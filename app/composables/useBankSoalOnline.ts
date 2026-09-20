import { fetch as tauriFetch } from "@tauri-apps/plugin-http";
import * as XLSX from "xlsx";

// jsDelivr's GitHub CDN (cdn.jsdelivr.net/gh/...) returns 403 Forbidden for Office document
// extensions (.xlsx, .docx, etc.) — verified against this exact repo, images work but every
// .xlsx returns 403. raw.githubusercontent.com serves the same files with 200 OK, so the bank
// soal fetch goes straight to GitHub's raw content host instead of the CDN.
export const CDN_BASE = "https://raw.githubusercontent.com/narr07/niluji/main/db-soal";
export const CDN_HEALTHCHECK_URL = "https://raw.githubusercontent.com/";
export const ONLINE_CLASSES = ["kelas_4", "kelas_5", "kelas_6"] as const;
export const ONLINE_CLASS_ITEMS = ONLINE_CLASSES.map((value) => ({
	label: `Kelas ${value.replace("kelas_", "")}`,
	value
}));

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
	essayFileName?: string
	folderPath: string
}

export interface BankSoalConnectionStatus {
	connected: boolean
	checkedAt: Date
	message: string
}

const clean = (value: unknown) => String(value ?? "").trim();

/** Kode di tabel subjects menjadi nama folder URL yang aman dan konsisten. */
export const subjectFolderCode = (subject: OnlineSubject) => {
	const source = clean(subject.code) || clean(subject.name);
	return source
		.toLocaleLowerCase("id-ID")
		.normalize("NFD")
		.replace(/[\u0300-\u036f]/g, "")
		.replace(/[^a-z0-9]+/g, "-")
		.replace(/^-+|-+$/g, "");
};

export const buildFolderPath = (kelas: string, mataPelajaran: string) => `${kelas}/${mataPelajaran}`;

export const buildQuestionFileName = (kelas: string, mataPelajaran: string) => {
	const grade = kelas.replace(/^kelas_/, "");
	return `${grade}-${mataPelajaran}.xlsx`;
};

export const buildEssayFileName = (kelas: string, mataPelajaran: string) => {
	const grade = kelas.replace(/^kelas_/, "");
	return `${grade}-${mataPelajaran}-esai.xlsx`;
};

const textValue = (value: unknown) => clean(value);

const normalizeAnswer = (value: unknown) => textValue(value).toLowerCase().replace(/[^a-z]/g, "");

// Kolom asli di template yang benar-benar dipakai (no, question, image, answer_a..d,
// correct_answer, score) berbeda dari nama kolom Bahasa Indonesia yang tadinya direncanakan —
// mapping di bawah menerjemahkan skema nyata itu ke bentuk internal `BankSoalOnlineRow`.
const parseRows = (sheet: XLSX.WorkSheet, folderPath: string, tipe: "pg" | "esai", rowOffset: number) => {
	const rawRows = XLSX.utils.sheet_to_json<Record<string, unknown>>(sheet, { defval: "" });

	return rawRows.map((raw, index): BankSoalOnlineRow => {
		const soal = textValue(raw.question);
		const pilihan_a = tipe === "pg" ? textValue(raw.answer_a) : "";
		const pilihan_b = tipe === "pg" ? textValue(raw.answer_b) : "";
		const pilihan_c = tipe === "pg" ? textValue(raw.answer_c) : "";
		const pilihan_d = tipe === "pg" ? textValue(raw.answer_d) : "";
		const kunci_jawaban = tipe === "pg" ? textValue(raw.correct_answer) : "";
		const filledChoices = [pilihan_a, pilihan_b, pilihan_c, pilihan_d]
			.map((value, choiceIndex) => ({ key: String.fromCharCode(97 + choiceIndex), value }))
			.filter((choice) => choice.value);
		const invalidReasons: string[] = [];

		if (!soal) invalidReasons.push("teks soal kosong");
		if (tipe === "pg") {
			if (filledChoices.length < 2) invalidReasons.push("kurang dari 2 pilihan terisi");
			if (!kunci_jawaban) {
				invalidReasons.push("kunci jawaban kosong");
			} else if (!filledChoices.some((choice) => choice.key === normalizeAnswer(kunci_jawaban))) {
				invalidReasons.push("kunci jawaban tidak cocok dengan pilihan yang terisi");
			}
		}

		const nama_file_gambar = textValue(raw.image);
		const pathRelatifGambar = nama_file_gambar ? `${folderPath}/${nama_file_gambar}` : undefined;
		const rowNumber = index + 2;
		return {
			id: `${tipe}-${rowNumber}`,
			rowNumber: rowOffset + rowNumber,
			tipe,
			soal,
			jenis: "",
			pilihan_a,
			pilihan_b,
			pilihan_c,
			pilihan_d,
			kunci_jawaban,
			skor: raw.score === "" || raw.score === undefined ? 1 : (raw.score as number | string),
			nama_file_gambar,
			...(pathRelatifGambar
				? {
					pathRelatifGambar,
					urlGambar: `${CDN_BASE}/${pathRelatifGambar.split("/").map(encodeURIComponent).join("/")}`
				}
				: {}),
			valid: invalidReasons.length === 0,
			...(invalidReasons.length ? { alasan_invalid: invalidReasons.join("; ") } : {})
		};
	});
};

