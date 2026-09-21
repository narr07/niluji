use crate::db::Db;
use calamine::{open_workbook_auto, Reader};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::Path;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportSummary {
	pub questions_imported: usize,
	pub subjects_created: usize
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionSummary {
	pub id: i64,
	pub subject: String,
	pub class: Option<String>,
	pub jenis: Option<String>,
	pub question_text: String,
	pub question_type: String,
	pub image: Option<String>,
	pub score: f64,
	pub option_count: i64
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardStats {
	pub students: i64,
	pub subjects: i64,
	pub jenis_ujian: i64
}

pub fn dashboard_stats(db: &Db) -> Result<DashboardStats, String> {
	let conn = db.lock().unwrap();
	let count = |table: &str| -> Result<i64, String> {
		conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0)).map_err(|e| e.to_string())
	};
	// Jumlah jenis ujian yang TERDAFTAR (dari registry question_types) — sama seperti badge di
	// Bank Soal, dihitung dari jenis yang sudah dibuat, bukan cuma yang sudah ada isi soalnya.
	let jenis_ujian = count("question_types")?;
	Ok(DashboardStats { students: count("students")?, subjects: count("subjects")?, jenis_ujian })
}

pub fn list_questions(db: &Db) -> Result<Vec<QuestionSummary>, String> {
	let conn = db.lock().unwrap();
	let mut stmt = conn
		.prepare(
			"SELECT q.id, s.name, q.class, q.jenis, q.question_text, q.question_type, q.image, q.score,
				(SELECT COUNT(*) FROM question_options o WHERE o.question_id = q.id)
			 FROM questions q
			 JOIN subjects s ON s.id = q.subject_id
			 ORDER BY s.name, q.class, q.id"
		)
		.map_err(|e| e.to_string())?;

	let rows = stmt
		.query_map([], |r| {
			Ok(QuestionSummary {
				id: r.get(0)?,
				subject: r.get(1)?,
				class: r.get(2)?,
				jenis: r.get(3)?,
				question_text: r.get(4)?,
				question_type: r.get(5)?,
				image: r.get(6)?,
				score: r.get(7)?,
				option_count: r.get(8)?
			})
		})
		.map_err(|e| e.to_string())?;

	rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

// Header name variants accepted per field, matched case-insensitively. Mirrors the
// flexible-column approach used by the scrom project's Excel quiz importer.
const SUBJECT_COLS: &[&str] = &["subject", "mata_pelajaran", "mapel", "mata pelajaran"];
pub(crate) const CLASS_COLS: &[&str] = &["class", "kelas", "grade", "tingkat"];
const QUESTION_COLS: &[&str] = &["question_text", "question", "soal", "pertanyaan"];
const OPTION_COLS: [(&str, &[&str]); 5] = [
	("A", &["option_a", "answer_a", "jawaban_a", "pilihan_a", "a"]),
	("B", &["option_b", "answer_b", "jawaban_b", "pilihan_b", "b"]),
	("C", &["option_c", "answer_c", "jawaban_c", "pilihan_c", "c"]),
	("D", &["option_d", "answer_d", "jawaban_d", "pilihan_d", "d"]),
	("E", &["option_e", "answer_e", "jawaban_e", "pilihan_e", "e"])
];
const CORRECT_COLS: &[&str] = &["correct_option", "correct_answer", "jawaban_benar", "kunci", "kunci_jawaban"];
const SCORE_COLS: &[&str] = &["score", "skor", "nilai", "bobot"];
const IMAGE_COLS: &[&str] = &["image", "gambar", "img", "foto"];
const JENIS_COLS: &[&str] = &["jenis", "jenis_soal", "paket", "paket_soal", "tipe"];

// Matches the "image" cell value against the uploaded files' original names. Teachers
// often type the name a bit differently than the actual file (wrong case, or without the
// extension), so this falls back from an exact match to case-insensitive, then to
// case-insensitive comparing just the base name (no extension) on both sides.
fn resolve_image(image_map: &std::collections::HashMap<String, String>, name: &str) -> Option<String> {
	if let Some(path) = image_map.get(name) {
		return Some(path.clone());
	}

	let lower = name.to_lowercase();
	if let Some((_, path)) = image_map.iter().find(|(k, _)| k.to_lowercase() == lower) {
		return Some(path.clone());
	}

	let stem = Path::new(name).file_stem().map(|s| s.to_string_lossy().to_lowercase())?;
	image_map
		.iter()
		.find(|(k, _)| Path::new(k).file_stem().map(|s| s.to_string_lossy().to_lowercase()).as_deref() == Some(stem.as_str()))
		.map(|(_, path)| path.clone())
}

pub(crate) fn find_column(headers: &[String], row: &[String], names: &[&str]) -> Option<String> {
	for name in names {
		if let Some(idx) = headers.iter().position(|h| h.trim().eq_ignore_ascii_case(name)) {
			if let Some(val) = row.get(idx) {
				let trimmed = val.trim();
				if !trimmed.is_empty() {
					return Some(trimmed.to_string());
				}
			}
		}
	}
	None
}

fn read_csv_rows(path: &str) -> Result<(Vec<String>, Vec<Vec<String>>), String> {
	let file = File::open(path).map_err(|e| format!("gagal buka file: {e}"))?;
	let mut reader = csv::ReaderBuilder::new().from_reader(file);
	let headers = reader.headers().map_err(|e| e.to_string())?.iter().map(String::from).collect();

	let mut rows = Vec::new();
	for result in reader.records() {
		let record = result.map_err(|e| e.to_string())?;
		rows.push(record.iter().map(String::from).collect());
	}
	Ok((headers, rows))
}

fn read_excel_rows(path: &str) -> Result<(Vec<String>, Vec<Vec<String>>), String> {
	let mut workbook = open_workbook_auto(path).map_err(|e| format!("gagal buka file excel: {e}"))?;
	let range = workbook
		.worksheet_range_at(0)
		.ok_or("File Excel tidak punya sheet")?
		.map_err(|e| e.to_string())?;

	let mut rows_iter = range.rows();
	let headers: Vec<String> = rows_iter.next().ok_or("File Excel kosong")?.iter().map(|c| c.to_string()).collect();
	let rows: Vec<Vec<String>> = rows_iter.map(|r| r.iter().map(|c| c.to_string()).collect()).collect();

	Ok((headers, rows))
}

pub(crate) fn read_rows(path: &str) -> Result<(Vec<String>, Vec<Vec<String>>), String> {
	let ext = Path::new(path).extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
	match ext.as_str() {
		"csv" => read_csv_rows(path),
		"xlsx" | "xls" => read_excel_rows(path),
		other => Err(format!("Format file .{other} tidak didukung. Gunakan .csv atau .xlsx.")),
	}
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportOptions {
	pub default_subject_id: Option<i64>,
	pub default_class: Option<String>,
	pub default_jenis: Option<String>,
	// "multiple_choice" or "essay" — applies to every row in this file. A file mixing both
	// types isn't supported; import PG and esai as two separate files instead.
	pub question_type: String,
	// Maps the raw filename referenced in the file's image column (e.g. "Soal 10.jpg") to
	// the already-saved relative path (e.g. "uploads/171234.jpg") from a prior image upload.
	pub image_map: std::collections::HashMap<String, String>
}

pub fn import_questions(db: &Db, path: &str, opts: ImportOptions) -> Result<ImportSummary, String> {
	let (headers, rows) = read_rows(path)?;
	let is_essay = opts.question_type == "essay";

	let mut conn = db.lock().unwrap();
	let tx = conn.transaction().map_err(|e| e.to_string())?;

	let mut subjects_created = 0usize;
	let mut questions_imported = 0usize;

	for (line, row) in rows.iter().enumerate() {
		let Some(question_text) = find_column(&headers, row, QUESTION_COLS) else {
			continue;
		};

		let subject_id: i64 = if let Some(name) = find_column(&headers, row, SUBJECT_COLS) {
			match tx.query_row("SELECT id FROM subjects WHERE name = ?1", params![name], |r| r.get(0)) {
				Ok(id) => id,
				Err(rusqlite::Error::QueryReturnedNoRows) => {
					tx.execute("INSERT INTO subjects (name) VALUES (?1)", params![name]).map_err(|e| e.to_string())?;
					subjects_created += 1;
					tx.last_insert_rowid()
				}
				Err(e) => return Err(e.to_string())
			}
		} else {
			opts.default_subject_id.ok_or_else(|| format!("baris {}: kolom mata pelajaran kosong", line + 2))?
		};

		let class = find_column(&headers, row, CLASS_COLS).or_else(|| opts.default_class.clone());
		let jenis = find_column(&headers, row, JENIS_COLS).or_else(|| opts.default_jenis.clone());
		let score: f64 = find_column(&headers, row, SCORE_COLS).and_then(|s| s.parse().ok()).unwrap_or(1.0);
		let image = find_column(&headers, row, IMAGE_COLS).and_then(|filename| resolve_image(&opts.image_map, &filename));
		let correct_option = if is_essay { String::new() } else { find_column(&headers, row, CORRECT_COLS).unwrap_or_default() };
		let question_type = if is_essay { "essay" } else { "multiple_choice" };

		tx.execute(
			"INSERT INTO questions (subject_id, class, jenis, question_text, question_type, image, answer_key, score) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
			params![subject_id, class, jenis, question_text, question_type, image, correct_option, score]
		)
		.map_err(|e| e.to_string())?;
		let question_id = tx.last_insert_rowid();

		if !is_essay {
			for (key, cols) in OPTION_COLS {
				if let Some(text) = find_column(&headers, row, cols) {
					let is_correct = correct_option.eq_ignore_ascii_case(key);
					tx.execute(
						"INSERT INTO question_options (question_id, option_key, option_text, is_correct) VALUES (?1, ?2, ?3, ?4)",
						params![question_id, key, text, is_correct]
					)
					.map_err(|e| e.to_string())?;
				}
			}
		}

		questions_imported += 1;
	}

	tx.commit().map_err(|e| e.to_string())?;
	Ok(ImportSummary { questions_imported, subjects_created })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionInput {
	pub subject_id: i64,
	pub class: Option<String>,
	pub jenis: Option<String>,
	pub question_text: String,
	pub question_type: Option<String>,
	pub image: Option<String>,
	pub score: f64,
	pub option_a: String,
	pub option_b: String,
	pub option_c: Option<String>,
	pub option_d: Option<String>,
	pub option_e: Option<String>,
	pub correct_option: String
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionDetail {
	pub id: i64,
	pub subject_id: i64,
	pub class: Option<String>,
	pub jenis: Option<String>,
	pub question_text: String,
	pub question_type: String,
	pub image: Option<String>,
	pub score: f64,
	pub option_a: String,
	pub option_b: String,
	pub option_c: String,
	pub option_d: String,
	pub option_e: String,
	pub correct_option: String
}

pub fn get_question(db: &Db, id: i64) -> Result<QuestionDetail, String> {
	let conn = db.lock().unwrap();
	#[allow(clippy::type_complexity)]
	let (subject_id, class, jenis, question_text, question_type, image, answer_key, score): (
		i64,
		Option<String>,
		Option<String>,
		String,
		String,
		Option<String>,
		Option<String>,
		f64
	) = conn
		.query_row(
			"SELECT subject_id, class, jenis, question_text, question_type, image, answer_key, score FROM questions WHERE id = ?1",
			params![id],
			|r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?, r.get(7)?))
		)
		.map_err(|_| "Soal tidak ditemukan".to_string())?;

	let mut options: std::collections::HashMap<String, String> = std::collections::HashMap::new();
	let mut stmt =
		conn.prepare("SELECT option_key, option_text FROM question_options WHERE question_id = ?1").map_err(|e| e.to_string())?;
	let mapped = stmt.query_map(params![id], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))).map_err(|e| e.to_string())?;
	for row in mapped {
		let (k, v) = row.map_err(|e| e.to_string())?;
		options.insert(k, v);
	}

	Ok(QuestionDetail {
		id,
		subject_id,
		class,
		jenis,
		question_text,
		question_type,
		image,
		score,
		option_a: options.get("A").cloned().unwrap_or_default(),
		option_b: options.get("B").cloned().unwrap_or_default(),
		option_c: options.get("C").cloned().unwrap_or_default(),
		option_d: options.get("D").cloned().unwrap_or_default(),
		option_e: options.get("E").cloned().unwrap_or_default(),
		correct_option: answer_key.unwrap_or_default()
	})
}

