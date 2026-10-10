use rusqlite::types::{Value as SqlValue, ValueRef};
use rusqlite::{params_from_iter, Connection};
use serde::Deserialize;
use serde_json::{Map, Value};
use std::path::Path;
use std::sync::Mutex;

pub type Db = Mutex<Connection>;

// Akun bawaan sama dengan e-Rapor SD resmi (Panduan hal. 16), supaya kebiasaan operator tetap terpakai.
pub const DEFAULT_ADMIN_USER: &str = "administrator";
pub const DEFAULT_ADMIN_PASS: &str = "administrator";

pub fn open(data_dir: &Path) -> Db {
	std::fs::create_dir_all(data_dir).expect("gagal membuat folder data aplikasi");
	let conn = Connection::open(data_dir.join("erapor.sqlite")).expect("gagal membuka database");
	siapkan(&conn).expect("gagal menyiapkan database");
	reset_darurat(&conn, data_dir);
	Mutex::new(conn)
}

// Jalur darurat kalau semua admin lupa password: buat file RESET-ADMIN.txt (isi bebas) di folder data
// aplikasi, lalu buka aplikasi. Akun "administrator" dipulihkan ke password bawaan dan diaktifkan,
// lalu file dihapus. Butuh akses langsung ke laptop, jadi tidak bisa dipakai dari jaringan.
fn reset_darurat(conn: &Connection, data_dir: &Path) {
	let file = data_dir.join("RESET-ADMIN.txt");
	if !file.exists() {
		return;
	}
	let Ok(hash) = hash_password(DEFAULT_ADMIN_PASS) else { return };
	let ok = conn.execute(
		"INSERT INTO users (username, password_hash, nama, level, aktif) VALUES (?1, ?2, 'Administrator', 'admin', 1)
		ON CONFLICT (username) DO UPDATE SET password_hash = excluded.password_hash, level = 'admin', aktif = 1, online = 0",
		rusqlite::params![DEFAULT_ADMIN_USER, hash],
	);
	if ok.is_ok() {
		let _ = std::fs::remove_file(&file);
	}
}

// Migrasi + akun admin bawaan. Dipanggil saat aplikasi dibuka dan setelah restore backup
// (backup lama mungkin belum punya tabel/kolom terbaru).
pub fn siapkan(conn: &Connection) -> Result<(), String> {
	let e = |e: rusqlite::Error| e.to_string();
	migrate(conn).map_err(e)?;
	migrate_sinkron(conn).map_err(e)?;
	migrate_koreksi(conn).map_err(e)?;
	migrate_walas(conn).map_err(e)?;
	// Deskripsi kokurikuler di rapor (NULL = kalimat otomatis dari capaian).
	add_column(conn, "rapor_siswa", "kokurikuler", "TEXT").map_err(e)?;
	// Subdimensi per dimensi kegiatan kokurikuler: JSON {kode_dimensi: [nama subdimensi, ...]} (opsional).
	add_column(conn, "kokurikuler_kegiatan", "subdimensi", "TEXT NOT NULL DEFAULT '{}'").map_err(e)?;
	// Mapel ikut transkrip nilai ijazah atau tidak (padanan "Mapel Transkrip" di e-Rapor).
	add_column(conn, "mapel_rapor", "transkrip", "INTEGER NOT NULL DEFAULT 1").map_err(e)?;
	seed_admin(conn)?;
	// Aplikasi desktop baru dibuka = belum ada yang login. Tanpa ini, akun yang aplikasinya
	// ditutup tanpa Keluar akan tercatat "online" selamanya.
	conn.execute("UPDATE users SET online = 0 WHERE online <> 0", []).map_err(e)?;
	Ok(())
}

