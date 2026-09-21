use crate::db::Db;
use rusqlite::params;
use serde::Serialize;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

fn now_secs() -> i64 {
	SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionView {
	pub key: String,
	pub text: String
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionView {
	pub id: i64,
	pub question_text: String,
	pub question_type: String,
	pub image: Option<String>,
	pub options: Vec<OptionView>
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JoinResponse {
	pub session_id: i64,
	pub exam_title: String,
	pub duration_seconds: i64,
	pub started_at: i64,
	pub pg_submitted_at: Option<i64>,
	pub submitted_at: Option<i64>,
	pub score: Option<f64>,
	pub questions: Vec<QuestionView>,
	pub answers: HashMap<i64, String>
}

pub fn join(db: &Db, token: &str, nisn: &str, name: &str) -> Result<JoinResponse, String> {
	crate::db::find_student(db, nisn, name).ok_or("NISN atau nama tidak cocok")?;

	let conn = db.lock().unwrap();

	#[allow(clippy::type_complexity)]
	let (exam_id, exam_title, subject_id, class, jenis, duration, window_start, window_end): (
		i64,
		String,
		i64,
		Option<String>,
		Option<String>,
		i64,
		Option<i64>,
		Option<i64>
	) = conn
		.query_row(
			"SELECT id, title, subject_id, class, jenis, duration, scheduled_at, window_end FROM exams WHERE token = ?1",
			params![token.trim()],
			|r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?, r.get(7)?))
		)
		.map_err(|_| "Token ujian tidak ditemukan".to_string())?;

	#[allow(clippy::type_complexity)]
	let existing: Option<(i64, String, i64, Option<i64>, Option<i64>, Option<f64>)> = conn
		.query_row(
			"SELECT id, question_ids, started_at, pg_submitted_at, submitted_at, score
			 FROM exam_sessions WHERE exam_id = ?1 AND student_nisn = ?2",
			params![exam_id, nisn.trim()],
			|r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?))
		)
		.ok();

	let (session_id, question_ids, started_at, pg_submitted_at, submitted_at, score) = if let Some(row) = existing {
		row
	} else {
		// A brand-new attempt must fall inside the exam's access window; an exam left without
		// one (older data) has no window and is always open. Someone already mid-exam can
		// still resume even if the window just closed — see the `existing` branch above. The
		// window only gates WHEN a student can start — see `duration_seconds` below for why
		// it doesn't also cap how long they get once in.
		if let Some(window_start) = window_start {
			let now = now_secs();
			if now < window_start {
				return Err("Ujian belum dimulai".to_string());
			}
			if let Some(window_end) = window_end {
				if now > window_end {
					return Err("Waktu ujian sudah berakhir".to_string());
				}
			}
		}

		// When the exam specifies a class and/or a jenis (question package), only draw
		// questions tagged for that class/jenis; exams left without one (older data, or a
		// jenis-less exam) draw from the whole subject as before — the "?2 IS NULL OR ..."
		// pattern lets one query cover every combination. Multiple-choice questions come
		// before essay ones, so students finish PG before moving on to essay. Every
		// matching question is included — no random subset.
		let mut stmt = conn
			.prepare(
				"SELECT id FROM questions
				 WHERE subject_id = ?1 AND (?2 IS NULL OR class = ?2) AND (?3 IS NULL OR jenis = ?3)
				 ORDER BY CASE WHEN question_type = 'essay' THEN 1 ELSE 0 END, RANDOM()"
			)
			.map_err(|e| e.to_string())?;
		let mapped = stmt.query_map(params![subject_id, class, jenis], |r| r.get(0)).map_err(|e| e.to_string())?;
		let ids: Vec<i64> = mapped.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

		if ids.is_empty() {
			return Err("Belum ada soal untuk mata pelajaran ujian ini".to_string());
		}

		let question_ids = ids.iter().map(i64::to_string).collect::<Vec<_>>().join(",");
		let started_at = now_secs();
		conn.execute(
			"INSERT INTO exam_sessions (exam_id, student_nisn, student_name, question_ids, started_at) VALUES (?1, ?2, ?3, ?4, ?5)",
			params![exam_id, nisn.trim(), name.trim(), question_ids, started_at]
		)
		.map_err(|e| e.to_string())?;
		(conn.last_insert_rowid(), question_ids, started_at, None, None, None)
	};

	let ids: Vec<i64> = question_ids.split(',').filter_map(|s| s.parse().ok()).collect();

	let mut questions = Vec::new();
	for qid in &ids {
		let (question_text, question_type, image): (String, String, Option<String>) = conn
			.query_row("SELECT question_text, question_type, image FROM questions WHERE id = ?1", params![qid], |r| {
				Ok((r.get(0)?, r.get(1)?, r.get(2)?))
			})
			.map_err(|e| e.to_string())?;

		let mut opt_stmt = conn
			.prepare("SELECT option_key, option_text FROM question_options WHERE question_id = ?1 ORDER BY option_key")
			.map_err(|e| e.to_string())?;
		let options = opt_stmt
			.query_map(params![qid], |r| Ok(OptionView { key: r.get(0)?, text: r.get(1)? }))
			.map_err(|e| e.to_string())?
			.collect::<Result<Vec<_>, _>>()
			.map_err(|e| e.to_string())?;

		questions.push(QuestionView { id: *qid, question_text, question_type, image, options });
	}

	let mut answers = HashMap::new();
	let mut ans_stmt = conn
		.prepare("SELECT question_id, option_key FROM exam_answers WHERE session_id = ?1")
		.map_err(|e| e.to_string())?;
	let rows = ans_stmt
		.query_map(params![session_id], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))
		.map_err(|e| e.to_string())?;
	for row in rows {
		let (qid, key) = row.map_err(|e| e.to_string())?;
		answers.insert(qid, key);
	}

	// Every student gets the full flat duration from whenever THEY start, regardless of
	// where in the access window that was — the window and the duration are deliberately
	// two separate settings (see Kelola Ujian).
	let duration_seconds = duration * 60;

	Ok(JoinResponse {
		session_id,
		exam_title,
		duration_seconds,
		started_at,
		pg_submitted_at,
		submitted_at,
		score,
		questions,
		answers
	})
}

