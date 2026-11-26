// Bank soal per gugus yang dipublikasikan pembuat soal ke repo GitHub, lalu ditarik sekolah.
//
// Susunan repo:
//   katalog.json                                     daftar wilayah, gugus, dan semua paket soal
//   <kecamatan>/<gugus>/kelas_<k>/<mapel>/<jenis>/soal.md (+ gambar/)
//
// Hanya pemilik token GitHub yang punya hak tulis ke repo yang bisa mempublikasikan: GitHub
// sendiri yang menolak penulisan tanpa hak itu, dan aplikasi menyembunyikan tombolnya bila
// token belum tersimpan. Token disimpan di penyimpanan kunci sistem operasi, tidak di database.

use crate::db::Db;
use crate::export::{render_paket, slugify};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub const REPO_OWNER: &str = "narr07";
pub const REPO_NAME: &str = "niluji-bank-soal";
const BRANCH: &str = "main";
const KATALOG_PATH: &str = "katalog.json";
const PAKET_FILE: &str = "soal.md";
pub const SEMUA_GUGUS: &str = "semua-gugus";
const KEYRING_SERVICE: &str = "niluji-bank-soal";
const KEYRING_USER: &str = "github-token";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Gugus {
	pub id: String,
	pub nama: String
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Kecamatan {
	pub id: String,
	pub nama: String,
	pub gugus: Vec<Gugus>
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PaketEntry {
	pub kecamatan: String,
	// Id gugus, atau "semua-gugus" untuk soal yang sama di semua gugus.
	pub gugus: String,
	pub kelas: String,
	pub mapel_kode: String,
	pub mapel_nama: String,
	pub jenis: String,
	// Folder paket di repo; file soalnya `<folder>/soal.md`, gambarnya `<folder>/gambar/`.
	pub folder: String,
	pub pg: usize,
	pub esai: usize,
	pub diperbarui: u64
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Katalog {
	pub versi: u32,
	pub diperbarui: u64,
	pub kecamatan: Vec<Kecamatan>,
	pub paket: Vec<PaketEntry>
}

impl Default for Katalog {
	fn default() -> Self {
		Katalog { versi: 1, diperbarui: 0, kecamatan: Vec::new(), paket: Vec::new() }
	}
}

fn now_secs() -> u64 {
	SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

pub fn paket_folder(kecamatan: &str, gugus: &str, kelas: &str, mapel_slug: &str, jenis: &str) -> String {
	format!("{kecamatan}/{gugus}/kelas_{}/{mapel_slug}/{}", slugify(kelas), slugify(jenis))
}

// Menetapkan nama kecamatan dan daftar gugusnya. Id dibentuk dari nama, jadi paket yang sudah
// terbit tetap tertaut selama nama gugusnya tidak diganti.
pub fn set_wilayah(katalog: &mut Katalog, kecamatan_nama: &str, gugus_nama: &[String]) -> Result<Kecamatan, String> {
	let nama = kecamatan_nama.trim();
	if nama.is_empty() {
		return Err("Nama kecamatan wajib diisi.".to_string());
	}
	let mut gugus = Vec::new();
	for g in gugus_nama.iter().map(|g| g.trim()).filter(|g| !g.is_empty()) {
		let id = slugify(g);
		if id == SEMUA_GUGUS || id.is_empty() {
			return Err(format!("Nama gugus \"{g}\" tidak bisa dipakai."));
		}
		if gugus.iter().any(|x: &Gugus| x.id == id) {
			return Err(format!("Nama gugus \"{g}\" ganda."));
		}
		gugus.push(Gugus { id, nama: g.to_string() });
	}
	if gugus.is_empty() {
		return Err("Isi minimal satu gugus.".to_string());
	}
	let kecamatan = Kecamatan { id: slugify(nama), nama: nama.to_string(), gugus };
	// Gugus yang dihapus (atau diganti namanya, karena id-nya ikut berubah) tidak boleh masih
	// punya paket terbit: sekolah tidak bisa lagi memilihnya, tapi soalnya tertinggal di repo.
	if let Some(existing) = katalog.kecamatan.iter().find(|k| k.id == kecamatan.id) {
		for lama in existing.gugus.iter().filter(|g| !kecamatan.gugus.iter().any(|b| b.id == g.id)) {
			let jumlah = katalog.paket.iter().filter(|p| p.kecamatan == kecamatan.id && p.gugus == lama.id).count();
			if jumlah > 0 {
				return Err(format!(
					"Gugus \"{}\" tidak bisa dihapus atau diganti namanya karena masih punya {jumlah} paket soal. Hapus paketnya dulu di daftar Sudah terbit.",
					lama.nama
				));
			}
		}
	}
	match katalog.kecamatan.iter_mut().find(|k| k.id == kecamatan.id) {
		Some(existing) => *existing = kecamatan.clone(),
		None => katalog.kecamatan.push(kecamatan.clone())
	}
	katalog.diperbarui = now_secs();
	Ok(kecamatan)
}

// Menambah atau mengganti paket di folder yang sama. Mengembalikan true bila menggantikan.
pub fn upsert_paket(katalog: &mut Katalog, entry: PaketEntry) -> bool {
	katalog.diperbarui = entry.diperbarui;
	match katalog.paket.iter_mut().find(|p| p.folder == entry.folder) {
		Some(existing) => {
			*existing = entry;
			true
		}
		None => {
			katalog.paket.push(entry);
			katalog.paket.sort_by(|a, b| (&a.kecamatan, &a.gugus, &a.kelas, &a.mapel_kode, &a.jenis).cmp(&(&b.kecamatan, &b.gugus, &b.kelas, &b.mapel_kode, &b.jenis)));
			false
		}
	}
}

// Mengeluarkan paket dari katalog. Hanya folder yang tercatat di katalog yang bisa dihapus, jadi
// perintah hapus tidak bisa dipakai untuk membuang folder lain di repo.
pub fn remove_paket(katalog: &mut Katalog, folder: &str) -> Result<PaketEntry, String> {
	let index = katalog
		.paket
		.iter()
		.position(|p| p.folder == folder)
		.ok_or("Paket ini sudah tidak ada di bank soal. Muat ulang daftarnya.")?;
	katalog.diperbarui = now_secs();
	Ok(katalog.paket.remove(index))
}

fn gugus_nama(katalog: &Katalog, kecamatan: &str, gugus: &str) -> String {
	katalog
		.kecamatan
		.iter()
		.filter(|k| k.id == kecamatan)
		.flat_map(|k| k.gugus.iter())
		.find(|g| g.id == gugus)
		.map(|g| g.nama.clone())
		.unwrap_or_else(|| "semua gugus".to_string())
}

// ---------------------------------------------------------------------------------------------
// Token pembuat soal
// ---------------------------------------------------------------------------------------------

fn keyring_entry() -> Result<keyring::Entry, String> {
	keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER).map_err(|e| format!("Penyimpanan kunci tidak tersedia: {e}"))
}

fn stored_token() -> Option<String> {
	keyring_entry().ok()?.get_password().ok().filter(|t| !t.trim().is_empty())
}

pub fn has_token() -> bool {
	stored_token().is_some()
}

pub fn clear_token() -> Result<(), String> {
	match keyring_entry()?.delete_credential() {
		Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
		Err(e) => Err(format!("Gagal menghapus token: {e}"))
	}
}

// Token hanya disimpan setelah terbukti punya hak tulis ke repo bank soal.
pub async fn save_token(token: &str) -> Result<(), String> {
	let token = token.trim();
	if token.is_empty() {
		return Err("Token kosong.".to_string());
	}
	let gh = GitHub::new(token.to_string());
	let repo = gh.get(&format!("/repos/{REPO_OWNER}/{REPO_NAME}")).await?;
	if repo["permissions"]["push"].as_bool() != Some(true) {
		return Err(format!("Token ini tidak punya hak tulis ke repo {REPO_OWNER}/{REPO_NAME}."));
	}
	keyring_entry()?.set_password(token).map_err(|e| format!("Gagal menyimpan token: {e}"))
}

// ---------------------------------------------------------------------------------------------
// GitHub API
// ---------------------------------------------------------------------------------------------

struct GitHub {
	token: String,
	client: reqwest::Client
}

impl GitHub {
	fn new(token: String) -> Self {
		GitHub { token, client: reqwest::Client::new() }
	}

	fn from_store() -> Result<Self, String> {
		stored_token()
			.map(GitHub::new)
			.ok_or_else(|| "Token pembuat soal belum disimpan. Isi dulu di bagian Akses pembuat soal.".to_string())
	}

	async fn send(&self, request: reqwest::RequestBuilder) -> Result<Option<Value>, String> {
		let response = request
			.header("Authorization", format!("Bearer {}", self.token))
			.header("Accept", "application/vnd.github+json")
			.header("X-GitHub-Api-Version", "2022-11-28")
			.header("User-Agent", "NILUJI")
			.send()
			.await
			.map_err(|e| format!("Gagal menghubungi GitHub: {e}"))?;
		let status = response.status();
		let text = response.text().await.map_err(|e| format!("Gagal membaca jawaban GitHub: {e}"))?;
		if status.as_u16() == 404 {
			return Ok(None);
		}
		if status.as_u16() == 401 {
			return Err("Token GitHub ditolak (kedaluwarsa atau dicabut). Simpan token baru.".to_string());
		}
		if !status.is_success() {
			let message = serde_json::from_str::<Value>(&text).ok().and_then(|v| v["message"].as_str().map(String::from)).unwrap_or(text);
			return Err(format!("GitHub menolak permintaan ({status}): {message}"));
		}
		serde_json::from_str(&text).map(Some).map_err(|e| format!("Jawaban GitHub tidak terbaca: {e}"))
	}

	fn url(path: &str) -> String {
		format!("https://api.github.com{path}")
	}

	async fn get(&self, path: &str) -> Result<Value, String> {
		self.get_optional(path).await?.ok_or_else(|| format!("Tidak ditemukan di GitHub: {path}. Pastikan repo {REPO_OWNER}/{REPO_NAME} sudah dibuat."))
	}

	async fn get_optional(&self, path: &str) -> Result<Option<Value>, String> {
		self.send(self.client.get(Self::url(path))).await
	}

	async fn post(&self, path: &str, body: Value) -> Result<Value, String> {
		let request = self.client.post(Self::url(path)).header("Content-Type", "application/json").body(body.to_string());
		self.send(request).await?.ok_or_else(|| format!("Tidak ditemukan di GitHub: {path}"))
	}

	async fn patch(&self, path: &str, body: Value) -> Result<Value, String> {
		let request = self.client.patch(Self::url(path)).header("Content-Type", "application/json").body(body.to_string());
		self.send(request).await?.ok_or_else(|| format!("Tidak ditemukan di GitHub: {path}"))
	}

	async fn read_katalog(&self) -> Result<Katalog, String> {
		let Some(file) = self.get_optional(&format!("/repos/{REPO_OWNER}/{REPO_NAME}/contents/{KATALOG_PATH}?ref={BRANCH}")).await? else {
			return Ok(Katalog::default());
		};
		let encoded: String = file["content"].as_str().unwrap_or_default().split_whitespace().collect();
		let bytes = STANDARD.decode(encoded).map_err(|e| format!("katalog.json rusak: {e}"))?;
		serde_json::from_slice(&bytes).map_err(|e| format!("katalog.json tidak terbaca: {e}"))
	}

	async fn create_blob(&self, bytes: &[u8]) -> Result<String, String> {
		let blob = self
			.post(&format!("/repos/{REPO_OWNER}/{REPO_NAME}/git/blobs"), json!({ "content": STANDARD.encode(bytes), "encoding": "base64" }))
			.await?;
		blob["sha"].as_str().map(String::from).ok_or_else(|| "GitHub tidak mengembalikan sha blob.".to_string())
	}

	// Satu commit berisi semua file baru, dan menghapus file lama di folder `replace_folder`
	// yang tidak ikut terkirim (misalnya gambar dari versi paket sebelumnya).
	async fn commit(&self, files: Vec<(String, Vec<u8>)>, replace_folder: Option<&str>, message: &str) -> Result<String, String> {
		let repo = format!("/repos/{REPO_OWNER}/{REPO_NAME}");
		let reference = self
			.get_optional(&format!("{repo}/git/ref/heads/{BRANCH}"))
			.await?
			.ok_or_else(|| format!("Repo {REPO_OWNER}/{REPO_NAME} masih kosong. Buat ulang dengan centang \"Add a README file\"."))?;
		let head = reference["object"]["sha"].as_str().ok_or("Ref GitHub tidak terbaca.")?.to_string();
		let head_commit = self.get(&format!("{repo}/git/commits/{head}")).await?;
		let base_tree = head_commit["tree"]["sha"].as_str().ok_or("Tree GitHub tidak terbaca.")?.to_string();

		let mut entries = Vec::new();
		for (path, bytes) in &files {
			let sha = self.create_blob(bytes).await?;
			entries.push(json!({ "path": path, "mode": "100644", "type": "blob", "sha": sha }));
		}
		if let Some(folder) = replace_folder {
			let tree = self.get(&format!("{repo}/git/trees/{base_tree}?recursive=1")).await?;
			let prefix = format!("{folder}/");
			for item in tree["tree"].as_array().into_iter().flatten() {
				let (Some(path), Some("blob")) = (item["path"].as_str(), item["type"].as_str()) else { continue };
				if path.starts_with(&prefix) && !files.iter().any(|(p, _)| p == path) {
					entries.push(json!({ "path": path, "mode": "100644", "type": "blob", "sha": Value::Null }));
				}
			}
		}

		let tree = self.post(&format!("{repo}/git/trees"), json!({ "base_tree": base_tree, "tree": entries })).await?;
		let tree_sha = tree["sha"].as_str().ok_or("Tree baru tidak terbaca.")?;
		let commit = self.post(&format!("{repo}/git/commits"), json!({ "message": message, "tree": tree_sha, "parents": [head] })).await?;
		let commit_sha = commit["sha"].as_str().ok_or("Commit baru tidak terbaca.")?.to_string();
		self.patch(&format!("{repo}/git/refs/heads/{BRANCH}"), json!({ "sha": commit_sha })).await?;
		Ok(commit_sha)
	}
}

// ---------------------------------------------------------------------------------------------
// Perintah pembuat soal
// ---------------------------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishRequest {
	pub kelas: String,
	pub subject_id: i64,
	// Jenis ujian di bank soal lokal pembuat soal, mis. "STS Ganjil G1".
	pub jenis: String,
	pub kecamatan: String,
	// Id gugus tujuan, atau "semua-gugus".
	pub gugus: String,
	// Nama jenis ujian yang dilihat sekolah, mis. "STS Ganjil 2026".
	pub judul: String
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishPlan {
	pub folder: String,
	pub pg: usize,
	pub esai: usize,
	pub gambar: usize,
	pub menggantikan: bool,
	pub mapel_kode: String,
	pub mapel_nama: String
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishResult {
	pub folder: String,
	pub file: usize,
	pub menggantikan: bool,
	pub commit: String
}

fn validate_target(katalog: &Katalog, req: &PublishRequest) -> Result<(), String> {
	if req.judul.trim().is_empty() {
		return Err("Nama jenis ujian untuk sekolah wajib diisi.".to_string());
	}
	let kecamatan = katalog
		.kecamatan
		.iter()
		.find(|k| k.id == req.kecamatan)
		.ok_or("Kecamatan belum terdaftar. Simpan daftar gugus dulu.")?;
	if req.gugus != SEMUA_GUGUS && !kecamatan.gugus.iter().any(|g| g.id == req.gugus) {
		return Err("Gugus tujuan tidak terdaftar.".to_string());
	}
	Ok(())
}

struct Prepared {
	plan: PublishPlan,
	files: Vec<(String, Vec<u8>)>,
	entry: PaketEntry
}

fn prepare(db: &Db, uploads_dir: &Path, katalog: &Katalog, req: &PublishRequest) -> Result<Prepared, String> {
	validate_target(katalog, req)?;
	let paket = render_paket(&db.lock().unwrap(), uploads_dir, &req.kelas, req.subject_id, &req.jenis, &req.judul)?;
	let mapel_kode = paket.subject_code.clone().filter(|c| !c.trim().is_empty()).unwrap_or_else(|| paket.subject_name.clone());
	let folder = paket_folder(&req.kecamatan, &req.gugus, &req.kelas, &paket.subject_slug(), &req.judul);
	let mut files = vec![(format!("{folder}/{PAKET_FILE}"), paket.markdown.clone().into_bytes())];
	for (name, source) in &paket.images {
		let bytes = std::fs::read(source).map_err(|e| format!("Gagal membaca gambar {name}: {e}"))?;
		files.push((format!("{folder}/gambar/{name}"), bytes));
	}
	let entry = PaketEntry {
		kecamatan: req.kecamatan.clone(),
		gugus: req.gugus.clone(),
		kelas: req.kelas.trim().to_string(),
		mapel_kode: mapel_kode.clone(),
		mapel_nama: paket.subject_name.clone(),
		jenis: req.judul.trim().to_string(),
		folder: folder.clone(),
		pg: paket.pg,
		esai: paket.esai,
		diperbarui: now_secs()
	};
	let plan = PublishPlan {
		menggantikan: katalog.paket.iter().any(|p| p.folder == folder),
		folder,
		pg: paket.pg,
		esai: paket.esai,
		gambar: paket.images.len(),
		mapel_kode,
		mapel_nama: paket.subject_name
	};
	Ok(Prepared { plan, files, entry })
}

pub async fn katalog_terbaru() -> Result<Katalog, String> {
	GitHub::from_store()?.read_katalog().await
}

pub async fn simpan_wilayah(kecamatan: &str, gugus: &[String]) -> Result<Katalog, String> {
	let gh = GitHub::from_store()?;
	let mut katalog = gh.read_katalog().await?;
	let wilayah = set_wilayah(&mut katalog, kecamatan, gugus)?;
	let json = serde_json::to_vec_pretty(&katalog).map_err(|e| e.to_string())?;
	gh.commit(vec![(KATALOG_PATH.to_string(), json)], None, &format!("Perbarui daftar gugus {}", wilayah.nama)).await?;
	Ok(katalog)
}

pub async fn rencana_publikasi(db: &Db, uploads_dir: &Path, req: &PublishRequest) -> Result<PublishPlan, String> {
	let katalog = GitHub::from_store()?.read_katalog().await?;
	Ok(prepare(db, uploads_dir, &katalog, req)?.plan)
}

pub async fn publikasikan(db: &Db, uploads_dir: &Path, req: &PublishRequest) -> Result<PublishResult, String> {
	let gh = GitHub::from_store()?;
	let mut katalog = gh.read_katalog().await?;
	let Prepared { plan, mut files, entry } = prepare(db, uploads_dir, &katalog, req)?;
	let tujuan = gugus_nama(&katalog, &req.kecamatan, &req.gugus);
	let menggantikan = upsert_paket(&mut katalog, entry);
	files.push((KATALOG_PATH.to_string(), serde_json::to_vec_pretty(&katalog).map_err(|e| e.to_string())?));
	let file = files.len();
	let message = format!("Publikasikan {} kelas {} {} untuk {}", req.judul.trim(), req.kelas.trim(), plan.mapel_nama, tujuan);
	let commit = gh.commit(files, Some(&plan.folder), &message).await?;
	Ok(PublishResult { folder: plan.folder, file, menggantikan, commit })
}

// Menghapus satu paket: entri katalog, soal.md, dan gambarnya dalam satu commit, supaya daftar di
// sekolah tidak pernah menunjuk ke folder yang sudah tidak ada.
pub async fn hapus_paket(folder: &str) -> Result<Katalog, String> {
	let gh = GitHub::from_store()?;
	let mut katalog = gh.read_katalog().await?;
	let entry = remove_paket(&mut katalog, folder)?;
	let asal = gugus_nama(&katalog, &entry.kecamatan, &entry.gugus);
	let json = serde_json::to_vec_pretty(&katalog).map_err(|e| e.to_string())?;
	let message = format!("Hapus {} kelas {} {} dari {}", entry.jenis, entry.kelas, entry.mapel_nama, asal);
	gh.commit(vec![(KATALOG_PATH.to_string(), json)], Some(&entry.folder), &message).await?;
	Ok(katalog)
}

// ---------------------------------------------------------------------------------------------
// Pilihan gugus sekolah
// ---------------------------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PilihanSekolah {
	pub kecamatan: Option<String>,
	pub gugus: Option<String>
}

pub fn pilihan_sekolah(db: &Db) -> Result<PilihanSekolah, String> {
	db.lock()
		.unwrap()
		.query_row("SELECT bank_kecamatan, bank_gugus FROM school WHERE id = 1", [], |r| {
			Ok(PilihanSekolah { kecamatan: r.get(0)?, gugus: r.get(1)? })
		})
		.map_err(|e| e.to_string())
}

pub fn simpan_pilihan_sekolah(db: &Db, pilihan: &PilihanSekolah) -> Result<(), String> {
	db.lock()
		.unwrap()
		.execute(
			"UPDATE school SET bank_kecamatan = ?1, bank_gugus = ?2 WHERE id = 1",
			rusqlite::params![pilihan.kecamatan, pilihan.gugus]
		)
		.map_err(|e| e.to_string())?;
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn school_remembers_its_gugus() {
		let dir = std::env::temp_dir().join(format!("niluji_bank_soal_{}_{}", std::process::id(), now_secs()));
		let db = crate::db::open(&dir);
		assert_eq!(pilihan_sekolah(&db).unwrap(), PilihanSekolah { kecamatan: None, gugus: None });
		let pilihan = PilihanSekolah { kecamatan: Some("rajagaluh".into()), gugus: Some("gugus-2".into()) };
		simpan_pilihan_sekolah(&db, &pilihan).unwrap();
		assert_eq!(pilihan_sekolah(&db).unwrap(), pilihan);
		drop(db);
		let _ = std::fs::remove_dir_all(&dir);
	}

	fn entry(folder: &str, jenis: &str) -> PaketEntry {
		PaketEntry {
			kecamatan: "rajagaluh".into(),
			gugus: "gugus-1".into(),
			kelas: "4".into(),
			mapel_kode: "MTK".into(),
			mapel_nama: "Matematika".into(),
			jenis: jenis.into(),
			folder: folder.into(),
			pg: 30,
			esai: 10,
			diperbarui: 1
		}
	}

	#[test]
	fn wilayah_gets_stable_ids_and_rejects_bad_names() {
		let mut katalog = Katalog::default();
		let kec = set_wilayah(&mut katalog, "Kecamatan Rajagaluh", &["Gugus 1".into(), " Gugus 2 ".into(), "".into()]).unwrap();
		assert_eq!(kec.id, "kecamatan-rajagaluh");
		assert_eq!(kec.gugus.iter().map(|g| g.id.as_str()).collect::<Vec<_>>(), ["gugus-1", "gugus-2"]);
		set_wilayah(&mut katalog, "Kecamatan Rajagaluh", &["Gugus 1".into(), "Gugus 2".into(), "Gugus 3".into()]).unwrap();
		assert_eq!(katalog.kecamatan.len(), 1, "kecamatan yang sama diperbarui, tidak ditambah");
		assert_eq!(katalog.kecamatan[0].gugus.len(), 3);

		assert!(set_wilayah(&mut katalog, "Rajagaluh", &["Semua Gugus".into()]).is_err());
		assert!(set_wilayah(&mut katalog, "Rajagaluh", &["Gugus A".into(), "gugus a".into()]).is_err());
		assert!(set_wilayah(&mut katalog, "  ", &["Gugus 1".into()]).is_err());
	}

	#[test]
	fn publishing_the_same_folder_replaces_the_entry() {
		let mut katalog = Katalog::default();
		assert!(!upsert_paket(&mut katalog, entry("a/gugus-1/kelas_4/mtk/sts", "STS")));
		assert!(!upsert_paket(&mut katalog, entry("a/gugus-1/kelas_4/mtk/uh", "UH")));
		assert!(upsert_paket(&mut katalog, entry("a/gugus-1/kelas_4/mtk/sts", "STS Revisi")));
		assert_eq!(katalog.paket.len(), 2);
		assert!(katalog.paket.iter().any(|p| p.jenis == "STS Revisi"));
	}

	#[test]
	fn gugus_with_published_paket_cannot_be_removed_or_renamed() {
		let mut katalog = Katalog::default();
		let gugus = |names: &[&str]| names.iter().map(|n| n.to_string()).collect::<Vec<_>>();
		set_wilayah(&mut katalog, "Rajagaluh", &gugus(&["Gugus 1", "Gugus 2", "Gugus 3"])).unwrap();
		upsert_paket(&mut katalog, entry("rajagaluh/gugus-1/kelas_4/mtk/sts", "STS"));

		let dihapus = set_wilayah(&mut katalog, "Rajagaluh", &gugus(&["Gugus 2", "Gugus 3"])).unwrap_err();
		assert!(dihapus.contains("\"Gugus 1\"") && dihapus.contains("1 paket"), "{dihapus}");
		assert!(set_wilayah(&mut katalog, "Rajagaluh", &gugus(&["Gugus Satu", "Gugus 2", "Gugus 3"])).is_err());
		assert_eq!(katalog.kecamatan[0].gugus.len(), 3, "daftar lama tetap utuh setelah ditolak");

		// Gugus tanpa paket boleh dihapus.
		set_wilayah(&mut katalog, "Rajagaluh", &gugus(&["Gugus 1", "Gugus 2"])).unwrap();
		// Setelah paketnya dihapus, gugus itu juga boleh dihapus.
		remove_paket(&mut katalog, "rajagaluh/gugus-1/kelas_4/mtk/sts").unwrap();
		set_wilayah(&mut katalog, "Rajagaluh", &gugus(&["Gugus 2"])).unwrap();
	}

	#[test]
	fn only_cataloged_paket_can_be_removed() {
		let mut katalog = Katalog::default();
		upsert_paket(&mut katalog, entry("rajagaluh/gugus-1/kelas_4/mtk/sts", "STS"));
		upsert_paket(&mut katalog, entry("rajagaluh/gugus-1/kelas_4/mtk/uh", "UH"));
		assert!(remove_paket(&mut katalog, "rajagaluh").is_err());
		assert!(remove_paket(&mut katalog, "").is_err());
		assert_eq!(remove_paket(&mut katalog, "rajagaluh/gugus-1/kelas_4/mtk/sts").unwrap().jenis, "STS");
		assert_eq!(katalog.paket.iter().map(|p| p.jenis.as_str()).collect::<Vec<_>>(), ["UH"]);
		assert!(remove_paket(&mut katalog, "rajagaluh/gugus-1/kelas_4/mtk/sts").is_err());
	}

	#[test]
	fn paket_folder_is_url_safe() {
		assert_eq!(paket_folder("rajagaluh", "gugus-2", "4", "mtk", "STS Ganjil 2026/2027"), "rajagaluh/gugus-2/kelas_4/mtk/sts-ganjil-2026-2027");
	}

	#[test]
	fn katalog_round_trips_as_camel_case_json() {
		let mut katalog = Katalog::default();
		upsert_paket(&mut katalog, entry("x", "STS"));
		let json = serde_json::to_string(&katalog).unwrap();
		assert!(json.contains(r#""mapelKode":"MTK""#));
		assert_eq!(serde_json::from_str::<Katalog>(&json).unwrap(), katalog);
	}
}