fn migrate(conn: &Connection) -> rusqlite::Result<()> {
	conn.execute_batch(
		"
		PRAGMA journal_mode = WAL;
		PRAGMA synchronous = NORMAL;

		-- Pengaturan bebas (status input nilai, dll.)
		CREATE TABLE IF NOT EXISTS settings (
			key TEXT PRIMARY KEY,
			value TEXT
		);

		-- Menu Web Service Dapodik
		CREATE TABLE IF NOT EXISTS webservice (
			id INTEGER PRIMARY KEY AUTOINCREMENT,
			nama_aplikasi TEXT NOT NULL,
			ip_erapor TEXT NOT NULL DEFAULT 'localhost',
			ip_dapodik TEXT NOT NULL DEFAULT 'localhost',
			port INTEGER NOT NULL DEFAULT 5774,
			token TEXT NOT NULL,
			npsn TEXT NOT NULL
		);

		CREATE TABLE IF NOT EXISTS users (
			id INTEGER PRIMARY KEY AUTOINCREMENT,
			username TEXT NOT NULL UNIQUE COLLATE NOCASE,
			password_hash TEXT NOT NULL,
			nama TEXT NOT NULL,
			email TEXT,
			level TEXT NOT NULL CHECK (level IN ('admin','guru','siswa')),
			ptk_id TEXT,
			peserta_didik_id TEXT,
			aktif INTEGER NOT NULL DEFAULT 1,
			online INTEGER NOT NULL DEFAULT 0,
			last_login TEXT,
			created_at TEXT NOT NULL DEFAULT (datetime('now','localtime'))
		);

		-- ===== Data dari Dapodik (master, read-only kecuali kolom lokal) =====
		-- Kolom raw menyimpan JSON utuh dari Web Service supaya tidak ada field yang hilang.

		CREATE TABLE IF NOT EXISTS semester (
			semester_id TEXT PRIMARY KEY,
			tahun_ajaran TEXT NOT NULL,
			nama TEXT NOT NULL,
			aktif INTEGER NOT NULL DEFAULT 0,
			synced_at TEXT
		);

		CREATE TABLE IF NOT EXISTS sekolah (
			sekolah_id TEXT PRIMARY KEY,
			npsn TEXT NOT NULL,
			nama TEXT NOT NULL,
			alamat TEXT,
			kecamatan TEXT,
			kabupaten_kota TEXT,
			provinsi TEXT,
			email TEXT,
			-- lokal (tidak ada di Web Service): kepala sekolah dipilih dari GTK
			kepsek_ptk_id TEXT,
			raw TEXT,
			synced_at TEXT
		);

		CREATE TABLE IF NOT EXISTS ptk (
			ptk_id TEXT PRIMARY KEY,
			nama TEXT NOT NULL,
			nip TEXT,
			nuptk TEXT,
			nik TEXT,
			jenis_kelamin TEXT,
			tempat_lahir TEXT,
			tanggal_lahir TEXT,
			jenis_ptk TEXT,
			jabatan_ptk TEXT,
			status_kepegawaian TEXT,
			-- lokal
			gelar_depan TEXT,
			gelar_belakang TEXT,
			raw TEXT,
			synced_at TEXT
		);

		CREATE TABLE IF NOT EXISTS peserta_didik (
			peserta_didik_id TEXT PRIMARY KEY,
			nama TEXT NOT NULL,
			nisn TEXT,
			nipd TEXT,
			nik TEXT,
			jenis_kelamin TEXT,
			tempat_lahir TEXT,
			tanggal_lahir TEXT,
			agama TEXT,
			nama_ayah TEXT,
			nama_ibu TEXT,
			raw TEXT,
			synced_at TEXT
		);

		CREATE TABLE IF NOT EXISTS rombel (
			rombongan_belajar_id TEXT PRIMARY KEY,
			semester_id TEXT NOT NULL,
			nama TEXT NOT NULL,
			tingkat TEXT,
			jenis_rombel TEXT,
			jenis_rombel_str TEXT,
			kurikulum TEXT,
			ptk_id TEXT,
			raw TEXT,
			synced_at TEXT
		);
		CREATE INDEX IF NOT EXISTS idx_rombel_semester ON rombel(semester_id);

		CREATE TABLE IF NOT EXISTS anggota_rombel (
			anggota_rombel_id TEXT PRIMARY KEY,
			rombongan_belajar_id TEXT NOT NULL,
			peserta_didik_id TEXT NOT NULL,
			semester_id TEXT NOT NULL,
			jenis_pendaftaran TEXT
		);
		CREATE INDEX IF NOT EXISTS idx_anggota_rombel ON anggota_rombel(rombongan_belajar_id);
		CREATE INDEX IF NOT EXISTS idx_anggota_pd ON anggota_rombel(peserta_didik_id);

		CREATE TABLE IF NOT EXISTS pembelajaran (
			pembelajaran_id TEXT PRIMARY KEY,
			rombongan_belajar_id TEXT NOT NULL,
			semester_id TEXT NOT NULL,
			mata_pelajaran_id TEXT,
			mata_pelajaran_str TEXT,
			nama_mata_pelajaran TEXT,
			ptk_id TEXT,
			jam_per_minggu INTEGER,
			raw TEXT
		);
		CREATE INDEX IF NOT EXISTS idx_pemb_rombel ON pembelajaran(rombongan_belajar_id);
		CREATE INDEX IF NOT EXISTS idx_pemb_ptk ON pembelajaran(ptk_id);

		CREATE TABLE IF NOT EXISTS mata_pelajaran (
			mata_pelajaran_id TEXT PRIMARY KEY,
			nama TEXT NOT NULL,
			raw TEXT
		);

		-- ===== Data lokal e-Rapor =====

		-- Mapel yang tampil di rapor. kode = mata_pelajaran_id Dapodik (untuk kirim nilai nanti).
		CREATE TABLE IF NOT EXISTS mapel_rapor (
			kode TEXT PRIMARY KEY,
			nama TEXT NOT NULL,
			singkat TEXT NOT NULL,
			kelompok TEXT NOT NULL DEFAULT 'Mata Pelajaran Wajib',
			urutan INTEGER NOT NULL DEFAULT 99
		);

		-- Pembelajaran per rombel yang muncul di rapor. Di SD, Dapodik hanya mencatat satu
		-- pembelajaran 'Guru Kelas SD/MI/SLB'; di sini dipecah per mapel (sumber = 'sub').
		CREATE TABLE IF NOT EXISTS pembelajaran_rapor (
			id INTEGER PRIMARY KEY AUTOINCREMENT,
			semester_id TEXT NOT NULL,
			rombongan_belajar_id TEXT NOT NULL,
			kode_mapel TEXT NOT NULL,
			ptk_id TEXT,
			sumber TEXT NOT NULL DEFAULT 'sub',
			pembelajaran_id TEXT,
			UNIQUE (semester_id, rombongan_belajar_id, kode_mapel)
		);
		CREATE INDEX IF NOT EXISTS idx_pr_ptk ON pembelajaran_rapor(ptk_id, semester_id);

		-- Bank Tujuan Pembelajaran: per mapel + tingkat + semester (1 ganjil, 2 genap),
		-- tidak terikat tahun ajaran, jadi otomatis terpakai lagi tahun berikutnya.
		CREATE TABLE IF NOT EXISTS tujuan_pembelajaran (
			id INTEGER PRIMARY KEY AUTOINCREMENT,
			kode_mapel TEXT NOT NULL,
			tingkat INTEGER NOT NULL,
			semester INTEGER NOT NULL,
			deskripsi TEXT NOT NULL,
			urutan INTEGER NOT NULL DEFAULT 0,
			created_at TEXT NOT NULL DEFAULT (datetime('now','localtime'))
		);
		CREATE INDEX IF NOT EXISTS idx_tp ON tujuan_pembelajaran(kode_mapel, tingkat, semester);

		-- Nilai akhir rapor + TP yang dicapai optimal / perlu pendampingan (JSON array id TP).
		CREATE TABLE IF NOT EXISTS nilai_rapor (
			pembelajaran_rapor_id INTEGER NOT NULL,
			peserta_didik_id TEXT NOT NULL,
			nilai INTEGER,
			tp_optimal TEXT NOT NULL DEFAULT '[]',
			tp_perlu TEXT NOT NULL DEFAULT '[]',
			updated_at TEXT NOT NULL DEFAULT (datetime('now','localtime')),
			PRIMARY KEY (pembelajaran_rapor_id, peserta_didik_id)
		);
		",
	)
}

