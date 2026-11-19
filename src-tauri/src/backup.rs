// Backup database CBT memakai SQLite online backup API supaya tetap konsisten walau database
// sedang dipakai (mode WAL). Riwayat backup dibaca dari isi folder, bukan dari tabel di database
// yang sama, karena tabel itu ikut rusak kalau file database rusak.
//
// Ada tiga jenis cadangan dengan batas masing-masing, supaya backup harian tidak terdorong keluar
// oleh backup manual atau cadangan sebelum operasi berbahaya.
use crate::db::{self, Db};
use rusqlite::{backup::Backup, params, Connection};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const MANUAL_PREFIX: &str = "cbt-backup-";
const DAILY_PREFIX: &str = "cbt-harian-";
const SAFETY_PREFIX: &str = "cbt-sebelum-";

const KEEP_MANUAL: usize = 14;
const KEEP_DAILY: usize = 14;
const KEEP_SAFETY: usize = 5;

const PAGES_PER_STEP: i32 = 2_000;

fn backups_dir(data_dir: &Path) -> PathBuf {
	data_dir.join("backups")
}

fn now_secs() -> i64 {
	SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64
}

// Tanggal UTC YYYY-MM-DD dari jumlah detik Unix (algoritma civil-from-days).
fn date_stamp(unix_seconds: i64) -> String {
	let z = unix_seconds.div_euclid(86_400) + 719_468;
	let era = z.div_euclid(146_097);
	let doe = z.rem_euclid(146_097);
	let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
	let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
	let mp = (5 * doy + 2) / 153;
	let day = doy - (153 * mp + 2) / 5 + 1;
	let month = if mp < 10 { mp + 3 } else { mp - 9 };
	let year = yoe + era * 400 + i64::from(month <= 2);
	format!("{year:04}-{month:02}-{day:02}")
}

// Ditulis ke file sementara lalu diganti namanya, supaya kegagalan di tengah jalan tidak merusak
// backup yang sudah ada. Kunci database dilepas segera sesudah salinan selesai.
fn write_backup(db: &Db, dest_path: &Path) -> Result<(), String> {
	let temp_path = dest_path.with_extension("tmp");
	let _ = std::fs::remove_file(&temp_path);
	{
		let source = db.lock().unwrap();
		let mut dest = Connection::open(&temp_path).map_err(|e| format!("Gagal membuat file backup: {e}"))?;
		let backup = Backup::new(&source, &mut dest).map_err(|e| format!("Gagal memulai backup: {e}"))?;
		backup.run_to_completion(PAGES_PER_STEP, Duration::ZERO, None).map_err(|e| format!("Backup gagal: {e}"))?;
	}
	std::fs::rename(&temp_path, dest_path).map_err(|e| format!("Gagal menyimpan file backup: {e}"))
}

fn prune(dir: &Path, prefix: &str, keep: usize) -> Result<(), String> {
	let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)
		.map_err(|e| e.to_string())?
		.filter_map(|e| e.ok())
		.map(|e| e.path())
		.filter(|p| {
			p.extension().is_some_and(|ext| ext == "sqlite")
				&& p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with(prefix))
		})
		.collect();
	entries.sort();
	if entries.len() > keep {
		for old in &entries[..entries.len() - keep] {
			let _ = std::fs::remove_file(old);
		}
	}
	Ok(())
}

// Nama gambar soal unik dan tidak pernah diubah, jadi cukup menyalin file yang belum ada.
fn sync_uploads(data_dir: &Path) -> Result<(), String> {
	let source = data_dir.join("uploads");
	if !source.is_dir() {
		return Ok(());
	}
	let target = backups_dir(data_dir).join("uploads");
	std::fs::create_dir_all(&target).map_err(|e| format!("Gagal membuat folder backup gambar: {e}"))?;
	for entry in std::fs::read_dir(&source).map_err(|e| e.to_string())?.flatten() {
		let dest = target.join(entry.file_name());
		if entry.path().is_file() && !dest.exists() {
			std::fs::copy(entry.path(), dest).map_err(|e| format!("Gagal menyalin gambar ke backup: {e}"))?;
		}
	}
	Ok(())
}

