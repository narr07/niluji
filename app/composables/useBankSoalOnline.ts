import { invoke } from "@tauri-apps/api/core";
import { fetch as tauriFetch } from "@tauri-apps/plugin-http";
import JSZip from "jszip";

// Sumber bank soal = folder di repo GitHub publik, berisi file Markdown hasil fitur Export:
//   {path}/kelas_4/{kode-mapel}/{jenis-ujian}.md   (+ folder gambar/ di sebelahnya)
// Bawaan memakai repo NILUJI sendiri; kecamatan lain bisa memakai repo yang disiapkan operator
// kecamatannya. File diambil dari raw.githubusercontent.com (jsDelivr menolak sebagian tipe file
// dengan 403), sedangkan LISTING folder lewat GitHub API karena raw host tidak bisa listing.
export interface BankSoalSource {
	owner: string
	repo: string
	branch: string
	/** Folder di dalam repo tempat folder kelas_* berada ("" = root repo). */
	path: string
}

export const DEFAULT_BANK_SOAL_SOURCE: BankSoalSource = { owner: "narr07", repo: "niluji", branch: "main", path: "db-soal" };
export const CDN_HEALTHCHECK_URL = "https://api.github.com/";

export interface OnlineSubject {
	id: number
	name: string
	code: string | null
}

export interface BankSoalOnlineRow {
	id: string
	rowNumber: number
	tipe: "pg" | "esai"
	soal: string
	jenis: string
	pilihan_a: string
	pilihan_b: string
	pilihan_c: string
	pilihan_d: string
	kunci_jawaban: string
	skor: number | string
	nama_file_gambar: string
	urlGambar?: string
	pathRelatifGambar?: string
	valid: boolean
	alasan_invalid?: string
}

export interface BankSoalOnlineResult {
	rows: BankSoalOnlineRow[]
	fileName: string
	fileUrl: string
	folderPath: string
	jenis: string
}

export interface BankSoalConnectionStatus {
	connected: boolean
	checkedAt: Date
	message: string
}

export interface JenisFileOption {
	fileName: string
	label: string
}

const clean = (value: unknown) => String(value ?? "").trim();
const encodePath = (path: string) => path.split("/").filter(Boolean).map(encodeURIComponent).join("/");
const joinPath = (...parts: string[]) => parts.filter(Boolean).join("/");

export const sourceLabel = (source: BankSoalSource) => `${source.owner}/${source.repo}${source.path ? `/${source.path}` : ""}`;
export const sourceRepoUrl = (source: BankSoalSource) =>
	`https://github.com/${source.owner}/${source.repo}${source.path ? `/tree/${source.branch}/${encodePath(source.path)}` : ""}`;
const rawUrl = (source: BankSoalSource, relative: string) =>
	`https://raw.githubusercontent.com/${source.owner}/${source.repo}/${encodeURIComponent(source.branch)}/${encodePath(joinPath(source.path, relative))}`;

/** Kode di tabel subjects menjadi nama folder URL yang aman dan konsisten. */
export const subjectFolderCode = (subject: OnlineSubject) => {
	const source = clean(subject.code) || clean(subject.name);
	return slugify(source);
};

const slugify = (value: string) =>
	value
		.toLocaleLowerCase("id-ID")
		.normalize("NFD")
		.replace(/[̀-ͯ]/g, "")
		.replace(/[^a-z0-9]+/g, "-")
		.replace(/^-+|-+$/g, "");

/** Cocokkan folder mapel di repo dengan mapel lokal — lewat kode dulu, lalu nama. */
export const matchLocalSubject = (folder: string, subjects: OnlineSubject[]) =>
	subjects.find((s) => subjectFolderCode(s) === folder) ?? subjects.find((s) => slugify(s.name) === folder);

export const buildFolderPath = (kelas: string, mataPelajaran: string) => `${kelas}/${mataPelajaran}`;

