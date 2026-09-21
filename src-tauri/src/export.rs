use crate::db::Db;
use rusqlite::params;
use serde::Serialize;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportSummary {
	pub file_path: String,
	pub questions_exported: usize,
	pub images_exported: usize
}

// Nama file/folder harus aman dipakai lintas OS & enak dibaca di GitHub — huruf kecil, spasi dan
// simbol lain diganti "-", tanpa tanda hubung berulang di ujung.
fn slugify(s: &str) -> String {
	let raw: String = s.trim().to_lowercase().chars().map(|c| if c.is_alphanumeric() { c } else { '-' }).collect();
	raw.split('-').filter(|part| !part.is_empty()).collect::<Vec<_>>().join("-")
}

// Format ekspor: satu file Markdown per jenis ujian (frontmatter YAML + soal bernomor dengan
// heading, pilihan sebagai list, kunci & skor sebagai baris "Label: nilai"). Sengaja bukan .docx
// — supaya gampang dibaca ulang (manusia maupun parser sederhana) nanti kalau mau diimpor lagi,
// dan gampang diunggah ke repo GitHub seperti halnya folder db-soal Excel yang sudah ada.
pub fn export_jenis_soal(
	db: &Db,
	uploads_dir: &Path,
	output_dir: &str,
	kelas: &str,
	subject_id: i64,
	jenis: &str,
	pin: &str
) -> Result<ExportSummary, String> {
	let conn = db.lock().unwrap();

	let export_pin: String = conn
		.query_row("SELECT export_pin FROM school WHERE id = 1", [], |r| r.get(0))
		.map_err(|e| e.to_string())?;
	if pin.trim() != export_pin.trim() {
		return Err("PIN salah.".to_string());
	}

	let subject_name: String = conn
		.query_row("SELECT name FROM subjects WHERE id = ?1", params![subject_id], |r| r.get(0))
		.map_err(|_| "Mata pelajaran tidak ditemukan".to_string())?;
	let subject_code: Option<String> =
		conn.query_row("SELECT code FROM subjects WHERE id = ?1", params![subject_id], |r| r.get(0)).unwrap_or(None);

	let mut stmt = conn
		.prepare(
			"SELECT id, question_text, question_type, image, score, answer_key
			 FROM questions WHERE subject_id = ?1 AND class = ?2 AND jenis = ?3
			 ORDER BY id"
		)
		.map_err(|e| e.to_string())?;
	#[allow(clippy::type_complexity)]
	let rows = stmt
		.query_map(params![subject_id, kelas, jenis], |r| {
			Ok((
				r.get::<_, i64>(0)?,
				r.get::<_, String>(1)?,
				r.get::<_, String>(2)?,
				r.get::<_, Option<String>>(3)?,
				r.get::<_, f64>(4)?,
				r.get::<_, Option<String>>(5)?
			))
		})
		.map_err(|e| e.to_string())?;
	let question_rows: Vec<(i64, String, String, Option<String>, f64, Option<String>)> =
		rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;
	drop(stmt);

	if question_rows.is_empty() {
		return Err("Belum ada soal untuk jenis ujian ini.".to_string());
	}

	let subject_slug = slugify(subject_code.as_deref().filter(|c| !c.is_empty()).unwrap_or(&subject_name));
	let jenis_slug = slugify(jenis);
	let target_dir = Path::new(output_dir).join(format!("kelas_{kelas}")).join(&subject_slug);
	let image_dir = target_dir.join("gambar");
	std::fs::create_dir_all(&target_dir).map_err(|e| format!("Gagal membuat folder tujuan: {e}"))?;

	let exported_at = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

	let mut md = String::new();
	md.push_str("---\n");
	md.push_str(&format!("kelas: {kelas}\n"));
	md.push_str(&format!("pelajaran: {subject_name}\n"));
	md.push_str(&format!("jenis: {jenis}\n"));
	md.push_str(&format!("diekspor_unix: {exported_at}\n"));
	md.push_str("---\n\n");

	let mut images_exported = 0usize;

	for (idx, (qid, question_text, question_type, image, score, answer_key)) in question_rows.iter().enumerate() {
		let num = idx + 1;
		let tipe_label = if question_type == "essay" { "Esai" } else { "Pilihan Ganda" };
		md.push_str(&format!("## {num}. {tipe_label}\n\n"));

		if let Some(img) = image {
			let filename = img.strip_prefix("uploads/").unwrap_or(img);
			let source = uploads_dir.join(filename);
			if source.exists() {
				std::fs::create_dir_all(&image_dir).map_err(|e| format!("Gagal membuat folder gambar: {e}"))?;
				let ext = Path::new(filename).extension().and_then(|e| e.to_str()).unwrap_or("jpg");
				let dest_name = format!("soal-{qid}.{ext}");
				std::fs::copy(&source, image_dir.join(&dest_name)).map_err(|e| format!("Gagal menyalin gambar soal #{num}: {e}"))?;
				md.push_str(&format!("![gambar](gambar/{dest_name})\n\n"));
				images_exported += 1;
			}
		}

		md.push_str(question_text.trim());
		md.push_str("\n\n");

		if question_type != "essay" {
			let mut opt_stmt = conn
				.prepare("SELECT option_key, option_text FROM question_options WHERE question_id = ?1 ORDER BY option_key")
				.map_err(|e| e.to_string())?;
			let opts: Vec<(String, String)> = opt_stmt
				.query_map(params![qid], |r| Ok((r.get(0)?, r.get(1)?)))
				.map_err(|e| e.to_string())?
				.collect::<Result<_, _>>()
				.map_err(|e| e.to_string())?;
			drop(opt_stmt);

			for (key, text) in &opts {
				md.push_str(&format!("- {key}. {text}\n"));
			}
			md.push('\n');
			if let Some(key) = answer_key {
				if !key.trim().is_empty() {
					md.push_str(&format!("Kunci: {}\n", key.trim()));
				}
			}
		}
		md.push_str(&format!("Skor: {score}\n\n"));
	}

	let file_path = target_dir.join(format!("{jenis_slug}.md"));
	std::fs::write(&file_path, md).map_err(|e| format!("Gagal menulis file ekspor: {e}"))?;

	Ok(ExportSummary {
		file_path: file_path.to_string_lossy().to_string(),
		questions_exported: question_rows.len(),
		images_exported
	})
}