fn create(db: &Db, data_dir: &Path, file_name: String, prefix: &str, keep: usize) -> Result<PathBuf, String> {
	let dir = backups_dir(data_dir);
	std::fs::create_dir_all(&dir).map_err(|e| format!("Gagal membuat folder backup: {e}"))?;
	let dest_path = dir.join(file_name);
	write_backup(db, &dest_path)?;
	prune(&dir, prefix, keep)?;
	sync_uploads(data_dir)?;
	Ok(dest_path)
}

pub fn backup_now(db: &Db, data_dir: &Path) -> Result<PathBuf, String> {
	create(db, data_dir, format!("{MANUAL_PREFIX}{}.sqlite", now_secs()), MANUAL_PREFIX, KEEP_MANUAL)
}

// Satu file per hari yang diperbarui selama aplikasi terbuka, disimpan selama 14 hari pemakaian.
pub fn backup_daily(db: &Db, data_dir: &Path) -> Result<PathBuf, String> {
	create(db, data_dir, format!("{DAILY_PREFIX}{}.sqlite", date_stamp(now_secs())), DAILY_PREFIX, KEEP_DAILY)
}

pub fn backup_before_change(db: &Db, data_dir: &Path) -> Result<PathBuf, String> {
	create(db, data_dir, format!("{SAFETY_PREFIX}{}.sqlite", now_secs()), SAFETY_PREFIX, KEEP_SAFETY)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupInfo {
	pub file_name: String,
	pub path: String,
	pub size_bytes: u64,
	pub created_at: i64,
	pub kind: String
}

fn kind_of(file_name: &str) -> &'static str {
	if file_name.starts_with(DAILY_PREFIX) {
		"harian"
	} else if file_name.starts_with(SAFETY_PREFIX) {
		"sebelum-perubahan"
	} else {
		"manual"
	}
}

pub fn describe(path: &Path) -> Result<BackupInfo, String> {
	let metadata = std::fs::metadata(path).map_err(|e| e.to_string())?;
	let file_name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
	let created_at = metadata.modified().ok().and_then(|t| t.duration_since(UNIX_EPOCH).ok()).map_or(0, |d| d.as_secs() as i64);
	Ok(BackupInfo {
		kind: kind_of(&file_name).to_string(),
		file_name,
		path: path.to_string_lossy().to_string(),
		size_bytes: metadata.len(),
		created_at
	})
}

pub fn list_backups(data_dir: &Path) -> Result<Vec<BackupInfo>, String> {
	let dir = backups_dir(data_dir);
	if !dir.exists() {
		return Ok(Vec::new());
	}

	let mut result: Vec<BackupInfo> = std::fs::read_dir(&dir)
		.map_err(|e| e.to_string())?
		.filter_map(|e| e.ok())
		.map(|entry| entry.path())
		.filter(|path| path.extension().is_some_and(|ext| ext == "sqlite"))
		.filter_map(|path| describe(&path).ok())
		.collect();
	result.sort_by_key(|b| std::cmp::Reverse(b.created_at));
	Ok(result)
}

fn resolve_backup_file(data_dir: &Path, file_name: &str) -> Result<PathBuf, String> {
	let is_plain_name = !file_name.is_empty()
		&& file_name.ends_with(".sqlite")
		&& file_name.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
		&& !file_name.contains("..");
	if !is_plain_name {
		return Err("Nama file backup tidak valid.".to_string());
	}
	let path = backups_dir(data_dir).join(file_name);
	if !path.is_file() {
		return Err("File backup tidak ditemukan.".to_string());
	}
	Ok(path)
}

