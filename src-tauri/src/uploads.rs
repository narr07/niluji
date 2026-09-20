use base64::{engine::general_purpose::STANDARD, Engine};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const ALLOWED_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "webp"];

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

	let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
	let filename = format!("{nanos}.{ext}");
	let dest = uploads_dir.join(&filename);

	std::fs::write(&dest, bytes).map_err(|e| format!("gagal menyimpan gambar: {e}"))?;
	let data_url = to_data_url(&dest)?;

	Ok(SavedImage { path: format!("uploads/{filename}"), data_url })
}

pub fn load_data_url(uploads_dir: &Path, relative_path: &str) -> Result<String, String> {
	let filename = relative_path.strip_prefix("uploads/").unwrap_or(relative_path);
	to_data_url(&uploads_dir.join(filename))
}

pub fn resolve_dir(app_data_dir: &Path) -> PathBuf {
	app_data_dir.join("uploads")
}
