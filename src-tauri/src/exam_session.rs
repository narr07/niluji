use crate::db::{generate_secret, secrets_match, Db};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub const ERR_IDENTITY: &str = "NISN atau nama tidak cocok";
pub const ERR_EXAM_TOKEN: &str = "Token ujian tidak ditemukan";
pub const ERR_PIN: &str = "PIN Kartu Peserta salah atau belum diisi.";
pub const ERR_SESSION: &str = "Sesi ujian tidak valid. Masuk ulang memakai token ujian.";
pub const ERR_SUBMITTED: &str = "Ujian sudah dikirim, jawaban tidak bisa diubah lagi";
pub const ERR_TIME_UP: &str = "Waktu pengerjaan ujian telah habis";
pub const ERR_PG_LOCKED: &str = "Sesi Pilihan Ganda sudah dikirim, jawaban tidak bisa diubah lagi";
pub const ERR_QUESTION: &str = "Soal ini bukan bagian dari sesi ujianmu";
pub const ERR_OPTION: &str = "Pilihan jawaban tidak tersedia pada soal ini";
pub const ERR_ANSWER_TOO_LONG: &str = "Jawaban esai terlalu panjang";

const MAX_ESSAY_CHARS: usize = 20_000;
// Jawaban yang datang sedikit terlambat karena WiFi lemah masih diterima selama masa tenggang ini.
const ANSWER_GRACE_SECONDS: i64 = 60;
// Sesi yang ditinggalkan dikumpulkan server setelah tenggang ini, sehingga perangkat yang masih
// hidup selalu mendapat kesempatan mengirim jawaban terakhirnya lebih dulu.
const AUTO_SUBMIT_GRACE_SECONDS: i64 = 180;

// Kegagalan yang dihitung oleh pembatas percobaan login: tebakan identitas, token, atau PIN.
pub fn is_credential_error(message: &str) -> bool {
	[ERR_IDENTITY, ERR_EXAM_TOKEN, ERR_PIN].contains(&message)
}

// Penolakan final: mengirim ulang permintaan yang sama tidak akan mengubah hasilnya.
pub fn is_final_rejection(message: &str) -> bool {
	[ERR_SESSION, ERR_SUBMITTED, ERR_TIME_UP, ERR_PG_LOCKED, ERR_QUESTION, ERR_OPTION, ERR_ANSWER_TOO_LONG].contains(&message)
}

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

fn non_blank(value: &Option<String>) -> Option<&str> {
	value.as_deref().map(str::trim).filter(|s| !s.is_empty())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
	pub session_token: String,
	pub exam_title: String,
	pub duration_seconds: i64,
	pub remaining_seconds: i64,
	pub started_at: i64,
	pub pg_submitted_at: Option<i64>,
	pub submitted_at: Option<i64>,
	pub device_token: String,
	pub questions: Vec<QuestionView>,
	pub answers: HashMap<i64, String>,
	pub doubtful: HashMap<i64, bool>
}

struct ExamRow {
	id: i64,
	title: String,
	subject_id: i64,
	class: Option<String>,
	jenis: Option<String>,
	duration: i64,
	window_start: Option<i64>,
	window_end: Option<i64>,
	randomize_pg: bool,
	randomize_essay: bool,
	randomize_options: bool,
	require_pin: bool
}

struct SessionRow {
	id: i64,
	started_at: i64,
	pg_submitted_at: Option<i64>,
	submitted_at: Option<i64>,
	extra_minutes: i64,
	device_token: Option<String>,
	session_token: Option<String>
}

// Salinan soal yang dipakai satu sesi. Mengubah atau menghapus Bank Soal sesudah ujian dimulai
// tidak lagi mengubah apa yang dikerjakan siswa maupun dasar penilaiannya.
struct SnapshotQuestion {
	id: i64,
	text: String,
	question_type: String,
	image: Option<String>,
	options_json: String
}

impl SnapshotQuestion {
	fn is_essay(&self) -> bool {
		self.question_type == "essay"
	}

	fn options(&self) -> Vec<OptionView> {
		serde_json::from_str(&self.options_json).unwrap_or_default()
	}
}

pub(crate) fn snapshot_session(conn: &Connection, session_id: i64, question_ids: &[i64]) -> rusqlite::Result<()> {
	let mut read_question =
		conn.prepare_cached("SELECT question_text, question_type, image, answer_key, score FROM questions WHERE id = ?1")?;
	let mut read_options =
		conn.prepare_cached("SELECT option_key, option_text, image FROM question_options WHERE question_id = ?1 ORDER BY option_key")?;
	let mut insert = conn.prepare_cached(
		"INSERT OR IGNORE INTO exam_session_questions
		 (session_id, question_id, position, question_text, question_type, image, answer_key, score, options_json)
		 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"
	)?;

	for (position, question_id) in question_ids.iter().enumerate() {
		#[allow(clippy::type_complexity)]
		let row: Option<(String, String, Option<String>, Option<String>, f64)> = read_question
			.query_row(params![question_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)))
			.optional()?;
		let Some((text, question_type, image, answer_key, score)) = row else {
			continue;
		};
		let options = read_options
			.query_map(params![question_id], |r| Ok(OptionView { key: r.get(0)?, text: r.get(1)?, image: r.get(2)? }))?
			.collect::<rusqlite::Result<Vec<_>>>()?;
		let options_json = serde_json::to_string(&options).unwrap_or_else(|_| "[]".to_string());
		insert.execute(params![session_id, question_id, position as i64, text, question_type, image, answer_key, score, options_json])?;
	}
	Ok(())
}