fn active_session_count(conn: &Connection) -> Result<i64, String> {
	conn.query_row(
		"SELECT COUNT(*) FROM exam_sessions es
		 JOIN exams e ON e.id = es.exam_id
		 WHERE es.submitted_at IS NULL AND es.started_at + (e.duration + COALESCE(es.extra_minutes, 0)) * 60 > ?1",
		params![now_secs()],
		|r| r.get(0)
	)
	.map_err(|e| e.to_string())
}

// Memulihkan database dari backup. Kondisi saat ini dicadangkan lebih dulu, jadi pemulihan yang
// keliru masih bisa dibatalkan dengan memulihkan cadangan "sebelum perubahan".
pub fn restore_backup(db: &Db, data_dir: &Path, file_name: &str) -> Result<(), String> {
	let source_path = resolve_backup_file(data_dir, file_name)?;

	let source = Connection::open(&source_path).map_err(|e| format!("Gagal membuka backup: {e}"))?;
	let integrity: String =
		source.query_row("PRAGMA integrity_check", [], |r| r.get(0)).map_err(|e| format!("File backup tidak terbaca: {e}"))?;
	if integrity != "ok" {
		return Err("File backup rusak, pemulihan dibatalkan.".to_string());
	}
	source
		.query_row("SELECT COUNT(*) FROM exam_sessions", [], |r| r.get::<_, i64>(0))
		.map_err(|_| "File ini bukan backup NILUJI yang valid.".to_string())?;

	let active = active_session_count(&db.lock().unwrap())?;
	if active > 0 {
		return Err(format!("Masih ada {active} siswa yang sedang ujian. Tunggu sampai selesai sebelum memulihkan backup."));
	}

	backup_before_change(db, data_dir)?;

	let mut conn = db.lock().unwrap();
	{
		let backup = Backup::new(&source, &mut conn).map_err(|e| format!("Gagal memulai pemulihan: {e}"))?;
		backup.run_to_completion(PAGES_PER_STEP, Duration::ZERO, None).map_err(|e| format!("Pemulihan gagal: {e}"))?;
	}
	db::migrate(&conn).map_err(|e| format!("Gagal menyesuaikan struktur database: {e}"))
}

#[cfg(test)]
mod tests {
	use super::*;

	fn temp_dir(tag: &str) -> PathBuf {
		let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
		std::env::temp_dir().join(format!("niluji_backup_test_{tag}_{}_{nanos}", std::process::id()))
	}

	#[test]
	fn date_stamp_handles_known_dates() {
		assert_eq!(date_stamp(0), "1970-01-01");
		assert_eq!(date_stamp(951_782_400), "2000-02-29");
		assert_eq!(date_stamp(1_798_761_599), "2026-12-31");
	}

	#[test]
	fn daily_backup_is_replaced_within_the_same_day_and_kinds_are_listed() {
		let dir = temp_dir("daily");
		let db = db::open(&dir);
		let first = backup_daily(&db, &dir).unwrap();
		let second = backup_daily(&db, &dir).unwrap();
		assert_eq!(first, second);
		backup_now(&db, &dir).unwrap();

		let list = list_backups(&dir).unwrap();
		assert_eq!(list.len(), 2);
		assert!(list.iter().any(|b| b.kind == "harian"));
		assert!(list.iter().any(|b| b.kind == "manual"));
		drop(db);
		let _ = std::fs::remove_dir_all(&dir);
	}

	#[test]
	fn each_kind_is_pruned_separately() {
		let dir = temp_dir("prune");
		let backups = backups_dir(&dir);
		std::fs::create_dir_all(&backups).unwrap();
		for i in 0..20 {
			std::fs::write(backups.join(format!("{SAFETY_PREFIX}{i:02}.sqlite")), b"x").unwrap();
		}
		std::fs::write(backups.join(format!("{DAILY_PREFIX}2026-01-01.sqlite")), b"x").unwrap();
		prune(&backups, SAFETY_PREFIX, KEEP_SAFETY).unwrap();

		let names: Vec<String> = std::fs::read_dir(&backups).unwrap().flatten().map(|e| e.file_name().to_string_lossy().to_string()).collect();
		assert_eq!(names.iter().filter(|n| n.starts_with(SAFETY_PREFIX)).count(), KEEP_SAFETY);
		assert!(names.iter().any(|n| n.starts_with(DAILY_PREFIX)));
		assert!(names.contains(&format!("{SAFETY_PREFIX}19.sqlite")));
		let _ = std::fs::remove_dir_all(&dir);
	}