// GitHub API tanpa login dibatasi ±60 permintaan/jam per IP — satu sekolah yang berbagi
// internet bisa kena, jadi pesan errornya dibuat jelas, bukan sekadar "status 403".
const githubJson = async <T>(url: string): Promise<T | null> => {
	const response = await tauriFetch(url, { headers: { Accept: "application/vnd.github+json" } });
	if (response.status === 404) return null;
	if (response.status === 403 || response.status === 429) {
		throw new Error("Batas akses GitHub tercapai (sekitar 60 permintaan per jam per jaringan). Tunggu beberapa menit lalu coba lagi.");
	}
	if (!response.ok) throw new Error(`GitHub merespons status ${response.status}.`);
	return (await response.json()) as T;
};

interface GithubEntry { name: string, type: string }

const listDir = async (source: BankSoalSource, relative: string) => {
	const path = encodePath(joinPath(source.path, relative));
	const items = await githubJson<GithubEntry[] | GithubEntry>(
		`https://api.github.com/repos/${source.owner}/${source.repo}/contents/${path}?ref=${encodeURIComponent(source.branch)}`
	);
	return Array.isArray(items) ? items : [];
};

const KELAS_FOLDER = /^kelas_(\w+)$/i;

/** Daftar folder kelas_* yang ada di sumber. */
export const listOnlineClasses = async (source: BankSoalSource) => {
	const entries = await listDir(source, "");
	return entries
		.filter((e) => e.type === "dir" && KELAS_FOLDER.test(e.name))
		.map((e) => ({ label: `Kelas ${e.name.match(KELAS_FOLDER)![1]!.toUpperCase()}`, value: e.name }))
		.sort((a, b) => a.value.localeCompare(b.value, "id", { numeric: true }));
};

/** Daftar folder mata pelajaran di dalam satu folder kelas. */
export const listSubjectFolders = async (source: BankSoalSource, kelas: string) => {
	const entries = await listDir(source, kelas);
	return entries.filter((e) => e.type === "dir").map((e) => e.name).sort();
};

/** Daftar file jenis ujian (.md hasil export) yang ada di folder kelas+pelajaran tertentu. */
export const listJenisFiles = async (source: BankSoalSource, kelas: string, mataPelajaran: string): Promise<JenisFileOption[]> => {
	const entries = await listDir(source, buildFolderPath(kelas, mataPelajaran));
	return entries
		.filter((item) => item.type === "file" && item.name.toLowerCase().endsWith(".md") && item.name.toLowerCase() !== "readme.md")
		.map((item) => ({ fileName: item.name, label: jenisLabel(item.name) }));
};

// Terima link repo dalam bentuk apa pun yang biasa disalin dari browser:
//   https://github.com/nama/repo
//   https://github.com/nama/repo/tree/main/db-soal
//   github.com/nama/repo.git
// Branch & folder yang tidak disebut dicari otomatis: branch default repo, lalu folder "db-soal"
// kalau ada, selain itu root repo. Sumber dianggap valid kalau ada minimal satu folder kelas_*.
export const resolveGithubSource = async (link: string): Promise<BankSoalSource> => {
	const trimmed = link.trim().replace(/^(?!https?:\/\/)/, "https://");
	let url: URL;
	try {
		url = new URL(trimmed);
	} catch {
		throw new Error("Link tidak valid. Contoh link yang benar: https://github.com/nama-akun/nama-repo");
	}
	if (url.hostname !== "github.com" && url.hostname !== "www.github.com") {
		throw new Error("Link harus link repo GitHub (diawali https://github.com/...).");
	}

	const [owner, repoRaw, kind, branchFromLink, ...rest] = url.pathname.split("/").filter(Boolean).map(decodeURIComponent);
	const repo = repoRaw?.replace(/\.git$/, "");
	if (!owner || !repo) throw new Error("Link harus menyebut akun dan nama repo, misalnya https://github.com/nama-akun/nama-repo");

	const info = await githubJson<{ default_branch: string, private: boolean }>(`https://api.github.com/repos/${owner}/${repo}`);
	if (!info) throw new Error(`Repo ${owner}/${repo} tidak ditemukan. Pastikan ejaan link benar dan repo diatur Public.`);

	const branch = kind === "tree" && branchFromLink ? branchFromLink : info.default_branch;
	let path = kind === "tree" ? rest.join("/") : "";

	if (!path) {
		const root = await listDir({ owner, repo, branch, path: "" }, "");
		if (root.some((e) => e.type === "dir" && e.name.toLowerCase() === "db-soal")) {
			path = root.find((e) => e.name.toLowerCase() === "db-soal")!.name;
		}
	}

	const source = { owner, repo, branch, path };
	const classes = await listOnlineClasses(source);
	if (!classes.length) {
		throw new Error(`Repo ditemukan, tapi tidak ada folder kelas_4, kelas_5, dst. di ${path ? `folder "${path}"` : "root repo"}. Ikuti struktur folder di panduan.`);
	}
	return source;
};

