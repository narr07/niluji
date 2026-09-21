use crate::db::Db;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchoolInfo {
	pub name: String,
	pub npsn: String,
	pub address: String,
	pub principal: String,
	// Gerbang fitur Export bank soal — supaya tidak sembarang guru di komputer yang sama bisa
	// mengekspor. Diset & diganti sendiri lewat form Data Sekolah ini.
	pub export_pin: String
}

pub fn get_school(db: &Db) -> Result<SchoolInfo, String> {
	db.lock()
		.unwrap()
		.query_row("SELECT name, npsn, address, principal, export_pin FROM school WHERE id = 1", [], |r| {
			Ok(SchoolInfo { name: r.get(0)?, npsn: r.get(1)?, address: r.get(2)?, principal: r.get(3)?, export_pin: r.get(4)? })
		})
		.map_err(|e| e.to_string())
}

pub fn update_school(db: &Db, info: SchoolInfo) -> Result<(), String> {
	db.lock()
		.unwrap()
		.execute(
			"UPDATE school SET name = ?1, npsn = ?2, address = ?3, principal = ?4, export_pin = ?5 WHERE id = 1",
			params![info.name.trim(), info.npsn.trim(), info.address.trim(), info.principal.trim(), info.export_pin.trim()]
		)
		.map_err(|e| e.to_string())?;
	Ok(())
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassRecord {
	pub id: i64,
	pub title: String,
	pub description: Option<String>
}

pub fn list_classes_full(db: &Db) -> Result<Vec<ClassRecord>, String> {
	let conn = db.lock().unwrap();
	let mut stmt = conn.prepare("SELECT id, title, description FROM classes ORDER BY sort_order, id").map_err(|e| e.to_string())?;
	let rows = stmt
		.query_map([], |r| Ok(ClassRecord { id: r.get(0)?, title: r.get(1)?, description: r.get(2)? }))
		.map_err(|e| e.to_string())?;
	rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

pub fn create_class(db: &Db, title: String, description: Option<String>) -> Result<i64, String> {
	let conn = db.lock().unwrap();
	// Kelas baru selalu masuk ke urutan paling akhir; guru bisa geser lewat reorder_classes
	// kalau mau ditaruh di posisi lain.
	let next_order: i64 =
		conn.query_row("SELECT COALESCE(MAX(sort_order), 0) + 1 FROM classes", [], |r| r.get(0)).map_err(|e| e.to_string())?;
	conn.execute(
		"INSERT INTO classes (title, description, sort_order) VALUES (?1, ?2, ?3)",
		params![title.trim(), description, next_order]
	)
	.map_err(|e| e.to_string())?;
	Ok(conn.last_insert_rowid())
}

pub fn update_class(db: &Db, id: i64, title: String, description: Option<String>) -> Result<(), String> {
	db.lock()
		.unwrap()
		.execute("UPDATE classes SET title = ?1, description = ?2 WHERE id = ?3", params![title.trim(), description, id])
		.map_err(|e| e.to_string())?;
	Ok(())
}

pub fn delete_class(db: &Db, id: i64) -> Result<(), String> {
	db.lock().unwrap().execute("DELETE FROM classes WHERE id = ?1", params![id]).map_err(|e| e.to_string())?;
	Ok(())
}

// Dipanggil dengan daftar id LENGKAP dalam urutan baru yang diinginkan (misal setelah drag &
// drop di UI) — posisinya di array jadi nilai sort_order barunya. Urutan ini nentuin urutan
// kelas di semua menu lain yang nampilin daftar kelas, bukan cuma tampilan di halaman ini saja.
pub fn reorder_classes(db: &Db, ordered_ids: Vec<i64>) -> Result<(), String> {
	let mut conn = db.lock().unwrap();
	let tx = conn.transaction().map_err(|e| e.to_string())?;
	for (index, id) in ordered_ids.iter().enumerate() {
		tx.execute("UPDATE classes SET sort_order = ?1 WHERE id = ?2", params![index as i64, id]).map_err(|e| e.to_string())?;
	}
	tx.commit().map_err(|e| e.to_string())?;
	Ok(())
}

// A "jenis soal" is a named question package (e.g. "UTS Ganjil 2026") so a teacher can build
// a new exam from a fresh set of questions without disturbing the ones already used elsewhere
// for the same kelas + mata pelajaran. `class`/`subject_id` scope which kelas/mapel it shows
// up for — NULL on either means "every class" / "every subject" on that axis, so a jenis can
// be as global or as narrow (this exact kelas + this exact mapel) as the teacher wants.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionTypeRecord {
	pub id: i64,
	pub name: String,
	pub description: Option<String>,
	pub class: Option<String>,
	pub subject_id: Option<i64>
}

// `class`/`subject_id` here are the CURRENT kelas + mapel a page is asking for — a jenis row
// matches if it's scoped to exactly that class/subject, or left NULL (wildcard) on that axis.
pub fn list_question_types(db: &Db, class: Option<String>, subject_id: Option<i64>) -> Result<Vec<QuestionTypeRecord>, String> {
	let conn = db.lock().unwrap();
	let mut stmt = conn
		.prepare(
			"SELECT id, name, description, class, subject_id FROM question_types
			 WHERE (?1 IS NULL OR class IS NULL OR class = ?1)
			   AND (?2 IS NULL OR subject_id IS NULL OR subject_id = ?2)
			 ORDER BY name"
		)
		.map_err(|e| e.to_string())?;
	let rows = stmt
		.query_map(params![class, subject_id], |r| {
			Ok(QuestionTypeRecord { id: r.get(0)?, name: r.get(1)?, description: r.get(2)?, class: r.get(3)?, subject_id: r.get(4)? })
		})
		.map_err(|e| e.to_string())?;
	rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

pub fn create_question_type(
	db: &Db,
	name: String,
	description: Option<String>,
	class: Option<String>,
	subject_id: Option<i64>
) -> Result<i64, String> {
	let conn = db.lock().unwrap();
	conn.execute(
		"INSERT INTO question_types (name, description, class, subject_id) VALUES (?1, ?2, ?3, ?4)",
		params![name.trim(), description, class, subject_id]
	)
	.map_err(|e| e.to_string())?;
	Ok(conn.last_insert_rowid())
}

pub fn update_question_type(db: &Db, id: i64, name: String, description: Option<String>) -> Result<(), String> {
	db.lock()
		.unwrap()
		.execute("UPDATE question_types SET name = ?1, description = ?2 WHERE id = ?3", params![name.trim(), description, id])
		.map_err(|e| e.to_string())?;
	Ok(())
}

pub fn delete_question_type(db: &Db, id: i64) -> Result<(), String> {
	db.lock().unwrap().execute("DELETE FROM question_types WHERE id = ?1", params![id]).map_err(|e| e.to_string())?;
	Ok(())
}

// Hapus jenis ujian dengan scope pilihan guru — perlu ini karena jenis yang dibuat "berlaku
// untuk semua kelas" (class IS NULL) atau "semua pelajaran" (subject_id IS NULL) itu SATU baris
// data yang sama dipakai di banyak kelas/pelajaran sekaligus. Menghapus baris itu langsung akan
// menghilangkannya dari SEMUA kelas/pelajaran yang tadinya ikut memakai wildcard tsb — bukan cuma
// dari kelas/pelajaran yang sedang dibuka. Untuk scope "subject"/"class", baris wildcard itu
// dipecah dulu: dibuatkan baris eksplisit untuk tiap kelas/pelajaran LAIN yang masih perlu tetap
// memakainya, baru baris aslinya dihapus — supaya jenis ini cuma hilang dari kelas/pelajaran yang
// dipilih guru, sisanya tetap ada.
pub fn delete_question_type_scoped(
	db: &Db,
	id: i64,
	scope: &str,
	current_class: String,
	current_subject_id: i64
) -> Result<(), String> {
	let mut conn = db.lock().unwrap();
	let tx = conn.transaction().map_err(|e| e.to_string())?;

	let (name, description, row_class, row_subject_id): (String, Option<String>, Option<String>, Option<i64>) = tx
		.query_row("SELECT name, description, class, subject_id FROM question_types WHERE id = ?1", params![id], |r| {
			Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
		})
		.map_err(|_| "Jenis ujian tidak ditemukan".to_string())?;

	match scope {
		"all" => {}
		// Row berlaku untuk SEMUA pelajaran (subject_id NULL) — pecah jadi baris eksplisit untuk
		// tiap pelajaran LAIN (selain yang sedang dibuka) supaya jenis ini tetap ada di sana.
		"subject" if row_subject_id.is_none() => {
			let mut stmt = tx.prepare("SELECT id FROM subjects WHERE id != ?1").map_err(|e| e.to_string())?;
			let other_subject_ids: Vec<i64> = stmt
				.query_map(params![current_subject_id], |r| r.get(0))
				.map_err(|e| e.to_string())?
				.collect::<Result<_, _>>()
				.map_err(|e| e.to_string())?;
			drop(stmt);
			for sid in other_subject_ids {
				tx.execute(
					"INSERT OR IGNORE INTO question_types (name, description, class, subject_id) VALUES (?1, ?2, ?3, ?4)",
					params![name, description, row_class, sid]
				)
				.map_err(|e| e.to_string())?;
			}
		}
		// Row berlaku untuk SEMUA kelas (class NULL) — pecah jadi baris eksplisit untuk tiap
		// kelas LAIN (selain yang sedang dibuka) supaya jenis ini tetap ada di sana.
		"class" if row_class.is_none() => {
			let mut stmt = tx.prepare("SELECT title FROM classes WHERE title != ?1").map_err(|e| e.to_string())?;
			let other_classes: Vec<String> = stmt
				.query_map(params![current_class], |r| r.get(0))
				.map_err(|e| e.to_string())?
				.collect::<Result<_, _>>()
				.map_err(|e| e.to_string())?;
			drop(stmt);
			for cls in other_classes {
				tx.execute(
					"INSERT OR IGNORE INTO question_types (name, description, class, subject_id) VALUES (?1, ?2, ?3, ?4)",
					params![name, description, cls, row_subject_id]
				)
				.map_err(|e| e.to_string())?;
			}
		}
		// Row sudah sespesifik kelas/pelajaran yang sedang dibuka (tidak ada wildcard di axis
		// itu) — tidak ada kelas/pelajaran lain yang ikut memakainya, jadi hapus langsung saja.
		"subject" | "class" => {}
		other => return Err(format!("Scope tidak dikenal: {other}"))
	}

	tx.execute("DELETE FROM question_types WHERE id = ?1", params![id]).map_err(|e| e.to_string())?;
	tx.commit().map_err(|e| e.to_string())?;
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn reads_and_updates_school() {
		let dir = std::env::temp_dir().join(format!(
			"nuxtor_cbt_settings_test_{}_{}",
			std::process::id(),
			std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
		));
		let db = crate::db::open(&dir);

		let info = get_school(&db).unwrap();
		assert_eq!(info.name, "Nama Sekolah");

		update_school(
			&db,
			SchoolInfo {
				name: "SDN Uji Coba".into(),
				npsn: "12345678".into(),
				address: "Jl. Contoh".into(),
				principal: "Budi".into(),
				export_pin: "9999".into()
			}
		)
		.unwrap();
		let updated = get_school(&db).unwrap();
		assert_eq!(updated.name, "SDN Uji Coba");

		let classes = list_classes_full(&db).unwrap();
		assert_eq!(classes.len(), 3);

		let id = create_class(&db, "7".into(), Some("Kelas tambahan".into())).unwrap();
		update_class(&db, id, "7A".into(), None).unwrap();
		delete_class(&db, id).unwrap();

		let qt_id = create_question_type(&db, "UTS Ganjil".into(), Some("Ujian tengah semester".into()), None, None).unwrap();
		update_question_type(&db, qt_id, "UTS Ganjil 2026".into(), None).unwrap();
		assert_eq!(list_question_types(&db, None, None).unwrap().len(), 1);
		delete_question_type(&db, qt_id).unwrap();
		assert_eq!(list_question_types(&db, None, None).unwrap().len(), 0);

		// Nama yang sama boleh dipakai lagi asal scope-nya beda (mis. jenis "Ulangan Harian"
		// khusus Kelas 4 + Matematika, terpisah dari yang berlaku global) — UNIQUE gabungan,
		// bukan cuma UNIQUE(name), yang bikin ini valid.
		create_question_type(&db, "Ulangan Harian".into(), None, None, None).unwrap();
		create_question_type(&db, "Ulangan Harian".into(), None, Some("4".into()), Some(1)).unwrap();
		let scoped = list_question_types(&db, Some("4".into()), Some(1)).unwrap();
		assert_eq!(scoped.len(), 2, "jenis global + jenis khusus kelas 4/mapel 1 sama-sama harus muncul untuk kelas 4/mapel 1");
		let other_class = list_question_types(&db, Some("5".into()), Some(1)).unwrap();
		assert_eq!(other_class.len(), 1, "jenis khusus kelas 4 tidak boleh muncul untuk kelas 5");

		drop(db);
		let _ = std::fs::remove_dir_all(&dir);
	}
}
