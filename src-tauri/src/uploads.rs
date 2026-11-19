use base64::{engine::general_purpose::STANDARD, Engine};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const ALLOWED_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "webp"];
// Batas wajar buat gambar soal — cukup longgar untuk foto scan resolusi tinggi, tapi cegah
// guru tidak sengaja pilih file yang salah (video, atau foto mentah kamera puluhan MB) yang
// bikin database & folder backup membengkak tanpa perlu.
const MAX_IMAGE_BYTES: u64 = 10 * 1024 * 1024;

fn mime_for_ext(ext: &str) -> &'static str {
	match ext {
		"png" => "image/png",
		"gif" => "image/gif",
		"webp" => "image/webp",
		_ => "image/jpeg"
	}
}

fn to_data_url(path: &Path) -> Result<String, String> {
	let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("jpg").to_lowercase();
	let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
	Ok(format!("data:{};base64,{}", mime_for_ext(&ext), STANDARD.encode(bytes)))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedImage {
	pub path: String,
	pub data_url: String
}

pub fn save_image(uploads_dir: &Path, source_path: &str) -> Result<SavedImage, String> {
	std::fs::create_dir_all(uploads_dir).map_err(|e| e.to_string())?;

	let ext = Path::new(source_path)
		.extension()
		.and_then(|e| e.to_str())
		.map(|e| e.to_lowercase())
		.filter(|e| ALLOWED_EXTENSIONS.contains(&e.as_str()))
		.ok_or("Format gambar tidak didukung. Gunakan JPG, PNG, GIF, atau WEBP.")?;

	let size = std::fs::metadata(source_path).map_err(|e| e.to_string())?.len();
	if size > MAX_IMAGE_BYTES {
		return Err(format!("Ukuran gambar {:.1} MB melebihi batas maksimal 10 MB.", size as f64 / 1024.0 / 1024.0));
	}

	let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
	let filename = format!("{nanos}.{ext}");
	let dest = uploads_dir.join(&filename);

	std::fs::copy(source_path, &dest).map_err(|e| format!("gagal menyalin gambar: {e}"))?;
	let data_url = to_data_url(&dest)?;

	Ok(SavedImage { path: format!("uploads/{filename}"), data_url })
}

// Same naming/validation scheme as `save_image`, but for bytes already fetched in memory
// (e.g. downloaded from a URL on the frontend) instead of an existing local file.
pub fn save_image_bytes(uploads_dir: &Path, source_name: &str, bytes: &[u8]) -> Result<SavedImage, String> {
	std::fs::create_dir_all(uploads_dir).map_err(|e| e.to_string())?;

	let ext = Path::new(source_name)
		.extension()
		.and_then(|e| e.to_str())
		.map(|e| e.to_lowercase())
		.filter(|e| ALLOWED_EXTENSIONS.contains(&e.as_str()))
		.ok_or("Format gambar tidak didukung. Gunakan JPG, PNG, GIF, atau WEBP.")?;

	if bytes.len() as u64 > MAX_IMAGE_BYTES {
		return Err(format!("Ukuran gambar {:.1} MB melebihi batas maksimal 10 MB.", bytes.len() as f64 / 1024.0 / 1024.0));
	}

	let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
	let filename = format!("{nanos}.{ext}");
	let dest = uploads_dir.join(&filename);

	std::fs::write(&dest, bytes).map_err(|e| format!("gagal menyimpan gambar: {e}"))?;
	let data_url = to_data_url(&dest)?;

	Ok(SavedImage { path: format!("uploads/{filename}"), data_url })
}

pub fn load_data_url(uploads_dir: &Path, relative_path: &str) -> Result<String, String> {
	let filename = relative_path.strip_prefix("uploads/").unwrap_or(relative_path);
	let is_plain_image = !filename.contains(['/', '\\', ':'])
		&& Path::new(filename)
			.extension()
			.and_then(|e| e.to_str())
			.is_some_and(|e| ALLOWED_EXTENSIONS.contains(&e.to_lowercase().as_str()));
	if !is_plain_image {
		return Err("Lokasi gambar tidak valid.".to_string());
	}
	to_data_url(&uploads_dir.join(filename))
}

pub fn resolve_dir(app_data_dir: &Path) -> PathBuf {
	app_data_dir.join("uploads")
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn image_loader_stays_inside_the_uploads_folder() {
		let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
		let root = std::env::temp_dir().join(format!("niluji_uploads_test_{}_{nanos}", std::process::id()));
		let uploads = root.join("uploads");
		std::fs::create_dir_all(&uploads).unwrap();
		std::fs::write(uploads.join("1.png"), b"gambar").unwrap();
		std::fs::write(root.join("rahasia.png"), b"rahasia").unwrap();
		std::fs::write(root.join("cbt.sqlite"), b"db").unwrap();

		assert!(load_data_url(&uploads, "uploads/1.png").is_ok());
		assert!(load_data_url(&uploads, "1.png").is_ok());
		for attack in ["../rahasia.png", "uploads/../rahasia.png", "..\\rahasia.png", "../cbt.sqlite", "C:/Windows/win.png", "cbt.sqlite"] {
			assert!(load_data_url(&uploads, attack).is_err(), "{attack}");
		}
		let _ = std::fs::remove_dir_all(&root);
	}
}