// Parser buat format Markdown hasil fitur Export (lihat src-tauri/src/export.rs) — frontmatter
// YAML sederhana (kelas/pelajaran/jenis) diikuti soal bernomor pakai heading "## N. Tipe",
// gambar (opsional) sebagai baris pertama isi blok, pilihan sebagai list "- A. teks", lalu baris
// "Kunci: X" dan "Skor: N". Sengaja regex manual (bukan library markdown) karena strukturnya
// sudah pasti — kita sendiri yang nulis exporter-nya.
// Dipakai juga untuk import file .md LOKAL (lihat SoalImportMarkdownModal.vue) — tanpa `source`,
// urlGambar tidak diisi dan pemanggil mencari gambarnya sendiri dari `nama_file_gambar`.
export const parseSoalMarkdown = (text: string, folderPath: string, source?: BankSoalSource): { jenis: string, rows: BankSoalOnlineRow[] } => {
	const fmMatch = text.match(/^---\r?\n([\s\S]*?)\r?\n---\r?\n/);
	const frontmatter: Record<string, string> = {};
	let body = text;
	if (fmMatch) {
		for (const line of fmMatch[1]!.split(/\r?\n/)) {
			const m = line.match(/^([a-z_]+):\s*(.*)$/i);
			if (m) frontmatter[m[1]!] = m[2]!.trim();
		}
		body = text.slice(fmMatch[0].length);
	}

	const headingRegex = /^##\s*(\d+)\.\s*(Pilihan Ganda|Esai)\s*$/gm;
	const matches = [...body.matchAll(headingRegex)];
	const rows: BankSoalOnlineRow[] = [];

	matches.forEach((m, i) => {
		const number = Number(m[1]);
		const tipeLabel = m[2];
		const start = m.index! + m[0].length;
		const end = i + 1 < matches.length ? matches[i + 1]!.index! : body.length;
		let content = body.slice(start, end).trim();

		let imageFile: string | undefined;
		const imgMatch = content.match(/^!\[gambar]\(gambar\/([^)]+)\)\s*\n*/);
		if (imgMatch) {
			imageFile = imgMatch[1];
			content = content.slice(imgMatch[0].length);
		}

		const optionMatches = [...content.matchAll(/^-\s*([A-D])\.\s*(.+)$/gm)];
		const kunciMatch = content.match(/^Kunci:\s*(.+)$/m);
		const skorMatch = content.match(/^Skor:\s*([\d.]+)$/m);

		let soalEnd = content.length;
		if (optionMatches.length) soalEnd = optionMatches[0]!.index!;
		else if (skorMatch) soalEnd = skorMatch.index!;
		const soal = content.slice(0, soalEnd).trim();

		const tipe: "pg" | "esai" = tipeLabel === "Esai" ? "esai" : "pg";
		const pilihan: Record<string, string> = {};
		for (const om of optionMatches) pilihan[om[1]!.toLowerCase()] = om[2]!.trim();
		const kunci_jawaban = kunciMatch ? kunciMatch[1]!.trim() : "";
		const skor = skorMatch ? Number(skorMatch[1]) : 1;

		const filledChoices = Object.values(pilihan).filter(Boolean);
		const invalidReasons: string[] = [];
		if (!soal) invalidReasons.push("teks soal kosong");
		if (tipe === "pg") {
			if (filledChoices.length < 2) invalidReasons.push("kurang dari 2 pilihan terisi");
			if (!kunci_jawaban) invalidReasons.push("kunci jawaban kosong");
		}

		const nama_file_gambar = imageFile ?? "";
		const pathRelatifGambar = imageFile ? `${folderPath}/gambar/${imageFile}` : undefined;

		rows.push({
			id: `${tipe}-${number}`,
			rowNumber: number,
			tipe,
			soal,
			jenis: frontmatter.jenis ?? "",
			pilihan_a: pilihan.a ?? "",
			pilihan_b: pilihan.b ?? "",
			pilihan_c: pilihan.c ?? "",
			pilihan_d: pilihan.d ?? "",
			kunci_jawaban,
			skor,
			nama_file_gambar,
			...(pathRelatifGambar
				? { pathRelatifGambar, ...(source ? { urlGambar: rawUrl(source, pathRelatifGambar) } : {}) }
				: {}),
			valid: invalidReasons.length === 0,
			...(invalidReasons.length ? { alasan_invalid: invalidReasons.join("; ") } : {})
		});
	});

	return { jenis: frontmatter.jenis ?? "", rows };
};