fn load_snapshot(conn: &Connection, session_id: i64) -> Result<Vec<SnapshotQuestion>, String> {
	let mut stmt = conn
		.prepare_cached(
			"SELECT question_id, question_text, question_type, image, options_json
			 FROM exam_session_questions WHERE session_id = ?1 ORDER BY position"
		)
		.map_err(|e| e.to_string())?;
	let rows = stmt
		.query_map(params![session_id], |r| {
			Ok(SnapshotQuestion {
				id: r.get(0)?,
				text: r.get(1)?,
				question_type: r.get(2)?,
				image: r.get(3)?,
				options_json: r.get(4)?
			})
		})
		.map_err(|e| e.to_string())?;
	rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

fn find_exam(conn: &Connection, token: &str) -> Result<ExamRow, String> {
	conn.query_row(
		"SELECT id, title, subject_id, class, jenis, duration, scheduled_at, window_end,
		        randomize_pg, randomize_essay, randomize_options, require_pin
		 FROM exams WHERE token = ?1 COLLATE NOCASE",
		params![token.trim()],
		|r| {
			Ok(ExamRow {
				id: r.get(0)?,
				title: r.get(1)?,
				subject_id: r.get(2)?,
				class: r.get(3)?,
				jenis: r.get(4)?,
				duration: r.get(5)?,
				window_start: r.get(6)?,
				window_end: r.get(7)?,
				randomize_pg: r.get::<_, Option<i64>>(8)?.unwrap_or(1) != 0,
				randomize_essay: r.get::<_, Option<i64>>(9)?.unwrap_or(1) != 0,
				randomize_options: r.get::<_, Option<i64>>(10)?.unwrap_or(1) != 0,
				require_pin: r.get::<_, Option<i64>>(11)?.unwrap_or(0) != 0
			})
		}
	)
	.map_err(|_| ERR_EXAM_TOKEN.to_string())
}

fn find_session(conn: &Connection, exam_id: i64, nisn: &str) -> Result<Option<SessionRow>, String> {
	conn.query_row(
		"SELECT id, started_at, pg_submitted_at, submitted_at, COALESCE(extra_minutes, 0), device_token, session_token
		 FROM exam_sessions WHERE exam_id = ?1 AND student_nisn = ?2",
		params![exam_id, nisn],
		|r| {
			Ok(SessionRow {
				id: r.get(0)?,
				started_at: r.get(1)?,
				pg_submitted_at: r.get(2)?,
				submitted_at: r.get(3)?,
				extra_minutes: r.get(4)?,
				device_token: r.get(5)?,
				session_token: r.get(6)?
			})
		}
	)
	.optional()
	.map_err(|e| e.to_string())
}

fn check_window(exam: &ExamRow) -> Result<(), String> {
	if let Some(window_start) = exam.window_start {
		let now = now_secs();
		if now < window_start {
			return Err("Ujian belum dimulai".to_string());
		}
		if exam.window_end.is_some_and(|end| now > end) {
			return Err("Waktu ujian sudah berakhir".to_string());
		}
	}
	Ok(())
}

fn create_session(conn: &Connection, exam: &ExamRow, nisn: &str, student_name: &str, device_token: &str) -> Result<SessionRow, String> {
	check_window(exam)?;

	let ids: Vec<i64> = {
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
			.query_map(
				params![exam.subject_id, exam.class, exam.jenis, exam.randomize_pg as i64, exam.randomize_essay as i64],
				|r| r.get(0)
			)
			.map_err(|e| e.to_string())?;
		mapped.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?
	};
	if ids.is_empty() {
		return Err("Belum ada soal untuk mata pelajaran ujian ini".to_string());
	}

	let question_ids = ids.iter().map(i64::to_string).collect::<Vec<_>>().join(",");
	let started_at = now_secs();
	let session_token = generate_secret();

	let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
	tx.execute(
		"INSERT INTO exam_sessions (exam_id, student_nisn, student_name, question_ids, started_at, extra_minutes, device_token, session_token)
		 VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6, ?7)",
		params![exam.id, nisn, student_name, question_ids, started_at, device_token, session_token]
	)
	.map_err(|e| e.to_string())?;
	let id = tx.last_insert_rowid();
	snapshot_session(&tx, id, &ids).map_err(|e| e.to_string())?;
	tx.commit().map_err(|e| e.to_string())?;

	Ok(SessionRow {
		id,
		started_at,
		pg_submitted_at: None,
		submitted_at: None,
		extra_minutes: 0,
		device_token: Some(device_token.to_string()),
		session_token: Some(session_token)
	})
}

// Mengikat sesi yang sedang berjalan ke perangkat ini. Token sesi diganti setiap kali sesi pindah
// perangkat, sehingga perangkat lama langsung kehilangan akses.
fn resume_session(conn: &Connection, mut session: SessionRow, device_token: &str) -> Result<SessionRow, String> {
	let locked_to = non_blank(&session.device_token).map(str::to_string);
	if locked_to.as_deref().is_some_and(|locked| locked != device_token) {
		return Err(
			"Sesi ujian Anda sedang aktif di perangkat lain. Hubungi pengawas untuk membuka kunci perangkat jika Anda berganti HP/laptop."
				.to_string()
		);
	}

	let needs_new_token = locked_to.is_none() || non_blank(&session.session_token).is_none();
	if needs_new_token {
		let session_token = generate_secret();
		conn.execute(
			"UPDATE exam_sessions SET device_token = ?1, session_token = ?2 WHERE id = ?3",
			params![device_token, session_token, session.id]
		)
		.map_err(|e| e.to_string())?;
		session.device_token = Some(device_token.to_string());
		session.session_token = Some(session_token);
	}
	Ok(session)
}

pub fn join(
	db: &Db,
	token: &str,
	nisn: &str,
	name: &str,
	pin: Option<&str>,
	device_token: Option<&str>
) -> Result<JoinResponse, String> {
	let conn = db.lock().unwrap();
	let nisn = nisn.trim();

	let (student_name, student_class, stored_pin): (String, Option<String>, String) = conn
		.query_row(
			"SELECT name, class, COALESCE(pin, '') FROM students WHERE nisn = ?1 AND name = ?2 COLLATE NOCASE",
			params![nisn, name.trim()],
			|r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))
		)
		.map_err(|_| ERR_IDENTITY.to_string())?;

	let exam = find_exam(&conn, token)?;

	if exam.require_pin {
		let expected = stored_pin.trim();
		if expected.is_empty() || !secrets_match(pin.unwrap_or("").trim(), expected) {
			return Err(ERR_PIN.to_string());
		}
	}

	// Siswa tanpa data kelas tetap boleh masuk supaya kekurangan data tidak menggagalkan ujian.
	if let (Some(exam_class), Some(own_class)) = (non_blank(&exam.class), non_blank(&student_class)) {
		if exam_class.to_lowercase() != own_class.to_lowercase() {
			return Err(format!(
				"Ujian ini untuk Kelas {exam_class}, sedangkan kamu terdaftar di Kelas {own_class}. Hubungi pengawas bila data kelasmu keliru."
			));
		}
	}

	let device_token = device_token
		.map(str::trim)
		.filter(|s| !s.is_empty())
		.map(String::from)
		.unwrap_or_else(|| format!("dev-{}", generate_secret()));

	let existing = find_session(&conn, exam.id, nisn)?;
	let duration_for = |extra: i64| (exam.duration + extra) * 60;

	// Sesi yang sudah selesai hanya memberi tahu statusnya. Soal, jawaban, nilai, dan token
	// perangkat tidak dikirim lagi ke siapa pun yang sekadar mengetahui NISN dan nama.
	if let Some(done) = existing.as_ref().filter(|s| s.submitted_at.is_some()) {
		return Ok(JoinResponse {
			session_id: done.id,
			session_token: String::new(),
			exam_title: exam.title.clone(),
			duration_seconds: duration_for(done.extra_minutes),
			remaining_seconds: 0,
			started_at: done.started_at,
			pg_submitted_at: done.pg_submitted_at,
			submitted_at: done.submitted_at,
			device_token,
			questions: Vec::new(),
			answers: HashMap::new(),
			doubtful: HashMap::new()
		});
	}

	let session = match existing {
		Some(session) => resume_session(&conn, session, &device_token)?,
		None => create_session(&conn, &exam, nisn, &student_name, &device_token)?
	};

	let questions = load_snapshot(&conn, session.id)?
		.into_iter()
		.map(|q| {
			let mut options = q.options();
			if exam.randomize_options && !q.is_essay() && options.len() > 1 {
				options.sort_by_key(|opt| shuffle_weight(session.id, q.id, &opt.key));
			}
			QuestionView { id: q.id, question_text: q.text, question_type: q.question_type, image: q.image, options }
		})
		.collect();

	let mut answers = HashMap::new();
	let mut doubtful = HashMap::new();
	let mut ans_stmt = conn
		.prepare("SELECT question_id, option_key, COALESCE(is_doubtful, 0) FROM exam_answers WHERE session_id = ?1")
		.map_err(|e| e.to_string())?;
	let rows = ans_stmt
		.query_map(params![session.id], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, i64>(2)? != 0)))
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

	let duration_seconds = duration_for(session.extra_minutes);
	let elapsed = (now_secs() - session.started_at).max(0);

	Ok(JoinResponse {
		session_id: session.id,
		session_token: session.session_token.unwrap_or_default(),
		exam_title: exam.title,
		duration_seconds,
		remaining_seconds: (duration_seconds - elapsed).max(0),
		started_at: session.started_at,
		pg_submitted_at: session.pg_submitted_at,
		submitted_at: None,
		device_token: session.device_token.unwrap_or(device_token),
		questions,
		answers,
		doubtful
	})
}

