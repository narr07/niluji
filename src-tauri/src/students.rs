use crate::db::{self, Db};
use crate::import::{find_column, read_rows, CLASS_COLS};
use rusqlite::params;
use serde::{Deserialize, Serialize};

const NISN_COLS: &[&str] = &["nisn", "nis"];
const NAME_COLS: &[&str] = &["name", "nama", "nama_siswa", "nama siswa"];

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StudentImportSummary {
	pub students_imported: usize
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StudentRecord {
	pub id: i64,
	pub nisn: String,
	pub name: String,
	pub class: Option<String>,
	pub school: Option<String>,
	pub pin: String
}

pub fn list_students(db: &Db) -> Result<Vec<StudentRecord>, String> {
	let conn = db.lock().unwrap();
	let mut stmt = conn
		.prepare("SELECT id, nisn, name, class, school, COALESCE(pin, '') FROM students ORDER BY class, name")
		.map_err(|e| e.to_string())?;
	let rows = stmt
		.query_map([], |r| {
			let id: i64 = r.get(0)?;
			let nisn: String = r.get(1)?;
			let raw_pin: String = r.get(5)?;
			let pin = if raw_pin.trim().is_empty() {
				db::generate_pin_from_seed(id, &nisn)
			} else {
				raw_pin
			};
			Ok(StudentRecord { id, nisn, name: r.get(2)?, class: r.get(3)?, school: r.get(4)?, pin })
		})
		.map_err(|e| e.to_string())?;
	rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

// Nama sekolah yang sudah pernah ditarik dari sumber online (dipakai sebagai dropdown di UI
// "Tarik Data Siswa" — jadi guru bisa pilih sekolah yang pernah dipakai, bukan mengetik ulang).
pub fn list_schools(db: &Db) -> Result<Vec<String>, String> {
	let conn = db.lock().unwrap();
	let mut stmt = conn
		.prepare("SELECT DISTINCT school FROM students WHERE school IS NOT NULL AND TRIM(school) != '' ORDER BY school")
		.map_err(|e| e.to_string())?;
	let rows = stmt.query_map([], |r| r.get::<_, String>(0)).map_err(|e| e.to_string())?;
	rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

pub fn students_by_school(db: &Db, school: &str) -> Result<Vec<StudentRecord>, String> {
	let conn = db.lock().unwrap();
	let mut stmt = conn
		.prepare("SELECT id, nisn, name, class, school, COALESCE(pin, '') FROM students WHERE school = ?1 COLLATE NOCASE ORDER BY class, name")
		.map_err(|e| e.to_string())?;
	let rows = stmt
		.query_map(params![school], |r| {
			let id: i64 = r.get(0)?;
			let nisn: String = r.get(1)?;
			let raw_pin: String = r.get(5)?;
			let pin = if raw_pin.trim().is_empty() {
				db::generate_pin_from_seed(id, &nisn)
			} else {
				raw_pin
			};
			Ok(StudentRecord { id, nisn, name: r.get(2)?, class: r.get(3)?, school: r.get(4)?, pin })
		})
		.map_err(|e| e.to_string())?;
	rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

// Baris CSV persis seperti yang dipublish dari Google Sheets untuk fitur "Tarik Data Siswa".
// NISN & KELAS tetap String dengan sengaja: NISN bisa berawalan nol ("0166955541"), dan KELAS
// kadang berupa teks ("IVA", "IVB") bukan cuma angka — keduanya tidak boleh pernah diperlakukan
// sebagai angka.
#[derive(Debug, Deserialize)]
struct TarikSiswaRow {
	#[serde(rename = "NISN")]
	nisn: String,
	#[serde(rename = "NAMA SISWA")]
	name: String,
	#[serde(rename = "KELAS")]
	class: String,
	#[serde(rename = "Nama Sekolah")]
	school: String
}

fn upsert_students(db: &Db, records: &[TarikSiswaRow]) -> Result<usize, String> {
	let mut conn = db.lock().unwrap();
	let tx = conn.transaction().map_err(|e| e.to_string())?;

	let mut count = 0usize;
	for (i, r) in records.iter().enumerate() {
		let pin = db::generate_random_pin(i as u64 + 1);
		tx.execute(
			"INSERT INTO students (nisn, name, class, school, pin) VALUES (?1, ?2, ?3, ?4, ?5)
			 ON CONFLICT(nisn) DO UPDATE SET name = excluded.name, class = excluded.class, school = excluded.school,
			 pin = COALESCE(NULLIF(TRIM(students.pin), ''), excluded.pin)",
			params![r.nisn.trim(), r.name.trim(), r.class.trim(), r.school.trim(), pin]
		)
		.map_err(|e| e.to_string())?;
		count += 1;
	}

	tx.commit().map_err(|e| e.to_string())?;
	Ok(count)
}

// Ambil daftar nama sekolah PERSIS seperti tertulis di sumber CSV (bukan dari database lokal) —
// dipakai buat dropdown di UI "Tarik Data Siswa", supaya guru memilih satu nama sekolah yang
// benar-benar ada & lengkap (mis. "SD NEGERI CIPINANG I" vs "SD NEGERI CIPINANG II" tetap
// kebeda, tidak ketuker gara-gara sama-sama mengandung kata "cipinang").
pub async fn list_schools_from_source(csv_url: &str) -> Result<Vec<String>, String> {
	let rows = fetch_csv(csv_url).await?;
	let mut seen = std::collections::BTreeSet::new();
	for row in &rows {
		seen.insert(row.school.trim().to_string());
	}
	Ok(seen.into_iter().collect())
}

async fn fetch_csv(csv_url: &str) -> Result<Vec<TarikSiswaRow>, String> {
	let response = reqwest::get(csv_url).await.map_err(|e| format!("Gagal mengambil data: {e}"))?;
	if !response.status().is_success() {
		return Err(format!("Gagal mengambil data: server merespons status {}", response.status()));
	}
	let body = response.text().await.map_err(|e| format!("Gagal membaca respons: {e}"))?;

	let mut reader = csv::Reader::from_reader(body.as_bytes());
	reader
		.deserialize::<TarikSiswaRow>()
		.map(|record| record.map_err(|e| format!("Format CSV dari sumber tidak sesuai: {e}")))
		.collect()
}

// Ambil CSV yang dipublish dari Google Sheets, filter baris sesuai nama sekolah, lalu simpan/update
// ke database lokal. `school` di sini harus nama sekolah yang PERSIS (dipilih dari dropdown hasil
// list_schools_from_source, bukan sekadar potongan kata) — supaya sekolah dengan nama mirip (mis.
// "SD NEGERI CIPINANG I" vs "SD NEGERI CIPINANG II") tidak ketarik gabung jadi satu.
pub async fn tarik_data_siswa(db: &Db, csv_url: &str, school: &str) -> Result<usize, String> {
	let rows = fetch_csv(csv_url).await?;
	let school_trimmed = school.trim();
	let matched: Vec<TarikSiswaRow> = rows.into_iter().filter(|row| row.school.trim().eq_ignore_ascii_case(school_trimmed)).collect();

	if matched.is_empty() {
		return Err(format!("Nama sekolah \"{school_trimmed}\" tidak ditemukan di sumber data."));
	}

	upsert_students(db, &matched)
}

// Classes are managed centrally in the "classes" table (see Pengaturan > Data Kelas),
// so Bank Soal, Kelola Ujian, Siswa, and Hasil Ujian all pick "kelas" from the same list.
pub fn list_classes(db: &Db) -> Result<Vec<String>, String> {
	let conn = db.lock().unwrap();
	let mut stmt = conn.prepare("SELECT title FROM classes ORDER BY sort_order, id").map_err(|e| e.to_string())?;
	let rows = stmt.query_map([], |r| r.get::<_, String>(0)).map_err(|e| e.to_string())?;
	rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

pub fn create_student(db: &Db, nisn: String, name: String, class: Option<String>) -> Result<i64, String> {
	let conn = db.lock().unwrap();
	let pin = db::generate_random_pin(1);
	conn.execute(
		"INSERT INTO students (nisn, name, class, pin) VALUES (?1, ?2, ?3, ?4)",
		params![nisn.trim(), name.trim(), class, pin]
	)
	.map_err(|e| e.to_string())?;
	Ok(conn.last_insert_rowid())
}

pub fn update_student(db: &Db, id: i64, nisn: String, name: String, class: Option<String>) -> Result<(), String> {
	db.lock()
		.unwrap()
		.execute(
			"UPDATE students SET nisn = ?1, name = ?2, class = ?3 WHERE id = ?4",
			params![nisn.trim(), name.trim(), class, id]
		)
		.map_err(|e| e.to_string())?;
	Ok(())
}

pub fn delete_student(db: &Db, id: i64) -> Result<(), String> {
	db.lock().unwrap().execute("DELETE FROM students WHERE id = ?1", params![id]).map_err(|e| e.to_string())?;
	Ok(())
}

pub fn regenerate_student_pins(db: &Db, class: Option<String>) -> Result<usize, String> {
	let mut conn = db.lock().unwrap();
	let tx = conn.transaction().map_err(|e| e.to_string())?;

	let ids: Vec<i64> = {
		let mut stmt = tx
			.prepare("SELECT id FROM students WHERE (?1 IS NULL OR class = ?1)")
			.map_err(|e| e.to_string())?;
		let mapped = stmt.query_map(params![class], |r| r.get(0)).map_err(|e| e.to_string())?;
		mapped.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?
	};

	for (idx, id) in ids.iter().enumerate() {
		let pin = db::generate_random_pin((idx as u64).wrapping_add(*id as u64));
		tx.execute("UPDATE students SET pin = ?1 WHERE id = ?2", params![pin, id]).map_err(|e| e.to_string())?;
	}

	tx.commit().map_err(|e| e.to_string())?;
	Ok(ids.len())
}

// Baris hasil parsing file CSV/Excel siswa, buat ditampilkan sebagai preview yang bisa
// diedit di UI dulu (sama seperti alur Import dari Word) SEBELUM benar-benar ditulis ke
// database — jadi kalau ada NISN/nama yang kebaca salah/kosong, guru bisa benerin langsung
// tanpa perlu edit ulang file sumbernya lalu import ulang dari awal.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StudentImportRow {
	pub nisn: String,
	pub name: String,
	pub class: Option<String>
}

pub fn parse_students_file(path: &str) -> Result<Vec<StudentImportRow>, String> {
	let (headers, rows) = read_rows(path)?;

	let mut result = Vec::new();
	for row in &rows {
		let nisn = find_column(&headers, row, NISN_COLS).unwrap_or_default();
		let name = find_column(&headers, row, NAME_COLS).unwrap_or_default();
		let class = find_column(&headers, row, CLASS_COLS);
		// Baris yang dua-duanya kosong biasanya cuma baris kosong di akhir file — dilewati
		// diam-diam. Baris yang salah satunya saja kosong tetap disertakan supaya kelihatan
		// di preview dan bisa dibenerin manual, bukan hilang tanpa jejak.
		if nisn.is_empty() && name.is_empty() {
			continue;
		}
		result.push(StudentImportRow { nisn, name, class });
	}
	Ok(result)
}

pub fn import_students_rows(db: &Db, rows: Vec<StudentImportRow>) -> Result<StudentImportSummary, String> {
	let mut conn = db.lock().unwrap();
	let tx = conn.transaction().map_err(|e| e.to_string())?;

	let mut students_imported = 0usize;
	for (i, row) in rows.iter().enumerate() {
		let nisn = row.nisn.trim();
		let name = row.name.trim();
		if nisn.is_empty() || name.is_empty() {
			continue;
		}
		let class = row.class.as_deref().map(str::trim).filter(|c| !c.is_empty());
		let pin = db::generate_random_pin(i as u64 + 10);

		tx.execute(
			"INSERT INTO students (nisn, name, class, pin) VALUES (?1, ?2, ?3, ?4)
			 ON CONFLICT(nisn) DO UPDATE SET name = excluded.name, class = excluded.class,
			 pin = COALESCE(NULLIF(TRIM(students.pin), ''), excluded.pin)",
			params![nisn, name, class, pin]
		)
		.map_err(|e| e.to_string())?;
		students_imported += 1;
	}

	tx.commit().map_err(|e| e.to_string())?;
	Ok(StudentImportSummary { students_imported })
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::io::Write;

	#[test]
	fn imports_students_from_csv_and_generates_pins() {
		let dir = std::env::temp_dir().join(format!("nuxtor_cbt_students_test_{}", std::process::id()));
		let db = crate::db::open(&dir);

		let csv_path = dir.join("siswa.csv");
		let mut f = std::fs::File::create(&csv_path).unwrap();
		writeln!(f, "nisn,nama,kelas").unwrap();
		writeln!(f, "1111111111,Rina Wati,4").unwrap();
		writeln!(f, "2222222222,Dedi Kurnia,4").unwrap();
		drop(f);

		let rows = parse_students_file(csv_path.to_str().unwrap()).unwrap();
		assert_eq!(rows.len(), 2);
		let summary = import_students_rows(&db, rows).unwrap();
		assert_eq!(summary.students_imported, 2);

		let list = list_students(&db).unwrap();
		assert_eq!(list.len(), 2);
		assert_eq!(list[0].pin.len(), 6);

		let regenerated = regenerate_student_pins(&db, Some("4".to_string())).unwrap();
		assert_eq!(regenerated, 2);

		let classes = list_classes(&db).unwrap();
		assert_eq!(classes, vec!["4".to_string(), "5".to_string(), "6".to_string()]);

		drop(db);
		let _ = std::fs::remove_dir_all(&dir);
	}
}