fn add_column(conn: &Connection, table: &str, column: &str, decl: &str) -> rusqlite::Result<()> {
	let exists: bool = conn
		.prepare(&format!("SELECT 1 FROM pragma_table_info('{table}') WHERE name = ?1"))?
		.exists([column])?;
	if !exists {
		conn.execute_batch(&format!("ALTER TABLE {table} ADD COLUMN {column} {decl}"))?;
	}
	Ok(())
}

// Kolom untuk sinkron admin ↔ laptop guru (ditambahkan setelah versi awal, jadi lewat ALTER).
fn migrate_sinkron(conn: &Connection) -> rusqlite::Result<()> {
	// TP butuh id yang sama di semua laptop: id angka beda tiap database, uid tidak.
	add_column(conn, "tujuan_pembelajaran", "uid", "TEXT")?;
	add_column(conn, "tujuan_pembelajaran", "sumber", "TEXT NOT NULL DEFAULT 'admin'")?;
	// Setelah guru mengirim nilai, pembelajaran dikunci sampai admin membukanya lagi.
	add_column(conn, "pembelajaran_rapor", "terkunci", "INTEGER NOT NULL DEFAULT 0")?;
	add_column(conn, "pembelajaran_rapor", "dikirim_at", "TEXT")?;
	conn.execute_batch(
		"
		UPDATE tujuan_pembelajaran SET uid = lower(hex(randomblob(16))) WHERE uid IS NULL;
		CREATE UNIQUE INDEX IF NOT EXISTS idx_tp_uid ON tujuan_pembelajaran(uid);
		CREATE TRIGGER IF NOT EXISTS trg_tp_uid AFTER INSERT ON tujuan_pembelajaran
		WHEN NEW.uid IS NULL BEGIN
			UPDATE tujuan_pembelajaran SET uid = lower(hex(randomblob(16))) WHERE id = NEW.id;
		END;
		",
	)
}

