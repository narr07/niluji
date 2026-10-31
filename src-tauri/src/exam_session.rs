use crate::db::Db;
use rusqlite::{params, OptionalExtension};
use serde::Serialize;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

fn now_secs() -> i64 {
	SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64
}

// Deterministic hash for shuffling options per (session_id, question_id, option_key)
fn shuffle_weight(session_id: i64, question_id: i64, key: &str) -> u64 {
	let mut h = (session_id as u64)
		.wrapping_mul(0x9E37_79B9_7F4A_7C15)
		.wrapping_add((question_id as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9));
	for b in key.bytes() {
		h ^= b as u64;
		h = h.wrapping_mul(0x94D0_49BB_1331_11EB);
		h ^= h >> 29;
	}
	h
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionView {
	pub key: String,
	pub text: String,
	pub image: Option<String>
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
	pub remaining_seconds: i64,
	pub started_at: i64,
	pub pg_submitted_at: Option<i64>,
	pub submitted_at: Option<i64>,
	pub score: Option<f64>,
	pub device_token: String,
	pub questions: Vec<QuestionView>,
	pub answers: HashMap<i64, String>,
	pub doubtful: HashMap<i64, bool>
}

pub fn join(
	db: &Db,
	token: &str,
	nisn: &str,
	name: &str,
	pin: Option<&str>,
	device_token: Option<&str>
) -> Result<JoinResponse, String> {
	crate::db::find_student(db, nisn, name).ok_or("NISN atau nama tidak cocok")?;

	let conn = db.lock().unwrap();

	#[allow(clippy::type_complexity)]
	let (
		exam_id,
		exam_title,
		subject_id,
		class,
		jenis,
		duration,
		window_start,
		window_end,
		randomize_pg,
		randomize_essay,
		randomize_options,
		require_pin
	): (
		i64,
		String,
		i64,
		Option<String>,
		Option<String>,
		i64,
		Option<i64>,
		Option<i64>,
		bool,
		bool,
		bool,
		bool
	) = conn
		.query_row(
			"SELECT id, title, subject_id, class, jenis, duration, scheduled_at, window_end,
			        randomize_pg, randomize_essay, randomize_options, require_pin
			 FROM exams WHERE token = ?1 COLLATE NOCASE",
			params![token.trim()],
			|r| {
				Ok((
					r.get(0)?,
					r.get(1)?,
					r.get(2)?,
					r.get(3)?,
					r.get(4)?,
					r.get(5)?,
					r.get(6)?,
					r.get(7)?,
					r.get::<_, Option<i64>>(8)?.unwrap_or(1) != 0,
					r.get::<_, Option<i64>>(9)?.unwrap_or(1) != 0,
					r.get::<_, Option<i64>>(10)?.unwrap_or(1) != 0,
					r.get::<_, Option<i64>>(11)?.unwrap_or(0) != 0
				))
			}
		)
		.map_err(|_| "Token ujian tidak ditemukan".to_string())?;

	if require_pin {
		let stored_pin: String = conn
			.query_row("SELECT COALESCE(pin, '') FROM students WHERE nisn = ?1", params![nisn.trim()], |r| r.get(0))
			.unwrap_or_default();
		let supplied = pin.unwrap_or("").trim();
		if !stored_pin.trim().is_empty() && !supplied.eq_ignore_ascii_case(stored_pin.trim()) {
			return Err("PIN Kartu Peserta salah atau belum diisi.".to_string());
		}
	}

	let client_dev_token = device_token
		.map(str::trim)
		.filter(|s| !s.is_empty())
		.map(String::from)
		.unwrap_or_else(|| format!("dev-{}-{}", now_secs(), crate::db::generate_random_pin(77)));

	#[allow(clippy::type_complexity)]
	let existing: Option<(i64, String, i64, Option<i64>, Option<i64>, Option<f64>, i64, Option<String>)> = conn
		.query_row(
			"SELECT id, question_ids, started_at, pg_submitted_at, submitted_at, score, COALESCE(extra_minutes, 0), device_token
			 FROM exam_sessions WHERE exam_id = ?1 AND student_nisn = ?2",
			params![exam_id, nisn.trim()],
			|r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?, r.get(7)?))
		)
		.ok();

	let (session_id, question_ids, started_at, pg_submitted_at, submitted_at, score, extra_minutes, active_dev_token) =
		if let Some((sid, qids, started, pg_sub, sub, sc, extra, stored_dev)) = existing {
			// Enforce single-device lock while the session is still in progress.
			if sub.is_none() {
				if let Some(ref locked_token) = stored_dev {
					if !locked_token.trim().is_empty() && locked_token != &client_dev_token {
						return Err(
							"Sesi ujian Anda sedang aktif di perangkat lain. Hubungi pengawas untuk membuka kunci perangkat jika Anda berganti HP/laptop."
								.to_string()
						);
					}
				}
				// If unlocked (device_token is NULL or empty), bind to this device now.
				if stored_dev.as_deref().unwrap_or("").trim().is_empty() {
					let _ = conn.execute(
						"UPDATE exam_sessions SET device_token = ?1 WHERE id = ?2",
						params![&client_dev_token, sid]
					);
				}
			}
			let effective_token = stored_dev.filter(|s| !s.trim().is_empty()).unwrap_or(client_dev_token);
			(sid, qids, started, pg_sub, sub, sc, extra, effective_token)
		} else {
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

			let mut stmt = conn
				.prepare(
					"SELECT id FROM questions
					 WHERE subject_id = ?1 AND (?2 IS NULL OR class = ?2) AND (?3 IS NULL OR jenis = ?3)
					 ORDER BY
					 	CASE WHEN question_type = 'essay' THEN 1 ELSE 0 END,
					 	CASE
					 		WHEN question_type = 'essay' THEN (CASE WHEN ?5 = 1 THEN RANDOM() ELSE id END)
					 		ELSE (CASE WHEN ?4 = 1 THEN RANDOM() ELSE id END)
					 	END"
				)
				.map_err(|e| e.to_string())?;
			let mapped = stmt
				.query_map(params![subject_id, class, jenis, randomize_pg as i64, randomize_essay as i64], |r| r.get(0))
				.map_err(|e| e.to_string())?;
			let ids: Vec<i64> = mapped.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

			if ids.is_empty() {
				return Err("Belum ada soal untuk mata pelajaran ujian ini".to_string());
			}

			let question_ids = ids.iter().map(i64::to_string).collect::<Vec<_>>().join(",");
			let started_at = now_secs();
			conn.execute(
				"INSERT INTO exam_sessions (exam_id, student_nisn, student_name, question_ids, started_at, extra_minutes, device_token)
				 VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6)",
				params![exam_id, nisn.trim(), name.trim(), question_ids, started_at, &client_dev_token]
			)
			.map_err(|e| e.to_string())?;
			(conn.last_insert_rowid(), question_ids, started_at, None, None, None, 0i64, client_dev_token)
		};

	let ids: Vec<i64> = question_ids.split(',').filter_map(|s| s.parse().ok()).collect();

	let mut questions = Vec::new();
	for qid in &ids {
		let question_row: Option<(String, String, Option<String>)> = conn
			.query_row("SELECT question_text, question_type, image FROM questions WHERE id = ?1", params![qid], |r| {
				Ok((r.get(0)?, r.get(1)?, r.get(2)?))
			})
			.optional()
			.map_err(|e| e.to_string())?;
		let Some((question_text, question_type, image)) = question_row else {
			continue;
		};

		let mut opt_stmt = conn
			.prepare("SELECT option_key, option_text, image FROM question_options WHERE question_id = ?1 ORDER BY option_key")
			.map_err(|e| e.to_string())?;
		let mut options = opt_stmt
			.query_map(params![qid], |r| Ok(OptionView { key: r.get(0)?, text: r.get(1)?, image: r.get(2)? }))
			.map_err(|e| e.to_string())?
			.collect::<Result<Vec<_>, _>>()
			.map_err(|e| e.to_string())?;

		if randomize_options && question_type != "essay" && options.len() > 1 {
			options.sort_by_key(|opt| shuffle_weight(session_id, *qid, &opt.key));
		}

		questions.push(QuestionView { id: *qid, question_text, question_type, image, options });
	}

	let mut answers = HashMap::new();
	let mut doubtful = HashMap::new();
	let mut ans_stmt = conn
		.prepare("SELECT question_id, option_key, COALESCE(is_doubtful, 0) FROM exam_answers WHERE session_id = ?1")
		.map_err(|e| e.to_string())?;
	let rows = ans_stmt
		.query_map(params![session_id], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, i64>(2)? != 0)))
		.map_err(|e| e.to_string())?;
	for row in rows {
		let (qid, key, is_doubt) = row.map_err(|e| e.to_string())?;
		if !key.is_empty() {
			answers.insert(qid, key);
		}
		if is_doubt {
			doubtful.insert(qid, true);
		}
	}

	let duration_seconds = (duration + extra_minutes) * 60;
	let elapsed = (now_secs() - started_at).max(0);
	let remaining_seconds = (duration_seconds - elapsed).max(0);

	Ok(JoinResponse {
		session_id,
		exam_title,
		duration_seconds,
		remaining_seconds,
		started_at,
		pg_submitted_at,
		submitted_at,
		score,
		device_token: active_dev_token,
		questions,
		answers,
		doubtful
	})
}