fn authorize(conn: &Connection, session_id: i64, session_token: &str) -> Result<(), String> {
	let stored: Option<String> = conn
		.query_row("SELECT session_token FROM exam_sessions WHERE id = ?1", params![session_id], |r| r.get(0))
		.optional()
		.map_err(|e| e.to_string())?
		.flatten();
	match stored.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
		Some(expected) if secrets_match(session_token.trim(), expected) => Ok(()),
		_ => Err(ERR_SESSION.to_string())
	}
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
		.execute("UPDATE exam_sessions SET device_token = NULL, session_token = NULL WHERE id = ?1", params![session_id])
		.map_err(|e| e.to_string())?;
	Ok(())
}

// Menambah waktu. Sesi yang tadinya dikumpulkan otomatis dibuka kembali dan mendapat sisa waktu
// sebanyak menit yang ditambahkan, misalnya untuk siswa yang HP-nya mati saat ujian.
pub fn add_extra_time(db: &Db, session_id: i64, minutes: i64) -> Result<i64, String> {
	let conn = db.lock().unwrap();
	let (started_at, duration, extra, auto_submitted): (i64, i64, i64, i64) = conn
		.query_row(
			"SELECT es.started_at, e.duration, COALESCE(es.extra_minutes, 0), COALESCE(es.auto_submitted, 0)
			 FROM exam_sessions es JOIN exams e ON e.id = es.exam_id WHERE es.id = ?1",
			params![session_id],
			|r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
		)
		.map_err(|_| "Sesi ujian tidak ditemukan".to_string())?;

	let mut updated = extra + minutes;
	if auto_submitted == 0 {
		conn.execute("UPDATE exam_sessions SET extra_minutes = ?1 WHERE id = ?2", params![updated, session_id])
			.map_err(|e| e.to_string())?;
		return Ok(updated);
	}

	let elapsed_minutes = ((now_secs() - started_at).max(0) + 59) / 60;
	updated = updated.max(elapsed_minutes + minutes - duration);
	conn.execute(
		"UPDATE exam_sessions
		 SET extra_minutes = ?1, submitted_at = NULL, score = NULL, auto_submitted = 0,
		     pg_submitted_at = CASE WHEN ?2 = 2 THEN NULL ELSE pg_submitted_at END
		 WHERE id = ?3",
		params![updated, auto_submitted, session_id]
	)
	.map_err(|e| e.to_string())?;
	Ok(updated)
}

// Mengumpulkan sesi yang waktunya sudah habis tetapi tidak pernah dikumpulkan.
pub fn finalize_expired(db: &Db) -> Result<usize, String> {
	let conn = db.lock().unwrap();
	let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
	let expired: Vec<(i64, Option<i64>)> = {
		let mut stmt = tx
			.prepare(
				"SELECT es.id, es.pg_submitted_at
				 FROM exam_sessions es
				 JOIN exams e ON e.id = es.exam_id
				 WHERE es.submitted_at IS NULL
				   AND es.started_at + (e.duration + COALESCE(es.extra_minutes, 0)) * 60 + ?1 < ?2"
			)
			.map_err(|e| e.to_string())?;
		let rows = stmt
			.query_map(params![AUTO_SUBMIT_GRACE_SECONDS, now_secs()], |r| Ok((r.get(0)?, r.get(1)?)))
			.map_err(|e| e.to_string())?;
		rows.collect::<Result<_, _>>().map_err(|e| e.to_string())?
	};

	for (session_id, pg_submitted_at) in &expired {
		finalize(&tx, *session_id)?;
		let marker = if pg_submitted_at.is_some() { 1 } else { 2 };
		tx.execute("UPDATE exam_sessions SET auto_submitted = ?1 WHERE id = ?2", params![marker, session_id])
			.map_err(|e| e.to_string())?;
	}
	tx.commit().map_err(|e| e.to_string())?;
	Ok(expired.len())
}

pub fn force_submit(db: &Db, session_id: i64) -> Result<SubmitResponse, String> {
	finalize(&db.lock().unwrap(), session_id)
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
	pub essay_pending: i64,
	pub pg_weight: f64,
	pub pg_submitted_at: Option<i64>,
	pub submitted_at: Option<i64>,
	pub items: Vec<AnswerDetail>
}

#[derive(Debug, Default, Clone, Copy)]
struct EssayTally {
	earned: f64,
	max: f64,
	// Jawaban esai yang sudah diisi siswa tetapi belum diberi nilai oleh guru.
	pending: i64
}

// Esai yang tidak dijawab bernilai 0. Esai yang dijawab tetapi belum dinilai membuat nilai akhir
// tetap kosong, kecuali bobot esainya 0% sehingga hasilnya tidak bergantung pada esai.
fn combine_scores(pg_score: Option<f64>, has_pg: bool, essay: EssayTally, pg_weight: f64, submitted: bool) -> (Option<f64>, Option<f64>) {
	let has_essay = essay.max > 0.0;
	let essay_pct = (submitted && has_essay && essay.pending == 0).then(|| (essay.earned / essay.max) * 100.0);

	let final_score = match (has_pg, has_essay) {
		(true, false) => pg_score,
		(false, true) => essay_pct,
		(true, true) => {
			let weight = (pg_weight / 100.0).clamp(0.0, 1.0);
			if weight >= 1.0 {
				pg_score
			} else if weight <= 0.0 {
				essay_pct
			} else {
				pg_score.zip(essay_pct).map(|(pg, es)| pg * weight + es * (1.0 - weight))
			}
		}
		(false, false) => None
	};

	(essay_pct, final_score)
}

