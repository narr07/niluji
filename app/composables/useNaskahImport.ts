import type { KunciMaps, ReadingLine } from "./naskahSoalParser";
import { buildSoalRows, parseKunciLines, parseNaskahLines } from "./naskahSoalParser";

// Baris hasil import naskah (Word maupun PDF) yang ditampilkan di tabel pratinjau.
export interface SoalImportRow {
	id: string
	number: number
	tipe: "pg" | "esai"
	soal: string
	pilihan_a: string
	pilihan_b: string
	pilihan_c: string
	pilihan_d: string
	pilihan_e: string
	kunci_jawaban: string
	catatanJawabanEsai?: string
	imageFileName?: string
	imageBytes?: Uint8Array
	valid: boolean
	alasanInvalid?: string
}

export interface NaskahImportSummary {
	format: "docx" | "pdf"
	pages?: number
	twoColumnPages: number
	pgCount: number
	essayCount: number
	expectedPg?: number
	expectedEssay?: number
	kunciFromMarks: number
	kunciFromFile: number
	kunciFromNaskah: number
	pgWithoutKunci: number
	kunciFileEmpty: boolean
	images: number
	imagesFailed: number
	imagesUnsupported: number
	missingNumbers: number[]
	// Salah ketik di naskah yang dibetulkan otomatis, mis. "Soal 3: nomor tertulis 1, ...".
	autoFixes: string[]
}

export interface NaskahImportResult {
	rows: SoalImportRow[]
	summary: NaskahImportSummary
}

export interface LoadedImages {
	loaded: Map<string, { bytes: Uint8Array, fileName: string }>
	failed: number
	unsupported: number
}

interface AssembleInput {
	format: NaskahImportSummary["format"]
	lines: ReadingLine[]
	kunciFile?: KunciMaps
	pages?: number
	twoColumnPages?: number
	loadImages: (ids: string[]) => Promise<LoadedImages>
}

export const assembleNaskahImport = async (input: AssembleInput): Promise<NaskahImportResult> => {
	const naskah = parseNaskahLines(input.lines);
	if (!naskah.blocks.length) {
		throw new Error("Tidak ada nomor soal yang terbaca. Pastikan setiap soal diawali nomor seperti \"1.\" atau \"1)\".");
	}
	const kunciNaskah = naskah.kunciLines.length ? parseKunciLines(naskah.kunciLines) : undefined;
	const drafts = buildSoalRows({ blocks: naskah.blocks, kunciFile: input.kunciFile, kunciNaskah });

	const imageIds = [...new Set(drafts.flatMap((row) => (row.imageId ? [row.imageId] : [])))];
	const images = await input.loadImages(imageIds);

	const rows: SoalImportRow[] = drafts.map((row) => {
		const image = row.imageId ? images.loaded.get(row.imageId) : undefined;
		return {
			id: row.id,
			number: row.number,
			tipe: row.tipe,
			soal: row.soal,
			pilihan_a: row.options[0],
			pilihan_b: row.options[1],
			pilihan_c: row.options[2],
			pilihan_d: row.options[3],
			pilihan_e: row.options[4],
			kunci_jawaban: row.kunci,
			catatanJawabanEsai: row.catatanJawabanEsai,
			imageFileName: image?.fileName,
			imageBytes: image?.bytes,
			valid: row.reasons.length === 0,
			alasanInvalid: row.reasons.length ? row.reasons.join("; ") : undefined
		};
	});

	const pgRows = drafts.filter((r) => r.tipe === "pg");
	const { kunciFile } = input;
	return {
		rows,
		summary: {
			format: input.format,
			pages: input.pages,
			twoColumnPages: input.twoColumnPages ?? 0,
			pgCount: pgRows.length,
			essayCount: drafts.length - pgRows.length,
			expectedPg: kunciFile?.expectedPg ?? kunciNaskah?.expectedPg,
			expectedEssay: kunciFile?.expectedEssay ?? kunciNaskah?.expectedEssay,
			kunciFromMarks: pgRows.filter((r) => r.kunciSource === "tanda").length,
			kunciFromFile: pgRows.filter((r) => r.kunciSource === "file").length,
			kunciFromNaskah: pgRows.filter((r) => r.kunciSource === "naskah").length,
			pgWithoutKunci: pgRows.filter((r) => !r.kunci).length,
			kunciFileEmpty: Boolean(kunciFile && !kunciFile.pg.size && !kunciFile.esai.size),
			images: images.loaded.size,
			imagesFailed: images.failed,
			imagesUnsupported: images.unsupported,
			missingNumbers: naskah.missingNumbers,
			autoFixes: drafts.flatMap((row) => row.fixes.map((fix) => `Soal ${row.number}: ${fix}`))
		}
	};
};
