// Sesi Online: laptop admin membuka server kecil supaya laptop guru bisa TARIK data dan KIRIM
// nilai (bukan online terus seperti e-Rapor). Server hanya punya dua jalur itu — database tidak
// pernah dibuka langsung ke luar. Akses dari luar sekolah lewat Cloudflare Quick Tunnel (tanpa
// akun): cloudflared diunduh sekali ke folder data aplikasi, link trycloudflare.com berganti tiap sesi.
use crate::erapor::db;
use axum::{extract::State, http::StatusCode, routing::{get, post}, Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasher, Hasher};
use std::net::{SocketAddr, UdpSocket};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::oneshot;

const PREFERRED_PORT: u16 = 8789;
const MAX_GAGAL: u32 = 30; // percobaan PIN/password salah sebelum sesi otomatis ditutup
pub const TP_MAX: usize = 100;

#[derive(Default)]
pub struct SesiState(pub Mutex<Option<Sesi>>);

pub struct Sesi {
	pin: String,
	port: u16,
	dibuka: String,
	public_url: Option<String>,
	tunnel_error: Option<String>,
	shutdown: Option<oneshot::Sender<()>>,
	tunnel: Option<tokio::process::Child>,
	log: Vec<LogItem>,
	gagal: u32,
}

#[derive(Clone, Serialize)]
pub struct LogItem {
	waktu: String,
	nama: String,
	aksi: String,
	ok: bool,
	detail: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SesiInfo {
	aktif: bool,
	pin: Option<String>,
	lan_url: Option<String>,
	public_url: Option<String>,
	tunnel_error: Option<String>,
	dibuka: Option<String>,
	log: Vec<LogItem>,
}

fn now() -> String {
	// Jam lokal dari SQLite supaya sama dengan kolom waktu lain di database.
	rusqlite::Connection::open_in_memory()
		.and_then(|c| c.query_row("SELECT datetime('now','localtime')", [], |r| r.get(0)))
		.unwrap_or_default()
}

fn random_pin() -> String {
	let n = std::collections::hash_map::RandomState::new().build_hasher().finish();
	format!("{:06}", n % 1_000_000)
}

pub fn local_lan_ip() -> Option<String> {
	let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
	socket.connect("8.8.8.8:80").ok()?;
	socket.local_addr().ok().map(|a| a.ip().to_string())
}

fn info(s: Option<&Sesi>) -> SesiInfo {
	match s {
		None => SesiInfo { aktif: false, pin: None, lan_url: None, public_url: None, tunnel_error: None, dibuka: None, log: vec![] },
		Some(s) => SesiInfo {
			aktif: true,
			pin: Some(s.pin.clone()),
			lan_url: local_lan_ip().map(|ip| format!("http://{ip}:{}", s.port)),
			public_url: s.public_url.clone(),
			tunnel_error: s.tunnel_error.clone(),
			dibuka: Some(s.dibuka.clone()),
			log: s.log.iter().rev().cloned().collect(),
		},
	}
}

fn catat(app: &AppHandle, nama: &str, aksi: &str, ok: bool, detail: impl Into<String>) {
	if let Some(s) = app.state::<SesiState>().0.lock().unwrap().as_mut() {
		s.log.push(LogItem { waktu: now(), nama: nama.into(), aksi: aksi.into(), ok, detail: detail.into() });
	}
}

// ===== Perintah Tauri =====

#[tauri::command]
pub fn sesi_status(app: AppHandle) -> SesiInfo {
	info(app.state::<SesiState>().0.lock().unwrap().as_ref())
}

#[tauri::command]
pub async fn sesi_buka(app: AppHandle, online: bool) -> Result<SesiInfo, String> {
	if app.state::<SesiState>().0.lock().unwrap().is_some() {
		return Ok(sesi_status(app));
	}

	let mut bound = None;
	for port in PREFERRED_PORT..PREFERRED_PORT + 20 {
		if let Ok(l) = tokio::net::TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], port))).await {
			bound = Some((l, port));
			break;
		}
	}
	let (listener, port) = bound.ok_or("Tidak ada port kosong untuk server sinkron")?;

	let (tx, rx) = oneshot::channel::<()>();
	let router = Router::new()
		.route("/", get(halaman_depan))
		.route("/api/info", get(api_info))
		.route("/api/tarik", post(api_tarik))
		.route("/api/kirim", post(api_kirim))
		.route("/api/kirim-walas", post(api_kirim_walas))
		.route("/api/kirim-ekskul", post(api_kirim_ekskul))
		.with_state(app.clone());
	tauri::async_runtime::spawn(async move {
		let _ = axum::serve(listener, router).with_graceful_shutdown(async { let _ = rx.await; }).await;
	});

	*app.state::<SesiState>().0.lock().unwrap() = Some(Sesi {
		pin: random_pin(),
		port,
		dibuka: now(),
		public_url: None,
		tunnel_error: None,
		shutdown: Some(tx),
		tunnel: None,
		log: vec![],
		gagal: 0,
	});

	if online {
		match start_tunnel(&app, port).await {
			Ok((child, url)) => {
				if let Some(s) = app.state::<SesiState>().0.lock().unwrap().as_mut() {
					s.public_url = Some(url);
					s.tunnel = Some(child);
				}
			}
			Err(e) => {
				if let Some(s) = app.state::<SesiState>().0.lock().unwrap().as_mut() {
					s.tunnel_error = Some(e);
				}
			}
		}
	}
	Ok(sesi_status(app))
}

#[tauri::command]
pub fn sesi_tutup(app: AppHandle) {
	tutup(&app);
}

pub fn tutup(app: &AppHandle) {
	if let Some(mut s) = app.state::<SesiState>().0.lock().unwrap().take() {
		if let Some(tx) = s.shutdown.take() {
			let _ = tx.send(());
		}
		if let Some(mut child) = s.tunnel.take() {
			let _ = child.start_kill();
		}
	}
}

// ===== Cloudflare Quick Tunnel =====