pub fn session_detail(db: &Db, session_id: i64) -> Result<SessionDetail, String> {
	let conn = db.lock().unwrap();

	#[allow(clippy::type_complexity)]
	let (student_name, exam_title, class, subject, score, pg_submitted_at, submitted_at, pg_weight): (
		String,
		String,
		Option<String>,
		String,
		Option<f64>,
		Option<i64>,
		Option<i64>,
		f64
	) = conn
		.query_row(
			"SELECT es.student_name, e.title, e.class, s.name, es.score, es.pg_submitted_at, es.submitted_at, COALESCE(e.pg_weight, 100.0)
			 FROM exam_sessions es
			 JOIN exams e ON e.id = es.exam_id
			 JOIN subjects s ON s.id = e.subject_id
			 WHERE es.id = ?1",
			params![session_id],
			|r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?, r.get(7)?))
		)
		.map_err(|_| "Sesi ujian tidak ditemukan".to_string())?;

	let mut stmt = conn
		.prepare(
			"SELECT q.question_id, q.question_text, q.question_type, q.answer_key, q.score, a.option_key, a.essay_score
			 FROM exam_session_questions q
			 LEFT JOIN exam_answers a ON a.session_id = q.session_id AND a.question_id = q.question_id
			 WHERE q.session_id = ?1
			 ORDER BY q.question_id"
		)
		.map_err(|e| e.to_string())?;
	#[allow(clippy::type_complexity)]
	let rows: Vec<(i64, String, String, Option<String>, f64, Option<String>, Option<f64>)> = stmt
		.query_map(params![session_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?)))
		.map_err(|e| e.to_string())?
		.collect::<Result<_, _>>()
		.map_err(|e| e.to_string())?;

	let mut items = Vec::with_capacity(rows.len());
	let mut has_pg = false;
	let mut essay = EssayTally::default();

	for (question_id, question_text, question_type, correct_answer, max_score, student_answer, essay_score) in rows {
		let answered = student_answer.as_deref().is_some_and(|a| !a.trim().is_empty());
		let is_essay = question_type == "essay";
		if is_essay {
			essay.max += max_score;
			essay.earned += essay_score.unwrap_or(0.0);
			if answered && essay_score.is_none() {
				essay.pending += 1;
			}
		} else {
			has_pg = true;
		}

		let is_correct = !is_essay
			&& matches!((&student_answer, &correct_answer), (Some(given), Some(key)) if given.trim().eq_ignore_ascii_case(key.trim()));

		items.push(AnswerDetail {
			question_id,
			question_text,
			question_type,
			student_answer: student_answer.filter(|s| !s.is_empty()),
			correct_answer,
			is_correct,
			essay_score,
			max_score
		});
	}

	let (essay_score, final_score) = combine_scores(score, has_pg, essay, pg_weight, submitted_at.is_some());

	Ok(SessionDetail {
		student_name,
		exam_title,
		class,
		subject,
		score,
		essay_score,
		final_score,
		essay_pending: essay.pending,
		pg_weight,
		pg_submitted_at,
		submitted_at,
		items
	})
}

pub fn grade_essay_answer(db: &Db, session_id: i64, question_id: i64, score: f64) -> Result<(), String> {
	let conn = db.lock().unwrap();

	let (question_type, max_score): (String, f64) = conn
		.query_row(
			"SELECT question_type, score FROM exam_session_questions WHERE session_id = ?1 AND question_id = ?2",
			params![session_id, question_id],
			|r| Ok((r.get(0)?, r.get(1)?))
		)
		.map_err(|_| "Soal tidak ditemukan pada sesi ini".to_string())?;
	if question_type != "essay" {
		return Err("Hanya soal esai yang dinilai manual".to_string());
	}
	if !score.is_finite() || score < 0.0 || score > max_score {
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
	pub essay_pending: i64,
	pub extra_minutes: i64,
	pub device_locked: bool,
	pub auto_submitted: bool
}

pub fn list_sessions(db: &Db) -> Result<Vec<SessionProgress>, String> {
	let conn = db.lock().unwrap();
	let mut stmt = conn
		.prepare_cached(
			"SELECT es.id, es.student_name, es.student_nisn, e.title, e.class, s.name, e.jenis,
			        es.submitted_at, es.score, COALESCE(es.extra_minutes, 0), es.device_token, COALESCE(e.pg_weight, 100.0),
			        COUNT(q.question_id),
			        COALESCE(SUM(CASE WHEN TRIM(COALESCE(a.option_key, '')) != '' THEN 1 ELSE 0 END), 0),
			        COALESCE(SUM(CASE WHEN q.question_type != 'essay' THEN 1 ELSE 0 END), 0),
			        COALESCE(SUM(CASE WHEN q.question_type != 'essay' AND TRIM(COALESCE(a.option_key, '')) != '' THEN 1 ELSE 0 END), 0),
			        COALESCE(SUM(CASE WHEN q.question_type = 'essay' THEN q.score ELSE 0 END), 0),
			        COALESCE(SUM(CASE WHEN q.question_type = 'essay' THEN COALESCE(a.essay_score, 0) ELSE 0 END), 0),
			        COALESCE(SUM(CASE WHEN q.question_type = 'essay' AND TRIM(COALESCE(a.option_key, '')) != '' AND a.essay_score IS NULL THEN 1 ELSE 0 END), 0),
			        COALESCE(es.auto_submitted, 0)
			 FROM exam_sessions es
			 JOIN exams e ON e.id = es.exam_id
			 JOIN subjects s ON s.id = e.subject_id
			 LEFT JOIN exam_session_questions q ON q.session_id = es.id
			 LEFT JOIN exam_answers a ON a.session_id = es.id AND a.question_id = q.question_id
			 GROUP BY es.id
			 ORDER BY es.started_at DESC"
		)
		.map_err(|e| e.to_string())?;

	let rows = stmt
		.query_map([], |r| {
			let submitted_at: Option<i64> = r.get(7)?;
			let score: Option<f64> = r.get(8)?;
			let device_token: Option<String> = r.get(10)?;
			let pg_weight: f64 = r.get(11)?;
			let pg_total: i64 = r.get(14)?;
			let essay = EssayTally { max: r.get(16)?, earned: r.get(17)?, pending: r.get(18)? };
			let (essay_score, final_score) = combine_scores(score, pg_total > 0, essay, pg_weight, submitted_at.is_some());
			Ok(SessionProgress {
				session_id: r.get(0)?,
				student_name: r.get(1)?,
				student_nisn: r.get(2)?,
				exam_title: r.get(3)?,
				class: r.get(4)?,
				subject: r.get(5)?,
				jenis: r.get(6)?,
				total_questions: r.get(12)?,
				answered_count: r.get(13)?,
				pg_total,
				pg_answered: r.get(15)?,
				submitted_at,
				score,
				essay_score,
				final_score,
				essay_pending: essay.pending,
				extra_minutes: r.get(9)?,
				device_locked: device_token.as_deref().is_some_and(|t| !t.trim().is_empty()),
				auto_submitted: r.get::<_, i64>(19)? != 0
			})
		})
		.map_err(|e| e.to_string())?;

	rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
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
			"SELECT es.id, es.student_name, q.question_id, q.question_text, q.answer_key, a.option_key
			 FROM exam_sessions es
			 JOIN exams e ON e.id = es.exam_id
			 LEFT JOIN exam_session_questions q ON q.session_id = es.id AND q.question_type != 'essay'
			 LEFT JOIN exam_answers a ON a.session_id = es.id AND a.question_id = q.question_id
			 WHERE e.subject_id = ?1 AND (?2 IS NULL OR e.class = ?2) AND (?3 IS NULL OR e.jenis = ?3) AND es.pg_submitted_at IS NOT NULL
			 ORDER BY es.id, q.position"
		)
		.map_err(|e| e.to_string())?;
	#[allow(clippy::type_complexity)]
	let rows: Vec<(i64, String, Option<i64>, Option<String>, Option<String>, Option<String>)> = stmt
		.query_map(params![subject_id, class, jenis], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?)))
		.map_err(|e| e.to_string())?
		.collect::<Result<_, _>>()
		.map_err(|e| e.to_string())?;

	let mut stats_by_question: HashMap<i64, QuestionStat> = HashMap::new();
	let mut student_stats: Vec<StudentStat> = Vec::new();

	for (session_id, student_name, question_id, question_text, answer_key, given) in rows {
		if student_stats.last().is_none_or(|s| s.session_id != session_id) {
			student_stats.push(StudentStat { session_id, name: student_name, correct_count: 0, incorrect_count: 0 });
		}
		let Some(question_id) = question_id else {
			continue;
		};
		let key = answer_key.unwrap_or_default().trim().to_uppercase();
		let stat = stats_by_question.entry(question_id).or_insert_with(|| QuestionStat {
			id: question_id,
			question_text: question_text.unwrap_or_default(),
			correct_option: key.clone(),
			correct_count: 0,
			incorrect_count: 0,
			option_counts: HashMap::new()
		});

		let picked = given.as_deref().map(str::trim).filter(|s| !s.is_empty()).map(str::to_uppercase);
		let is_correct = !key.is_empty() && picked.as_deref() == Some(key.as_str());
		*stat.option_counts.entry(picked.unwrap_or_else(|| "KOSONG".to_string())).or_insert(0) += 1;

		let student = student_stats.last_mut().expect("baris siswa sudah ditambahkan");
		if is_correct {
			stat.correct_count += 1;
			student.correct_count += 1;
		} else {
			stat.incorrect_count += 1;
			student.incorrect_count += 1;
		}
	}

	let mut question_stats: Vec<QuestionStat> = stats_by_question.into_values().collect();
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
	session_token: &str,
	question_id: i64,
	option_key: &str,
	is_doubtful: Option<bool>
) -> Result<AnswerResponse, String> {
	let conn = db.lock().unwrap();
	authorize(&conn, session_id, session_token)?;

	let (pg_submitted_at, submitted_at, started_at, extra_minutes, duration): (Option<i64>, Option<i64>, i64, i64, i64) = conn
		.query_row(
			"SELECT es.pg_submitted_at, es.submitted_at, es.started_at, COALESCE(es.extra_minutes, 0), e.duration
			 FROM exam_sessions es
			 JOIN exams e ON e.id = es.exam_id
			 WHERE es.id = ?1",
			params![session_id],
			|r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
		)
		.map_err(|_| ERR_SESSION.to_string())?;

	if submitted_at.is_some() {
		return Err(ERR_SUBMITTED.to_string());
	}

	let duration_seconds = (duration + extra_minutes) * 60;
	let elapsed = (now_secs() - started_at).max(0);
	if elapsed > duration_seconds + ANSWER_GRACE_SECONDS {
		return Err(ERR_TIME_UP.to_string());
	}

	let (question_type, options_json): (String, String) = conn
		.query_row(
			"SELECT question_type, options_json FROM exam_session_questions WHERE session_id = ?1 AND question_id = ?2",
			params![session_id, question_id],
			|r| Ok((r.get(0)?, r.get(1)?))
		)
		.optional()
		.map_err(|e| e.to_string())?
		.ok_or(ERR_QUESTION)?;

	if question_type == "essay" {
		if option_key.chars().count() > MAX_ESSAY_CHARS {
			return Err(ERR_ANSWER_TOO_LONG.to_string());
		}
	} else {
		if pg_submitted_at.is_some() {
			return Err(ERR_PG_LOCKED.to_string());
		}
		// String kosong sah: dipakai saat siswa menandai ragu-ragu sebelum memilih jawaban.
		let options: Vec<OptionView> = serde_json::from_str(&options_json).unwrap_or_default();
		if !option_key.trim().is_empty() && !options.iter().any(|o| o.key.eq_ignore_ascii_case(option_key.trim())) {
			return Err(ERR_OPTION.to_string());
		}
	}

	conn.execute(
		"INSERT INTO exam_answers (session_id, question_id, option_key, is_doubtful) VALUES (?1, ?2, ?3, COALESCE(?4, 0))
		 ON CONFLICT(session_id, question_id) DO UPDATE SET
		 	option_key = excluded.option_key,
		 	is_doubtful = COALESCE(?4, exam_answers.is_doubtful)",
		params![session_id, question_id, option_key, is_doubtful.map(i64::from)]
	)
	.map_err(|e| e.to_string())?;

	Ok(AnswerResponse { remaining_seconds: (duration_seconds - elapsed).max(0) })
}