	#[test]
	fn uploads_are_copied_into_the_backup_folder() {
		let dir = temp_dir("uploads");
		let db = db::open(&dir);
		std::fs::create_dir_all(dir.join("uploads")).unwrap();
		std::fs::write(dir.join("uploads").join("1.png"), b"gambar").unwrap();
		backup_daily(&db, &dir).unwrap();
		assert!(backups_dir(&dir).join("uploads").join("1.png").is_file());
		drop(db);
		let _ = std::fs::remove_dir_all(&dir);
	}

	#[test]
	fn restore_brings_data_back_and_keeps_a_safety_copy() {
		let dir = temp_dir("restore");
		let db = db::open(&dir);
		db.lock().unwrap().execute("INSERT INTO students (nisn, name, pin) VALUES ('1', 'Asli', '111111')", []).unwrap();
		let backup = backup_now(&db, &dir).unwrap();
		db.lock().unwrap().execute("DELETE FROM students", []).unwrap();

		restore_backup(&db, &dir, backup.file_name().unwrap().to_str().unwrap()).unwrap();

		let count: i64 = db.lock().unwrap().query_row("SELECT COUNT(*) FROM students WHERE name = 'Asli'", [], |r| r.get(0)).unwrap();
		assert_eq!(count, 1);
		assert!(list_backups(&dir).unwrap().iter().any(|b| b.kind == "sebelum-perubahan"));
		drop(db);
		let _ = std::fs::remove_dir_all(&dir);
	}

	#[test]
	fn restore_rejects_paths_outside_the_backup_folder_and_invalid_files() {
		let dir = temp_dir("restore_bad");
		let db = db::open(&dir);
		assert!(restore_backup(&db, &dir, "../cbt.sqlite").is_err());
		assert!(restore_backup(&db, &dir, "C:\\Windows\\x.sqlite").is_err());

		std::fs::create_dir_all(backups_dir(&dir)).unwrap();
		std::fs::write(backups_dir(&dir).join("cbt-backup-1.sqlite"), b"bukan database").unwrap();
		assert!(restore_backup(&db, &dir, "cbt-backup-1.sqlite").is_err());
		drop(db);
		let _ = std::fs::remove_dir_all(&dir);
	}

	#[test]
	fn restore_is_refused_while_students_are_taking_an_exam() {
		let dir = temp_dir("restore_active");
		let db = db::open(&dir);
		let backup = backup_now(&db, &dir).unwrap();
		{
			let conn = db.lock().unwrap();
			conn.execute("INSERT INTO subjects (name) VALUES ('Uji')", []).unwrap();
			let subject = conn.last_insert_rowid();
			conn.execute("INSERT INTO exams (subject_id, title, duration, question_count, token) VALUES (?1, 'U', 60, 1, 'TOKEN1')", params![subject]).unwrap();
			let exam = conn.last_insert_rowid();
			conn.execute(
				"INSERT INTO exam_sessions (exam_id, student_nisn, student_name, question_ids, started_at) VALUES (?1, '1', 'A', '', ?2)",
				params![exam, now_secs()]
			)
			.unwrap();
		}
		let err = restore_backup(&db, &dir, backup.file_name().unwrap().to_str().unwrap()).unwrap_err();
		assert!(err.contains("sedang ujian"));
		drop(db);
		let _ = std::fs::remove_dir_all(&dir);
	}
}