async fn ensure_cloudflared(app: &AppHandle) -> Result<PathBuf, String> {
	let dir = app.path().app_data_dir().map_err(|e| e.to_string())?.join("bin");
	std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
	#[cfg(target_os = "windows")]
	let (name, url) = ("cloudflared.exe", "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-windows-amd64.exe");
	#[cfg(target_os = "linux")]
	let (name, url) = ("cloudflared", "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-amd64");
	#[cfg(not(any(target_os = "windows", target_os = "linux")))]
	return Err("Akses online otomatis baru tersedia untuk Windows dan Linux".into());

	#[cfg(any(target_os = "windows", target_os = "linux"))]
	{
		let exe = dir.join(name);
		if exe.exists() {
			return Ok(exe);
		}
		let bytes = reqwest::get(url)
			.await
			.and_then(|r| r.error_for_status())
			.map_err(|e| format!("Gagal mengunduh cloudflared (butuh internet): {e}"))?
			.bytes()
			.await
			.map_err(|e| format!("Gagal mengunduh cloudflared: {e}"))?;
		let tmp = dir.join(format!("{name}.part"));
		std::fs::write(&tmp, &bytes).map_err(|e| e.to_string())?;
		std::fs::rename(&tmp, &exe).map_err(|e| e.to_string())?;
		#[cfg(unix)]
		{
			use std::os::unix::fs::PermissionsExt;
			let _ = std::fs::set_permissions(&exe, std::fs::Permissions::from_mode(0o755));
		}
		Ok(exe)
	}
}

// Jalankan program tunnel dan tunggu link publiknya muncul di stdout/stderr.
async fn jalankan_tunnel(
	mut cmd: tokio::process::Command,
	cocok: fn(&str) -> bool,
	batas_detik: u64,
) -> Result<(tokio::process::Child, String), String> {
	cmd.stdin(std::process::Stdio::null())
		.stdout(std::process::Stdio::piped())
		.stderr(std::process::Stdio::piped())
		.kill_on_drop(true);
	#[cfg(target_os = "windows")]
	cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW: jangan munculkan jendela konsol
	let mut child = cmd.spawn().map_err(|e| format!("tidak bisa dijalankan ({e})"))?;

	// Baca kedua aliran terus-menerus (supaya pipe tidak penuh), kirim link pertama yang cocok.
	let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();
	let out = child.stdout.take();
	let err = child.stderr.take();
	for stream in [out.map(|s| Box::new(s) as Box<dyn tokio::io::AsyncRead + Unpin + Send>), err.map(|s| Box::new(s) as _)].into_iter().flatten() {
		let tx = tx.clone();
		tauri::async_runtime::spawn(async move {
			let mut lines = BufReader::new(stream).lines();
			while let Ok(Some(line)) = lines.next_line().await {
				let mut rest = line.as_str();
				while let Some(i) = rest.find("https://") {
					let url: String = rest[i..].chars().take_while(|c| !c.is_whitespace() && !matches!(c, '|' | '"' | ',')).collect();
					if cocok(&url) {
						let _ = tx.send(url.clone());
					}
					rest = &rest[i + 8..];
				}
			}
		});
	}
	drop(tx);

	match tokio::time::timeout(std::time::Duration::from_secs(batas_detik), rx.recv()).await {
		Ok(Some(url)) => Ok((child, url)),
		_ => {
			let _ = child.start_kill();
			Err(format!("tidak memberi link dalam {batas_detik} detik"))
		}
	}
}

fn ssh_exe() -> PathBuf {
	#[cfg(target_os = "windows")]
	{
		let bawaan = PathBuf::from(r"C:\Windows\System32\OpenSSH\ssh.exe");
		if bawaan.exists() {
			return bawaan;
		}
	}
	PathBuf::from("ssh")
}

// Coba penyedia tunnel tanpa akun secara berurutan sampai ada yang berhasil:
//   1. Cloudflare Quick Tunnel (cloudflared, diunduh otomatis sekali)
//   2. localhost.run (lewat SSH bawaan Windows 10/11, tanpa unduhan)
// (Pinggy butuh ketik password kosong dan Serveo sering mati, jadi tidak dipakai otomatis.)
async fn start_tunnel(app: &AppHandle, port: u16) -> Result<(tokio::process::Child, String), String> {
	let target = format!("http://127.0.0.1:{port}");
	let mut gagal = Vec::new();

	match ensure_cloudflared(app).await {
		Ok(exe) => {
			let mut cmd = tokio::process::Command::new(exe);
			cmd.args(["tunnel", "--no-autoupdate", "--url", &target]);
			match jalankan_tunnel(cmd, |u| u.contains(".trycloudflare.com"), 45).await {
				Ok(r) => return Ok(r),
				Err(e) => gagal.push(format!("Cloudflare: {e}")),
			}
		}
		Err(e) => gagal.push(format!("Cloudflare: {e}")),
	}

	let mut cmd = tokio::process::Command::new(ssh_exe());
	#[cfg(target_os = "windows")]
	let known = "UserKnownHostsFile=NUL";
	#[cfg(not(target_os = "windows"))]
	let known = "UserKnownHostsFile=/dev/null";
	cmd.args([
		"-o", "StrictHostKeyChecking=no",
		"-o", known,
		"-o", "ServerAliveInterval=30",
		"-o", "ExitOnForwardFailure=yes",
		"-T",
		"-R", &format!("80:127.0.0.1:{port}"),
		"nokey@localhost.run",
	]);
	match jalankan_tunnel(cmd, |u| u.contains(".lhr.") && !u.contains("localhost.run"), 30).await {
		Ok(r) => return Ok(r),
		Err(e) => gagal.push(format!("localhost.run: {e}")),
	}

	Err(format!("Semua jalur online gagal. Cek internet laptop admin.\n{}", gagal.join("\n")))
}

// ===== API untuk laptop guru =====

type ApiResult = Result<Json<Value>, (StatusCode, Json<Value>)>;

fn fail(code: StatusCode, msg: impl Into<String>) -> (StatusCode, Json<Value>) {
	(code, Json(json!({ "error": msg.into() })))
}

#[derive(Deserialize)]
struct Auth {
	pin: String,
	username: String,
	password: String,
}

struct Guru {
	nama: String,
	ptk_id: String,
}