const fetchWorkbookSheet = async (folderPath: string, candidates: string[]) => {
	for (const candidate of candidates) {
		const response = await tauriFetch(`${CDN_BASE}/${folderPath.split("/").map(encodeURIComponent).join("/")}/${encodeURIComponent(candidate)}`);
		if (!response.ok) continue;
		const bytes = await response.arrayBuffer();
		const workbook = XLSX.read(bytes, { type: "array" });
		const firstSheetName = workbook.SheetNames[0];
		if (!firstSheetName) continue;
		return { sheet: workbook.Sheets[firstSheetName]!, fileName: candidate, fileUrl: response.url };
	}
	return undefined;
};

export const fetchBankSoal = async (kelas: string, mataPelajaran: string): Promise<BankSoalOnlineResult> => {
	if (!ONLINE_CLASSES.includes(kelas as (typeof ONLINE_CLASSES)[number])) {
		throw new Error("Kelas online hanya tersedia untuk kelas 4, 5, dan 6.");
	}
	if (!mataPelajaran) throw new Error("Kode mata pelajaran belum dipilih.");

	const folderPath = buildFolderPath(kelas, mataPelajaran);

	const pg = await fetchWorkbookSheet(folderPath, [buildQuestionFileName(kelas, mataPelajaran), "soal.xlsx"]);
	if (!pg) {
		throw new Error(`File ${buildQuestionFileName(kelas, mataPelajaran)} tidak ditemukan untuk folder ${folderPath}. Pastikan template sudah diunggah ke repo niluji.`);
	}

	const pgRows = parseRows(pg.sheet, folderPath, "pg", 0);

	// Soal esai ada di file terpisah dan sifatnya opsional — tidak semua mapel punya esai.
	const esai = await fetchWorkbookSheet(folderPath, [buildEssayFileName(kelas, mataPelajaran)]);
	const esaiRows = esai ? parseRows(esai.sheet, folderPath, "esai", pgRows.length) : [];

	return {
		rows: [...pgRows, ...esaiRows],
		fileName: pg.fileName,
		fileUrl: pg.fileUrl,
		essayFileName: esai?.fileName,
		folderPath
	};
};

/** Memastikan CDN yang dipakai bank soal benar-benar dapat diakses dari aplikasi offline. */
export const checkBankSoalConnection = async (): Promise<BankSoalConnectionStatus> => {
	try {
		const response = await tauriFetch(CDN_HEALTHCHECK_URL, { cache: "no-store" });
		if (!response.ok) throw new Error(`Server merespons status ${response.status}`);
		return {
			connected: true,
			checkedAt: new Date(),
			message: "Internet terhubung dan server bank soal dapat dijangkau."
		};
	} catch {
		return {
			connected: false,
			checkedAt: new Date(),
			message: "Tidak ada koneksi internet atau server bank soal tidak dapat dijangkau."
		};
	}
};

export const useBankSoalOnline = () => ({
	cdnBase: CDN_BASE,
	classes: ONLINE_CLASS_ITEMS,
	buildFolderPath,
	buildQuestionFileName,
	checkBankSoalConnection,
	fetchBankSoal,
	subjectFolderCode
});