fn save_options(tx: &rusqlite::Transaction, question_id: i64, correct_option: &str, opts: &[(&str, &str)]) -> Result<(), String> {
	tx.execute("DELETE FROM question_options WHERE question_id = ?1", params![question_id]).map_err(|e| e.to_string())?;
	for (key, text) in opts {
		if text.trim().is_empty() {
			continue;
		}
		let is_correct = correct_option.eq_ignore_ascii_case(key);
		tx.execute(
			"INSERT INTO question_options (question_id, option_key, option_text, is_correct) VALUES (?1, ?2, ?3, ?4)",
			params![question_id, key, text.trim(), is_correct]
		)
		.map_err(|e| e.to_string())?;
	}
	Ok(())
}

pub fn create_question(db: &Db, input: QuestionInput) -> Result<i64, String> {
	let mut conn = db.lock().unwrap();
	let tx = conn.transaction().map_err(|e| e.to_string())?;
	let question_type = input.question_type.unwrap_or_else(|| "multiple_choice".to_string());

	tx.execute(
		"INSERT INTO questions (subject_id, class, jenis, question_text, question_type, image, answer_key, score) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
		params![
			input.subject_id,
			input.class,
			input.jenis,
			input.question_text.trim(),
			question_type,
			input.image,
			input.correct_option.trim(),
			input.score
		]
	)
	.map_err(|e| e.to_string())?;
	let id = tx.last_insert_rowid();

	save_options(
		&tx,
		id,
		&input.correct_option,
		&[
			("A", &input.option_a),
			("B", &input.option_b),
			("C", input.option_c.as_deref().unwrap_or("")),
			("D", input.option_d.as_deref().unwrap_or("")),
			("E", input.option_e.as_deref().unwrap_or(""))
		]
	)?;

	tx.commit().map_err(|e| e.to_string())?;
	Ok(id)
}