pub fn reset(db: &Db, session_id: i64) -> Result<(), String> {
	db.lock()
		.unwrap()
		.execute("DELETE FROM exam_sessions WHERE id = ?1", params![session_id])
		.map_err(|e| e.to_string())?;
	Ok(())
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnswerDetail {
	pub question_id: i64,
	pub question_text: String,
	pub question_type: String,
	pub student_answer: Option<String>,
	pub correct_answer: Option<String>,
	pub is_correct: bool,
	// Only meaningful for essay: teacher-assigned points (None = not graded yet) out of max_score.
	pub essay_score: Option<f64>,
	pub max_score: f64
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionDetail {
	pub student_name: String,
	pub exam_title: String,
	pub class: Option<String>,
	pub subject: String,
	pub score: Option<f64>,
	pub pg_submitted_at: Option<i64>,
	pub submitted_at: Option<i64>,
	pub items: Vec<AnswerDetail>
}

pub fn session_detail(db: &Db, session_id: i64) -> Result<SessionDetail, String> {
	let conn = db.lock().unwrap();

	#[allow(clippy::type_complexity)]
	let (student_name, exam_title, class, subject, question_ids, score, pg_submitted_at, submitted_at): (
		String,
		String,
		Option<String>,
		String,
		String,
		Option<f64>,
		Option<i64>,
		Option<i64>
	) = conn
		.query_row(
			"SELECT es.student_name, e.title, e.class, s.name, es.question_ids, es.score, es.pg_submitted_at, es.submitted_at
			 FROM exam_sessions es
			 JOIN exams e ON e.id = es.exam_id
			 JOIN subjects s ON s.id = e.subject_id
			 WHERE es.id = ?1",
			params![session_id],
			|r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?, r.get(7)?))
		)
		.map_err(|_| "Sesi ujian tidak ditemukan".to_string())?;

	let ids: Vec<i64> = question_ids.split(',').filter_map(|s| s.parse().ok()).collect();

	let mut items = Vec::new();
	for qid in ids {
		let (question_text, question_type, max_score): (String, String, f64) = conn
			.query_row("SELECT question_text, question_type, score FROM questions WHERE id = ?1", params![qid], |r| {
				Ok((r.get(0)?, r.get(1)?, r.get(2)?))
			})
			.map_err(|e| e.to_string())?;
		let correct_answer: Option<String> =
			conn.query_row("SELECT answer_key FROM questions WHERE id = ?1", params![qid], |r| r.get(0)).ok();
		let (student_answer, essay_score): (Option<String>, Option<f64>) = conn
			.query_row(
				"SELECT option_key, essay_score FROM exam_answers WHERE session_id = ?1 AND question_id = ?2",
				params![session_id, qid],
				|r| Ok((r.get(0)?, r.get(1)?))
			)
			.unwrap_or((None, None));

		let is_correct = match (&student_answer, &correct_answer) {
			(Some(g), Some(k)) if question_type != "essay" => g.trim().eq_ignore_ascii_case(k.trim()),
			_ => false
		};

		items.push(AnswerDetail { question_id: qid, question_text, question_type, student_answer, correct_answer, is_correct, essay_score, max_score });
	}

	Ok(SessionDetail { student_name, exam_title, class, subject, score, pg_submitted_at, submitted_at, items })
}

