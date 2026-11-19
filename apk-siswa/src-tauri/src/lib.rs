mod address;
mod probe;

use address::ServerAddress;
use serde::Serialize;
use std::sync::Mutex;
use tauri::{Manager, State, Url, WebviewUrl, WebviewWindowBuilder};

/// Server ujian yang sedang dibuka. Hanya halaman di server ini yang boleh dimuat WebView.
#[derive(Default)]
struct ExamState(Mutex<Option<ServerAddress>>);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AppInfo {
	version: String,
	mobile: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ServerCheck {
	base_url: String,
	exam_url: String,
}

#[tauri::command]
fn app_info(app: tauri::AppHandle) -> AppInfo {
	AppInfo { version: app.package_info().version.to_string(), mobile: cfg!(mobile) }
}

async fn verify(input: String) -> Result<ServerAddress, String> {
	let address = address::parse(&input)?;
	let probed = address.clone();
	tauri::async_runtime::spawn_blocking(move || probe::check(&probed))
		.await
		.map_err(|_| probe::ERR_UNREACHABLE.to_string())??;
	Ok(address)
}

#[tauri::command(async)]
async fn check_server(address: String) -> Result<ServerCheck, String> {
	let address = verify(address).await?;
	Ok(ServerCheck { base_url: address.base_url(), exam_url: address.exam_url() })
}

#[tauri::command(async)]
async fn open_exam(app: tauri::AppHandle, state: State<'_, ExamState>, address: String) -> Result<(), String> {
	let address = verify(address).await?;
	let url = Url::parse(&address.exam_url()).map_err(|e| e.to_string())?;
	*state.0.lock().unwrap() = Some(address);
	let window = app.get_webview_window("main").ok_or("Jendela aplikasi tidak ditemukan.")?;
	window.navigate(url).map_err(|e| e.to_string())
}

/// Halaman milik aplikasi sendiri (layar awal), di semua platform dan saat pengembangan.
fn is_app_page(url: &Url, dev_url: Option<&Url>) -> bool {
	if url.scheme() == "tauri" || url.scheme() == "about" {
		return true;
	}
	let host = url.host_str().unwrap_or_default();
	if host.eq_ignore_ascii_case("tauri.localhost") && matches!(url.scheme(), "http" | "https") {
		return true;
	}
	dev_url.is_some_and(|dev| dev.scheme() == url.scheme() && dev.host_str() == url.host_str() && dev.port_or_known_default() == url.port_or_known_default())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
	let builder = tauri::Builder::default();
	#[cfg(mobile)]
	let builder = builder.plugin(tauri_plugin_barcode_scanner::init());

	builder
		.manage(ExamState::default())
		.invoke_handler(tauri::generate_handler![app_info, check_server, open_exam])
		.setup(|app| {
			let dev_url = if tauri::is_dev() { app.config().build.dev_url.clone() } else { None };
			let handle = app.handle().clone();
			let builder = WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into())).on_navigation(move |url| {
				if is_app_page(url, dev_url.as_ref()) {
					return true;
				}
				let exam = handle.state::<ExamState>();
				let allowed = exam.0.lock().unwrap().as_ref().is_some_and(|server| server.allows(url));
				allowed
			});
			#[cfg(desktop)]
			let builder = builder.title("NILUJI Siswa").inner_size(420.0, 820.0).min_inner_size(360.0, 600.0);
			builder.build()?;
			Ok(())
		})
		.run(tauri::generate_context!())
		.expect("gagal menjalankan NILUJI Siswa");
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn recognises_the_app_start_page() {
		assert!(is_app_page(&Url::parse("http://tauri.localhost/").unwrap(), None));
		assert!(is_app_page(&Url::parse("tauri://localhost/index.html").unwrap(), None));
		assert!(!is_app_page(&Url::parse("http://192.168.1.10/cbt").unwrap(), None));
		let dev = Url::parse("http://localhost:3000").unwrap();
		assert!(is_app_page(&Url::parse("http://localhost:3000/").unwrap(), Some(&dev)));
		assert!(!is_app_page(&Url::parse("http://localhost:3000/").unwrap(), None));
	}
}