// Koreksi data siswa yang salah input di Dapodik: hanya berlaku di e-Rapor (tampilan & cetak),
// tidak dikirim ke Dapodik dan tidak terhapus saat sinkron ulang. Kolom NULL = pakai data Dapodik.
// View siswa_rapor = data yang dipakai rapor (koreksi kalau ada, kalau tidak data Dapodik).
fn migrate_koreksi(conn: &Connection) -> rusqlite::Result<()> {
	conn.execute_batch(
		"
		CREATE TABLE IF NOT EXISTS koreksi_siswa (
			peserta_didik_id TEXT PRIMARY KEY,
			nama TEXT,
			nisn TEXT,
			nipd TEXT,
			jenis_kelamin TEXT,
			tempat_lahir TEXT,
			tanggal_lahir TEXT,
			agama TEXT,
			nama_ayah TEXT,
			nama_ibu TEXT,
			catatan TEXT,
			updated_at TEXT NOT NULL DEFAULT (datetime('now','localtime')),
			updated_by TEXT
		);

		CREATE VIEW IF NOT EXISTS siswa_rapor AS
		SELECT p.peserta_didik_id,
			COALESCE(k.nama, p.nama) AS nama,
			COALESCE(k.nisn, p.nisn) AS nisn,
			COALESCE(k.nipd, p.nipd) AS nipd,
			COALESCE(k.jenis_kelamin, p.jenis_kelamin) AS jenis_kelamin,
			COALESCE(k.tempat_lahir, p.tempat_lahir) AS tempat_lahir,
			COALESCE(k.tanggal_lahir, p.tanggal_lahir) AS tanggal_lahir,
			COALESCE(k.agama, p.agama) AS agama,
			COALESCE(k.nama_ayah, p.nama_ayah) AS nama_ayah,
			COALESCE(k.nama_ibu, p.nama_ibu) AS nama_ibu
		FROM peserta_didik p LEFT JOIN koreksi_siswa k USING (peserta_didik_id);
		",
	)
}