export const fetchBankSoalMarkdown = async (
	source: BankSoalSource,
	kelas: string,
	mataPelajaran: string,
	fileName: string
): Promise<BankSoalOnlineResult> => {
	const folderPath = buildFolderPath(kelas, mataPelajaran);
	const response = await tauriFetch(rawUrl(source, joinPath(folderPath, fileName)));
	if (!response.ok) throw new Error(`File ${fileName} tidak ditemukan di folder ${folderPath}.`);
	const text = await response.text();
	const { jenis, rows } = parseSoalMarkdown(text, folderPath, source);
	if (!rows.length) throw new Error(`File ${fileName} terbaca, tapi tidak ada soal dengan format "## 1. Pilihan Ganda" / "## 1. Esai" di dalamnya.`);
	return { rows, fileName, fileUrl: response.url, folderPath, jenis };
};

// ---------------------------------------------------------------------------------------------
// Provider: satu antarmuka untuk semua jenis sumber (repo GitHub, atau file ZIP dari link
// Google Drive/Dropbox/dll. maupun dari flashdisk), supaya komponen Tarik Soal tidak perlu tahu
// datanya datang dari mana. Struktur foldernya sama persis: kelas_*/{mapel}/{jenis}.md + gambar/.
// ---------------------------------------------------------------------------------------------

export interface BankSoalProvider {
	label: string
	listClasses: () => Promise<{ label: string, value: string }[]>
	listSubjects: (kelas: string) => Promise<string[]>
	listJenis: (kelas: string, mapel: string) => Promise<JenisFileOption[]>
	readMarkdown: (kelas: string, mapel: string, fileName: string) => Promise<BankSoalOnlineResult>
	/** Ambil gambar soal; ekstensi ditebak kalau nama filenya tanpa ekstensi. */
	readImage: (pathRelatif: string) => Promise<{ bytes: Uint8Array, fileName: string } | null>
}

const IMAGE_EXTENSIONS = ["jpg", "jpeg", "png", "gif", "webp"];
const imageCandidates = (path: string) =>
	/\.(jpg|jpeg|png|gif|webp)$/i.test(path) ? [path] : IMAGE_EXTENSIONS.map((ext) => `${path}.${ext}`);