pub fn reset(db: &Db, session_id: i64) -> Result<(), String> {
	db.lock()
		.unwrap()
		.execute("DELETE FROM exam_sessions WHERE id = ?1", params![session_id])
		.map_err(|e| e.to_string())?;
	Ok(())
}

pub fn unlock_device(db: &Db, session_id: i64) -> Result<(), String> {
	db.lock()
		.unwrap()
		.execute("UPDATE exam_sessions SET device_token = NULL WHERE id = ?1", params![session_id])
		.map_err(|e| e.to_string())?;
	Ok(())
}

pub fn add_extra_time(db: &Db, session_id: i64, minutes: i64) -> Result<i64, String> {
	let conn = db.lock().unwrap();
	conn.execute(
		"UPDATE exam_sessions SET extra_minutes = COALESCE(extra_minutes, 0) + ?1 WHERE id = ?2",
		params![minutes, session_id]
	)
	.map_err(|e| e.to_string())?;
	let updated: i64 = conn
		.query_row("SELECT COALESCE(extra_minutes, 0) FROM exam_sessions WHERE id = ?1", params![session_id], |r| r.get(0))
		.map_err(|e| e.to_string())?;
	Ok(updated)
}

pub fn force_submit(db: &Db, session_id: i64) -> Result<SubmitResponse, String> {
	let _ = submit_pg(db, session_id);
	submit(db, session_id)
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
	pub essay_score: Option<f64>,
	pub final_score: Option<f64>,
	pub pg_weight: f64,
	pub pg_submitted_at: Option<i64>,
	pub submitted_at: Option<i64>,
	pub items: Vec<AnswerDetail>
}