// Isian wali kelas per siswa per semester (padanan Input Kelengkapan e-Rapor):
// kehadiran, catatan wali kelas, dan keputusan kenaikan (hanya semester genap). Juga nilai ekskul.
// naik: NULL = belum diputuskan, 1 = naik/lulus, 0 = tinggal kelas/tidak lulus.
fn migrate_walas(conn: &Connection) -> rusqlite::Result<()> {
	conn.execute_batch(
		"
		CREATE TABLE IF NOT EXISTS rapor_siswa (
			semester_id TEXT NOT NULL,
			peserta_didik_id TEXT NOT NULL,
			rombongan_belajar_id TEXT NOT NULL,
			sakit INTEGER,
			izin INTEGER,
			alpa INTEGER,
			catatan TEXT,
			naik INTEGER,
			updated_at TEXT NOT NULL DEFAULT (datetime('now','localtime')),
			PRIMARY KEY (semester_id, peserta_didik_id)
		);
		CREATE INDEX IF NOT EXISTS idx_rapor_siswa_rombel ON rapor_siswa(rombongan_belajar_id);

		-- Nilai ekstrakurikuler oleh pembina (rombel ekskul = jenis_rombel 51).
		-- predikat: SB / B / C / K. keterangan NULL = kalimat otomatis dari predikat saat dicetak.
		CREATE TABLE IF NOT EXISTS nilai_ekskul (
			semester_id TEXT NOT NULL,
			rombongan_belajar_id TEXT NOT NULL,
			peserta_didik_id TEXT NOT NULL,
			predikat TEXT,
			keterangan TEXT,
			updated_at TEXT NOT NULL DEFAULT (datetime('now','localtime')),
			PRIMARY KEY (semester_id, rombongan_belajar_id, peserta_didik_id)
		);

		-- Kokurikuler (versi ringkas e-Rapor 2025): kegiatan per semester untuk tingkat tertentu,
		-- dinilai wali kelas per dimensi profil lulusan. tingkat & dimensi = JSON array.
		CREATE TABLE IF NOT EXISTS kokurikuler_kegiatan (
			id INTEGER PRIMARY KEY AUTOINCREMENT,
			semester_id TEXT NOT NULL,
			nama TEXT NOT NULL,
			tema TEXT,
			tujuan TEXT,
			tingkat TEXT NOT NULL DEFAULT '[]',
			dimensi TEXT NOT NULL DEFAULT '[]',
			urutan INTEGER NOT NULL DEFAULT 0
		);
		-- Foto siswa untuk pelengkap rapor (data URL JPEG kecil, ±30 KB).
		CREATE TABLE IF NOT EXISTS foto_siswa (
			peserta_didik_id TEXT PRIMARY KEY,
			foto TEXT NOT NULL,
			updated_at TEXT NOT NULL DEFAULT (datetime('now','localtime'))
		);
		-- Transkrip nilai ijazah (kelas 6, semester genap): nomor ijazah + nilai per mapel transkrip.
		CREATE TABLE IF NOT EXISTS transkrip_siswa (
			peserta_didik_id TEXT PRIMARY KEY,
			no_ijazah TEXT,
			no_transkrip TEXT,
			tanggal_lulus TEXT
		);
		CREATE TABLE IF NOT EXISTS nilai_transkrip (
			peserta_didik_id TEXT NOT NULL,
			kode_mapel TEXT NOT NULL,
			nilai REAL,
			PRIMARY KEY (peserta_didik_id, kode_mapel)
		);
		-- capaian = JSON {kode_dimensi: 1 Berkembang | 2 Cakap | 3 Mahir}
		CREATE TABLE IF NOT EXISTS nilai_kokurikuler (
			kegiatan_id INTEGER NOT NULL,
			peserta_didik_id TEXT NOT NULL,
			capaian TEXT NOT NULL DEFAULT '{}',
			updated_at TEXT NOT NULL DEFAULT (datetime('now','localtime')),
			PRIMARY KEY (kegiatan_id, peserta_didik_id)
		);
		",
	)
}

