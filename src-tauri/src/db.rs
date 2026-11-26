use rusqlite::Connection;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

// ponytail: single Mutex<Connection>, fine for one exam room's concurrency;
// switch to a connection pool (r2d2_sqlite) if multiple exam sessions hammer it at once.
pub type Db = Arc<Mutex<Connection>>;

pub fn ping(db: &Db) -> bool {
	db.lock().unwrap().query_row("SELECT 1", [], |row| row.get::<_, i32>(0)).is_ok()
}

pub fn find_student(db: &Db, nisn: &str) -> Option<String> {
	db.lock()
		.unwrap()
		.query_row(
			"SELECT name FROM students WHERE nisn = ?1",
			rusqlite::params![nisn.trim()],
			|r| r.get(0),
		)
		.ok()
}

fn fill_random(buffer: &mut [u8]) {
	getrandom::getrandom(buffer).expect("generator acak sistem operasi tidak tersedia");
}

// Rumus PIN versi lama. Hasilnya bisa dihitung siapa pun dari ID dan NISN, jadi tidak lagi dipakai
// untuk membuat PIN dan hanya tersisa untuk mendeteksi PIN lama yang perlu diacak ulang.
fn legacy_pin_from_seed(id: i64, nisn: &str) -> String {
	let mut hash: u64 = (id as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(0xBF58_476D_1CE4_E5B9);
	for b in nisn.bytes() {
		hash = hash.wrapping_mul(31).wrapping_add(b as u64);
	}
	hash ^= hash >> 30;
	hash = hash.wrapping_mul(0xBF58_476D_1CE4_E5B9);
	hash ^= hash >> 27;
	let num = 100_000 + (hash % 900_000);
	num.to_string()
}

pub fn count_legacy_pins(db: &Db) -> Result<usize, String> {
	let conn = db.lock().unwrap();
	let mut stmt = conn.prepare("SELECT id, nisn, COALESCE(pin, '') FROM students").map_err(|e| e.to_string())?;
	let rows = stmt
		.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?)))
		.map_err(|e| e.to_string())?;
	let mut count = 0;
	for row in rows {
		let (id, nisn, pin) = row.map_err(|e| e.to_string())?;
		if pin.trim() == legacy_pin_from_seed(id, &nisn) {
			count += 1;
		}
	}
	Ok(count)
}

// PIN 6 digit dari generator acak sistem operasi, tanpa bias modulo.
pub fn generate_random_pin() -> String {
	const RANGE: u32 = 900_000;
	let limit = u32::MAX - (u32::MAX % RANGE);
	loop {
		let mut bytes = [0u8; 4];
		fill_random(&mut bytes);
		let value = u32::from_le_bytes(bytes);
		if value < limit {
			return (100_000 + value % RANGE).to_string();
		}
	}
}

pub fn generate_secret() -> String {
	let mut bytes = [0u8; 32];
	fill_random(&mut bytes);
	bytes.iter().map(|b| format!("{b:02x}")).collect()
}

// Perbandingan yang waktunya tidak bergantung pada posisi karakter pertama yang berbeda.
pub fn secrets_match(given: &str, expected: &str) -> bool {
	let (given, expected) = (given.as_bytes(), expected.as_bytes());
	given.len() == expected.len() && given.iter().zip(expected).fold(0u8, |diff, (a, b)| diff | (a ^ b)) == 0
}

// Migrate data from the legacy template directory (`com.nicolaspadari.nuxtor`) if `cbt.sqlite`
// does not exist yet in the new `com.narr07.niluji` directory, preserving existing user data.
fn migrate_legacy_data_dir(data_dir: &Path) {
	if data_dir.join("cbt.sqlite").exists() {
		return;
	}
	let Some(parent) = data_dir.parent() else {
		return;
	};
	let legacy_dir = parent.join("com.nicolaspadari.nuxtor");
	if !legacy_dir.join("cbt.sqlite").exists() {
		return;
	}
	let _ = std::fs::create_dir_all(data_dir);
	for file in &["cbt.sqlite", "cbt.sqlite-wal", "cbt.sqlite-shm"] {
		let src = legacy_dir.join(file);
		if src.exists() {
			let _ = std::fs::copy(&src, data_dir.join(file));
		}
	}
	let legacy_uploads = legacy_dir.join("uploads");
	if legacy_uploads.is_dir() {
		let target_uploads = data_dir.join("uploads");
		let _ = std::fs::create_dir_all(&target_uploads);
		if let Ok(entries) = std::fs::read_dir(&legacy_uploads) {
			for entry in entries.flatten() {
				let dest = target_uploads.join(entry.file_name());
				if !dest.exists() {
					let _ = std::fs::copy(entry.path(), dest);
				}
			}
		}
	}
}

