pub mod cmds;
pub mod db;
pub mod sesi;

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tauri::Manager;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DapodikRequest {
	pub base_url: String,
	pub path: String,
	pub token: String,
	pub npsn: String,
	#[serde(default)]
	pub semester_id: Option<String>,
	#[serde(default)]
	pub method: Option<String>,
	#[serde(default)]
	pub body: Option<serde_json::Value>,
	#[serde(default)]
	pub raw_body: Option<String>,
	#[serde(default)]
	pub timeout_secs: Option<u64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DapodikResponse {
	pub url: String,
	pub status: u16,
	pub elapsed_ms: u128,
	pub size_bytes: usize,
	pub content_type: String,
	pub body: String,
}

#[tauri::command]
pub async fn dapodik_request(req: DapodikRequest) -> Result<DapodikResponse, String> {
	let base = req.base_url.trim().trim_end_matches('/');
	let path = req.path.trim().trim_start_matches('/');
	let url = format!("{base}/{path}");

	let client = reqwest::Client::builder()
		.timeout(Duration::from_secs(req.timeout_secs.unwrap_or(30)))
		.build()
		.map_err(|e| e.to_string())?;

	let method = req.method.as_deref().unwrap_or("GET").to_uppercase();
	let mut builder = match method.as_str() {
		"POST" => client.post(&url),
		_ => client.get(&url),
	};
	builder = builder.query(&[("npsn", req.npsn.trim())]);
	if let Some(sem) = req.semester_id.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
		builder = builder.query(&[("semester_id", sem)]);
	}
	builder = builder
		.bearer_auth(req.token.trim())
		.header("Accept", "application/json");
	if let Some(raw) = &req.raw_body {
		builder = builder.header("Content-Type", "application/x-www-form-urlencoded").body(raw.clone());
	} else if let Some(body) = &req.body {
		builder = builder.json(body);
	}

	let started = Instant::now();
	let res = builder.send().await.map_err(|e| {
		if e.is_timeout() {
			format!("Timeout: Dapodik tidak menjawab dalam {} detik ({url})", req.timeout_secs.unwrap_or(30))
		} else if e.is_connect() {
			format!("Tidak bisa terhubung ke {url}. Cek IP/port, pastikan Dapodik menyala dan port 5774 tidak diblok firewall. Detail: {e}")
		} else {
			format!("Request gagal: {e}")
		}
	})?;

	let status = res.status().as_u16();
	let final_url = res.url().to_string();
	let content_type = res
		.headers()
		.get(reqwest::header::CONTENT_TYPE)
		.and_then(|v| v.to_str().ok())
		.unwrap_or("")
		.to_string();
	let body = res.text().await.map_err(|e| format!("Gagal membaca response: {e}"))?;

	Ok(DapodikResponse {
		url: final_url,
		status,
		elapsed_ms: started.elapsed().as_millis(),
		size_bytes: body.len(),
		content_type,
		body,
	})
}

pub fn data_dir(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
	match std::env::var("ERAPOR_DATA_DIR") {
		Ok(d) if !d.trim().is_empty() => Ok(std::path::PathBuf::from(d)),
		_ => app.path().app_data_dir().map_err(|e| e.to_string()),
	}
}

fn samples_dir(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
	let dir = data_dir(app)?.join("samples");
	std::fs::create_dir_all(&dir).map_err(|e| format!("Gagal membuat folder sampel: {e}"))?;
	Ok(dir)
}

#[tauri::command]
pub fn save_sample(app: tauri::AppHandle, name: String, content: String) -> Result<String, String> {
	let safe: String = name
		.chars()
		.map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
		.collect();
	let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
	let dest = samples_dir(&app)?.join(format!("{safe}-{ts}.json"));
	std::fs::write(&dest, content).map_err(|e| format!("Gagal menyimpan sampel: {e}"))?;
	Ok(dest.to_string_lossy().into_owned())
}

#[tauri::command]
pub fn open_samples_dir(app: tauri::AppHandle) -> Result<String, String> {
	let dir = samples_dir(&app)?;
	cmds::open_in_explorer(&dir)?;
	Ok(dir.to_string_lossy().into_owned())
}