fn seed_admin(conn: &Connection) -> Result<(), String> {
	let count: i64 = conn
		.query_row("SELECT COUNT(*) FROM users WHERE level = 'admin'", [], |r| r.get(0))
		.map_err(|e| e.to_string())?;
	if count == 0 {
		let hash = hash_password(DEFAULT_ADMIN_PASS)?;
		conn.execute(
			"INSERT INTO users (username, password_hash, nama, level) VALUES (?1, ?2, 'Administrator', 'admin')",
			rusqlite::params![DEFAULT_ADMIN_USER, hash],
		)
		.map_err(|e| e.to_string())?;
	}
	Ok(())
}

// Password bebas (keputusan sekolah: supaya mudah diingat guru), asal tidak kosong.
// Kalau nanti ingin aturan panjang minimal, cukup ubah di sini — semua perintah lewat fungsi ini.
pub fn cek_password(_level: &str, password: &str) -> Result<(), String> {
	if password.is_empty() {
		return Err("Password tidak boleh kosong".into());
	}
	Ok(())
}

// Cost 10 = setara default e-Rapor/Dapodik ($2y$10$...).
pub fn hash_password(plain: &str) -> Result<String, String> {
	bcrypt::hash(plain, 10).map_err(|e| e.to_string())
}

pub fn verify_password(plain: &str, hash: &str) -> bool {
	bcrypt::verify(plain, hash).unwrap_or(false)
}

// ===== Query generik untuk frontend =====
// Aplikasi ini desktop (webview lokal = kode kita sendiri), jadi frontend boleh kirim SQL
// berparameter. Kalau nanti dibuka sebagai server LAN, perintah ini TIDAK boleh diekspos ke
// klien jaringan — ganti dengan endpoint per fitur.

fn to_sql(v: &Value) -> SqlValue {
	match v {
		Value::Null => SqlValue::Null,
		Value::Bool(b) => SqlValue::Integer(*b as i64),
		Value::Number(n) => n.as_i64().map(SqlValue::Integer).unwrap_or_else(|| SqlValue::Real(n.as_f64().unwrap_or(0.0))),
		Value::String(s) => SqlValue::Text(s.clone()),
		other => SqlValue::Text(other.to_string()),
	}
}

fn from_sql(v: ValueRef) -> Value {
	match v {
		ValueRef::Null => Value::Null,
		ValueRef::Integer(i) => Value::from(i),
		ValueRef::Real(f) => Value::from(f),
		ValueRef::Text(t) => Value::String(String::from_utf8_lossy(t).into_owned()),
		ValueRef::Blob(_) => Value::Null,
	}
}

pub fn query(conn: &Connection, sql: &str, params: &[Value]) -> Result<Vec<Map<String, Value>>, String> {
	let mut stmt = conn.prepare(sql).map_err(|e| format!("{e} — SQL: {sql}"))?;
	let names: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();
	let rows = stmt
		.query_map(params_from_iter(params.iter().map(to_sql)), |row| {
			let mut obj = Map::new();
			for (i, name) in names.iter().enumerate() {
				obj.insert(name.clone(), from_sql(row.get_ref(i)?));
			}
			Ok(obj)
		})
		.map_err(|e| e.to_string())?;
	rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

pub fn execute(conn: &Connection, sql: &str, params: &[Value]) -> Result<usize, String> {
	conn.execute(sql, params_from_iter(params.iter().map(to_sql))).map_err(|e| format!("{e} — SQL: {sql}"))
}

#[derive(Debug, Deserialize)]
pub struct Statement {
	pub sql: String,
	#[serde(default)]
	pub params: Vec<Value>,
}

// Semua statement dalam satu transaksi: gagal satu, batal semua (dipakai saat tarik data Dapodik).
pub fn batch(conn: &mut Connection, stmts: &[Statement]) -> Result<usize, String> {
	let tx = conn.transaction().map_err(|e| e.to_string())?;
	let mut total = 0;
	for s in stmts {
		total += tx
			.execute(&s.sql, params_from_iter(s.params.iter().map(to_sql)))
			.map_err(|e| format!("{e} — SQL: {}", s.sql))?;
	}
	tx.commit().map_err(|e| e.to_string())?;
	Ok(total)
}
