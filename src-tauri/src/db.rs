use rusqlite::Connection;
use std::path::Path;
use std::sync::{Arc, Mutex};

// ponytail: single Mutex<Connection>, fine for one exam room's concurrency;
// switch to a connection pool (r2d2_sqlite) if multiple exam sessions hammer it at once.
pub type Db = Arc<Mutex<Connection>>;

pub fn ping(db: &Db) -> bool {
	db.lock().unwrap().query_row("SELECT 1", [], |row| row.get::<_, i32>(0)).is_ok()
}

pub fn find_student(db: &Db, nisn: &str, name: &str) -> Option<String> {
	db.lock()
		.unwrap()
		.query_row(
			"SELECT name FROM students WHERE nisn = ?1 AND name = ?2 COLLATE NOCASE",
			rusqlite::params![nisn.trim(), name.trim()],
			|r| r.get(0),
		)
		.ok()
}

pub fn open(data_dir: &Path) -> Db {
	std::fs::create_dir_all(data_dir).expect("failed to create app data dir");
	let conn = Connection::open(data_dir.join("cbt.sqlite")).expect("failed to open sqlite db");
	migrate(&conn).expect("failed to run sqlite migrations");
	Arc::new(Mutex::new(conn))
}

fn migrate(conn: &Connection) -> rusqlite::Result<()> {
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
		",
	)?;

	// ponytail: dummy roster so the student login page has something to test against
	// before real import (CSV/Dapodik) exists. Remove once that ships.
	conn.execute_batch(
		"
		INSERT OR IGNORE INTO students (nisn, name) VALUES ('0012345678', 'Ahmad Fauzi');
		INSERT OR IGNORE INTO students (nisn, name) VALUES ('0012345679', 'Siti Aminah');
		INSERT OR IGNORE INTO students (nisn, name) VALUES ('0012345680', 'Budi Santoso');
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
	// Teacher-assigned points for one essay answer; never rolled into the automatic score.
	ensure_column(conn, "exam_answers", "essay_score", "REAL")?;
	// PIN buat gerbang fitur Export soal — supaya tidak sembarang guru di komputer yang sama bisa
	// mengekspor bank soal ke file. Default "1234" biar tidak langsung terkunci di instalasi baru;
	// admin sekolah ganti sendiri lewat Pengaturan > Data Sekolah.
	ensure_column(conn, "school", "export_pin", "TEXT")?;
	// Acak urutan soal per siswa saat ujian ini dimulai — dipisah PG dan esai (guru bisa mau
	// PG-nya diacak tapi esainya tidak, atau sebaliknya). "randomize" (kolom lama, sekarang
	// tidak dipakai lagi) jadi sumber nilai awal keduanya biar ujian yang sudah dibuat sebelum
	// pemisahan ini tetap berperilaku sama seperti sebelumnya.
	ensure_column(conn, "exams", "randomize", "INTEGER")?;
	ensure_column(conn, "exams", "randomize_pg", "INTEGER")?;
	ensure_column(conn, "exams", "randomize_essay", "INTEGER")?;
	conn.execute("UPDATE exams SET randomize_pg = COALESCE(randomize, 1) WHERE randomize_pg IS NULL", [])?;
	conn.execute("UPDATE exams SET randomize_essay = COALESCE(randomize, 1) WHERE randomize_essay IS NULL", [])?;

	relax_question_types_uniqueness(conn)?;

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
		UPDATE school SET export_pin = '1234' WHERE export_pin IS NULL;
		",
	)
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
}