// Cek PIN sesi + akun guru. Terlalu banyak gagal = sesi ditutup otomatis.
async fn auth(app: &AppHandle, a: &Auth) -> Result<Guru, (StatusCode, Json<Value>)> {
	let pin_ok = {
		let state = app.state::<SesiState>();
		let guard = state.0.lock().unwrap();
		guard.as_ref().map(|s| s.pin == a.pin.trim()).unwrap_or(false)
	};
	let gagal = |app: &AppHandle, msg: &str| {
		let mut tutup_sesi = false;
		if let Some(s) = app.state::<SesiState>().0.lock().unwrap().as_mut() {
			s.gagal += 1;
			tutup_sesi = s.gagal >= MAX_GAGAL;
		}
		catat(app, &a.username, "login", false, msg);
		if tutup_sesi {
			tutup(app);
		}
		fail(StatusCode::UNAUTHORIZED, msg)
	};
	if !pin_ok {
		return Err(gagal(app, "PIN sesi salah"));
	}

	let found: Option<(String, String, Option<String>, i64)> = {
		let state = app.state::<db::Db>();
		let conn = state.lock().unwrap();
		conn.query_row(
			"SELECT password_hash, nama, ptk_id, aktif FROM users WHERE username = ?1 AND level = 'guru'",
			[a.username.trim()],
			|r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
		)
		.ok()
	};
	let Some((hash, nama, ptk_id, aktif)) = found else {
		return Err(gagal(app, "Username atau password salah"));
	};
	let pass = a.password.clone();
	let ok = tokio::task::spawn_blocking(move || db::verify_password(&pass, &hash)).await.unwrap_or(false);
	if !ok {
		return Err(gagal(app, "Username atau password salah"));
	}
	if aktif == 0 {
		return Err(fail(StatusCode::FORBIDDEN, "Akun dinonaktifkan admin"));
	}
	let ptk_id = ptk_id.ok_or_else(|| fail(StatusCode::FORBIDDEN, "Akun belum terhubung dengan data guru Dapodik"))?;
	Ok(Guru { nama, ptk_id })
}

fn semester_aktif(conn: &rusqlite::Connection) -> Option<String> {
	conn.query_row(
		"SELECT semester_id FROM pembelajaran_rapor GROUP BY semester_id ORDER BY semester_id DESC LIMIT 1",
		[],
		|r| r.get(0),
	)
	.ok()
}

// Yang membuka link di browser/HP diberi penjelasan, bukan halaman error kosong.
async fn halaman_depan(State(app): State<AppHandle>) -> axum::response::Html<String> {
	let sekolah: String = {
		let state = app.state::<db::Db>();
		let conn = state.lock().unwrap();
		conn.query_row("SELECT nama FROM sekolah LIMIT 1", [], |r| r.get(0)).unwrap_or_default()
	};
	let sekolah = sekolah.replace('<', "&lt;");
	axum::response::Html(format!(
		r#"<!doctype html><html lang="id"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>Server e-Rapor</title><style>body{{font-family:system-ui,sans-serif;max-width:480px;margin:40px auto;padding:0 16px;line-height:1.6;color:#222}}
.ok{{color:#16a34a;font-weight:600}}code{{background:#f3f4f6;padding:2px 6px;border-radius:4px}}</style></head><body>
<h2>Server e-Rapor {sekolah}</h2><p class="ok">● Sesi sinkron sedang aktif</p>
<p>Link ini dipakai oleh <b>aplikasi e-Rapor di laptop guru</b>:</p>
<ol><li>Buka aplikasi e-Rapor di laptop guru</li><li>Pilih <b>Sinkron ke Admin</b> (atau "Laptop guru? Tarik data" di halaman masuk)</li><li>Tempel link ini dan masukkan PIN dari admin</li></ol>
<p style="color:#666">Pengisian nilai langsung lewat browser/HP belum tersedia.</p></body></html>"#
	))
}

async fn api_info(State(app): State<AppHandle>) -> Json<Value> {
	let state = app.state::<db::Db>();
	let conn = state.lock().unwrap();
	let sekolah: Option<String> = conn.query_row("SELECT nama FROM sekolah LIMIT 1", [], |r| r.get(0)).ok();
	Json(json!({ "app": "nuxt-erapor", "sekolah": sekolah, "semesterId": semester_aktif(&conn) }))
}

// Rombel yang ia ajar + yang ia walikan (dasar pembelajaran yang dikirim).
const ROMBEL_UTAMA: &str = "(SELECT rombongan_belajar_id FROM pembelajaran_rapor WHERE ptk_id = ?1 AND semester_id = ?2
	UNION SELECT rombongan_belajar_id FROM rombel WHERE ptk_id = ?1 AND semester_id = ?2 AND jenis_rombel = '1')";

// Rombel milik guru: yang ia ajar + yang ia walikan + ekskul yang ia bina (beserta kelas reguler
// anggota ekskul itu, supaya pembina tahu kelas tiap siswa).
const ROMBEL_GURU: &str = "(SELECT rombongan_belajar_id FROM pembelajaran_rapor WHERE ptk_id = ?1 AND semester_id = ?2
	UNION SELECT rombongan_belajar_id FROM rombel WHERE ptk_id = ?1 AND semester_id = ?2 AND jenis_rombel IN ('1', '51')
	UNION SELECT k.rombongan_belajar_id FROM anggota_rombel k JOIN rombel rk ON rk.rombongan_belajar_id = k.rombongan_belajar_id AND rk.jenis_rombel = '1'
		WHERE rk.semester_id = ?2 AND k.peserta_didik_id IN (SELECT a.peserta_didik_id FROM anggota_rombel a JOIN rombel e USING (rombongan_belajar_id)
			WHERE e.ptk_id = ?1 AND e.semester_id = ?2 AND e.jenis_rombel = '51'))
	UNION SELECT x.rombongan_belajar_id FROM anggota_rombel x JOIN rombel ex ON ex.rombongan_belajar_id = x.rombongan_belajar_id AND ex.jenis_rombel = '51'
		WHERE ex.semester_id = ?2 AND x.peserta_didik_id IN (SELECT a.peserta_didik_id FROM anggota_rombel a JOIN rombel w USING (rombongan_belajar_id) WHERE w.ptk_id = ?1 AND w.semester_id = ?2 AND w.jenis_rombel = '1'))";