// Mengunci jawaban PG dan memindahkan siswa ke bagian esai. Aman dipanggil ulang: waktu
// penguncian pertama tetap dipertahankan.
pub fn submit_pg(db: &Db, session_id: i64, session_token: &str) -> Result<(), String> {
	let conn = db.lock().unwrap();
	authorize(&conn, session_id, session_token)?;
	conn.execute(
		"UPDATE exam_sessions SET pg_submitted_at = COALESCE(pg_submitted_at, ?1) WHERE id = ?2",
		params![now_secs(), session_id]
	)
	.map_err(|e| e.to_string())?;
	Ok(())
}

#[derive(Debug, Serialize)]
pub struct SubmitResponse {
	pub score: f64
}

fn pg_score(conn: &Connection, session_id: i64) -> Result<f64, String> {
	let mut stmt = conn
		.prepare_cached(
			"SELECT q.answer_key, q.score, a.option_key
			 FROM exam_session_questions q
			 LEFT JOIN exam_answers a ON a.session_id = q.session_id AND a.question_id = q.question_id
			 WHERE q.session_id = ?1 AND q.question_type != 'essay'"
		)
		.map_err(|e| e.to_string())?;
	let rows = stmt
		.query_map(params![session_id], |r| Ok((r.get::<_, Option<String>>(0)?, r.get::<_, f64>(1)?, r.get::<_, Option<String>>(2)?)))
		.map_err(|e| e.to_string())?;

	let mut total = 0.0;
	let mut earned = 0.0;
	for row in rows {
		let (key, score, given) = row.map_err(|e| e.to_string())?;
		total += score;
		if let (Some(key), Some(given)) = (key, given) {
			if !key.trim().is_empty() && given.trim().eq_ignore_ascii_case(key.trim()) {
				earned += score;
			}
		}
	}
	Ok(if total > 0.0 { (earned / total) * 100.0 } else { 0.0 })
}

// Mengumpulkan sesi sekali saja. Pemanggilan berikutnya (coba ulang karena jaringan, atau Paksa
// Kumpul pada sesi yang sudah selesai) mengembalikan nilai yang sudah tersimpan.
fn finalize(conn: &Connection, session_id: i64) -> Result<SubmitResponse, String> {
	let (submitted_at, stored_score): (Option<i64>, Option<f64>) = conn
		.query_row("SELECT submitted_at, score FROM exam_sessions WHERE id = ?1", params![session_id], |r| Ok((r.get(0)?, r.get(1)?)))
		.map_err(|_| "Sesi ujian tidak ditemukan".to_string())?;
	if submitted_at.is_some() {
		return Ok(SubmitResponse { score: stored_score.unwrap_or(0.0) });
	}

	let score = pg_score(conn, session_id)?;
	let now = now_secs();
	conn.execute(
		"UPDATE exam_sessions SET submitted_at = ?1, pg_submitted_at = COALESCE(pg_submitted_at, ?1), score = ?2 WHERE id = ?3",
		params![now, score, session_id]
	)
	.map_err(|e| e.to_string())?;
	Ok(SubmitResponse { score })
}

