use crate::db::Db;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchoolInfo {
	pub name: String,
	pub npsn: String,
	pub address: String,
	pub principal: String
}

pub fn get_school(db: &Db) -> Result<SchoolInfo, String> {
	db.lock()
		.unwrap()
		.query_row("SELECT name, npsn, address, principal FROM school WHERE id = 1", [], |r| {
			Ok(SchoolInfo { name: r.get(0)?, npsn: r.get(1)?, address: r.get(2)?, principal: r.get(3)? })
		})
		.map_err(|e| e.to_string())
}

pub fn update_school(db: &Db, info: SchoolInfo) -> Result<(), String> {
	db.lock()
		.unwrap()
		.execute(
			"UPDATE school SET name = ?1, npsn = ?2, address = ?3, principal = ?4 WHERE id = 1",
			params![info.name.trim(), info.npsn.trim(), info.address.trim(), info.principal.trim()]
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
	let mut stmt = conn.prepare("SELECT id, title, description FROM classes ORDER BY id").map_err(|e| e.to_string())?;
	let rows = stmt
		.query_map([], |r| Ok(ClassRecord { id: r.get(0)?, title: r.get(1)?, description: r.get(2)? }))
		.map_err(|e| e.to_string())?;
	rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

pub fn create_class(db: &Db, title: String, description: Option<String>) -> Result<i64, String> {
	let conn = db.lock().unwrap();
	conn.execute("INSERT INTO classes (title, description) VALUES (?1, ?2)", params![title.trim(), description])
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

// A "jenis soal" is a named question package (e.g. "UTS Ganjil 2026") so a teacher can build
// a new exam from a fresh set of questions without disturbing the ones already used elsewhere
// for the same kelas + mata pelajaran.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionTypeRecord {
	pub id: i64,
	pub name: String,
	pub description: Option<String>
}

pub fn list_question_types(db: &Db) -> Result<Vec<QuestionTypeRecord>, String> {
	let conn = db.lock().unwrap();
	let mut stmt = conn.prepare("SELECT id, name, description FROM question_types ORDER BY name").map_err(|e| e.to_string())?;
	let rows = stmt
		.query_map([], |r| Ok(QuestionTypeRecord { id: r.get(0)?, name: r.get(1)?, description: r.get(2)? }))
		.map_err(|e| e.to_string())?;
	rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

pub fn create_question_type(db: &Db, name: String, description: Option<String>) -> Result<i64, String> {
	let conn = db.lock().unwrap();
	conn.execute("INSERT INTO question_types (name, description) VALUES (?1, ?2)", params![name.trim(), description])
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
			SchoolInfo { name: "SDN Uji Coba".into(), npsn: "12345678".into(), address: "Jl. Contoh".into(), principal: "Budi".into() }
		)
		.unwrap();
		let updated = get_school(&db).unwrap();
		assert_eq!(updated.name, "SDN Uji Coba");

		let classes = list_classes_full(&db).unwrap();
		assert_eq!(classes.len(), 3);

		let id = create_class(&db, "7".into(), Some("Kelas tambahan".into())).unwrap();
		update_class(&db, id, "7A".into(), None).unwrap();
		delete_class(&db, id).unwrap();

		let qt_id = create_question_type(&db, "UTS Ganjil".into(), Some("Ujian tengah semester".into())).unwrap();
		update_question_type(&db, qt_id, "UTS Ganjil 2026".into(), None).unwrap();
		assert_eq!(list_question_types(&db).unwrap().len(), 1);
		delete_question_type(&db, qt_id).unwrap();
		assert_eq!(list_question_types(&db).unwrap().len(), 0);

		drop(db);
		let _ = std::fs::remove_dir_all(&dir);
	}
}