const jenisLabel = (fileName: string) =>
	fileName
		.replace(/\.md$/i, "")
		.split("-")
		.map((w) => (w ? w.charAt(0).toUpperCase() + w.slice(1) : w))
		.join(" ");

export const githubProvider = (source: BankSoalSource): BankSoalProvider => ({
	label: sourceLabel(source),
	listClasses: () => listOnlineClasses(source),
	listSubjects: (kelas) => listSubjectFolders(source, kelas),
	listJenis: (kelas, mapel) => listJenisFiles(source, kelas, mapel),
	readMarkdown: (kelas, mapel, fileName) => fetchBankSoalMarkdown(source, kelas, mapel, fileName),
	readImage: async (pathRelatif) => {
		for (const candidate of imageCandidates(pathRelatif)) {
			const response = await tauriFetch(rawUrl(source, candidate));
			if (response.ok) return { bytes: new Uint8Array(await response.arrayBuffer()), fileName: candidate.split("/").pop()! };
		}
		return null;
	}
});

export const zipProvider = async (bytes: ArrayBuffer | Uint8Array, label: string): Promise<BankSoalProvider> => {
	const head = new Uint8Array(bytes instanceof Uint8Array ? bytes.subarray(0, 2) : bytes.slice(0, 2));
	if (head[0] !== 0x50 || head[1] !== 0x4B) {
		throw new Error("File yang didapat bukan ZIP. Kalau dari Google Drive, pastikan file dibagikan \"Siapa saja yang memiliki link\", dan yang dibagikan adalah file .zip (bukan folder).");
	}
	const zip = await JSZip.loadAsync(bytes);

	// Folder kelas_* boleh ada di root ZIP atau di dalam folder pembungkus (mis. "db-soal/" atau
	// "bank-soal-kecamatan/db-soal/") — cari prefix sampai folder kelas_* pertama.
	const files = Object.values(zip.files).filter((f) => !f.dir).map((f) => f.name.replace(/\\/g, "/"));
	let root: string | null = null;
	for (const name of files) {
		const parts = name.split("/");
		const idx = parts.findIndex((p) => KELAS_FOLDER.test(p));
		if (idx >= 0) {
			root = parts.slice(0, idx).join("/");
			break;
		}
	}
	if (root === null) {
		throw new Error("Isi ZIP tidak berisi folder kelas_4, kelas_5, dst. ZIP harus berisi folder db-soal hasil Export (lihat panduan).");
	}
	const prefix = root ? `${root}/` : "";
	// Path relatif terhadap root, dipetakan huruf kecil → path asli di ZIP (nama gambar di file .md
	// kadang beda besar-kecil hurufnya dengan file sebenarnya).
	const byLower = new Map<string, string>();
	for (const name of files) if (name.startsWith(prefix)) byLower.set(name.slice(prefix.length).toLowerCase(), name);
	const relPaths = [...byLower.values()].map((n) => n.slice(prefix.length));

	const childDirs = (dir: string) => {
		const base = dir ? `${dir}/` : "";
		const set = new Set<string>();
		for (const rel of relPaths) {
			if (!rel.startsWith(base)) continue;
			const rest = rel.slice(base.length).split("/");
			if (rest.length > 1) set.add(rest[0]!);
		}
		return [...set].sort();
	};

	const readText = async (rel: string) => {
		const real = byLower.get(rel.toLowerCase());
		return real ? zip.file(real)!.async("string") : null;
	};

	return {
		label,
		listClasses: async () =>
			childDirs("")
				.filter((d) => KELAS_FOLDER.test(d))
				.map((d) => ({ label: `Kelas ${d.match(KELAS_FOLDER)![1]!.toUpperCase()}`, value: d }))
				.sort((a, b) => a.value.localeCompare(b.value, "id", { numeric: true })),
		listSubjects: async (kelas) => childDirs(kelas),
		listJenis: async (kelas, mapel) => {
			const base = `${kelas}/${mapel}/`;
			return relPaths
				.filter((rel) => rel.startsWith(base) && !rel.slice(base.length).includes("/") && /\.md$/i.test(rel) && !/readme\.md$/i.test(rel))
				.map((rel) => rel.slice(base.length))
				.sort()
				.map((fileName) => ({ fileName, label: jenisLabel(fileName) }));
		},
		readMarkdown: async (kelas, mapel, fileName) => {
			const folderPath = buildFolderPath(kelas, mapel);
			const text = await readText(`${folderPath}/${fileName}`);
			if (text === null) throw new Error(`File ${fileName} tidak ditemukan di folder ${folderPath}.`);
			const { jenis, rows } = parseSoalMarkdown(text, folderPath);
			if (!rows.length) throw new Error(`File ${fileName} terbaca, tapi tidak ada soal dengan format "## 1. Pilihan Ganda" / "## 1. Esai" di dalamnya.`);
			return { rows, fileName, fileUrl: `${label}/${folderPath}/${fileName}`, folderPath, jenis };
		},
		readImage: async (pathRelatif) => {
			for (const candidate of imageCandidates(pathRelatif)) {
				const real = byLower.get(candidate.toLowerCase());
				if (real) return { bytes: await zip.file(real)!.async("uint8array"), fileName: real.split("/").pop()! };
			}
			return null;
		}
	};
};