pub fn submit(db: &Db, session_id: i64, session_token: &str) -> Result<SubmitResponse, String> {
	let conn = db.lock().unwrap();
	authorize(&conn, session_id, session_token)?;
	finalize(&conn, session_id)
}

// Menerapkan kunci jawaban dan skor terbaru dari Bank Soal ke sesi yang sudah ada, lalu menghitung
// ulang nilai PG. Hanya berjalan atas perintah guru, misalnya setelah membetulkan kunci yang salah.
pub fn regrade(db: &Db, class: Option<String>, subject_id: i64, jenis: Option<String>) -> Result<usize, String> {
	let mut conn = db.lock().unwrap();
	let tx = conn.transaction().map_err(|e| e.to_string())?;

	let sessions: Vec<(i64, Option<i64>)> = {
		let mut stmt = tx
			.prepare(
				"SELECT es.id, es.submitted_at
				 FROM exam_sessions es
				 JOIN exams e ON e.id = es.exam_id
				 WHERE e.subject_id = ?1 AND (?2 IS NULL OR e.class = ?2) AND (?3 IS NULL OR e.jenis = ?3)"
			)
			.map_err(|e| e.to_string())?;
		let rows = stmt.query_map(params![subject_id, class, jenis], |r| Ok((r.get(0)?, r.get(1)?))).map_err(|e| e.to_string())?;
		rows.collect::<Result<_, _>>().map_err(|e| e.to_string())?
	};

	for (session_id, submitted_at) in &sessions {
		tx.execute(
			"UPDATE exam_session_questions
			 SET answer_key = (SELECT q.answer_key FROM questions q WHERE q.id = exam_session_questions.question_id),
			     score = (SELECT q.score FROM questions q WHERE q.id = exam_session_questions.question_id)
			 WHERE session_id = ?1 AND EXISTS (SELECT 1 FROM questions q WHERE q.id = exam_session_questions.question_id)",
			params![session_id]
		)
		.map_err(|e| e.to_string())?;
		if submitted_at.is_some() {
			let score = pg_score(&tx, *session_id)?;
			tx.execute("UPDATE exam_sessions SET score = ?1 WHERE id = ?2", params![score, session_id]).map_err(|e| e.to_string())?;
		}
	}

	tx.commit().map_err(|e| e.to_string())?;
	Ok(sessions.len())
}

#[cfg(test)]
mod tests {
	use super::*;
	use rusqlite::params as p;

	struct Fixture {
		db: Db,
		dir: std::path::PathBuf,
		subject_id: i64,
		q1: i64
	}

	impl Drop for Fixture {
		fn drop(&mut self) {
			let _ = std::fs::remove_dir_all(&self.dir);
		}
	}

	fn setup() -> Fixture {
		let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
		let dir = std::env::temp_dir().join(format!("nuxtor_cbt_exam_session_test_{}_{nanos}", std::process::id()));
		let db = crate::db::open(&dir);
		let (subject_id, q1) = {
			let conn = db.lock().unwrap();
			conn.execute("INSERT INTO students (nisn, name, class, pin) VALUES ('111', 'Test Siswa', '6', '654321')", []).unwrap();
			conn.execute("INSERT INTO students (nisn, name, class, pin) VALUES ('222', 'Siswa Lain', '4', '123456')", []).unwrap();
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
			(subject_id, q1)
		};
		Fixture { db, dir, subject_id, q1 }
	}

	fn add_essay(f: &Fixture, score: f64) -> i64 {
		let conn = f.db.lock().unwrap();
		conn.execute(
			"INSERT INTO questions (subject_id, question_text, question_type, score) VALUES (?1, 'Jelaskan sesuatu', 'essay', ?2)",
			p![f.subject_id, score]
		)
		.unwrap();
		conn.last_insert_rowid()
	}

	fn set_weight(f: &Fixture, weight: f64) {
		f.db.lock().unwrap().execute("UPDATE exams SET pg_weight = ?1", p![weight]).unwrap();
	}

	fn join_a(f: &Fixture) -> JoinResponse {
		join(&f.db, "TOK123", "111", "Test Siswa", None, Some("device-A")).unwrap()
	}

	#[test]
	fn join_answer_submit_flow() {
		let f = setup();
		let joined = join_a(&f);
		assert_eq!(joined.questions.len(), 1);
		assert!(joined.session_token.len() >= 32);

		answer(&f.db, joined.session_id, &joined.session_token, f.q1, "A", Some(true)).unwrap();
		let result = submit(&f.db, joined.session_id, &joined.session_token).unwrap();
		assert_eq!(result.score, 100.0);
	}

	#[test]
	fn session_token_is_required_for_every_student_write() {
		let f = setup();
		let victim = join_a(&f);
		answer(&f.db, victim.session_id, &victim.session_token, f.q1, "A", None).unwrap();

		for forged in ["", "tebakan", "0000000000000000000000000000000000000000000000000000000000000000"] {
			assert_eq!(answer(&f.db, victim.session_id, forged, f.q1, "B", None).unwrap_err(), ERR_SESSION);
			assert_eq!(submit(&f.db, victim.session_id, forged).unwrap_err(), ERR_SESSION);
			assert_eq!(submit_pg(&f.db, victim.session_id, forged).unwrap_err(), ERR_SESSION);
		}

		let detail = session_detail(&f.db, victim.session_id).unwrap();
		assert_eq!(detail.items[0].student_answer.as_deref(), Some("A"));
		assert!(detail.submitted_at.is_none());
	}

	#[test]
	fn another_students_token_cannot_touch_the_session() {
		let f = setup();
		let victim = join_a(&f);
		let other = join(&f.db, "TOK123", "222", "Siswa Lain", None, Some("device-B")).unwrap();
		assert_eq!(answer(&f.db, victim.session_id, &other.session_token, f.q1, "B", None).unwrap_err(), ERR_SESSION);
	}

	#[test]
	fn answer_must_belong_to_the_session_and_use_a_real_option() {
		let f = setup();
		let joined = join_a(&f);
		assert_eq!(answer(&f.db, joined.session_id, &joined.session_token, f.q1 + 999, "A", None).unwrap_err(), ERR_QUESTION);
		assert_eq!(answer(&f.db, joined.session_id, &joined.session_token, f.q1, "Z", None).unwrap_err(), ERR_OPTION);
		answer(&f.db, joined.session_id, &joined.session_token, f.q1, "", Some(true)).unwrap();
		let rejoined = join_a(&f);
		assert_eq!(rejoined.doubtful.get(&f.q1), Some(&true));
	}

	#[test]
	fn doubtful_flag_survives_an_answer_without_the_flag() {
		let f = setup();
		let joined = join_a(&f);
		answer(&f.db, joined.session_id, &joined.session_token, f.q1, "A", Some(true)).unwrap();
		answer(&f.db, joined.session_id, &joined.session_token, f.q1, "B", None).unwrap();
		assert_eq!(join_a(&f).doubtful.get(&f.q1), Some(&true));
	}