// Compute combined final score given PG score (0..100), essay earned/max, and pg_weight (0..100).
fn compute_combined_scores(
	pg_score: Option<f64>,
	has_pg: bool,
	essay_earned: f64,
	essay_max: f64,
	essay_graded_count: usize,
	pg_weight: f64
) -> (Option<f64>, Option<f64>) {
	let essay_pct = if essay_max > 0.0 && essay_graded_count > 0 {
		Some((essay_earned / essay_max) * 100.0)
	} else {
		None
	};

	let final_score = match (has_pg, essay_max > 0.0) {
		(true, false) => pg_score,
		(false, true) => essay_pct,
		(true, true) => {
			let pg = pg_score.unwrap_or(0.0);
			let es = essay_pct.unwrap_or(0.0);
			// If teacher explicitly configured pg_weight < 100, use weighted average;
			// if pg_weight is 100 (default), still blend 70% PG + 30% Essay once essays are graded,
			// or use configured weight when < 100.
			let w_pg = if pg_weight < 100.0 { (pg_weight / 100.0).clamp(0.0, 1.0) } else { 0.7 };
			if pg_score.is_some() || essay_pct.is_some() {
				Some(pg * w_pg + es * (1.0 - w_pg))
			} else {
				None
			}
		}
		(false, false) => None
	};

	(essay_pct, final_score)
}