// Link berbagi dari layanan populer diubah ke link unduhan langsung. Link FOLDER Google Drive
// tidak bisa dibaca tanpa API key, jadi ditolak dengan petunjuk untuk membagikan file .zip-nya.
export const toDirectDownloadUrl = (link: string) => {
	let url: URL;
	try {
		url = new URL(link.trim());
	} catch {
		throw new Error("Link tidak valid. Tempel link lengkap yang diawali https://");
	}

	if (url.hostname === "drive.google.com" || url.hostname === "docs.google.com") {
		if (url.pathname.includes("/folders/")) {
			throw new Error("Ini link FOLDER Google Drive — tidak bisa dibaca langsung. Kompres folder db-soal jadi file .zip, unggah file zip-nya ke Drive, lalu bagikan link file tersebut.");
		}
		const id = url.pathname.match(/\/d\/([\w-]+)/)?.[1] ?? url.searchParams.get("id");
		if (!id) throw new Error("Link Google Drive tidak dikenali. Gunakan link dari tombol Bagikan → Salin link pada file .zip.");
		return `https://drive.usercontent.google.com/download?id=${id}&export=download&confirm=t`;
	}
	if (url.hostname.endsWith("dropbox.com")) {
		url.searchParams.set("dl", "1");
		return url.toString();
	}
	return url.toString();
};

export const zipProviderFromLink = async (link: string) => {
	const direct = toDirectDownloadUrl(link);
	const bytes = await invoke<ArrayBuffer>("download_bytes", { url: direct });
	const host = new URL(direct).hostname.replace(/^www\./, "");
	return zipProvider(bytes, host.includes("google") ? "ZIP dari Google Drive" : `ZIP dari ${host}`);
};

export const zipProviderFromFile = async (path: string) => {
	const bytes = await invoke<number[]>("read_file_bytes", { path });
	return zipProvider(new Uint8Array(bytes), path.split(/[/\\]/).pop() ?? path);
};

/** Memastikan GitHub benar-benar dapat diakses dari aplikasi offline. */
export const checkBankSoalConnection = async (): Promise<BankSoalConnectionStatus> => {
	try {
		const response = await tauriFetch(CDN_HEALTHCHECK_URL, { cache: "no-store" });
		if (!response.ok && response.status !== 403) throw new Error(`Server merespons status ${response.status}`);
		return {
			connected: true,
			checkedAt: new Date(),
			message: "Internet terhubung dan GitHub dapat dijangkau."
		};
	} catch {
		return {
			connected: false,
			checkedAt: new Date(),
			message: "Tidak ada koneksi internet atau GitHub tidak dapat dijangkau."
		};
	}
};