	#[test]
	fn single_device_lock_and_proctor_unlock() {
		let f = setup();
		let joined = join_a(&f);
		assert_eq!(joined.device_token, "device-A");

		let err = join(&f.db, "TOK123", "111", "Test Siswa", None, Some("device-B")).unwrap_err();
		assert!(err.contains("perangkat lain"));

		unlock_device(&f.db, joined.session_id).unwrap();
		assert_eq!(answer(&f.db, joined.session_id, &joined.session_token, f.q1, "A", None).unwrap_err(), ERR_SESSION);

		let rejoined = join(&f.db, "TOK123", "111", "Test Siswa", None, Some("device-B")).unwrap();
		assert_eq!(rejoined.device_token, "device-B");
		assert_ne!(rejoined.session_token, joined.session_token);
		answer(&f.db, rejoined.session_id, &rejoined.session_token, f.q1, "A", None).unwrap();
	}

	#[test]
	fn rejoining_on_the_same_device_keeps_the_session_token() {
		let f = setup();
		let first = join_a(&f);
		assert_eq!(join_a(&f).session_token, first.session_token);
	}

	#[test]
	fn finished_session_does_not_reveal_questions_answers_or_device_token() {
		let f = setup();
		let joined = join_a(&f);
		answer(&f.db, joined.session_id, &joined.session_token, f.q1, "A", None).unwrap();
		submit(&f.db, joined.session_id, &joined.session_token).unwrap();

		let later = join(&f.db, "TOK123", "111", "Test Siswa", None, Some("device-X")).unwrap();
		assert!(later.submitted_at.is_some());
		assert!(later.questions.is_empty());
		assert!(later.answers.is_empty());
		assert!(later.session_token.is_empty());
		assert_eq!(later.device_token, "device-X");
	}

	#[test]
	fn pin_is_enforced_and_missing_pin_fails_closed() {
		let f = setup();
		f.db.lock().unwrap().execute("UPDATE exams SET require_pin = 1", []).unwrap();
		assert_eq!(join(&f.db, "TOK123", "111", "Test Siswa", None, Some("d")).unwrap_err(), ERR_PIN);
		assert_eq!(join(&f.db, "TOK123", "111", "Test Siswa", Some("000000"), Some("d")).unwrap_err(), ERR_PIN);
		join(&f.db, "TOK123", "111", "Test Siswa", Some("654321"), Some("d")).unwrap();

		f.db.lock().unwrap().execute("UPDATE students SET pin = '' WHERE nisn = '222'", []).unwrap();
		assert_eq!(join(&f.db, "TOK123", "222", "Siswa Lain", Some(""), Some("e")).unwrap_err(), ERR_PIN);
	}

	#[test]
	fn student_from_another_class_is_rejected() {
		let f = setup();
		f.db.lock().unwrap().execute("UPDATE exams SET class = '6'", []).unwrap();
		f.db.lock().unwrap().execute("UPDATE questions SET class = '6'", []).unwrap();

		let err = join(&f.db, "TOK123", "222", "Siswa Lain", None, Some("d")).unwrap_err();
		assert!(err.contains("Kelas 6") && err.contains("Kelas 4"));
		join_a(&f);

		f.db.lock().unwrap().execute("UPDATE students SET class = NULL WHERE nisn = '222'", []).unwrap();
		join(&f.db, "TOK123", "222", "Siswa Lain", None, Some("d")).unwrap();
	}

	#[test]
	fn credential_errors_are_classified_for_rate_limiting() {
		assert!(is_credential_error(ERR_PIN));
		assert!(is_credential_error(ERR_IDENTITY));
		assert!(is_credential_error(ERR_EXAM_TOKEN));
		assert!(!is_credential_error("Ujian belum dimulai"));
	}

	#[test]
	fn rejects_join_outside_scheduled_window() {
		let f = setup();
		let now = now_secs();
		f.db.lock().unwrap().execute("UPDATE exams SET scheduled_at = ?1 WHERE token = 'TOK123'", p![now + 3600]).unwrap();
		assert_eq!(join(&f.db, "TOK123", "111", "Test Siswa", None, None).unwrap_err(), "Ujian belum dimulai");

		f.db.lock()
			.unwrap()
			.execute("UPDATE exams SET scheduled_at = ?1, window_end = ?2 WHERE token = 'TOK123'", p![now - 7200, now - 3600])
			.unwrap();
		assert_eq!(join(&f.db, "TOK123", "111", "Test Siswa", None, None).unwrap_err(), "Waktu ujian sudah berakhir");
	}

	#[test]
	fn full_duration_and_extra_minutes_work() {
		let f = setup();
		let joined = join_a(&f);
		assert_eq!(joined.duration_seconds, 30 * 60);
		assert_eq!(add_extra_time(&f.db, joined.session_id, 10).unwrap(), 10);
		assert_eq!(join_a(&f).duration_seconds, 40 * 60);
	}

	#[test]
	fn answers_after_grace_period_are_rejected_but_late_ones_are_kept() {
		let f = setup();
		let joined = join_a(&f);
		let set_elapsed = |seconds: i64| {
			f.db.lock().unwrap().execute("UPDATE exam_sessions SET started_at = ?1", p![now_secs() - seconds]).unwrap();
		};
		set_elapsed(30 * 60 + 30);
		answer(&f.db, joined.session_id, &joined.session_token, f.q1, "A", None).unwrap();
		set_elapsed(30 * 60 + 120);
		assert_eq!(answer(&f.db, joined.session_id, &joined.session_token, f.q1, "B", None).unwrap_err(), ERR_TIME_UP);
	}

	#[test]
	fn submit_is_idempotent() {
		let f = setup();
		let joined = join_a(&f);
		answer(&f.db, joined.session_id, &joined.session_token, f.q1, "A", None).unwrap();
		submit(&f.db, joined.session_id, &joined.session_token).unwrap();
		let first: i64 = f.db.lock().unwrap().query_row("SELECT submitted_at FROM exam_sessions", [], |r| r.get(0)).unwrap();
		f.db.lock().unwrap().execute("UPDATE exam_sessions SET submitted_at = ?1", p![first - 100]).unwrap();

		assert_eq!(submit(&f.db, joined.session_id, &joined.session_token).unwrap().score, 100.0);
		assert_eq!(force_submit(&f.db, joined.session_id).unwrap().score, 100.0);
		let after: i64 = f.db.lock().unwrap().query_row("SELECT submitted_at FROM exam_sessions", [], |r| r.get(0)).unwrap();
		assert_eq!(after, first - 100);
	}

	#[test]
	fn final_submit_also_locks_the_pg_section() {
		let f = setup();
		add_essay(&f, 5.0);
		let joined = join_a(&f);
		submit(&f.db, joined.session_id, &joined.session_token).unwrap();
		assert!(session_detail(&f.db, joined.session_id).unwrap().pg_submitted_at.is_some());
	}