pub fn open(data_dir: &Path) -> Db {
	std::fs::create_dir_all(data_dir).expect("failed to create app data dir");
	migrate_legacy_data_dir(data_dir);
	let conn = Connection::open(data_dir.join("cbt.sqlite")).expect("failed to open sqlite db");
	conn.busy_timeout(Duration::from_secs(5)).expect("failed to set sqlite busy timeout");
	migrate(&conn).expect("failed to run sqlite migrations");
	Arc::new(Mutex::new(conn))
}

pub(crate) fn migrate(conn: &Connection) -> rusqlite::Result<()> {
	conn.execute_batch(
		"
		PRAGMA foreign_keys = ON;
		-- WAL: pembacaan (dashboard guru, live-refresh tiap 3 detik) tidak saling mengunci
		-- dengan penulisan (siswa yang lagi menjawab) — penting begitu banyak siswa jalan
		-- bersamaan. synchronous=NORMAL aman dipakai bareng WAL (tetap tahan crash aplikasi,
		-- cuma sedikit lebih longgar dari FULL soal power-loss di tengah write, yang wajar
		-- untuk laptop/PC sekolah biasa).
		PRAGMA journal_mode = WAL;
		PRAGMA synchronous = NORMAL;

		CREATE TABLE IF NOT EXISTS subjects (
			id INTEGER PRIMARY KEY AUTOINCREMENT,
			name TEXT NOT NULL UNIQUE,
			code TEXT
		);

		CREATE TABLE IF NOT EXISTS classes (
			id INTEGER PRIMARY KEY AUTOINCREMENT,
			title TEXT NOT NULL UNIQUE,
			description TEXT
		);

		CREATE TABLE IF NOT EXISTS question_types (
			id INTEGER PRIMARY KEY AUTOINCREMENT,
			name TEXT NOT NULL UNIQUE,
			description TEXT
		);

		CREATE TABLE IF NOT EXISTS school (
			id INTEGER PRIMARY KEY CHECK (id = 1),
			name TEXT NOT NULL DEFAULT 'Nama Sekolah',
			npsn TEXT NOT NULL DEFAULT '-',
			address TEXT NOT NULL DEFAULT '-',
			principal TEXT NOT NULL DEFAULT '-'
		);

		CREATE TABLE IF NOT EXISTS questions (
			id INTEGER PRIMARY KEY AUTOINCREMENT,
			subject_id INTEGER NOT NULL REFERENCES subjects(id) ON DELETE CASCADE,
			class TEXT,
			question_text TEXT NOT NULL,
			question_type TEXT NOT NULL DEFAULT 'multiple_choice',
			answer_key TEXT,
			score REAL NOT NULL DEFAULT 1
		);

		CREATE TABLE IF NOT EXISTS question_options (
			id INTEGER PRIMARY KEY AUTOINCREMENT,
			question_id INTEGER NOT NULL REFERENCES questions(id) ON DELETE CASCADE,
			option_key TEXT NOT NULL,
			option_text TEXT NOT NULL,
			is_correct INTEGER NOT NULL DEFAULT 0
		);

		CREATE TABLE IF NOT EXISTS students (
			id INTEGER PRIMARY KEY AUTOINCREMENT,
			nisn TEXT NOT NULL UNIQUE,
			name TEXT NOT NULL
		);

		CREATE TABLE IF NOT EXISTS exams (
			id INTEGER PRIMARY KEY AUTOINCREMENT,
			subject_id INTEGER NOT NULL REFERENCES subjects(id) ON DELETE CASCADE,
			class TEXT,
			title TEXT NOT NULL,
			duration INTEGER NOT NULL DEFAULT 60,
			question_count INTEGER NOT NULL DEFAULT 10,
			token TEXT NOT NULL UNIQUE
		);

		CREATE TABLE IF NOT EXISTS exam_sessions (
			id INTEGER PRIMARY KEY AUTOINCREMENT,
			exam_id INTEGER NOT NULL REFERENCES exams(id) ON DELETE CASCADE,
			student_nisn TEXT NOT NULL,
			student_name TEXT NOT NULL,
			question_ids TEXT NOT NULL,
			started_at INTEGER NOT NULL,
			submitted_at INTEGER,
			score REAL,
			UNIQUE(exam_id, student_nisn)
		);

		CREATE TABLE IF NOT EXISTS exam_answers (
			session_id INTEGER NOT NULL REFERENCES exam_sessions(id) ON DELETE CASCADE,
			question_id INTEGER NOT NULL REFERENCES questions(id) ON DELETE CASCADE,
			option_key TEXT NOT NULL,
			PRIMARY KEY (session_id, question_id)
		);

		-- Salinan soal, kunci, dan skor pada saat sesi dimulai. Sengaja tanpa foreign key ke
		-- questions supaya perubahan Bank Soal tidak mengubah ujian yang sudah berjalan.
		CREATE TABLE IF NOT EXISTS exam_session_questions (
			session_id INTEGER NOT NULL REFERENCES exam_sessions(id) ON DELETE CASCADE,
			question_id INTEGER NOT NULL,
			position INTEGER NOT NULL,
			question_text TEXT NOT NULL,
			question_type TEXT NOT NULL,
			image TEXT,
			answer_key TEXT,
			score REAL NOT NULL,
			options_json TEXT NOT NULL DEFAULT '[]',
			PRIMARY KEY (session_id, question_id)
		);
		CREATE INDEX IF NOT EXISTS idx_exam_session_questions_question ON exam_session_questions(question_id);
		",
	)?;

	// Standard Kurikulum Merdeka SD subjects, seeded so they're pickable right away
	// without needing an import first.
	conn.execute_batch(
		"
		INSERT OR IGNORE INTO subjects (name) VALUES ('Pendidikan Agama Islam dan Budi Pekerti');
		INSERT OR IGNORE INTO subjects (name) VALUES ('Pendidikan Pancasila');
		INSERT OR IGNORE INTO subjects (name) VALUES ('Bahasa Indonesia');
		INSERT OR IGNORE INTO subjects (name) VALUES ('Matematika');
		INSERT OR IGNORE INTO subjects (name) VALUES ('IPAS (Ilmu Pengetahuan Alam dan Sosial)');
		INSERT OR IGNORE INTO subjects (name) VALUES ('Seni dan Budaya');
		INSERT OR IGNORE INTO subjects (name) VALUES ('PJOK');
		INSERT OR IGNORE INTO subjects (name) VALUES ('Bahasa Inggris');
		INSERT OR IGNORE INTO subjects (name) VALUES ('Bahasa Sunda');

		INSERT OR IGNORE INTO classes (title, description) VALUES ('4', 'Kelas 4');
		INSERT OR IGNORE INTO classes (title, description) VALUES ('5', 'Kelas 5');
		INSERT OR IGNORE INTO classes (title, description) VALUES ('6', 'Kelas 6');

		INSERT OR IGNORE INTO school (id, name, npsn, address, principal) VALUES (1, 'Nama Sekolah', '-', '-', '-');
		",
	)?;

	// Columns added after the initial CREATE TABLE need an explicit ALTER for databases
	// that already existed (CREATE TABLE IF NOT EXISTS is a no-op on those).
	ensure_column(conn, "questions", "class", "TEXT")?;
	ensure_column(conn, "questions", "image", "TEXT")?;
	// Gambar per PILIHAN jawaban (bukan cuma per soal) — buat tipe soal yang jawabannya sendiri
	// berupa gambar (mis. "gambar mana yang menunjukkan hewan mamalia?").
	ensure_column(conn, "question_options", "image", "TEXT")?;
	ensure_column(conn, "exams", "class", "TEXT")?;
	ensure_column(conn, "exams", "scheduled_at", "INTEGER")?;
	ensure_column(conn, "students", "class", "TEXT")?;
	ensure_column(conn, "students", "school", "TEXT")?;
	ensure_column(conn, "students", "pin", "TEXT")?;
	// Rombel (IVA, IVB) terpisah dari kelas/tingkat (4) — lihat kelas.rs.
	ensure_column(conn, "students", "rombel", "TEXT")?;
	ensure_column(conn, "classes", "sort_order", "INTEGER")?;
	ensure_column(conn, "subjects", "code", "TEXT")?;
	ensure_column(conn, "questions", "jenis", "TEXT")?;
	ensure_column(conn, "exams", "jenis", "TEXT")?;
	ensure_column(conn, "subjects", "sort_order", "INTEGER")?;
	// "scheduled_at" is the exam window's START; window_end is when access closes. Once a
	// student joins inside that window they get the full flat `duration`, even if it runs
	// past window_end — the window only gates WHEN they can start, not how long they get.
	ensure_column(conn, "exams", "window_end", "INTEGER")?;
	// Set once the student submits the PG portion — after that, PG answers are locked and
	// the student moves on to the essay portion (if any) as a separate step.
	ensure_column(conn, "exam_sessions", "pg_submitted_at", "INTEGER")?;
	// Extra minutes granted by the proctor for this specific session.
	ensure_column(conn, "exam_sessions", "extra_minutes", "INTEGER DEFAULT 0")?;
	// Single-device lock token so a student cannot open the same active session on two devices simultaneously.
	ensure_column(conn, "exam_sessions", "device_token", "TEXT")?;
	// Rahasia yang diberikan server saat siswa masuk, wajib dikirim pada setiap jawaban dan pengumpulan.
	ensure_column(conn, "exam_sessions", "session_token", "TEXT")?;
	// 0 = dikumpulkan siswa/pengawas, 1 = otomatis saat PG sudah terkunci, 2 = otomatis saat PG belum
	// dikunci. Nilainya dipakai untuk mengembalikan kondisi sesi kalau pengawas membukanya lagi.
	ensure_column(conn, "exam_sessions", "auto_submitted", "INTEGER DEFAULT 0")?;
	// Teacher-assigned points for one essay answer; never rolled into the automatic PG score.
	ensure_column(conn, "exam_answers", "essay_score", "REAL")?;
	// Flag "ragu-ragu" (doubtful/bookmarked) set by the student while taking the exam.
	ensure_column(conn, "exam_answers", "is_doubtful", "INTEGER DEFAULT 0")?;
	// PIN gerbang fitur Export soal. Sengaja tanpa nilai bawaan: export baru bisa dipakai setelah
	// sekolah menentukan PIN sendiri di Pengaturan > Data Sekolah.
	ensure_column(conn, "school", "export_pin", "TEXT")?;
	// Gugus bank soal yang dipilih sekolah ini di Tarik Soal Online (lihat bank_soal.rs).
	ensure_column(conn, "school", "bank_kecamatan", "TEXT")?;
	ensure_column(conn, "school", "bank_gugus", "TEXT")?;
	// Acak urutan soal per siswa saat ujian ini dimulai — dipisah PG dan esai (guru bisa mau
	// PG-nya diacak tapi esainya tidak, atau sebaliknya). "randomize" (kolom lama, sekarang
	// tidak dipakai lagi) jadi sumber nilai awal keduanya biar ujian yang sudah dibuat sebelum
	// pemisahan ini tetap berperilaku sama seperti sebelumnya.
	ensure_column(conn, "exams", "randomize", "INTEGER")?;
	ensure_column(conn, "exams", "randomize_pg", "INTEGER")?;
	ensure_column(conn, "exams", "randomize_essay", "INTEGER")?;
	ensure_column(conn, "exams", "randomize_options", "INTEGER DEFAULT 1")?;
	ensure_column(conn, "exams", "require_pin", "INTEGER DEFAULT 0")?;
	ensure_column(conn, "exams", "pg_weight", "REAL DEFAULT 100.0")?;
	conn.execute("UPDATE exams SET randomize_pg = COALESCE(randomize, 1) WHERE randomize_pg IS NULL", [])?;
	conn.execute("UPDATE exams SET randomize_essay = COALESCE(randomize, 1) WHERE randomize_essay IS NULL", [])?;
	conn.execute("UPDATE exams SET randomize_options = 1 WHERE randomize_options IS NULL", [])?;
	conn.execute("UPDATE exams SET require_pin = 0 WHERE require_pin IS NULL", [])?;
	conn.execute("UPDATE exams SET pg_weight = 100.0 WHERE pg_weight IS NULL", [])?;
	conn.execute("UPDATE exam_sessions SET extra_minutes = 0 WHERE extra_minutes IS NULL", [])?;
	conn.execute("UPDATE exam_answers SET is_doubtful = 0 WHERE is_doubtful IS NULL", [])?;

	// Backfill PIN for any existing student row that does not have one yet.
	{
		let mut stmt = conn.prepare("SELECT id FROM students WHERE pin IS NULL OR TRIM(pin) = ''")?;
		let missing: Vec<i64> = stmt.query_map([], |r| r.get(0))?.filter_map(Result::ok).collect();
		for id in missing {
			conn.execute("UPDATE students SET pin = ?1 WHERE id = ?2", rusqlite::params![generate_random_pin(), id])?;
		}
	}

	relax_question_types_uniqueness(conn)?;
	snapshot_existing_sessions(conn)?;

	// Short default codes so existing installs get compact labels without a manual edit.
	conn.execute_batch(
		"
		UPDATE subjects SET code = 'PAI' WHERE name = 'Pendidikan Agama Islam dan Budi Pekerti' AND code IS NULL;
		UPDATE subjects SET code = 'PPKn' WHERE name = 'Pendidikan Pancasila' AND code IS NULL;
		UPDATE subjects SET code = 'B.Indo' WHERE name = 'Bahasa Indonesia' AND code IS NULL;
		UPDATE subjects SET code = 'MTK' WHERE name = 'Matematika' AND code IS NULL;
		UPDATE subjects SET code = 'IPAS' WHERE name = 'IPAS (Ilmu Pengetahuan Alam dan Sosial)' AND code IS NULL;
		UPDATE subjects SET code = 'SBDP' WHERE name = 'Seni dan Budaya' AND code IS NULL;
		UPDATE subjects SET code = 'PJOK' WHERE name = 'PJOK' AND code IS NULL;
		UPDATE subjects SET code = 'B.Ing' WHERE name = 'Bahasa Inggris' AND code IS NULL;
		UPDATE subjects SET code = 'B.Sunda' WHERE name = 'Bahasa Sunda' AND code IS NULL;

		-- Baris lama belum punya sort_order — pakai id (urutan input) sebagai default awal,
		-- guru bisa geser urutannya sendiri lewat Pengaturan > Data Pelajaran sesudahnya.
		UPDATE subjects SET sort_order = id WHERE sort_order IS NULL;
		UPDATE classes SET sort_order = id WHERE sort_order IS NULL;
		UPDATE school SET export_pin = '' WHERE export_pin IS NULL OR export_pin = '1234';
		",
	)
}

