use crate::db::Db;
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
pub struct StudentRecord {
	pub id: i64,
	pub nisn: String,
	pub name: String,
	pub class: Option<String>,
	pub school: Option<String>
}

pub fn list_students(db: &Db) -> Result<Vec<StudentRecord>, String> {
	let conn = db.lock().unwrap();
	let mut stmt = conn.prepare("SELECT id, nisn, name, class, school FROM students ORDER BY class, name").map_err(|e| e.to_string())?;
	let rows = stmt
		.query_map([], |r| Ok(StudentRecord { id: r.get(0)?, nisn: r.get(1)?, name: r.get(2)?, class: r.get(3)?, school: r.get(4)? }))
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
		.prepare("SELECT id, nisn, name, class, school FROM students WHERE school = ?1 COLLATE NOCASE ORDER BY class, name")
		.map_err(|e| e.to_string())?;
	let rows = stmt
		.query_map(params![school], |r| Ok(StudentRecord { id: r.get(0)?, nisn: r.get(1)?, name: r.get(2)?, class: r.get(3)?, school: r.get(4)? }))
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
	for r in records {
		tx.execute(
			"INSERT INTO students (nisn, name, class, school) VALUES (?1, ?2, ?3, ?4)
			 ON CONFLICT(nisn) DO UPDATE SET name = excluded.name, class = excluded.class, school = excluded.school",
			params![r.nisn.trim(), r.name.trim(), r.class.trim(), r.school.trim()]
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
	let mut stmt = conn.prepare("SELECT title FROM classes ORDER BY id").map_err(|e| e.to_string())?;
	let rows = stmt.query_map([], |r| r.get::<_, String>(0)).map_err(|e| e.to_string())?;
	rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

pub fn create_student(db: &Db, nisn: String, name: String, class: Option<String>) -> Result<i64, String> {
	let conn = db.lock().unwrap();
	conn.execute(
		"INSERT INTO students (nisn, name, class) VALUES (?1, ?2, ?3)",
		params![nisn.trim(), name.trim(), class]
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
	for row in &rows {
		let nisn = row.nisn.trim();
		let name = row.name.trim();
		if nisn.is_empty() || name.is_empty() {
			continue;
		}
		let class = row.class.as_deref().map(str::trim).filter(|c| !c.is_empty());

		tx.execute(
			"INSERT INTO students (nisn, name, class) VALUES (?1, ?2, ?3)
			 ON CONFLICT(nisn) DO UPDATE SET name = excluded.name, class = excluded.class",
			params![nisn, name, class]
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
	fn imports_students_from_csv() {
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

		let name: String =
			db.lock().unwrap().query_row("SELECT name FROM students WHERE nisn = '1111111111'", [], |r| r.get(0)).unwrap();
		assert_eq!(name, "Rina Wati");

		let classes = list_classes(&db).unwrap();
		assert_eq!(classes, vec!["4".to_string(), "5".to_string(), "6".to_string()]);

		drop(db);
		let _ = std::fs::remove_dir_all(&dir);
	}
}