	#[test]
	fn pg_locks_and_essay_is_combined_with_the_configured_weight() {
		let f = setup();
		let essay = add_essay(&f, 5.0);
		set_weight(&f, 70.0);
		let joined = join_a(&f);
		assert_eq!(joined.questions.len(), 2);

		answer(&f.db, joined.session_id, &joined.session_token, f.q1, "A", None).unwrap();
		submit_pg(&f.db, joined.session_id, &joined.session_token).unwrap();
		assert_eq!(answer(&f.db, joined.session_id, &joined.session_token, f.q1, "B", None).unwrap_err(), ERR_PG_LOCKED);
		answer(&f.db, joined.session_id, &joined.session_token, essay, "Jawaban esai saya", None).unwrap();
		assert_eq!(submit(&f.db, joined.session_id, &joined.session_token).unwrap().score, 100.0);

		let pending = session_detail(&f.db, joined.session_id).unwrap();
		assert_eq!(pending.essay_pending, 1);
		assert_eq!(pending.essay_score, None);
		assert_eq!(pending.final_score, None, "nilai akhir tidak boleh muncul sebelum esai dinilai");

		grade_essay_answer(&f.db, joined.session_id, essay, 4.0).unwrap();
		let detail = session_detail(&f.db, joined.session_id).unwrap();
		assert_eq!(detail.essay_pending, 0);
		assert_eq!(detail.essay_score, Some(80.0));
		assert_eq!(detail.final_score.map(|v| v.round()), Some(94.0));

		let listed = list_sessions(&f.db).unwrap();
		assert_eq!(listed[0].final_score.map(|v| v.round()), Some(94.0));
		assert_eq!(listed[0].total_questions, 2);
		assert_eq!(listed[0].answered_count, 2);
		assert_eq!(listed[0].pg_total, 1);
		assert_eq!(listed[0].pg_answered, 1);
	}

	#[test]
	fn weight_of_100_means_pg_only_even_with_ungraded_essays() {
		let f = setup();
		let essay = add_essay(&f, 5.0);
		set_weight(&f, 100.0);
		let joined = join_a(&f);
		answer(&f.db, joined.session_id, &joined.session_token, f.q1, "A", None).unwrap();
		answer(&f.db, joined.session_id, &joined.session_token, essay, "Isi", None).unwrap();
		submit(&f.db, joined.session_id, &joined.session_token).unwrap();
		assert_eq!(session_detail(&f.db, joined.session_id).unwrap().final_score, Some(100.0));
	}

	#[test]
	fn unanswered_essay_counts_as_zero_without_blocking_the_final_score() {
		let f = setup();
		add_essay(&f, 5.0);
		set_weight(&f, 50.0);
		let joined = join_a(&f);
		answer(&f.db, joined.session_id, &joined.session_token, f.q1, "A", None).unwrap();
		submit(&f.db, joined.session_id, &joined.session_token).unwrap();
		let detail = session_detail(&f.db, joined.session_id).unwrap();
		assert_eq!(detail.essay_pending, 0);
		assert_eq!(detail.final_score, Some(50.0));
	}

	#[test]
	fn grading_rejects_pg_questions_and_invalid_scores() {
		let f = setup();
		let essay = add_essay(&f, 5.0);
		let joined = join_a(&f);
		assert!(grade_essay_answer(&f.db, joined.session_id, f.q1, 1.0).is_err());
		assert!(grade_essay_answer(&f.db, joined.session_id, essay, 6.0).is_err());
		assert!(grade_essay_answer(&f.db, joined.session_id, essay, f64::NAN).is_err());
	}

	#[test]
	fn results_are_protected_from_bank_changes_until_regraded() {
		let f = setup();
		let joined = join_a(&f);
		answer(&f.db, joined.session_id, &joined.session_token, f.q1, "A", None).unwrap();
		submit(&f.db, joined.session_id, &joined.session_token).unwrap();

		f.db.lock().unwrap().execute("UPDATE questions SET answer_key = 'B', question_text = 'Diubah'", []).unwrap();
		let detail = session_detail(&f.db, joined.session_id).unwrap();
		assert_eq!(detail.score, Some(100.0));
		assert!(detail.items[0].is_correct);
		assert_eq!(detail.items[0].question_text, "Soal 1");

		assert_eq!(regrade(&f.db, None, f.subject_id, None).unwrap(), 1);
		let regraded = session_detail(&f.db, joined.session_id).unwrap();
		assert_eq!(regraded.score, Some(0.0));
		assert!(!regraded.items[0].is_correct);
		assert_eq!(regraded.items[0].question_text, "Soal 1");
	}

	#[test]
	fn abandoned_session_is_submitted_after_the_grace_period() {
		let f = setup();
		let joined = join_a(&f);
		answer(&f.db, joined.session_id, &joined.session_token, f.q1, "A", None).unwrap();

		let set_elapsed = |seconds: i64| {
			f.db.lock().unwrap().execute("UPDATE exam_sessions SET started_at = ?1", p![now_secs() - seconds]).unwrap();
		};
		set_elapsed(30 * 60 + AUTO_SUBMIT_GRACE_SECONDS - 30);
		assert_eq!(finalize_expired(&f.db).unwrap(), 0);

		set_elapsed(30 * 60 + AUTO_SUBMIT_GRACE_SECONDS + 30);
		assert_eq!(finalize_expired(&f.db).unwrap(), 1);
		assert_eq!(finalize_expired(&f.db).unwrap(), 0);
		let listed = list_sessions(&f.db).unwrap();
		assert!(listed[0].auto_submitted);
		assert_eq!(listed[0].score, Some(100.0));
	}

	#[test]
	fn proctor_can_reopen_an_auto_submitted_session_with_extra_time() {
		let f = setup();
		let joined = join_a(&f);
		f.db.lock().unwrap().execute("UPDATE exam_sessions SET started_at = ?1", p![now_secs() - 40 * 60]).unwrap();
		finalize_expired(&f.db).unwrap();
		assert_eq!(answer(&f.db, joined.session_id, &joined.session_token, f.q1, "A", None).unwrap_err(), ERR_SUBMITTED);

		add_extra_time(&f.db, joined.session_id, 10).unwrap();
		let reopened = join_a(&f);
		assert!(reopened.submitted_at.is_none());
		assert!(reopened.pg_submitted_at.is_none(), "PG yang belum dikunci siswa harus bisa dikerjakan lagi");
		assert!((9 * 60..=10 * 60).contains(&reopened.remaining_seconds), "{}", reopened.remaining_seconds);
		answer(&f.db, reopened.session_id, &reopened.session_token, f.q1, "A", None).unwrap();
		assert_eq!(finalize_expired(&f.db).unwrap(), 0);
		assert!(!list_sessions(&f.db).unwrap()[0].auto_submitted);
	}

	#[test]
	fn analytics_counts_pg_correctness_and_distractors() {
		let f = setup();
		let joined = join_a(&f);
		assert!(analytics(&f.db, None, f.subject_id, None).unwrap().student_stats.is_empty());

		answer(&f.db, joined.session_id, &joined.session_token, f.q1, "A", None).unwrap();
		submit_pg(&f.db, joined.session_id, &joined.session_token).unwrap();

		let stats = analytics(&f.db, None, f.subject_id, None).unwrap();
		assert_eq!(stats.question_stats.len(), 1);
		assert_eq!(stats.question_stats[0].correct_count, 1);
		assert_eq!(stats.question_stats[0].incorrect_count, 0);
		assert_eq!(stats.question_stats[0].option_counts.get("A"), Some(&1));
		assert_eq!(stats.student_stats.len(), 1);
		assert_eq!(stats.student_stats[0].name, "Test Siswa");
		assert_eq!(stats.student_stats[0].correct_count, 1);
	}
}