pub fn session_detail(db: &Db, session_id: i64) -> Result<SessionDetail, String> {
	let conn = db.lock().unwrap();

	#[allow(clippy::type_complexity)]
	let (student_name, exam_title, class, subject, question_ids, score, pg_submitted_at, submitted_at, pg_weight): (
		String,
		String,
		Option<String>,
		String,
		String,
		Option<f64>,
		Option<i64>,
		Option<i64>,
		f64
	) = conn
		.query_row(
			"SELECT es.student_name, e.title, e.class, s.name, es.question_ids, es.score, es.pg_submitted_at, es.submitted_at, COALESCE(e.pg_weight, 100.0)
			 FROM exam_sessions es
			 JOIN exams e ON e.id = es.exam_id
			 JOIN subjects s ON s.id = e.subject_id
			 WHERE es.id = ?1",
			params![session_id],
			|r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?, r.get(7)?, r.get(8)?))
		)
		.map_err(|_| "Sesi ujian tidak ditemukan".to_string())?;

	let mut ids: Vec<i64> = question_ids.split(',').filter_map(|s| s.parse().ok()).collect();
	ids.sort_unstable();

	let mut items = Vec::new();
	let mut has_pg = false;
	let mut essay_earned = 0.0;
	let mut essay_max = 0.0;
	let mut essay_graded_count = 0usize;

	for qid in ids {
		let question_row: Option<(String, String, f64)> = conn
			.query_row("SELECT question_text, question_type, score FROM questions WHERE id = ?1", params![qid], |r| {
				Ok((r.get(0)?, r.get(1)?, r.get(2)?))
			})
			.optional()
			.map_err(|e| e.to_string())?;
		let Some((question_text, question_type, max_score)) = question_row else {
			continue;
		};
		let correct_answer: Option<String> =
			conn.query_row("SELECT answer_key FROM questions WHERE id = ?1", params![qid], |r| r.get(0)).ok();
		let (student_answer, essay_score): (Option<String>, Option<f64>) = conn
			.query_row(
				"SELECT option_key, essay_score FROM exam_answers WHERE session_id = ?1 AND question_id = ?2",
				params![session_id, qid],
				|r| Ok((r.get(0)?, r.get(1)?))
			)
			.unwrap_or((None, None));

		if question_type == "essay" {
			essay_max += max_score;
			if let Some(es) = essay_score {
				essay_earned += es;
				essay_graded_count += 1;
			}
		} else {
			has_pg = true;
		}

		let is_correct = match (&student_answer, &correct_answer) {
			(Some(g), Some(k)) if question_type != "essay" => g.trim().eq_ignore_ascii_case(k.trim()),
			_ => false
		};

		items.push(AnswerDetail {
			question_id: qid,
			question_text,
			question_type,
			student_answer: student_answer.filter(|s| !s.is_empty()),
			correct_answer,
			is_correct,
			essay_score,
			max_score
		});
	}

	let (essay_score_pct, final_score) =
		compute_combined_scores(score, has_pg, essay_earned, essay_max, essay_graded_count, pg_weight);

	Ok(SessionDetail {
		student_name,
		exam_title,
		class,
		subject,
		score,
		essay_score: essay_score_pct,
		final_score,
		pg_weight,
		pg_submitted_at,
		submitted_at,
		items
	})
}

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
	pub score: Option<f64>,
	pub essay_score: Option<f64>,
	pub final_score: Option<f64>,
	pub extra_minutes: i64,
	pub device_locked: bool
}

