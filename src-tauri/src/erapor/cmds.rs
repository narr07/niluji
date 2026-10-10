use crate::erapor::db;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::Manager;

// ===== Database =====

#[tauri::command]
pub fn db_query(db: tauri::State<db::Db>, sql: String, params: Option<Vec<Value>>) -> Result<Vec<Map<String, Value>>, String> {
	db::query(&db.lock().unwrap(), &sql, &params.unwrap_or_default())
}

#[tauri::command]
pub fn db_execute(db: tauri::State<db::Db>, sql: String, params: Option<Vec<Value>>) -> Result<usize, String> {
	db::execute(&db.lock().unwrap(), &sql, &params.unwrap_or_default())
}

#[tauri::command]
pub fn db_batch(db: tauri::State<db::Db>, statements: Vec<db::Statement>) -> Result<usize, String> {
	db::batch(&mut db.lock().unwrap(), &statements)
}

// ===== Login & pengguna =====
// Hash password tidak pernah dikirim ke frontend; verifikasi dan hashing hanya di sini.

const USER_COLUMNS: &str = "id, username, nama, email, level, ptk_id, peserta_didik_id, aktif, online, last_login";

#[tauri::command]
pub async fn auth_login(app: tauri::AppHandle, username: String, password: String) -> Result<Map<String, Value>, String> {
	// bcrypt butuh beberapa ratus ms — jangan di thread utama supaya UI tidak beku.
	tauri::async_runtime::spawn_blocking(move || {
		let state = app.state::<db::Db>();
		let conn = state.lock().unwrap();
		let found: Option<(i64, String, i64)> = conn
			.query_row(
				"SELECT id, password_hash, aktif FROM users WHERE username = ?1",
				[username.trim()],
				|r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
			)
			.ok();
		let Some((id, hash, aktif)) = found else {
			return Err("Username atau password salah".to_string());
		};
		if !db::verify_password(&password, &hash) {
			return Err("Username atau password salah".into());
		}
		if aktif == 0 {
			return Err("Akun dinonaktifkan. Hubungi administrator.".into());
		}
		conn.execute("UPDATE users SET online = 1, last_login = datetime('now','localtime') WHERE id = ?1", [id])
			.map_err(|e| e.to_string())?;
		db::query(&conn, &format!("SELECT {USER_COLUMNS} FROM users WHERE id = ?1"), &[Value::from(id)])?
			.into_iter()
			.next()
			.ok_or_else(|| "Pengguna tidak ditemukan".to_string())
	})
	.await
	.map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn auth_logout(db: tauri::State<db::Db>, user_id: i64) -> Result<(), String> {
	db.lock().unwrap().execute("UPDATE users SET online = 0 WHERE id = ?1", [user_id]).map_err(|e| e.to_string())?;
	Ok(())
}

#[tauri::command]
pub async fn auth_change_password(app: tauri::AppHandle, user_id: i64, old_password: String, new_password: String) -> Result<(), String> {
	tauri::async_runtime::spawn_blocking(move || {
		let state = app.state::<db::Db>();
		let conn = state.lock().unwrap();
		let (hash, level): (String, String) = conn
			.query_row("SELECT password_hash, level FROM users WHERE id = ?1", [user_id], |r| Ok((r.get(0)?, r.get(1)?)))
			.map_err(|_| "Pengguna tidak ditemukan".to_string())?;
		if !db::verify_password(&old_password, &hash) {
			return Err("Password lama salah".to_string());
		}
		db::cek_password(&level, &new_password)?;
		conn.execute(
			"UPDATE users SET password_hash = ?1 WHERE id = ?2",
			rusqlite::params![db::hash_password(&new_password)?, user_id],
		)
		.map_err(|e| e.to_string())?;
		Ok(())
	})
	.await
	.map_err(|e| e.to_string())?
}

// "Lupa password?" seperti e-Rapor: username, nama lengkap, dan email harus persis sama dengan
// yang terdaftar. Akun tanpa email tidak bisa reset sendiri (minta admin, atau jalur RESET-ADMIN.txt).
#[tauri::command]
pub async fn auth_reset_password(app: tauri::AppHandle, username: String, nama: String, email: String, password: String) -> Result<(), String> {
	tauri::async_runtime::spawn_blocking(move || {
		let state = app.state::<db::Db>();
		let conn = state.lock().unwrap();
		let gagal = "Data tidak cocok dengan pengguna terdaftar. Minta administrator mereset password lewat menu Pengguna.";
		let (id, level, nama_db, email_db): (i64, String, String, Option<String>) = conn
			.query_row("SELECT id, level, nama, email FROM users WHERE lower(username) = lower(?1)", [username.trim()], |r| {
				Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
			})
			.map_err(|_| gagal.to_string())?;
		let email_db = email_db.unwrap_or_default();
		if email_db.trim().is_empty() {
			return Err("Akun ini belum punya email, jadi tidak bisa reset sendiri. Minta administrator mereset password lewat menu Pengguna.".into());
		}
		let sama = |a: &str, b: &str| a.trim().to_lowercase() == b.trim().to_lowercase();
		if !sama(&nama, &nama_db) || !sama(&email, &email_db) {
			return Err(gagal.into());
		}
		db::cek_password(&level, &password)?;
		conn.execute("UPDATE users SET password_hash = ?1, online = 0 WHERE id = ?2", rusqlite::params![db::hash_password(&password)?, id])
			.map_err(|e| e.to_string())?;
		Ok(())
	})
	.await
	.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn user_set_password(app: tauri::AppHandle, user_id: i64, password: String) -> Result<(), String> {
	tauri::async_runtime::spawn_blocking(move || {
		let hash = db::hash_password(&password)?;
		let state = app.state::<db::Db>();
		let conn = state.lock().unwrap();
		conn.execute("UPDATE users SET password_hash = ?1 WHERE id = ?2", rusqlite::params![hash, user_id])
			.map_err(|e| e.to_string())?;
		Ok(())
	})
	.await
	.map_err(|e| e.to_string())?
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewUser {
	username: String,
	password: String,
	nama: String,
	level: String,
	#[serde(default)]
	email: Option<String>,
	#[serde(default)]
	ptk_id: Option<String>,
	#[serde(default)]
	peserta_didik_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateUsersResult {
	created: usize,
	skipped: Vec<String>,
}

// Dipakai "Tambah Pengguna" dan "Generate Semua Pengguna". Username yang sudah ada dilewati,
// begitu juga guru/siswa yang sudah punya akun (ptk_id / peserta_didik_id sama).
#[tauri::command]
pub async fn users_create(app: tauri::AppHandle, users: Vec<NewUser>) -> Result<CreateUsersResult, String> {
	tauri::async_runtime::spawn_blocking(move || {
		for u in &users {
			db::cek_password(&u.level, &u.password).map_err(|e| format!("{}: {e}", u.nama))?;
		}
		// Hash dulu di luar lock (bagian paling lambat), baru tulis dalam satu transaksi.
		let hashed: Vec<(NewUser, String)> = users
			.into_iter()
			.map(|u| {
				let h = db::hash_password(&u.password)?;
				Ok((u, h))
			})
			.collect::<Result<_, String>>()?;

		let state = app.state::<db::Db>();
		let mut conn = state.lock().unwrap();
		let tx = conn.transaction().map_err(|e| e.to_string())?;
		let mut created = 0;
		let mut skipped = Vec::new();
		for (u, hash) in hashed {
			let exists: i64 = tx
				.query_row(
					"SELECT COUNT(*) FROM users WHERE username = ?1
						OR (?2 IS NOT NULL AND ptk_id = ?2)
						OR (?3 IS NOT NULL AND peserta_didik_id = ?3)",
					rusqlite::params![u.username.trim(), u.ptk_id, u.peserta_didik_id],
					|r| r.get(0),
				)
				.map_err(|e| e.to_string())?;
			if exists > 0 {
				skipped.push(u.username);
				continue;
			}
			tx.execute(
				"INSERT INTO users (username, password_hash, nama, email, level, ptk_id, peserta_didik_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
				rusqlite::params![u.username.trim(), hash, u.nama, u.email, u.level, u.ptk_id, u.peserta_didik_id],
			)
			.map_err(|e| e.to_string())?;
			created += 1;
		}
		tx.commit().map_err(|e| e.to_string())?;
		Ok(CreateUsersResult { created, skipped })
	})
	.await
	.map_err(|e| e.to_string())?
}

// ===== Backup =====

#[tauri::command]
pub fn db_backup(app: tauri::AppHandle, db: tauri::State<db::Db>) -> Result<String, String> {
	let dir = crate::erapor::data_dir(&app)?.join("backups");
	std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
	let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
	let dest = dir.join(format!("erapor-backup-{ts}.sqlite"));
	let src = db.lock().unwrap();
	let mut out = rusqlite::Connection::open(&dest).map_err(|e| e.to_string())?;
	let backup = rusqlite::backup::Backup::new(&src, &mut out).map_err(|e| e.to_string())?;
	backup.run_to_completion(50, Duration::from_millis(20), None).map_err(|e| e.to_string())?;
	Ok(dest.to_string_lossy().into_owned())
}

// File backup yang ada di folder backups (terbaru dulu), untuk dipilih saat restore.
#[derive(Serialize)]
pub struct BerkasBackup {
	nama: String,
	ukuran: u64,
	waktu: u64,
}

#[tauri::command]
pub fn db_backup_list(app: tauri::AppHandle) -> Result<Vec<BerkasBackup>, String> {
	let dir = crate::erapor::data_dir(&app)?.join("backups");
	let Ok(rd) = std::fs::read_dir(&dir) else { return Ok(vec![]) };
	let mut out: Vec<BerkasBackup> = rd
		.filter_map(Result::ok)
		.filter(|f| f.path().extension().is_some_and(|x| x == "sqlite"))
		.filter_map(|f| {
			let m = f.metadata().ok()?;
			let waktu = m.modified().ok()?.duration_since(UNIX_EPOCH).ok()?.as_secs();
			Some(BerkasBackup { nama: f.file_name().to_string_lossy().into_owned(), ukuran: m.len(), waktu })
		})
		.collect();
	out.sort_by(|a, b| b.waktu.cmp(&a.waktu));
	Ok(out)
}

// Pulihkan database dari file backup (nama file di folder backups, atau isi file yang dipilih admin).
// Aman: file diperiksa dulu, database sekarang dibackup otomatis ("sebelum-restore"), lalu isinya
// disalin ke koneksi yang sedang terbuka (tidak perlu menutup aplikasi) dan migrasi dijalankan ulang.
#[tauri::command]
pub fn db_restore(app: tauri::AppHandle, db: tauri::State<db::Db>, nama: Option<String>, bytes: Option<Vec<u8>>) -> Result<String, String> {
	let dir = crate::erapor::data_dir(&app)?.join("backups");
	std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
	let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
	let (sumber, sementara) = match (nama, bytes) {
		(Some(n), _) => {
			let n = std::path::Path::new(&n).file_name().ok_or("Nama file backup tidak valid")?.to_owned();
			(dir.join(n), false)
		}
		(None, Some(b)) => {
			let p = dir.join(format!("restore-sementara-{ts}.sqlite"));
			std::fs::write(&p, b).map_err(|e| format!("Gagal menyimpan file sementara: {e}"))?;
			(p, true)
		}
		_ => return Err("Pilih file backup dulu".into()),
	};
	let hasil = (|| {
		let src = rusqlite::Connection::open_with_flags(&sumber, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
			.map_err(|_| "File bukan database e-Rapor".to_string())?;
		let cek: String = src.query_row("PRAGMA integrity_check", [], |r| r.get(0)).map_err(|_| "File bukan database e-Rapor".to_string())?;
		if cek != "ok" {
			return Err("File backup rusak (integrity check gagal)".to_string());
		}
		let admin: i64 = src
			.query_row("SELECT COUNT(*) FROM users WHERE level = 'admin'", [], |r| r.get(0))
			.map_err(|_| "File ini bukan backup e-Rapor (tabel pengguna tidak ada)".to_string())?;
		if admin == 0 {
			return Err("Backup tidak punya akun admin, tidak bisa dipakai".to_string());
		}
		let mut live = db.lock().unwrap();
		// Backup otomatis kondisi sekarang, supaya restore yang salah bisa dibatalkan.
		let aman = dir.join(format!("sebelum-restore-{ts}.sqlite"));
		{
			let mut out = rusqlite::Connection::open(&aman).map_err(|e| e.to_string())?;
			rusqlite::backup::Backup::new(&live, &mut out).map_err(|e| e.to_string())?
				.run_to_completion(50, Duration::from_millis(20), None).map_err(|e| e.to_string())?;
		}
		rusqlite::backup::Backup::new(&src, &mut live).map_err(|e| e.to_string())?
			.run_to_completion(50, Duration::from_millis(20), None).map_err(|e| format!("Restore gagal: {e}"))?;
		db::siapkan(&live)?;
		Ok(aman.file_name().unwrap().to_string_lossy().into_owned())
	})();
	if sementara {
		let _ = std::fs::remove_file(&sumber);
	}
	hasil
}

#[tauri::command]
pub fn open_data_dir(app: tauri::AppHandle, sub: Option<String>) -> Result<String, String> {
	let mut dir = crate::erapor::data_dir(&app)?;
	if let Some(sub) = sub.filter(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')) {
		dir = dir.join(sub);
	}
	std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
	open_in_explorer(&dir)?;
	Ok(dir.to_string_lossy().into_owned())
}

pub fn open_in_explorer(dir: &std::path::Path) -> Result<(), String> {
	#[cfg(target_os = "windows")]
	let cmd = "explorer";
	#[cfg(target_os = "macos")]
	let cmd = "open";
	#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
	let cmd = "xdg-open";
	std::process::Command::new(cmd).arg(dir).spawn().map_err(|e| e.to_string())?;
	Ok(())
}


// Buka Explorer dengan file tertentu terpilih.
#[tauri::command]
pub fn reveal_file(path: String) -> Result<(), String> {
	#[cfg(target_os = "windows")]
	std::process::Command::new("explorer").arg(format!("/select,{path}")).spawn().map_err(|e| e.to_string())?;
	#[cfg(not(target_os = "windows"))]
	open_in_explorer(std::path::Path::new(&path).parent().unwrap_or(std::path::Path::new(".")))?;
	Ok(())
}

// ===== Laptop guru =====

// POST JSON ke server sinkron admin (lewat Rust supaya tidak kena CORS di webview).
#[tauri::command]
pub async fn http_post_json(url: String, body: Value, timeout_secs: Option<u64>) -> Result<Value, String> {
	let client = reqwest::Client::builder()
		.timeout(Duration::from_secs(timeout_secs.unwrap_or(60)))
		.build()
		.map_err(|e| e.to_string())?;
	let res = client.post(&url).json(&body).send().await.map_err(|e| {
		if e.is_connect() || e.is_timeout() {
			"Tidak bisa terhubung ke laptop admin. Pastikan link benar dan Sesi Online di admin masih dibuka.".to_string()
		} else {
			format!("Gagal menghubungi admin: {e}")
		}
	})?;
	let status = res.status();
	let text = res.text().await.map_err(|e| e.to_string())?;
	let json: Value = serde_json::from_str(&text).unwrap_or(Value::String(text.clone()));
	if !status.is_success() {
		let msg = json.get("error").and_then(|v| v.as_str()).map(String::from)
			.unwrap_or_else(|| format!("Server admin menjawab HTTP {status}"));
		return Err(msg);
	}
	Ok(json)
}

// Akun guru di laptop guru: dibuat/diperbarui saat tarik data, dengan password yang baru saja
// dipakai (sudah diverifikasi admin), supaya guru bisa login offline dengan password yang sama.
#[tauri::command]
pub async fn guru_set_local_user(app: tauri::AppHandle, username: String, password: String, nama: String, ptk_id: String) -> Result<(), String> {
	tauri::async_runtime::spawn_blocking(move || {
		let hash = db::hash_password(&password)?;
		let state = app.state::<db::Db>();
		let conn = state.lock().unwrap();
		conn.execute(
			"INSERT INTO users (username, password_hash, nama, level, ptk_id) VALUES (?1, ?2, ?3, 'guru', ?4)
			ON CONFLICT (username) DO UPDATE SET password_hash = excluded.password_hash, nama = excluded.nama, ptk_id = excluded.ptk_id, aktif = 1",
			rusqlite::params![username.trim(), hash, nama, ptk_id],
		)
		.map_err(|e| e.to_string())?;
		Ok(())
	})
	.await
	.map_err(|e| e.to_string())?
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetPassword {
	user_id: i64,
	password: String,
}

// Ganti password banyak pengguna sekaligus (checklist di Data Pengguna). Di-hash satu per satu
// (salt berbeda walau passwordnya sama), lalu ditulis dalam satu transaksi.
#[tauri::command]
pub async fn users_set_password(app: tauri::AppHandle, items: Vec<SetPassword>) -> Result<usize, String> {
	tauri::async_runtime::spawn_blocking(move || {
		{
			let state = app.state::<db::Db>();
			let conn = state.lock().unwrap();
			for i in &items {
				let (nama, level): (String, String) = conn
					.query_row("SELECT nama, level FROM users WHERE id = ?1", [i.user_id], |r| Ok((r.get(0)?, r.get(1)?)))
					.map_err(|_| format!("Pengguna #{} tidak ditemukan", i.user_id))?;
				db::cek_password(&level, &i.password).map_err(|e| format!("{nama}: {e}"))?;
			}
		}
		let hashed: Vec<(i64, String)> = items
			.iter()
			.map(|i| Ok((i.user_id, db::hash_password(&i.password)?)))
			.collect::<Result<_, String>>()?;
		let state = app.state::<db::Db>();
		let mut conn = state.lock().unwrap();
		let tx = conn.transaction().map_err(|e| e.to_string())?;
		let mut n = 0;
		for (id, hash) in &hashed {
			n += tx
				.execute("UPDATE users SET password_hash = ?1 WHERE id = ?2", rusqlite::params![hash, id])
				.map_err(|e| e.to_string())?;
		}
		tx.commit().map_err(|e| e.to_string())?;
		Ok(n)
	})
	.await
	.map_err(|e| e.to_string())?
}
