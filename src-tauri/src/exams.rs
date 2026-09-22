use crate::db::Db;
use rusqlite::params;
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubjectSummary {
	pub id: i64,
	pub name: String,
	pub code: Option<String>
}

pub fn list_subjects(db: &Db) -> Result<Vec<SubjectSummary>, String> {
	let conn = db.lock().unwrap();
	let mut stmt = conn.prepare("SELECT id, name, code FROM subjects ORDER BY sort_order, name").map_err(|e| e.to_string())?;
	let rows = stmt
		.query_map([], |r| Ok(SubjectSummary { id: r.get(0)?, name: r.get(1)?, code: r.get(2)? }))
		.map_err(|e| e.to_string())?;
	rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

pub fn create_subject(db: &Db, name: String, code: Option<String>) -> Result<i64, String> {
	let conn = db.lock().unwrap();
	// Mata pelajaran baru selalu masuk ke urutan paling akhir; guru bisa geser lewat
	// reorder_subjects kalau mau ditaruh di posisi lain.
	let next_order: i64 =
		conn.query_row("SELECT COALESCE(MAX(sort_order), 0) + 1 FROM subjects", [], |r| r.get(0)).map_err(|e| e.to_string())?;
	conn.execute(
		"INSERT INTO subjects (name, code, sort_order) VALUES (?1, ?2, ?3)",
		params![name.trim(), code, next_order]
	)
	.map_err(|e| e.to_string())?;
	Ok(conn.last_insert_rowid())
}

pub fn update_subject(db: &Db, id: i64, name: String, code: Option<String>) -> Result<(), String> {
	db.lock()
		.unwrap()
		.execute("UPDATE subjects SET name = ?1, code = ?2 WHERE id = ?3", params![name.trim(), code, id])
		.map_err(|e| e.to_string())?;
	Ok(())
}

pub fn delete_subject(db: &Db, id: i64) -> Result<(), String> {
	db.lock().unwrap().execute("DELETE FROM subjects WHERE id = ?1", params![id]).map_err(|e| e.to_string())?;
	Ok(())
}

// Dipanggil dengan daftar id LENGKAP dalam urutan baru yang diinginkan (misal setelah geser
// naik/turun di UI) — posisinya di array jadi nilai sort_order barunya.
pub fn reorder_subjects(db: &Db, ordered_ids: Vec<i64>) -> Result<(), String> {
	let mut conn = db.lock().unwrap();
	let tx = conn.transaction().map_err(|e| e.to_string())?;
	for (index, id) in ordered_ids.iter().enumerate() {
		tx.execute("UPDATE subjects SET sort_order = ?1 WHERE id = ?2", params![index as i64, id]).map_err(|e| e.to_string())?;
	}
	tx.commit().map_err(|e| e.to_string())?;
	Ok(())
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExamSummary {
	pub id: i64,
	pub subject: String,
	pub class: Option<String>,
	pub jenis: Option<String>,
	pub title: String,
	pub duration: i64,
	pub scheduled_at: Option<i64>,
	pub window_end: Option<i64>,
	pub token: String,
	pub randomize_pg: bool,
	pub randomize_essay: bool
}

pub fn list_exams(db: &Db) -> Result<Vec<ExamSummary>, String> {
	let conn = db.lock().unwrap();
	let mut stmt = conn
		.prepare(
			"SELECT e.id, s.name, e.class, e.jenis, e.title, e.duration, e.scheduled_at, e.window_end, e.token, e.randomize_pg, e.randomize_essay
			 FROM exams e
			 JOIN subjects s ON s.id = e.subject_id
			 ORDER BY e.id DESC",
		)
		.map_err(|e| e.to_string())?;

	let rows = stmt
		.query_map([], |r| {
			Ok(ExamSummary {
				id: r.get(0)?,
				subject: r.get(1)?,
				class: r.get(2)?,
				jenis: r.get(3)?,
				title: r.get(4)?,
				duration: r.get(5)?,
				scheduled_at: r.get(6)?,
				window_end: r.get(7)?,
				token: r.get(8)?,
				randomize_pg: r.get::<_, Option<i64>>(9)?.unwrap_or(1) != 0,
				randomize_essay: r.get::<_, Option<i64>>(10)?.unwrap_or(1) != 0
			})
		})
		.map_err(|e| e.to_string())?;

	rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

// Dipanggil di awal create_exam/update_exam supaya guru tidak bisa menyimpan jadwal yang
// mustahil (durasi negatif/nol, atau ujian "selesai" sebelum "mulai").
fn validate_schedule(duration: i64, scheduled_at: i64, window_end: i64) -> Result<(), String> {
	if duration <= 0 {
		return Err("Durasi ujian harus lebih dari 0 menit.".to_string());
	}
	if window_end < scheduled_at {
		return Err("Jam selesai tidak boleh sebelum jam mulai.".to_string());
	}
	Ok(())
}

// `token UNIQUE` di database sudah mencegah dua ujian pakai token sama, tapi pesan error
// mentahnya ("UNIQUE constraint failed: exams.token") membingungkan buat guru — diganti pesan
// yang jelas begitu terdeteksi.
fn friendly_db_error(e: rusqlite::Error) -> String {
	if e.to_string().contains("UNIQUE constraint failed: exams.token") {
		"Token ujian ini sudah dipakai ujian lain — pakai token yang berbeda.".to_string()
	} else {
		e.to_string()
	}
}

#[allow(clippy::too_many_arguments)]
pub fn create_exam(
	db: &Db,
	subject_id: i64,
	class: Option<String>,
	jenis: Option<String>,
	title: String,
	duration: i64,
	scheduled_at: i64,
	window_end: i64,
	token: String,
	randomize_pg: bool,
	randomize_essay: bool
) -> Result<i64, String> {
	validate_schedule(duration, scheduled_at, window_end)?;

	let conn = db.lock().unwrap();
	conn.execute(
		"INSERT INTO exams (subject_id, class, jenis, title, duration, scheduled_at, window_end, token, randomize_pg, randomize_essay)
		 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
		params![
			subject_id,
			class,
			jenis,
			title.trim(),
			duration,
			scheduled_at,
			window_end,
			token.trim(),
			randomize_pg as i64,
			randomize_essay as i64
		]
	)
	.map_err(friendly_db_error)?;
	Ok(conn.last_insert_rowid())
}

#[allow(clippy::too_many_arguments)]
pub fn update_exam(
	db: &Db,
	id: i64,
	subject_id: i64,
	class: Option<String>,
	jenis: Option<String>,
	title: String,
	duration: i64,
	scheduled_at: i64,
	window_end: i64,
	token: String,
	randomize_pg: bool,
	randomize_essay: bool
) -> Result<(), String> {
	validate_schedule(duration, scheduled_at, window_end)?;

	db.lock()
		.unwrap()
		.execute(
			"UPDATE exams SET subject_id = ?1, class = ?2, jenis = ?3, title = ?4, duration = ?5, scheduled_at = ?6, window_end = ?7,
			 token = ?8, randomize_pg = ?9, randomize_essay = ?10 WHERE id = ?11",
			params![
				subject_id,
				class,
				jenis,
				title.trim(),
				duration,
				scheduled_at,
				window_end,
				token.trim(),
				randomize_pg as i64,
				randomize_essay as i64,
				id
			]
		)
		.map_err(friendly_db_error)?;
	Ok(())
}

pub fn delete_exam(db: &Db, id: i64) -> Result<(), String> {
	db.lock().unwrap().execute("DELETE FROM exams WHERE id = ?1", params![id]).map_err(|e| e.to_string())?;
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn creates_and_lists_exam() {
		let dir = std::env::temp_dir().join(format!("nuxtor_cbt_exams_test_{}", std::process::id()));
		let db = crate::db::open(&dir);

		db.lock().unwrap().execute("INSERT INTO subjects (name) VALUES ('Test Subject')", []).unwrap();
		let subject_id = db.lock().unwrap().last_insert_rowid();

		let exam_id = create_exam(
			&db,
			subject_id,
			Some("4".into()),
			Some("UTS Ganjil".into()),
			"UTS Matematika".into(),
			60,
			1_700_000_000,
			1_700_007_200,
			"UTS2026".into(),
			true,
			true
		)
		.unwrap();
		assert!(exam_id > 0);

		let exams = list_exams(&db).unwrap();
		assert_eq!(exams.len(), 1);
		assert_eq!(exams[0].subject, "Test Subject");
		assert_eq!(exams[0].class.as_deref(), Some("4"));
		assert_eq!(exams[0].scheduled_at, Some(1_700_000_000));
		assert_eq!(exams[0].window_end, Some(1_700_007_200));
		assert_eq!(exams[0].token, "UTS2026");

		drop(db);
		let _ = std::fs::remove_dir_all(&dir);
	}

	#[test]
	fn reorders_subjects() {
		let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
		let dir = std::env::temp_dir().join(format!("nuxtor_cbt_reorder_test_{}_{nanos}", std::process::id()));
		let db = crate::db::open(&dir);

		let a = create_subject(&db, "Uji A".into(), None).unwrap();
		let b = create_subject(&db, "Uji B".into(), None).unwrap();
		let c = create_subject(&db, "Uji C".into(), None).unwrap();

		let uji_names = |db: &Db| -> Vec<String> {
			list_subjects(db).unwrap().into_iter().filter(|s| s.name.starts_with("Uji ")).map(|s| s.name).collect()
		};

		assert_eq!(uji_names(&db), vec!["Uji A", "Uji B", "Uji C"]);

		reorder_subjects(&db, vec![c, a, b]).unwrap();
		assert_eq!(uji_names(&db), vec!["Uji C", "Uji A", "Uji B"]);

		drop(db);
		let _ = std::fs::remove_dir_all(&dir);
	}
}