fn build_paket(conn: &rusqlite::Connection, ptk_id: &str, sem: &str) -> Result<Value, String> {
	let p = [Value::from(ptk_id), Value::from(sem)];
	let q = |sql: &str| db::query(conn, sql, &p).map(Value::from);
	let q0 = |sql: &str| db::query(conn, sql, &[]).map(Value::from);
	// Data pribadi siswa/guru dikirim seperlunya saja (tanpa NIK, alamat, data orang tua).
	Ok(json!({
		"semesterId": sem,
		"sekolah": q0("SELECT sekolah_id, npsn, nama, alamat, kecamatan, kabupaten_kota, provinsi, email, kepsek_ptk_id FROM sekolah")?,
		"semester": db::query(conn, "SELECT * FROM semester WHERE semester_id = ?1", &[Value::from(sem)]).map(Value::from)?,
		"ptk": q0("SELECT ptk_id, nama, nip, nuptk, jenis_kelamin, gelar_depan, gelar_belakang FROM ptk")?,
		"rombel": q(&format!("SELECT rombongan_belajar_id, semester_id, nama, tingkat, jenis_rombel, jenis_rombel_str, kurikulum, ptk_id FROM rombel WHERE rombongan_belajar_id IN {ROMBEL_GURU}"))?,
		"anggota_rombel": q(&format!("SELECT * FROM anggota_rombel WHERE rombongan_belajar_id IN {ROMBEL_GURU}"))?,
		// Data hasil koreksi admin (view siswa_rapor), supaya nama di laptop guru sama dengan di rapor.
		"peserta_didik": q(&format!("SELECT peserta_didik_id, nama, nisn, nipd, jenis_kelamin, tempat_lahir, tanggal_lahir, agama FROM siswa_rapor
			WHERE peserta_didik_id IN (SELECT peserta_didik_id FROM anggota_rombel WHERE rombongan_belajar_id IN {ROMBEL_GURU})"))?,
		"mapel_rapor": q0("SELECT * FROM mapel_rapor")?,
		"pembelajaran_rapor": q(&format!("SELECT id, semester_id, rombongan_belajar_id, kode_mapel, ptk_id, sumber, terkunci, dikirim_at FROM pembelajaran_rapor
			WHERE semester_id = ?2 AND rombongan_belajar_id IN {ROMBEL_UTAMA}"))?,
		"tujuan_pembelajaran": q("SELECT uid, kode_mapel, tingkat, semester, deskripsi, urutan, sumber FROM tujuan_pembelajaran t
			WHERE EXISTS (SELECT 1 FROM pembelajaran_rapor pr JOIN rombel r USING (rombongan_belajar_id)
				WHERE pr.ptk_id = ?1 AND pr.semester_id = ?2 AND pr.kode_mapel = t.kode_mapel AND CAST(r.tingkat AS INTEGER) = t.tingkat)")?,
		"nilai_rapor": q("SELECT n.pembelajaran_rapor_id, n.peserta_didik_id, n.nilai,
				(SELECT json_group_array(t.uid) FROM json_each(n.tp_optimal) j JOIN tujuan_pembelajaran t ON t.id = j.value) AS tp_optimal,
				(SELECT json_group_array(t.uid) FROM json_each(n.tp_perlu) j JOIN tujuan_pembelajaran t ON t.id = j.value) AS tp_perlu
			FROM nilai_rapor n JOIN pembelajaran_rapor pr ON pr.id = n.pembelajaran_rapor_id
			WHERE pr.ptk_id = ?1 AND pr.semester_id = ?2")?,
		// Isian wali kelas (kehadiran, catatan, kenaikan) untuk kelas yang ia walikan.
		"rapor_siswa": q("SELECT semester_id, peserta_didik_id, rombongan_belajar_id, sakit, izin, alpa, catatan, naik, kokurikuler FROM rapor_siswa
			WHERE semester_id = ?2 AND rombongan_belajar_id IN (SELECT rombongan_belajar_id FROM rombel WHERE ptk_id = ?1 AND semester_id = ?2 AND jenis_rombel = '1')")?,
		"nilai_ekskul": q("SELECT semester_id, rombongan_belajar_id, peserta_didik_id, predikat, keterangan FROM nilai_ekskul
			WHERE semester_id = ?2 AND (rombongan_belajar_id IN (SELECT rombongan_belajar_id FROM rombel WHERE ptk_id = ?1 AND semester_id = ?2 AND jenis_rombel = '51')
				OR peserta_didik_id IN (SELECT a.peserta_didik_id FROM anggota_rombel a JOIN rombel w USING (rombongan_belajar_id) WHERE w.ptk_id = ?1 AND w.semester_id = ?2 AND w.jenis_rombel = '1'))")?,
		// Kokurikuler: semua kegiatan semester ini (kecil) + nilai siswa kelas yang ia walikan.
		"kokurikuler_kegiatan": q("SELECT id, semester_id, nama, tema, tujuan, tingkat, dimensi, subdimensi, urutan FROM kokurikuler_kegiatan WHERE semester_id = ?2")?,
		"nilai_kokurikuler": q("SELECT n.kegiatan_id, n.peserta_didik_id, n.capaian FROM nilai_kokurikuler n JOIN kokurikuler_kegiatan k ON k.id = n.kegiatan_id
			WHERE k.semester_id = ?2 AND n.peserta_didik_id IN (SELECT a.peserta_didik_id FROM anggota_rombel a JOIN rombel r USING (rombongan_belajar_id)
				WHERE r.ptk_id = ?1 AND r.semester_id = ?2 AND r.jenis_rombel = '1')")?,
	}))
}

async fn api_tarik(State(app): State<AppHandle>, Json(a): Json<Auth>) -> ApiResult {
	let guru = auth(&app, &a).await?;
	let paket = {
		let state = app.state::<db::Db>();
		let conn = state.lock().unwrap();
		let sem = semester_aktif(&conn).ok_or_else(|| fail(StatusCode::CONFLICT, "Admin belum menyusun pembelajaran rapor"))?;
		build_paket(&conn, &guru.ptk_id, &sem).map_err(|e| fail(StatusCode::INTERNAL_SERVER_ERROR, e))?
	};
	let jml = paket["pembelajaran_rapor"].as_array().map(|a| a.iter().filter(|p| p["ptk_id"] == guru.ptk_id.as_str()).count()).unwrap_or(0);
	catat(&app, &guru.nama, "tarik data", true, format!("{jml} pembelajaran"));
	Ok(Json(json!({ "guru": { "nama": guru.nama, "username": a.username.trim(), "ptkId": guru.ptk_id }, "paket": paket })))
}

#[derive(Deserialize)]
struct TpIn {
	uid: String,
	kode_mapel: String,
	tingkat: i64,
	semester: i64,
	deskripsi: String,
	urutan: i64,
}

#[derive(Deserialize)]
struct NilaiIn {
	pembelajaran_rapor_id: i64,
	peserta_didik_id: String,
	nilai: Option<i64>,
	tp_optimal: Vec<String>,
	tp_perlu: Vec<String>,
}

#[derive(Deserialize)]
struct KirimReq {
	#[serde(flatten)]
	auth: Auth,
	semester_id: String,
	pembelajaran_ids: Vec<i64>,
	tp: Vec<TpIn>,
	nilai: Vec<NilaiIn>,
}

async fn api_kirim(State(app): State<AppHandle>, Json(req): Json<KirimReq>) -> ApiResult {
	let guru = auth(&app, &req.auth).await?;
	let hasil = {
		let state = app.state::<db::Db>();
		let mut conn = state.lock().unwrap();
		simpan_kiriman(&mut conn, &guru, &req)
	};
	match hasil {
		Ok(v) => {
			catat(&app, &guru.nama, "kirim nilai", true, format!("{} mapel, {} nilai", v["pembelajaran"], v["nilai"]));
			Ok(Json(v))
		}
		Err(msg) => {
			catat(&app, &guru.nama, "kirim nilai", false, msg.clone());
			Err(fail(StatusCode::UNPROCESSABLE_ENTITY, msg))
		}
	}
}

fn simpan_kiriman(conn: &mut rusqlite::Connection, guru: &Guru, req: &KirimReq) -> Result<Value, String> {
	let tx = conn.transaction().map_err(|e| e.to_string())?;
	let e = |e: rusqlite::Error| e.to_string();

	// 1. Pembelajaran harus milik guru ini, semester yang sama, dan belum terkunci.
	let ids: HashSet<i64> = req.pembelajaran_ids.iter().copied().collect();
	if ids.is_empty() {
		return Err("Tidak ada pembelajaran yang dikirim".into());
	}
	let mut rombel_of = HashMap::new();
	for id in &ids {
		let row: Option<(Option<String>, String, i64, String, String)> = tx
			.query_row(
				"SELECT pr.ptk_id, pr.semester_id, pr.terkunci, pr.rombongan_belajar_id, m.singkat || ' ' || r.nama
				FROM pembelajaran_rapor pr JOIN mapel_rapor m ON m.kode = pr.kode_mapel JOIN rombel r USING (rombongan_belajar_id) WHERE pr.id = ?1",
				[id],
				|r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
			)
			.ok();
		let Some((ptk, sem, terkunci, rombel, label)) = row else {
			return Err(format!("Pembelajaran #{id} tidak ada di laptop admin. Tarik data ulang."));
		};
		if ptk.as_deref() != Some(guru.ptk_id.as_str()) {
			return Err(format!("{label} bukan pembelajaran Anda (pengampu sudah diganti admin). Tarik data ulang."));
		}
		if sem != req.semester_id {
			return Err(format!("Semester berbeda dengan admin ({sem}). Tarik data ulang."));
		}
		if terkunci != 0 {
			return Err(format!("{label} sudah terkirim dan terkunci. Minta admin membuka kunci."));
		}
		rombel_of.insert(*id, (rombel, label));
	}

	// 2. TP: buatan guru disimpan/diperbarui; TP buatan admin tidak diubah.
	for tp in &req.tp {
		if tp.deskripsi.chars().count() > TP_MAX {
			return Err(format!("TP \"{}…\" lebih dari {TP_MAX} karakter", tp.deskripsi.chars().take(30).collect::<String>()));
		}
		tx.execute(
			"INSERT INTO tujuan_pembelajaran (uid, kode_mapel, tingkat, semester, deskripsi, urutan, sumber) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'guru')
			ON CONFLICT (uid) DO UPDATE SET deskripsi = excluded.deskripsi, urutan = excluded.urutan WHERE tujuan_pembelajaran.sumber = 'guru'",
			rusqlite::params![tp.uid, tp.kode_mapel, tp.tingkat, tp.semester, tp.deskripsi, tp.urutan],
		)
		.map_err(e)?;
	}
	let mut uid_ke_id: HashMap<String, i64> = HashMap::new();
	let mut id_of = |uid: &str| -> Result<i64, String> {
		if let Some(id) = uid_ke_id.get(uid) {
			return Ok(*id);
		}
		let id: i64 = tx
			.query_row("SELECT id FROM tujuan_pembelajaran WHERE uid = ?1", [uid], |r| r.get(0))
			.map_err(|_| "Ada TP yang tidak ikut terkirim. Coba kirim ulang.".to_string())?;
		uid_ke_id.insert(uid.to_string(), id);
		Ok(id)
	};

	// 3. Lengkap: setiap siswa di rombel punya nilai dan minimal satu centang TP.
	let mut lengkap: HashMap<i64, HashSet<&str>> = HashMap::new();
	for n in &req.nilai {
		if !ids.contains(&n.pembelajaran_rapor_id) {
			return Err("Nilai berisi pembelajaran yang tidak ikut dikirim".into());
		}
		if n.nilai.is_some_and(|v| (0..=100).contains(&v)) && !n.tp_optimal.is_empty() && !n.tp_perlu.is_empty() {
			lengkap.entry(n.pembelajaran_rapor_id).or_default().insert(n.peserta_didik_id.as_str());
		}
	}
	for (id, (rombel, label)) in &rombel_of {
		let mut stmt = tx.prepare("SELECT peserta_didik_id FROM anggota_rombel WHERE rombongan_belajar_id = ?1").map_err(e)?;
		let siswa: Vec<String> = stmt.query_map([rombel], |r| r.get(0)).map_err(e)?.filter_map(Result::ok).collect();
		let ada = lengkap.get(id);
		let kurang = siswa.iter().filter(|s| !ada.is_some_and(|a| a.contains(s.as_str()))).count();
		if kurang > 0 {
			return Err(format!("{label}: {kurang} siswa belum lengkap (nilai + minimal 1 TP tercapai dan 1 TP perlu bantuan)"));
		}
	}

	// 4. Simpan nilai, lalu kunci pembelajaran.
	for n in &req.nilai {
		let opt: Vec<i64> = n.tp_optimal.iter().map(|u| id_of(u)).collect::<Result<_, _>>()?;
		let perlu: Vec<i64> = n.tp_perlu.iter().map(|u| id_of(u)).collect::<Result<_, _>>()?;
		tx.execute(
			"INSERT INTO nilai_rapor (pembelajaran_rapor_id, peserta_didik_id, nilai, tp_optimal, tp_perlu, updated_at)
			VALUES (?1, ?2, ?3, ?4, ?5, datetime('now','localtime'))
			ON CONFLICT (pembelajaran_rapor_id, peserta_didik_id) DO UPDATE SET
				nilai = excluded.nilai, tp_optimal = excluded.tp_optimal, tp_perlu = excluded.tp_perlu, updated_at = excluded.updated_at",
			rusqlite::params![n.pembelajaran_rapor_id, n.peserta_didik_id, n.nilai, json!(opt).to_string(), json!(perlu).to_string()],
		)
		.map_err(e)?;
	}
	for id in &ids {
		tx.execute("UPDATE pembelajaran_rapor SET terkunci = 1, dikirim_at = datetime('now','localtime') WHERE id = ?1", [id])
			.map_err(e)?;
	}
	tx.commit().map_err(e)?;
	Ok(json!({ "ok": true, "pembelajaran": ids.len(), "nilai": req.nilai.len(), "waktu": now() }))
}

// ===== Data wali kelas (kehadiran, catatan, kenaikan) =====
// Tidak dikunci seperti nilai: wali kelas boleh mengirim ulang kapan saja selama sesi dibuka.

#[derive(Deserialize)]
struct WalasIn {
	peserta_didik_id: String,
	sakit: Option<i64>,
	izin: Option<i64>,
	alpa: Option<i64>,
	catatan: Option<String>,
	naik: Option<i64>,
	#[serde(default)]
	kokurikuler: Option<String>,
}

// Nilai ekskul yang diisi wali kelas untuk siswanya. Hanya mengisi yang masih kosong di admin,
// supaya nilai pembina tidak tertimpa data lama dari laptop wali kelas.
#[derive(Deserialize)]
struct EkskulWalasIn {
	rombongan_belajar_id: String,
	peserta_didik_id: String,
	predikat: Option<String>,
	keterangan: Option<String>,
}

#[derive(Deserialize)]
struct KokurikulerIn {
	kegiatan_id: i64,
	peserta_didik_id: String,
	capaian: String,
}

#[derive(Deserialize)]
struct KirimWalasReq {
	#[serde(flatten)]
	auth: Auth,
	semester_id: String,
	rombongan_belajar_id: String,
	siswa: Vec<WalasIn>,
	#[serde(default)]
	kokurikuler: Vec<KokurikulerIn>,
	#[serde(default)]
	ekskul: Vec<EkskulWalasIn>,
}

async fn api_kirim_walas(State(app): State<AppHandle>, Json(req): Json<KirimWalasReq>) -> ApiResult {
	let guru = auth(&app, &req.auth).await?;
	let hasil = {
		let state = app.state::<db::Db>();
		let mut conn = state.lock().unwrap();
		simpan_walas(&mut conn, &guru, &req)
	};
	match hasil {
		Ok(v) => {
			catat(&app, &guru.nama, "kirim data wali kelas", true, format!("{} · {} siswa", v["rombel"].as_str().unwrap_or(""), v["siswa"]));
			Ok(Json(v))
		}
		Err(msg) => {
			catat(&app, &guru.nama, "kirim data wali kelas", false, msg.clone());
			Err(fail(StatusCode::UNPROCESSABLE_ENTITY, msg))
		}
	}
}

fn simpan_walas(conn: &mut rusqlite::Connection, guru: &Guru, req: &KirimWalasReq) -> Result<Value, String> {
	let tx = conn.transaction().map_err(|e| e.to_string())?;
	let e = |e: rusqlite::Error| e.to_string();
	// Hanya wali kelas rombel itu, di semester yang sama.
	let nama: String = tx
		.query_row(
			"SELECT nama FROM rombel WHERE rombongan_belajar_id = ?1 AND semester_id = ?2 AND ptk_id = ?3 AND jenis_rombel = '1'",
			rusqlite::params![req.rombongan_belajar_id, req.semester_id, guru.ptk_id],
			|r| r.get(0),
		)
		.map_err(|_| "Anda bukan wali kelas rombel ini di semester admin. Tarik data ulang.".to_string())?;
	let anggota: HashSet<String> = {
		let mut stmt = tx.prepare("SELECT peserta_didik_id FROM anggota_rombel WHERE rombongan_belajar_id = ?1").map_err(e)?;
		let rows = stmt.query_map([&req.rombongan_belajar_id], |r| r.get(0)).map_err(e)?;
		rows.filter_map(Result::ok).collect()
	};
	let hari_wajar = |v: Option<i64>| v.is_none_or(|n| (0..=366).contains(&n));
	for s in &req.siswa {
		if !anggota.contains(&s.peserta_didik_id) {
			return Err(format!("Ada siswa yang bukan anggota {nama}. Tarik data ulang."));
		}
		if !hari_wajar(s.sakit) || !hari_wajar(s.izin) || !hari_wajar(s.alpa) {
			return Err("Jumlah hari kehadiran tidak wajar (0–366).".into());
		}
		tx.execute(
			"INSERT INTO rapor_siswa (semester_id, peserta_didik_id, rombongan_belajar_id, sakit, izin, alpa, catatan, naik, kokurikuler, updated_at)
			VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, datetime('now','localtime'))
			ON CONFLICT (semester_id, peserta_didik_id) DO UPDATE SET rombongan_belajar_id = excluded.rombongan_belajar_id,
				sakit = excluded.sakit, izin = excluded.izin, alpa = excluded.alpa, catatan = excluded.catatan, naik = excluded.naik,
				kokurikuler = excluded.kokurikuler, updated_at = excluded.updated_at",
			rusqlite::params![req.semester_id, s.peserta_didik_id, req.rombongan_belajar_id, s.sakit, s.izin, s.alpa, s.catatan, s.naik, s.kokurikuler],
		)
		.map_err(e)?;
	}
	// Capaian kokurikuler: kegiatan harus ada di semester ini, siswa harus anggota kelas.
	for k in &req.kokurikuler {
		if !anggota.contains(&k.peserta_didik_id) {
			return Err(format!("Ada nilai kokurikuler siswa yang bukan anggota {nama}. Tarik data ulang."));
		}
		let ada: bool = tx
			.prepare("SELECT 1 FROM kokurikuler_kegiatan WHERE id = ?1 AND semester_id = ?2")
			.map_err(e)?
			.exists(rusqlite::params![k.kegiatan_id, req.semester_id])
			.map_err(e)?;
		if !ada {
			return Err("Ada kegiatan kokurikuler yang sudah dihapus admin. Tarik data ulang.".into());
		}
		serde_json::from_str::<serde_json::Map<String, Value>>(&k.capaian).map_err(|_| "Format capaian kokurikuler tidak valid".to_string())?;
		tx.execute(
			"INSERT INTO nilai_kokurikuler (kegiatan_id, peserta_didik_id, capaian, updated_at) VALUES (?1, ?2, ?3, datetime('now','localtime'))
			ON CONFLICT (kegiatan_id, peserta_didik_id) DO UPDATE SET capaian = excluded.capaian, updated_at = excluded.updated_at",
			rusqlite::params![k.kegiatan_id, k.peserta_didik_id, k.capaian],
		)
		.map_err(e)?;
	}
	for x in &req.ekskul {
		if !anggota.contains(&x.peserta_didik_id) {
			return Err(format!("Ada nilai ekskul siswa yang bukan anggota {nama}. Tarik data ulang."));
		}
		if x.predikat.as_deref().is_some_and(|p| !["SB", "B", "C", "K"].contains(&p)) {
			return Err("Predikat ekskul harus SB, B, C, atau K.".into());
		}
		let ikut: bool = tx
			.prepare("SELECT 1 FROM anggota_rombel a JOIN rombel e USING (rombongan_belajar_id) WHERE a.rombongan_belajar_id = ?1 AND a.peserta_didik_id = ?2 AND e.jenis_rombel = '51' AND e.semester_id = ?3")
			.map_err(e)?
			.exists(rusqlite::params![x.rombongan_belajar_id, x.peserta_didik_id, req.semester_id])
			.map_err(e)?;
		if !ikut {
			return Err("Ada nilai ekskul untuk ekskul yang tidak diikuti siswa. Tarik data ulang.".into());
		}
		tx.execute(
			"INSERT INTO nilai_ekskul (semester_id, rombongan_belajar_id, peserta_didik_id, predikat, keterangan, updated_at)
			VALUES (?1, ?2, ?3, ?4, ?5, datetime('now','localtime'))
			ON CONFLICT (semester_id, rombongan_belajar_id, peserta_didik_id) DO UPDATE SET
				predikat = excluded.predikat, keterangan = excluded.keterangan, updated_at = excluded.updated_at
			WHERE nilai_ekskul.predikat IS NULL",
			rusqlite::params![req.semester_id, x.rombongan_belajar_id, x.peserta_didik_id, x.predikat, x.keterangan],
		)
		.map_err(e)?;
	}
	tx.commit().map_err(e)?;
	Ok(json!({ "ok": true, "rombel": nama, "siswa": req.siswa.len(), "waktu": now() }))
}

// ===== Nilai ekstrakurikuler (oleh pembina) =====
// Seperti data wali kelas: tidak dikunci, pembina boleh mengirim ulang selama sesi dibuka.

#[derive(Deserialize)]
struct EkskulIn {
	peserta_didik_id: String,
	predikat: Option<String>,
	keterangan: Option<String>,
}

#[derive(Deserialize)]
struct KirimEkskulReq {
	#[serde(flatten)]
	auth: Auth,
	semester_id: String,
	rombongan_belajar_id: String,
	siswa: Vec<EkskulIn>,
}

async fn api_kirim_ekskul(State(app): State<AppHandle>, Json(req): Json<KirimEkskulReq>) -> ApiResult {
	let guru = auth(&app, &req.auth).await?;
	let hasil = {
		let state = app.state::<db::Db>();
		let mut conn = state.lock().unwrap();
		simpan_ekskul(&mut conn, &guru, &req)
	};
	match hasil {
		Ok(v) => {
			catat(&app, &guru.nama, "kirim nilai ekskul", true, format!("{} · {} siswa", v["ekskul"].as_str().unwrap_or(""), v["siswa"]));
			Ok(Json(v))
		}
		Err(msg) => {
			catat(&app, &guru.nama, "kirim nilai ekskul", false, msg.clone());
			Err(fail(StatusCode::UNPROCESSABLE_ENTITY, msg))
		}
	}
}

fn simpan_ekskul(conn: &mut rusqlite::Connection, guru: &Guru, req: &KirimEkskulReq) -> Result<Value, String> {
	let tx = conn.transaction().map_err(|e| e.to_string())?;
	let e = |e: rusqlite::Error| e.to_string();
	// Hanya pembina ekskul itu, di semester yang sama.
	let nama: String = tx
		.query_row(
			"SELECT nama FROM rombel WHERE rombongan_belajar_id = ?1 AND semester_id = ?2 AND ptk_id = ?3 AND jenis_rombel = '51'",
			rusqlite::params![req.rombongan_belajar_id, req.semester_id, guru.ptk_id],
			|r| r.get(0),
		)
		.map_err(|_| "Anda bukan pembina ekskul ini di semester admin. Tarik data ulang.".to_string())?;
	let anggota: HashSet<String> = {
		let mut stmt = tx.prepare("SELECT peserta_didik_id FROM anggota_rombel WHERE rombongan_belajar_id = ?1").map_err(e)?;
		let rows = stmt.query_map([&req.rombongan_belajar_id], |r| r.get(0)).map_err(e)?;
		rows.filter_map(Result::ok).collect()
	};
	for s in &req.siswa {
		if !anggota.contains(&s.peserta_didik_id) {
			return Err(format!("Ada siswa yang bukan anggota {nama}. Tarik data ulang."));
		}
		if s.predikat.as_deref().is_some_and(|p| !["SB", "B", "C", "K"].contains(&p)) {
			return Err("Predikat ekskul harus SB, B, C, atau K.".into());
		}
		tx.execute(
			"INSERT INTO nilai_ekskul (semester_id, rombongan_belajar_id, peserta_didik_id, predikat, keterangan, updated_at)
			VALUES (?1, ?2, ?3, ?4, ?5, datetime('now','localtime'))
			ON CONFLICT (semester_id, rombongan_belajar_id, peserta_didik_id) DO UPDATE SET
				predikat = excluded.predikat, keterangan = excluded.keterangan, updated_at = excluded.updated_at",
			rusqlite::params![req.semester_id, req.rombongan_belajar_id, s.peserta_didik_id, s.predikat, s.keterangan],
		)
		.map_err(e)?;
	}
	tx.commit().map_err(e)?;
	Ok(json!({ "ok": true, "ekskul": nama, "siswa": req.siswa.len(), "waktu": now() }))
}

#[cfg(test)]
mod tests {
	// Dijalankan manual dengan ERAPOR_TEST_DIR berisi admin/erapor.sqlite (salinan data asli).
	use super::*;

	fn dir() -> PathBuf {
		PathBuf::from(std::env::var("ERAPOR_TEST_DIR").expect("set ERAPOR_TEST_DIR"))
	}

	fn guru_kelas4(conn: &rusqlite::Connection) -> Guru {
		let (ptk_id, nama): (String, String) = conn
			.query_row("SELECT ptk_id, nama FROM ptk WHERE nama LIKE 'Maspupah%'", [], |r| Ok((r.get(0)?, r.get(1)?)))
			.unwrap();
		Guru { nama, ptk_id }
	}

	// Uji jalur online sungguhan (butuh internet): CF_EXE = path cloudflared.exe.
	#[tokio::test]
	#[ignore]
	async fn c_tunnel_nyata() {
		let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
		let port = listener.local_addr().unwrap().port();
		let router = Router::new().route("/api/info", get(|| async { Json(json!({ "app": "uji" })) }));
		tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
		let target = format!("http://127.0.0.1:{port}");

		let mut cf = tokio::process::Command::new(std::env::var("CF_EXE").unwrap());
		cf.args(["tunnel", "--no-autoupdate", "--url", &target]);
		let (mut c1, u1) = jalankan_tunnel(cf, |u| u.contains(".trycloudflare.com"), 45).await.unwrap();

		let mut ssh = tokio::process::Command::new(ssh_exe());
		ssh.args(["-o", "StrictHostKeyChecking=no", "-o", "UserKnownHostsFile=NUL", "-T", "-R", &format!("80:127.0.0.1:{port}"), "nokey@localhost.run"]);
		let (mut c2, u2) = jalankan_tunnel(ssh, |u| u.contains(".lhr.") && !u.contains("localhost.run"), 30).await.unwrap();

		for u in [&u1, &u2] {
			let mut body = String::new();
			for _ in 0..10 {
				tokio::time::sleep(std::time::Duration::from_secs(3)).await;
				if let Ok(r) = reqwest::get(format!("{u}/api/info")).await {
					body = r.text().await.unwrap_or_default();
					if body.contains("uji") {
						break;
					}
				}
			}
			println!("{u} → {body}");
			assert!(body.contains("uji"));
		}
		let _ = c1.start_kill();
		let _ = c2.start_kill();
	}

	#[test]
	#[ignore]
	fn a_dump_paket() {
		let admin = db::open(&dir().join("admin"));
		let conn = admin.lock().unwrap();
		let guru = guru_kelas4(&conn);
		let sem = semester_aktif(&conn).unwrap();
		let paket = build_paket(&conn, &guru.ptk_id, &sem).unwrap();
		for k in ["rombel", "anggota_rombel", "peserta_didik", "pembelajaran_rapor", "tujuan_pembelajaran", "nilai_rapor"] {
			println!("{k}: {}", paket[k].as_array().unwrap().len());
		}
		assert!(paket["peserta_didik"][0].get("nik").is_none(), "NIK tidak boleh ikut");
		let out = json!({ "guru": { "nama": guru.nama, "username": "maspupah", "ptkId": guru.ptk_id }, "paket": paket });
		std::fs::write(dir().join("paket.json"), out.to_string()).unwrap();
		// database kosong untuk laptop guru (skema lengkap + migrasi)
		drop(db::open(&dir().join("guru")));
	}

	#[test]
	#[ignore]
	fn b_terima_kiriman() {
		let admin = db::open(&dir().join("admin"));
		let mut conn = admin.lock().unwrap();
		let guru = guru_kelas4(&conn);
		let raw = std::fs::read_to_string(dir().join("kirim.json")).unwrap();
		let mut req: KirimReq = serde_json::from_str(&raw).unwrap();

		// Kurang satu siswa → ditolak.
		let simpan = req.nilai.pop().unwrap();
		let err = simpan_kiriman(&mut conn, &guru, &req).unwrap_err();
		println!("tidak lengkap → {err}");
		assert!(err.contains("belum lengkap"));
		req.nilai.push(simpan);

		let ok = simpan_kiriman(&mut conn, &guru, &req).unwrap();
		println!("diterima → {ok}");
		let terkunci: i64 = conn
			.query_row(&format!("SELECT COUNT(*) FROM pembelajaran_rapor WHERE terkunci = 1 AND id IN ({})",
				req.pembelajaran_ids.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(",")), [], |r| r.get(0))
			.unwrap();
		assert_eq!(terkunci as usize, req.pembelajaran_ids.len());
		let tp_guru: i64 = conn.query_row("SELECT COUNT(*) FROM tujuan_pembelajaran WHERE sumber = 'guru'", [], |r| r.get(0)).unwrap();
		println!("TP buatan guru di admin: {tp_guru}");

		// Kirim ulang → ditolak karena terkunci.
		let err = simpan_kiriman(&mut conn, &guru, &req).unwrap_err();
		println!("kirim ulang → {err}");
		assert!(err.contains("terkunci"));
	}
}