pub fn list_sessions(db: &Db) -> Result<Vec<SessionProgress>, String> {
	let conn = db.lock().unwrap();
	let mut stmt = conn
		.prepare(
			"SELECT es.id, es.student_name, es.student_nisn, e.title, e.class, s.name, e.jenis, es.question_ids,
			        es.submitted_at, es.score, COALESCE(es.extra_minutes, 0), es.device_token, COALESCE(e.pg_weight, 100.0)
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
				r.get::<_, Option<f64>>(9)?,
				r.get::<_, i64>(10)?,
				r.get::<_, Option<String>>(11)?,
				r.get::<_, f64>(12)?
			))
		})
		.map_err(|e| e.to_string())?;

	let mut result = Vec::new();
	for row in rows {
		let (
			session_id,
			student_name,
			student_nisn,
			exam_title,
			class,
			subject,
			jenis,
			question_ids,
			submitted_at,
			score,
			extra_minutes,
			device_token,
			pg_weight
		) = row.map_err(|e| e.to_string())?;
		let ids: Vec<i64> = question_ids.split(',').filter_map(|s| s.parse().ok()).collect();
		let total_questions = ids.len() as i64;
		let answered_count: i64 = conn
			.query_row(
				"SELECT COUNT(*) FROM exam_answers WHERE session_id = ?1 AND TRIM(option_key) != ''",
				params![session_id],
				|r| r.get(0)
			)
			.map_err(|e| e.to_string())?;

		let mut pg_total = 0i64;
		let mut pg_answered = 0i64;
		let mut essay_earned = 0.0;
		let mut essay_max = 0.0;
		let mut essay_graded_count = 0usize;

		for qid in &ids {
			let q_info: Option<(String, f64)> = conn
				.query_row("SELECT question_type, score FROM questions WHERE id = ?1", params![qid], |r| Ok((r.get(0)?, r.get(1)?)))
				.optional()
				.map_err(|e| e.to_string())?;
			let Some((question_type, q_score)) = q_info else {
				continue;
			};
			if question_type == "essay" {
				essay_max += q_score;
				if let Ok(Some(es)) = conn.query_row(
					"SELECT essay_score FROM exam_answers WHERE session_id = ?1 AND question_id = ?2",
					params![session_id, qid],
					|r| r.get::<_, Option<f64>>(0)
				) {
					essay_earned += es;
					essay_graded_count += 1;
				}
				continue;
			}
			pg_total += 1;
			let answered: bool = conn
				.query_row(
					"SELECT 1 FROM exam_answers WHERE session_id = ?1 AND question_id = ?2 AND TRIM(option_key) != ''",
					params![session_id, qid],
					|_| Ok(())
				)
				.is_ok();
			if answered {
				pg_answered += 1;
			}
		}

		let (essay_score, final_score) =
			compute_combined_scores(score, pg_total > 0, essay_earned, essay_max, essay_graded_count, pg_weight);

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
			score,
			essay_score,
			final_score,
			extra_minutes,
			device_locked: device_token.as_deref().is_some_and(|t| !t.trim().is_empty())
		});
	}

	Ok(result)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionStat {
	pub id: i64,
	pub question_text: String,
	pub correct_option: String,
	pub correct_count: i64,
	pub incorrect_count: i64,
	pub option_counts: HashMap<String, i64>
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
	let mut question_meta_by_id: HashMap<i64, (String, String)> = HashMap::new();
	let mut question_option_counts: HashMap<i64, HashMap<String, i64>> = HashMap::new();
	let mut student_stats = Vec::new();

	for (session_id, student_name, question_ids) in &sessions {
		let ids: Vec<i64> = question_ids.split(',').filter_map(|s| s.parse().ok()).collect();
		let mut correct = 0i64;
		let mut incorrect = 0i64;

		for qid in &ids {
			let row: Option<(String, String, Option<String>)> = conn
				.query_row("SELECT question_type, question_text, answer_key FROM questions WHERE id = ?1", params![qid], |r| {
					Ok((r.get(0)?, r.get(1)?, r.get(2)?))
				})
				.optional()
				.map_err(|e| e.to_string())?;
			let Some((question_type, question_text, answer_key)) = row else {
				continue;
			};
			if question_type == "essay" {
				continue;
			}
			let key_str = answer_key.clone().unwrap_or_default().trim().to_uppercase();
			question_meta_by_id.entry(*qid).or_insert((question_text, key_str));

			let given: Option<String> = conn
				.query_row(
					"SELECT option_key FROM exam_answers WHERE session_id = ?1 AND question_id = ?2",
					params![session_id, qid],
					|r| r.get(0)
				)
				.ok();

			let picked_bucket = given
				.as_deref()
				.map(str::trim)
				.filter(|s| !s.is_empty())
				.map(str::to_uppercase)
				.unwrap_or_else(|| "KOSONG".to_string());
			*question_option_counts.entry(*qid).or_default().entry(picked_bucket).or_insert(0) += 1;

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

	let mut question_stats: Vec<QuestionStat> = question_meta_by_id
		.into_iter()
		.map(|(id, (question_text, correct_option))| QuestionStat {
			id,
			question_text,
			correct_option,
			correct_count: *question_correct.get(&id).unwrap_or(&0),
			incorrect_count: *question_incorrect.get(&id).unwrap_or(&0),
			option_counts: question_option_counts.remove(&id).unwrap_or_default()
		})
		.collect();
	question_stats.sort_by_key(|q| q.id);

	Ok(AnalyticsResponse { question_stats, student_stats })
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnswerResponse {
	pub remaining_seconds: i64
}

pub fn answer(
	db: &Db,
	session_id: i64,
	question_id: i64,
	option_key: &str,
	is_doubtful: Option<bool>
) -> Result<AnswerResponse, String> {
	let conn = db.lock().unwrap();

	let (pg_submitted_at, submitted_at, started_at, extra_minutes, duration): (Option<i64>, Option<i64>, i64, i64, i64) = conn
		.query_row(
			"SELECT es.pg_submitted_at, es.submitted_at, es.started_at, COALESCE(es.extra_minutes, 0), e.duration
			 FROM exam_sessions es
			 JOIN exams e ON e.id = es.exam_id
			 WHERE es.id = ?1",
			params![session_id],
			|r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
		)
		.map_err(|_| "Sesi ujian tidak ditemukan".to_string())?;

	if submitted_at.is_some() {
		return Err("Ujian sudah dikirim, jawaban tidak bisa diubah lagi".to_string());
	}

	let duration_seconds = (duration + extra_minutes) * 60;
	let elapsed = (now_secs() - started_at).max(0);
	let remaining_seconds = (duration_seconds - elapsed).max(0);
	// Allow a 60-second grace period so last-second auto-retries from weak Wi-Fi still land cleanly.
	if elapsed > duration_seconds + 60 {
		return Err("Waktu pengerjaan ujian telah habis".to_string());
	}

	if pg_submitted_at.is_some() {
		let question_type: Option<String> = conn
			.query_row("SELECT question_type FROM questions WHERE id = ?1", params![question_id], |r| r.get(0))
			.optional()
			.map_err(|e| e.to_string())?;
		if question_type.as_deref() != Some("essay") {
			return Err("Sesi Pilihan Ganda sudah dikirim, jawaban tidak bisa diubah lagi".to_string());
		}
	}

	if let Some(doubt) = is_doubtful {
		conn.execute(
			"INSERT INTO exam_answers (session_id, question_id, option_key, is_doubtful) VALUES (?1, ?2, ?3, ?4)
			 ON CONFLICT(session_id, question_id) DO UPDATE SET option_key = excluded.option_key, is_doubtful = excluded.is_doubtful",
			params![session_id, question_id, option_key, doubt as i64]
		)
		.map_err(|e| e.to_string())?;
	} else {
		conn.execute(
			"INSERT INTO exam_answers (session_id, question_id, option_key) VALUES (?1, ?2, ?3)
			 ON CONFLICT(session_id, question_id) DO UPDATE SET option_key = excluded.option_key",
			params![session_id, question_id, option_key]
		)
		.map_err(|e| e.to_string())?;
	}

	Ok(AnswerResponse { remaining_seconds })
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
		let question_row: Option<(String, Option<String>, f64)> = conn
			.query_row("SELECT question_type, answer_key, score FROM questions WHERE id = ?1", params![qid], |r| {
				Ok((r.get(0)?, r.get(1)?, r.get(2)?))
			})
			.optional()
			.map_err(|e| e.to_string())?;
		let Some((question_type, answer_key, score)) = question_row else {
			continue;
		};

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
		conn.execute("INSERT INTO students (nisn, name, pin) VALUES ('111', 'Test Siswa', '654321')", []).unwrap();
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

		let joined = join(&db, "TOK123", "111", "Test Siswa", None, Some("dev-1")).unwrap();
		assert_eq!(joined.questions.len(), 1);
		let qid = joined.questions[0].id;

		answer(&db, joined.session_id, qid, "A", Some(true)).unwrap();

		let result = submit(&db, joined.session_id).unwrap();
		assert_eq!(result.score, 100.0);

		drop(db);
		let _ = std::fs::remove_dir_all(&dir);
	}

	#[test]
	fn single_device_lock_and_proctor_unlock() {
		let (db, dir) = setup();

		let joined = join(&db, "TOK123", "111", "Test Siswa", None, Some("device-A")).unwrap();
		assert_eq!(joined.device_token, "device-A");

		// Second device with different token must be blocked while session is open.
		let err = join(&db, "TOK123", "111", "Test Siswa", None, Some("device-B")).unwrap_err();
		assert!(err.contains("perangkat lain"));

		// Proctor unlocks the device, allowing device-B to take over.
		unlock_device(&db, joined.session_id).unwrap();
		let rejoined = join(&db, "TOK123", "111", "Test Siswa", None, Some("device-B")).unwrap();
		assert_eq!(rejoined.device_token, "device-B");

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
		let err = join(&db, "TOK123", "111", "Test Siswa", None, None).unwrap_err();
		assert_eq!(err, "Ujian belum dimulai");

		{
			let conn = db.lock().unwrap();
			conn.execute(
				"UPDATE exams SET scheduled_at = ?1, window_end = ?2 WHERE token = 'TOK123'",
				p![now - 7200, now - 3600]
			)
			.unwrap();
		}
		let err = join(&db, "TOK123", "111", "Test Siswa", None, None).unwrap_err();
		assert_eq!(err, "Waktu ujian sudah berakhir");

		drop(db);
		let _ = std::fs::remove_dir_all(&dir);
	}

	#[test]
	fn full_duration_and_extra_minutes_work() {
		let (db, dir) = setup();
		let now = now_secs();

		{
			let conn = db.lock().unwrap();
			conn.execute(
				"UPDATE exams SET scheduled_at = ?1, window_end = ?2 WHERE token = 'TOK123'",
				p![now - 3600, now + 3600]
			)
			.unwrap();
		}

		let joined = join(&db, "TOK123", "111", "Test Siswa", None, None).unwrap();
		assert_eq!(joined.duration_seconds, 30 * 60);

		let extra = add_extra_time(&db, joined.session_id, 10).unwrap();
		assert_eq!(extra, 10);

		let rejoined = join(&db, "TOK123", "111", "Test Siswa", None, Some(&joined.device_token)).unwrap();
		assert_eq!(rejoined.duration_seconds, 40 * 60);

		drop(db);
		let _ = std::fs::remove_dir_all(&dir);
	}

	#[test]
	fn pg_session_locks_and_essay_can_be_graded_with_combined_score() {
		let (db, dir) = setup();

		let subject_id: i64 = db.lock().unwrap().query_row("SELECT subject_id FROM exams WHERE token = 'TOK123'", [], |r| r.get(0)).unwrap();
		db.lock()
			.unwrap()
			.execute(
				"INSERT INTO questions (subject_id, question_text, question_type, score) VALUES (?1, 'Jelaskan sesuatu', 'essay', 5)",
				p![subject_id]
			)
			.unwrap();

		let joined = join(&db, "TOK123", "111", "Test Siswa", None, None).unwrap();
		assert_eq!(joined.questions.len(), 2);
		let pg_qid = joined.questions.iter().find(|q| q.question_type != "essay").unwrap().id;
		let essay_qid = joined.questions.iter().find(|q| q.question_type == "essay").unwrap().id;

		answer(&db, joined.session_id, pg_qid, "A", None).unwrap();
		submit_pg(&db, joined.session_id).unwrap();

		let err = answer(&db, joined.session_id, pg_qid, "B", None).unwrap_err();
		assert_eq!(err, "Sesi Pilihan Ganda sudah dikirim, jawaban tidak bisa diubah lagi");
		answer(&db, joined.session_id, essay_qid, "Jawaban esai saya", None).unwrap();

		let result = submit(&db, joined.session_id).unwrap();
		assert_eq!(result.score, 100.0);

		grade_essay_answer(&db, joined.session_id, essay_qid, 4.0).unwrap();

		let detail = session_detail(&db, joined.session_id).unwrap();
		let essay_item = detail.items.iter().find(|i| i.question_id == essay_qid).unwrap();
		assert_eq!(essay_item.essay_score, Some(4.0));
		assert_eq!(detail.essay_score, Some(80.0));
		// Default blend: 70% of 100 + 30% of 80 = 70 + 24 = 94
		assert_eq!(detail.final_score.map(|v| v.round()), Some(94.0));

		drop(db);
		let _ = std::fs::remove_dir_all(&dir);
	}

	#[test]
	fn analytics_counts_pg_correctness_and_distractors() {
		let (db, dir) = setup();
		let subject_id: i64 = db.lock().unwrap().query_row("SELECT subject_id FROM exams WHERE token = 'TOK123'", [], |r| r.get(0)).unwrap();

		let joined = join(&db, "TOK123", "111", "Test Siswa", None, None).unwrap();
		let qid = joined.questions[0].id;

		let early = analytics(&db, None, subject_id, None).unwrap();
		assert!(early.student_stats.is_empty());

		answer(&db, joined.session_id, qid, "A", None).unwrap();
		submit_pg(&db, joined.session_id).unwrap();

		let stats = analytics(&db, None, subject_id, None).unwrap();
		assert_eq!(stats.question_stats.len(), 1);
		assert_eq!(stats.question_stats[0].correct_count, 1);
		assert_eq!(stats.question_stats[0].incorrect_count, 0);
		assert_eq!(stats.question_stats[0].option_counts.get("A"), Some(&1));
		assert_eq!(stats.student_stats.len(), 1);
		assert_eq!(stats.student_stats[0].name, "Test Siswa");
		assert_eq!(stats.student_stats[0].correct_count, 1);

		drop(db);
		let _ = std::fs::remove_dir_all(&dir);
	}
}