// Teacher-assigned score for one essay answer, clamped to the question's max score. Never
// rolled into `exam_sessions.score` — essay grading stays outside the automatic calculation.
pub fn grade_essay_answer(db: &Db, session_id: i64, question_id: i64, score: f64) -> Result<(), String> {
	let conn = db.lock().unwrap();

	let max_score: f64 = conn
		.query_row("SELECT score FROM questions WHERE id = ?1", params![question_id], |r| r.get(0))
		.map_err(|_| "Soal tidak ditemukan".to_string())?;
	if score < 0.0 || score > max_score {
		return Err(format!("Nilai harus antara 0 dan {max_score}"));
	}

	conn.execute(
		"INSERT INTO exam_answers (session_id, question_id, option_key, essay_score) VALUES (?1, ?2, '', ?3)
		 ON CONFLICT(session_id, question_id) DO UPDATE SET essay_score = excluded.essay_score",
		params![session_id, question_id, score]
	)
	.map_err(|e| e.to_string())?;
	Ok(())
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionProgress {
	pub session_id: i64,
	pub student_name: String,
	pub student_nisn: String,
	pub exam_title: String,
	pub class: Option<String>,
	pub subject: String,
	pub jenis: Option<String>,
	pub total_questions: i64,
	pub answered_count: i64,
	pub pg_total: i64,
	pub pg_answered: i64,
	pub submitted_at: Option<i64>,
	pub score: Option<f64>
}

pub fn list_sessions(db: &Db) -> Result<Vec<SessionProgress>, String> {
	let conn = db.lock().unwrap();
	let mut stmt = conn
		.prepare(
			"SELECT es.id, es.student_name, es.student_nisn, e.title, e.class, s.name, e.jenis, es.question_ids, es.submitted_at, es.score
			 FROM exam_sessions es
			 JOIN exams e ON e.id = es.exam_id
			 JOIN subjects s ON s.id = e.subject_id
			 ORDER BY es.started_at DESC"
		)
		.map_err(|e| e.to_string())?;

	#[allow(clippy::type_complexity)]
	let rows = stmt
		.query_map([], |r| {
			Ok((
				r.get::<_, i64>(0)?,
				r.get::<_, String>(1)?,
				r.get::<_, String>(2)?,
				r.get::<_, String>(3)?,
				r.get::<_, Option<String>>(4)?,
				r.get::<_, String>(5)?,
				r.get::<_, Option<String>>(6)?,
				r.get::<_, String>(7)?,
				r.get::<_, Option<i64>>(8)?,
				r.get::<_, Option<f64>>(9)?
			))
		})
		.map_err(|e| e.to_string())?;

	let mut result = Vec::new();
	for row in rows {
		let (session_id, student_name, student_nisn, exam_title, class, subject, jenis, question_ids, submitted_at, score) =
			row.map_err(|e| e.to_string())?;
		let ids: Vec<i64> = question_ids.split(',').filter_map(|s| s.parse().ok()).collect();
		let total_questions = ids.len() as i64;
		let answered_count: i64 = conn
			.query_row("SELECT COUNT(*) FROM exam_answers WHERE session_id = ?1", params![session_id], |r| r.get(0))
			.map_err(|e| e.to_string())?;

		let mut pg_total = 0i64;
		let mut pg_answered = 0i64;
		for qid in &ids {
			let question_type: String = conn
				.query_row("SELECT question_type FROM questions WHERE id = ?1", params![qid], |r| r.get(0))
				.map_err(|e| e.to_string())?;
			if question_type == "essay" {
				continue;
			}
			pg_total += 1;
			let answered: bool = conn
				.query_row(
					"SELECT 1 FROM exam_answers WHERE session_id = ?1 AND question_id = ?2",
					params![session_id, qid],
					|_| Ok(())
				)
				.is_ok();
			if answered {
				pg_answered += 1;
			}
		}

		result.push(SessionProgress {
			session_id,
			student_name,
			student_nisn,
			exam_title,
			class,
			subject,
			jenis,
			total_questions,
			answered_count,
			pg_total,
			pg_answered,
			submitted_at,
			score
		});
	}

	Ok(result)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionStat {
	pub id: i64,
	pub question_text: String,
	pub correct_count: i64,
	pub incorrect_count: i64
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StudentStat {
	pub session_id: i64,
	pub name: String,
	pub correct_count: i64,
	pub incorrect_count: i64
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsResponse {
	pub question_stats: Vec<QuestionStat>,
	pub student_stats: Vec<StudentStat>
}

// Per-question and per-student PG correctness stats for a kelas + mata pelajaran, used by the
// Hasil Ujian analytics dashboard. Only counts sessions that finished the PG portion (matches
// existing scoring: an unanswered PG question counts as wrong, same as `submit()`). Essay
// questions are excluded entirely — their grading is manual, not something to chart.
pub fn analytics(db: &Db, class: Option<String>, subject_id: i64, jenis: Option<String>) -> Result<AnalyticsResponse, String> {
	let conn = db.lock().unwrap();

	let mut stmt = conn
		.prepare(
			"SELECT es.id, es.student_name, es.question_ids
			 FROM exam_sessions es
			 JOIN exams e ON e.id = es.exam_id
			 WHERE e.subject_id = ?1 AND (?2 IS NULL OR e.class = ?2) AND (?3 IS NULL OR e.jenis = ?3) AND es.pg_submitted_at IS NOT NULL"
		)
		.map_err(|e| e.to_string())?;
	let rows = stmt
		.query_map(params![subject_id, class, jenis], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?)))
		.map_err(|e| e.to_string())?;
	let sessions: Vec<(i64, String, String)> = rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

	let mut question_correct: HashMap<i64, i64> = HashMap::new();
	let mut question_incorrect: HashMap<i64, i64> = HashMap::new();
	let mut question_text_by_id: HashMap<i64, String> = HashMap::new();
	let mut student_stats = Vec::new();

	for (session_id, student_name, question_ids) in &sessions {
		let ids: Vec<i64> = question_ids.split(',').filter_map(|s| s.parse().ok()).collect();
		let mut correct = 0i64;
		let mut incorrect = 0i64;

		for qid in &ids {
			let (question_type, question_text, answer_key): (String, String, Option<String>) = conn
				.query_row("SELECT question_type, question_text, answer_key FROM questions WHERE id = ?1", params![qid], |r| {
					Ok((r.get(0)?, r.get(1)?, r.get(2)?))
				})
				.map_err(|e| e.to_string())?;
			if question_type == "essay" {
				continue;
			}
			question_text_by_id.entry(*qid).or_insert(question_text);

			let given: Option<String> = conn
				.query_row(
					"SELECT option_key FROM exam_answers WHERE session_id = ?1 AND question_id = ?2",
					params![session_id, qid],
					|r| r.get(0)
				)
				.ok();

			let is_correct = matches!((&given, &answer_key), (Some(g), Some(k)) if g.trim().eq_ignore_ascii_case(k.trim()));
			if is_correct {
				correct += 1;
				*question_correct.entry(*qid).or_insert(0) += 1;
			} else {
				incorrect += 1;
				*question_incorrect.entry(*qid).or_insert(0) += 1;
			}
		}

		student_stats.push(StudentStat {
			session_id: *session_id,
			name: student_name.clone(),
			correct_count: correct,
			incorrect_count: incorrect
		});
	}

	let mut question_stats: Vec<QuestionStat> = question_text_by_id
		.into_iter()
		.map(|(id, question_text)| QuestionStat {
			id,
			question_text,
			correct_count: *question_correct.get(&id).unwrap_or(&0),
			incorrect_count: *question_incorrect.get(&id).unwrap_or(&0)
		})
		.collect();
	question_stats.sort_by_key(|q| q.id);

	Ok(AnalyticsResponse { question_stats, student_stats })
}

pub fn answer(db: &Db, session_id: i64, question_id: i64, option_key: &str) -> Result<(), String> {
	let conn = db.lock().unwrap();

	let (pg_submitted_at, submitted_at): (Option<i64>, Option<i64>) = conn
		.query_row("SELECT pg_submitted_at, submitted_at FROM exam_sessions WHERE id = ?1", params![session_id], |r| {
			Ok((r.get(0)?, r.get(1)?))
		})
		.map_err(|_| "Sesi ujian tidak ditemukan".to_string())?;

	if submitted_at.is_some() {
		return Err("Ujian sudah dikirim, jawaban tidak bisa diubah lagi".to_string());
	}

	if pg_submitted_at.is_some() {
		let question_type: String = conn
			.query_row("SELECT question_type FROM questions WHERE id = ?1", params![question_id], |r| r.get(0))
			.map_err(|e| e.to_string())?;
		if question_type != "essay" {
			return Err("Sesi Pilihan Ganda sudah dikirim, jawaban tidak bisa diubah lagi".to_string());
		}
	}

	conn.execute(
		"INSERT INTO exam_answers (session_id, question_id, option_key) VALUES (?1, ?2, ?3)
		 ON CONFLICT(session_id, question_id) DO UPDATE SET option_key = excluded.option_key",
		params![session_id, question_id, option_key]
	)
	.map_err(|e| e.to_string())?;
	Ok(())
}

// Locks the PG answers in place and moves the student on to the essay portion (if the exam
// has any). Idempotent — calling it again just refreshes the timestamp, which is harmless.
pub fn submit_pg(db: &Db, session_id: i64) -> Result<(), String> {
	db.lock()
		.unwrap()
		.execute("UPDATE exam_sessions SET pg_submitted_at = ?1 WHERE id = ?2", params![now_secs(), session_id])
		.map_err(|e| e.to_string())?;
	Ok(())
}

#[derive(Debug, Serialize)]
pub struct SubmitResponse {
	pub score: f64
}

pub fn submit(db: &Db, session_id: i64) -> Result<SubmitResponse, String> {
	let conn = db.lock().unwrap();

	let question_ids: String = conn
		.query_row("SELECT question_ids FROM exam_sessions WHERE id = ?1", params![session_id], |r| r.get(0))
		.map_err(|_| "Sesi ujian tidak ditemukan".to_string())?;

	let ids: Vec<i64> = question_ids.split(',').filter_map(|s| s.parse().ok()).collect();

	let mut total_score = 0.0;
	let mut earned_score = 0.0;

	for qid in &ids {
		let (question_type, answer_key, score): (String, Option<String>, f64) = conn
			.query_row("SELECT question_type, answer_key, score FROM questions WHERE id = ?1", params![qid], |r| {
				Ok((r.get(0)?, r.get(1)?, r.get(2)?))
			})
			.map_err(|e| e.to_string())?;

		// Essay answers need a human to grade them; they don't count toward the auto score.
		if question_type == "essay" {
			continue;
		}

		total_score += score;

		let given: Option<String> = conn
			.query_row(
				"SELECT option_key FROM exam_answers WHERE session_id = ?1 AND question_id = ?2",
				params![session_id, qid],
				|r| r.get(0)
			)
			.ok();

		if let (Some(given), Some(key)) = (given, answer_key) {
			if given.trim().eq_ignore_ascii_case(key.trim()) {
				earned_score += score;
			}
		}
	}

	let final_score = if total_score > 0.0 { (earned_score / total_score) * 100.0 } else { 0.0 };

	conn.execute(
		"UPDATE exam_sessions SET submitted_at = ?1, score = ?2 WHERE id = ?3",
		params![now_secs(), final_score, session_id]
	)
	.map_err(|e| e.to_string())?;

	Ok(SubmitResponse { score: final_score })
}

#[cfg(test)]
mod tests {
	use super::*;
	use rusqlite::params as p;

	fn setup() -> (Db, std::path::PathBuf) {
		let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
		let dir = std::env::temp_dir().join(format!("nuxtor_cbt_exam_session_test_{}_{nanos}", std::process::id()));
		let db = crate::db::open(&dir);
		let conn = db.lock().unwrap();
		conn.execute("INSERT INTO students (nisn, name) VALUES ('111', 'Test Siswa')", []).unwrap();
		conn.execute("INSERT INTO subjects (name) VALUES ('Test Subject')", []).unwrap();
		let subject_id = conn.last_insert_rowid();
		conn.execute(
			"INSERT INTO questions (subject_id, question_text, answer_key, score) VALUES (?1, 'Soal 1', 'A', 1)",
			p![subject_id]
		)
		.unwrap();
		let q1 = conn.last_insert_rowid();
		conn.execute("INSERT INTO question_options (question_id, option_key, option_text) VALUES (?1, 'A', 'Benar')", p![q1]).unwrap();
		conn.execute("INSERT INTO question_options (question_id, option_key, option_text) VALUES (?1, 'B', 'Salah')", p![q1]).unwrap();
		conn.execute(
			"INSERT INTO exams (subject_id, title, duration, question_count, token) VALUES (?1, 'Ujian Test', 30, 1, 'TOK123')",
			p![subject_id]
		)
		.unwrap();
		drop(conn);
		(db, dir)
	}

	#[test]
	fn join_answer_submit_flow() {
		let (db, dir) = setup();

		let joined = join(&db, "TOK123", "111", "Test Siswa").unwrap();
		assert_eq!(joined.questions.len(), 1);
		let qid = joined.questions[0].id;

		answer(&db, joined.session_id, qid, "A").unwrap();

		let result = submit(&db, joined.session_id).unwrap();
		assert_eq!(result.score, 100.0);

		drop(db);
		let _ = std::fs::remove_dir_all(&dir);
	}

	#[test]
	fn rejects_join_outside_scheduled_window() {
		let (db, dir) = setup();
		let now = now_secs();

		{
			let conn = db.lock().unwrap();
			conn.execute("UPDATE exams SET scheduled_at = ?1 WHERE token = 'TOK123'", p![now + 3600]).unwrap();
		}
		let err = join(&db, "TOK123", "111", "Test Siswa").unwrap_err();
		assert_eq!(err, "Ujian belum dimulai");

		{
			let conn = db.lock().unwrap();
			conn.execute(
				"UPDATE exams SET scheduled_at = ?1, window_end = ?2 WHERE token = 'TOK123'",
				p![now - 7200, now - 3600]
			)
			.unwrap();
		}
		let err = join(&db, "TOK123", "111", "Test Siswa").unwrap_err();
		assert_eq!(err, "Waktu ujian sudah berakhir");

		drop(db);
		let _ = std::fs::remove_dir_all(&dir);
	}

	#[test]
	fn full_duration_regardless_of_when_in_window_student_joins() {
		let (db, dir) = setup();
		let now = now_secs();

		{
			let conn = db.lock().unwrap();
			// Window is open for 2 hours; duration (set in `setup`) is 30 minutes. Joining
			// halfway into the window should still grant the full 30 minutes, not whatever
			// is left until window_end.
			conn.execute(
				"UPDATE exams SET scheduled_at = ?1, window_end = ?2 WHERE token = 'TOK123'",
				p![now - 3600, now + 3600]
			)
			.unwrap();
		}

		let joined = join(&db, "TOK123", "111", "Test Siswa").unwrap();
		assert_eq!(joined.duration_seconds, 30 * 60);

		drop(db);
		let _ = std::fs::remove_dir_all(&dir);
	}

	#[test]
	fn pg_session_locks_and_essay_can_be_graded_separately() {
		let (db, dir) = setup();

		// Add an essay question alongside the existing PG one so this exam has both.
		let subject_id: i64 = db.lock().unwrap().query_row("SELECT subject_id FROM exams WHERE token = 'TOK123'", [], |r| r.get(0)).unwrap();
		db.lock()
			.unwrap()
			.execute(
				"INSERT INTO questions (subject_id, question_text, question_type, score) VALUES (?1, 'Jelaskan sesuatu', 'essay', 5)",
				p![subject_id]
			)
			.unwrap();

		let joined = join(&db, "TOK123", "111", "Test Siswa").unwrap();
		assert_eq!(joined.questions.len(), 2);
		let pg_qid = joined.questions.iter().find(|q| q.question_type != "essay").unwrap().id;
		let essay_qid = joined.questions.iter().find(|q| q.question_type == "essay").unwrap().id;

		answer(&db, joined.session_id, pg_qid, "A").unwrap();
		submit_pg(&db, joined.session_id).unwrap();

		// PG is locked now — changing it should fail, but the essay answer still goes through.
		let err = answer(&db, joined.session_id, pg_qid, "B").unwrap_err();
		assert_eq!(err, "Sesi Pilihan Ganda sudah dikirim, jawaban tidak bisa diubah lagi");
		answer(&db, joined.session_id, essay_qid, "Jawaban esai saya").unwrap();

		grade_essay_answer(&db, joined.session_id, essay_qid, 4.0).unwrap();

		let detail = session_detail(&db, joined.session_id).unwrap();
		let essay_item = detail.items.iter().find(|i| i.question_id == essay_qid).unwrap();
		assert_eq!(essay_item.essay_score, Some(4.0));
		assert_eq!(essay_item.student_answer.as_deref(), Some("Jawaban esai saya"));

		// Essay score must never leak into the automatic score.
		let result = submit(&db, joined.session_id).unwrap();
		assert_eq!(result.score, 100.0);

		drop(db);
		let _ = std::fs::remove_dir_all(&dir);
	}

	#[test]
	fn analytics_counts_pg_correctness_after_pg_is_submitted() {
		let (db, dir) = setup();
		let subject_id: i64 = db.lock().unwrap().query_row("SELECT subject_id FROM exams WHERE token = 'TOK123'", [], |r| r.get(0)).unwrap();

		let joined = join(&db, "TOK123", "111", "Test Siswa").unwrap();
		let qid = joined.questions[0].id;

		// Before the PG session is submitted, this session shouldn't count yet.
		let early = analytics(&db, None, subject_id, None).unwrap();
		assert!(early.student_stats.is_empty());

		answer(&db, joined.session_id, qid, "A").unwrap();
		submit_pg(&db, joined.session_id).unwrap();

		let stats = analytics(&db, None, subject_id, None).unwrap();
		assert_eq!(stats.question_stats.len(), 1);
		assert_eq!(stats.question_stats[0].correct_count, 1);
		assert_eq!(stats.question_stats[0].incorrect_count, 0);
		assert_eq!(stats.student_stats.len(), 1);
		assert_eq!(stats.student_stats[0].name, "Test Siswa");
		assert_eq!(stats.student_stats[0].correct_count, 1);

		drop(db);
		let _ = std::fs::remove_dir_all(&dir);
	}
}