pub fn update_question(db: &Db, id: i64, input: QuestionInput) -> Result<(), String> {
	let mut conn = db.lock().unwrap();
	let tx = conn.transaction().map_err(|e| e.to_string())?;
	let question_type = input.question_type.unwrap_or_else(|| "multiple_choice".to_string());

	tx.execute(
		"UPDATE questions SET subject_id = ?1, class = ?2, jenis = ?3, question_text = ?4, question_type = ?5, image = ?6, answer_key = ?7, score = ?8 WHERE id = ?9",
		params![
			input.subject_id,
			input.class,
			input.jenis,
			input.question_text.trim(),
			question_type,
			input.image,
			input.correct_option.trim(),
			input.score,
			id
		]
	)
	.map_err(|e| e.to_string())?;

	save_options(
		&tx,
		id,
		&input.correct_option,
		&[
			("A", &input.option_a),
			("B", &input.option_b),
			("C", input.option_c.as_deref().unwrap_or("")),
			("D", input.option_d.as_deref().unwrap_or("")),
			("E", input.option_e.as_deref().unwrap_or(""))
		]
	)?;

	tx.commit().map_err(|e| e.to_string())?;
	Ok(())
}

pub fn delete_question(db: &Db, id: i64) -> Result<(), String> {
	db.lock().unwrap().execute("DELETE FROM questions WHERE id = ?1", params![id]).map_err(|e| e.to_string())?;
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::io::Write;

	#[test]
	fn imports_questions_and_options() {
		let dir = std::env::temp_dir().join(format!("nuxtor_cbt_import_test_{}", std::process::id()));
		let db = crate::db::open(&dir);

		let csv_path = dir.join("soal.csv");
		let mut f = File::create(&csv_path).unwrap();
		writeln!(f, "subject,kelas,question_text,question_type,option_a,option_b,option_c,option_d,option_e,correct_option,score").unwrap();
		writeln!(f, "Uji Coba A,4,2+2=?,,3,4,5,,,B,1").unwrap();
		writeln!(f, "Uji Coba A,4,3+3=?,,5,6,7,,,B,1").unwrap();
		writeln!(f, "Uji Coba B,5,Air mendidih pada berapa derajat celcius?,,90,100,110,,,B,2").unwrap();
		drop(f);

		let opts = ImportOptions {
			default_subject_id: None,
			default_class: None,
			default_jenis: None,
			question_type: "multiple_choice".to_string(),
			image_map: Default::default()
		};
		let summary = import_questions(&db, csv_path.to_str().unwrap(), opts).unwrap();
		assert_eq!(summary.questions_imported, 3);
		assert_eq!(summary.subjects_created, 2);

		let conn = db.lock().unwrap();
		let option_count: i64 = conn.query_row("SELECT COUNT(*) FROM question_options", [], |r| r.get(0)).unwrap();
		assert_eq!(option_count, 9);
		let class: String = conn.query_row("SELECT class FROM questions WHERE question_text = '2+2=?'", [], |r| r.get(0)).unwrap();
		assert_eq!(class, "4");

		drop(conn);
		drop(db);
		let _ = std::fs::remove_dir_all(&dir);
	}

	#[test]
	fn imports_questions_with_jenis_column_and_default() {
		let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
		let dir = std::env::temp_dir().join(format!("nuxtor_cbt_import_jenis_test_{}_{nanos}", std::process::id()));
		let db = crate::db::open(&dir);

		let csv_path = dir.join("soal.csv");
		let mut f = File::create(&csv_path).unwrap();
		writeln!(f, "subject,kelas,jenis,question_text,option_a,option_b,correct_option,score").unwrap();
		writeln!(f, "Uji Coba A,4,UTS Ganjil,2+2=?,3,4,B,1").unwrap();
		writeln!(f, "Uji Coba A,4,,3+3=?,5,6,B,1").unwrap();
		drop(f);

		let opts = ImportOptions {
			default_subject_id: None,
			default_class: None,
			default_jenis: Some("Latihan".to_string()),
			question_type: "multiple_choice".to_string(),
			image_map: Default::default()
		};
		import_questions(&db, csv_path.to_str().unwrap(), opts).unwrap();

		let conn = db.lock().unwrap();
		let jenis_from_col: String =
			conn.query_row("SELECT jenis FROM questions WHERE question_text = '2+2=?'", [], |r| r.get(0)).unwrap();
		assert_eq!(jenis_from_col, "UTS Ganjil");
		let jenis_from_default: String =
			conn.query_row("SELECT jenis FROM questions WHERE question_text = '3+3=?'", [], |r| r.get(0)).unwrap();
		assert_eq!(jenis_from_default, "Latihan");

		drop(conn);
		drop(db);
		let _ = std::fs::remove_dir_all(&dir);
	}

	#[test]
	fn resolves_image_case_and_extension_insensitively() {
		let mut map = std::collections::HashMap::new();
		map.insert("Soal1.PNG".to_string(), "uploads/1.png".to_string());

		// exact match still works
		assert_eq!(resolve_image(&map, "Soal1.PNG"), Some("uploads/1.png".to_string()));
		// different case, same extension
		assert_eq!(resolve_image(&map, "soal1.png"), Some("uploads/1.png".to_string()));
		// no extension typed at all, different case
		assert_eq!(resolve_image(&map, "soal1"), Some("uploads/1.png".to_string()));
		// unrelated name doesn't match
		assert_eq!(resolve_image(&map, "soal2"), None);
	}
}
