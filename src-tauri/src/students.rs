use crate::db::Db;
use crate::import::{find_column, read_rows, CLASS_COLS};
use rusqlite::params;
use serde::Serialize;

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
	pub class: Option<String>
}

pub fn list_students(db: &Db) -> Result<Vec<StudentRecord>, String> {
	let conn = db.lock().unwrap();
	let mut stmt = conn.prepare("SELECT id, nisn, name, class FROM students ORDER BY class, name").map_err(|e| e.to_string())?;
	let rows = stmt
		.query_map([], |r| Ok(StudentRecord { id: r.get(0)?, nisn: r.get(1)?, name: r.get(2)?, class: r.get(3)? }))
		.map_err(|e| e.to_string())?;
	rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
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

pub fn import_students(db: &Db, path: &str) -> Result<StudentImportSummary, String> {
	let (headers, rows) = read_rows(path)?;

	let mut conn = db.lock().unwrap();
	let tx = conn.transaction().map_err(|e| e.to_string())?;

	let mut students_imported = 0usize;
	for row in &rows {
		let Some(nisn) = find_column(&headers, row, NISN_COLS) else {
			continue;
		};
		let Some(name) = find_column(&headers, row, NAME_COLS) else {
			continue;
		};
		let class = find_column(&headers, row, CLASS_COLS);

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

		let summary = import_students(&db, csv_path.to_str().unwrap()).unwrap();
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
