// Backup database CBT — pakai SQLite online backup API (bukan sekadar salin file mentah),
// supaya tetap konsisten walau lagi ada koneksi lain yang menulis (mode WAL). Riwayat backup
// TIDAK disimpan sebagai tabel di dalam database yang sama (kalau file databasenya rusak,
// baris riwayat itu ikut rusak juga) — daftar backup yang ada cukup dibaca langsung dari isi
// foldernya tiap kali dibutuhkan.
use crate::db::Db;
use rusqlite::{backup::Backup, Connection};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const KEEP_LAST_N: usize = 14;

fn backups_dir(data_dir: &Path) -> PathBuf {
	data_dir.join("backups")
}

fn now_secs() -> i64 {
	SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64
}

// Backup baru + buang backup lama di luar KEEP_LAST_N (diurut dari nama file, yang sudah
// berisi timestamp menaik) — supaya folder backup tidak numpuk tanpa batas di laptop guru.
pub fn backup_now(db: &Db, data_dir: &Path) -> Result<PathBuf, String> {
	let dir = backups_dir(data_dir);
	std::fs::create_dir_all(&dir).map_err(|e| format!("Gagal membuat folder backup: {e}"))?;

	let dest_path = dir.join(format!("cbt-backup-{}.sqlite", now_secs()));

	{
		let source = db.lock().unwrap();
		let mut dest = Connection::open(&dest_path).map_err(|e| format!("Gagal membuat file backup: {e}"))?;
		let backup = Backup::new(&source, &mut dest).map_err(|e| format!("Gagal memulai backup: {e}"))?;
		backup.run_to_completion(5, std::time::Duration::from_millis(50), None).map_err(|e| format!("Backup gagal: {e}"))?;
	}

	let mut entries: Vec<PathBuf> = std::fs::read_dir(&dir)
		.map_err(|e| e.to_string())?
		.filter_map(|e| e.ok())
		.map(|e| e.path())
		.filter(|p| p.extension().is_some_and(|ext| ext == "sqlite"))
		.collect();
	entries.sort();
	if entries.len() > KEEP_LAST_N {
		for old in &entries[..entries.len() - KEEP_LAST_N] {
			let _ = std::fs::remove_file(old);
		}
	}

	Ok(dest_path)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupInfo {
	pub file_name: String,
	pub path: String,
	pub size_bytes: u64,
	pub created_at: i64
}

pub fn list_backups(data_dir: &Path) -> Result<Vec<BackupInfo>, String> {
	let dir = backups_dir(data_dir);
	if !dir.exists() {
		return Ok(Vec::new());
	}

	let mut result: Vec<BackupInfo> = std::fs::read_dir(&dir)
		.map_err(|e| e.to_string())?
		.filter_map(|e| e.ok())
		.filter_map(|entry| {
			let path = entry.path();
			if path.extension().is_none_or(|ext| ext != "sqlite") {
				return None;
			}
			let metadata = entry.metadata().ok()?;
			let created_at = metadata.modified().ok().and_then(|t| t.duration_since(UNIX_EPOCH).ok())?.as_secs() as i64;
			Some(BackupInfo {
				file_name: path.file_name()?.to_string_lossy().to_string(),
				path: path.to_string_lossy().to_string(),
				size_bytes: metadata.len(),
				created_at
			})
		})
		.collect();
	result.sort_by_key(|b| std::cmp::Reverse(b.created_at));
	Ok(result)
}
