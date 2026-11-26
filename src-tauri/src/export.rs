use crate::db::{secrets_match, Db};
use rusqlite::{params, Connection};
use serde::Serialize;
use std::path::{Path, PathBuf};
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
pub fn slugify(s: &str) -> String {
	let raw: String = s.trim().to_lowercase().chars().map(|c| if c.is_alphanumeric() { c } else { '-' }).collect();
	raw.split('-').filter(|part| !part.is_empty()).collect::<Vec<_>>().join("-")
}

// Satu jenis ujian yang sudah diubah ke format Markdown, beserta gambar yang dirujuknya.
// Dipakai bersama oleh Export ke folder dan Publikasikan ke bank soal online.
pub struct RenderedPaket {
	pub markdown: String,
	// (nama file di folder "gambar/", lokasi file aslinya di folder uploads)
	pub images: Vec<(String, PathBuf)>,
	pub subject_name: String,
	pub subject_code: Option<String>,
	pub pg: usize,
	pub esai: usize
}

impl RenderedPaket {
	pub fn subject_slug(&self) -> String {
		slugify(self.subject_code.as_deref().filter(|c| !c.trim().is_empty()).unwrap_or(&self.subject_name))
	}
}

// Format ekspor: satu file Markdown per jenis ujian (frontmatter YAML + soal bernomor dengan
// heading, pilihan sebagai list, kunci & skor sebagai baris "Label: nilai"). Sengaja bukan .docx
// — supaya gampang dibaca ulang (manusia maupun parser sederhana) nanti kalau mau diimpor lagi,
// dan gampang diunggah ke repo GitHub. `title` adalah nama jenis ujian yang tertulis di file;
// bank soal lokal dibaca dari `jenis`.
pub fn render_paket(
	conn: &Connection,
	uploads_dir: &Path,
	kelas: &str,
	subject_id: i64,
	jenis: &str,
	title: &str
) -> Result<RenderedPaket, String> {
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
	#[allow(clippy::type_complexity)]
	let question_rows: Vec<(i64, String, String, Option<String>, f64, Option<String>)> =
		rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;
	drop(stmt);

	if question_rows.is_empty() {
		return Err("Belum ada soal untuk jenis ujian ini.".to_string());
	}

	let exported_at = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
	let mut md = String::new();
	md.push_str("---\n");
	md.push_str(&format!("kelas: {kelas}\n"));
	md.push_str(&format!("pelajaran: {subject_name}\n"));
	md.push_str(&format!("jenis: {}\n", title.trim()));
	md.push_str(&format!("diekspor_unix: {exported_at}\n"));
	md.push_str("---\n\n");

	let mut images = Vec::new();
	let mut pg = 0usize;
	let mut esai = 0usize;
	let image_source = |img: &str| {
		let filename = img.strip_prefix("uploads/").unwrap_or(img).to_string();
		let source = uploads_dir.join(&filename);
		source.exists().then(|| {
			let ext = Path::new(&filename).extension().and_then(|e| e.to_str()).unwrap_or("jpg").to_lowercase();
			(source, ext)
		})
	};

	for (idx, (qid, question_text, question_type, image, score, answer_key)) in question_rows.iter().enumerate() {
		let num = idx + 1;
		let is_essay = question_type == "essay";
		if is_essay {
			esai += 1;
		} else {
			pg += 1;
		}
		let tipe_label = if is_essay { "Esai" } else { "Pilihan Ganda" };
		md.push_str(&format!("## {num}. {tipe_label}\n\n"));

		if let Some((source, ext)) = image.as_deref().and_then(image_source) {
			let dest_name = format!("soal-{qid}.{ext}");
			md.push_str(&format!("![gambar](gambar/{dest_name})\n\n"));
			images.push((dest_name, source));
		}

		md.push_str(question_text.trim());
		md.push_str("\n\n");

		if !is_essay {
			let mut opt_stmt = conn
				.prepare("SELECT option_key, option_text, image FROM question_options WHERE question_id = ?1 ORDER BY option_key")
				.map_err(|e| e.to_string())?;
			let opts: Vec<(String, String, Option<String>)> = opt_stmt
				.query_map(params![qid], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
				.map_err(|e| e.to_string())?
				.collect::<Result<_, _>>()
				.map_err(|e| e.to_string())?;
			drop(opt_stmt);

			for (key, text, opt_img) in &opts {
				let mut opt_img_tag = String::new();
				if let Some((source, ext)) = opt_img.as_deref().and_then(image_source) {
					let dest_name = format!("soal-{qid}-opsi-{}.{ext}", key.to_lowercase());
					opt_img_tag = format!(" ![opsi-{key}](gambar/{dest_name})");
					images.push((dest_name, source));
				}
				md.push_str(&format!("- {key}. {}{opt_img_tag}\n", text.trim()));
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

	Ok(RenderedPaket { markdown: md, images, subject_name, subject_code, pg, esai })
}

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
		.query_row("SELECT COALESCE(export_pin, '') FROM school WHERE id = 1", [], |r| r.get(0))
		.map_err(|e| e.to_string())?;
	if export_pin.trim().is_empty() {
		return Err("PIN Export belum diatur. Isi dulu di Pengaturan > Data Sekolah.".to_string());
	}
	if !secrets_match(pin.trim(), export_pin.trim()) {
		return Err("PIN salah.".to_string());
	}

	let paket = render_paket(&conn, uploads_dir, kelas, subject_id, jenis, jenis)?;
	drop(conn);

	let target_dir = Path::new(output_dir).join(format!("kelas_{kelas}")).join(paket.subject_slug());
	std::fs::create_dir_all(&target_dir).map_err(|e| format!("Gagal membuat folder tujuan: {e}"))?;
	if !paket.images.is_empty() {
		let image_dir = target_dir.join("gambar");
		std::fs::create_dir_all(&image_dir).map_err(|e| format!("Gagal membuat folder gambar: {e}"))?;
		for (dest_name, source) in &paket.images {
			std::fs::copy(source, image_dir.join(dest_name)).map_err(|e| format!("Gagal menyalin gambar {dest_name}: {e}"))?;
		}
	}

	let file_path = target_dir.join(format!("{}.md", slugify(jenis)));
	std::fs::write(&file_path, &paket.markdown).map_err(|e| format!("Gagal menulis file ekspor: {e}"))?;

	Ok(ExportSummary {
		file_path: file_path.to_string_lossy().to_string(),
		questions_exported: paket.pg + paket.esai,
		images_exported: paket.images.len()
	})
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn export_requires_a_configured_pin() {
		let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
		let dir = std::env::temp_dir().join(format!("niluji_export_test_{}_{nanos}", std::process::id()));
		let db = crate::db::open(&dir);
		let subject = {
			let conn = db.lock().unwrap();
			conn.execute("INSERT INTO subjects (name) VALUES ('Uji Export')", []).unwrap();
			let subject = conn.last_insert_rowid();
			conn.execute(
				"INSERT INTO questions (subject_id, class, jenis, question_text, answer_key) VALUES (?1, '6', 'STS', 'Soal', 'A')",
				params![subject]
			)
			.unwrap();
			subject
		};
		let output = dir.join("hasil");
		let run = |pin: &str| export_jenis_soal(&db, &dir, output.to_str().unwrap(), "6", subject, "STS", pin);

		assert!(run("1234").unwrap_err().contains("belum diatur"));
		db.lock().unwrap().execute("UPDATE school SET export_pin = '8642'", []).unwrap();
		assert_eq!(run("1234").unwrap_err(), "PIN salah.");
		assert_eq!(run("8642").unwrap().questions_exported, 1);
		drop(db);
		let _ = std::fs::remove_dir_all(&dir);
	}

	#[test]
	fn rendered_paket_uses_the_public_title_and_counts_question_types() {
		let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
		let dir = std::env::temp_dir().join(format!("niluji_render_test_{}_{nanos}", std::process::id()));
		let db = crate::db::open(&dir);
		let conn = db.lock().unwrap();
		conn.execute("INSERT INTO subjects (name, code) VALUES ('Matematika Uji Render', 'MTK')", []).unwrap();
		let subject = conn.last_insert_rowid();
		conn.execute(
			"INSERT INTO questions (subject_id, class, jenis, question_text, question_type, answer_key) VALUES
			 (?1, '4', 'STS G1', 'Soal PG', 'multiple_choice', 'A'), (?1, '4', 'STS G1', 'Soal esai', 'essay', NULL)",
			params![subject]
		)
		.unwrap();
		let paket = render_paket(&conn, &dir, "4", subject, "STS G1", "STS Ganjil 2026").unwrap();
		assert!(paket.markdown.contains("jenis: STS Ganjil 2026\n"));
		assert_eq!((paket.pg, paket.esai, paket.subject_slug()), (1, 1, "mtk".to_string()));
		drop(conn);
		drop(db);
		let _ = std::fs::remove_dir_all(&dir);
	}
}