// Sesi dari versi sebelumnya belum punya salinan soal. Salinannya dibuat sekali dari Bank Soal saat
// ini; soal yang sudah terhapus memang tidak bisa dipulihkan dan dilewati.
fn snapshot_existing_sessions(conn: &Connection) -> rusqlite::Result<()> {
	let tx = conn.unchecked_transaction()?;
	let sessions: Vec<(i64, String)> = {
		let mut stmt = tx.prepare(
			"SELECT es.id, es.question_ids FROM exam_sessions es
			 WHERE NOT EXISTS (SELECT 1 FROM exam_session_questions q WHERE q.session_id = es.id)",
		)?;
		let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
		rows.collect::<rusqlite::Result<_>>()?
	};
	for (session_id, question_ids) in sessions {
		let ids: Vec<i64> = question_ids.split(',').filter_map(|s| s.parse().ok()).collect();
		crate::exam_session::snapshot_session(&tx, session_id, &ids)?;
	}
	tx.commit()
}

fn ensure_column(conn: &Connection, table: &str, column: &str, definition: &str) -> rusqlite::Result<()> {
	let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
	let exists = stmt.query_map([], |r| r.get::<_, String>(1))?.filter_map(Result::ok).any(|name| name == column);
	if !exists {
		conn.execute(&format!("ALTER TABLE {table} ADD COLUMN {column} {definition}"), [])?;
	}
	Ok(())
}

// `question_types.name` used to be globally UNIQUE, but the same jenis name (e.g. "Ulangan
// Harian") legitimately needs to exist independently per kelas + mata pelajaran once jenis
// creation became scoped — NULL on `class`/`subject_id` means "applies to every class/subject
// on that axis". SQLite can't ALTER a UNIQUE constraint directly, so this rebuilds the table
// when the old single-column-UNIQUE shape is detected (idempotent: skipped once already done).
fn relax_question_types_uniqueness(conn: &Connection) -> rusqlite::Result<()> {
	let mut stmt = conn.prepare("PRAGMA table_info(question_types)")?;
	let already_migrated = stmt.query_map([], |r| r.get::<_, String>(1))?.filter_map(Result::ok).any(|name| name == "class");
	if already_migrated {
		return Ok(());
	}

	conn.execute_batch(
		"
		CREATE TABLE question_types_new (
			id INTEGER PRIMARY KEY AUTOINCREMENT,
			name TEXT NOT NULL,
			description TEXT,
			class TEXT,
			subject_id INTEGER REFERENCES subjects(id) ON DELETE CASCADE,
			UNIQUE(name, class, subject_id)
		);
		INSERT INTO question_types_new (id, name, description) SELECT id, name, description FROM question_types;
		DROP TABLE question_types;
		ALTER TABLE question_types_new RENAME TO question_types;
		",
	)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn opens_and_responds() {
		let dir = std::env::temp_dir().join(format!("nuxtor_cbt_db_test_{}", std::process::id()));
		let db = open(&dir);
		let ok: i32 = db.lock().unwrap().query_row("SELECT 1", [], |row| row.get(0)).unwrap();
		assert_eq!(ok, 1);
		drop(db);
		let _ = std::fs::remove_dir_all(&dir);
	}

	fn temp_dir(tag: &str) -> std::path::PathBuf {
		let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
		std::env::temp_dir().join(format!("niluji_db_test_{tag}_{}_{nanos}", std::process::id()))
	}

	#[test]
	fn generated_pins_have_six_digits_and_secrets_are_long() {
		for _ in 0..200 {
			let pin = generate_random_pin();
			assert_eq!(pin.len(), 6);
			assert!(pin.chars().all(|c| c.is_ascii_digit()));
		}
		assert_eq!(generate_secret().len(), 64);
		assert_ne!(generate_secret(), generate_secret());
		assert!(secrets_match("abc", "abc"));
		assert!(!secrets_match("abc", "abd"));
		assert!(!secrets_match("abc", "abcd"));
	}

	#[test]
	fn backfilled_pin_is_random_and_old_formula_is_detected() {
		let dir = temp_dir("pin");
		let db = open(&dir);
		let id = {
			let conn = db.lock().unwrap();
			conn.execute("INSERT INTO students (nisn, name) VALUES ('0099887766', 'Lama')", []).unwrap();
			conn.last_insert_rowid()
		};
		drop(db);

		let db = open(&dir);
		let stored: String = db.lock().unwrap().query_row("SELECT pin FROM students WHERE id = ?1", [id], |r| r.get(0)).unwrap();
		assert_eq!(stored.len(), 6);
		assert_ne!(stored, legacy_pin_from_seed(id, "0099887766"));
		assert_eq!(count_legacy_pins(&db).unwrap(), 0);

		db.lock()
			.unwrap()
			.execute("UPDATE students SET pin = ?1 WHERE id = ?2", rusqlite::params![legacy_pin_from_seed(id, "0099887766"), id])
			.unwrap();
		assert_eq!(count_legacy_pins(&db).unwrap(), 1);
		drop(db);
		let _ = std::fs::remove_dir_all(&dir);
	}

	#[test]
	fn sessions_from_older_versions_get_a_snapshot_on_upgrade() {
		let dir = temp_dir("snapshot");
		let db = open(&dir);
		let session_id = {
			let conn = db.lock().unwrap();
			conn.execute("INSERT INTO subjects (name) VALUES ('Lama')", []).unwrap();
			let subject = conn.last_insert_rowid();
			conn.execute("INSERT INTO questions (subject_id, question_text, answer_key) VALUES (?1, 'Soal', 'A')", [subject]).unwrap();
			let question = conn.last_insert_rowid();
			conn.execute("INSERT INTO exams (subject_id, title, token) VALUES (?1, 'U', 'LAMA01')", [subject]).unwrap();
			let exam = conn.last_insert_rowid();
			conn.execute(
				"INSERT INTO exam_sessions (exam_id, student_nisn, student_name, question_ids, started_at) VALUES (?1, '1', 'A', ?2, 1)",
				rusqlite::params![exam, format!("{question},999999")],
			)
			.unwrap();
			conn.last_insert_rowid()
		};
		drop(db);

		let db = open(&dir);
		let count: i64 = db
			.lock()
			.unwrap()
			.query_row("SELECT COUNT(*) FROM exam_session_questions WHERE session_id = ?1", [session_id], |r| r.get(0))
			.unwrap();
		assert_eq!(count, 1);
		drop(db);
		let _ = std::fs::remove_dir_all(&dir);
	}
}
