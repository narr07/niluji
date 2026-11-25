import type { LoadedImages, NaskahImportResult } from "./useNaskahImport";
import { isSupportedDocxImage, readDocxNaskah, readDocxTextLines } from "./docxNaskahReader";
import { parseKunciLines } from "./naskahSoalParser";
import { assembleNaskahImport } from "./useNaskahImport";

// Import naskah soal Word. Dokumen diubah menjadi baris teks seperti tampilannya di Word
// (termasuk nomor dan huruf otomatis), lalu diurai oleh pengurai yang sama dengan import PDF.
// File kunci jawaban opsional; kunci juga bisa berada di akhir naskah atau ditandai di naskah.
export const importSoalFromDocx = async (naskahBytes: Uint8Array, kunciBytes?: Uint8Array): Promise<NaskahImportResult> => {
	const kunciFile = kunciBytes ? parseKunciLines(await readDocxTextLines(kunciBytes)) : undefined;
	const docx = await readDocxNaskah(naskahBytes);

	const loadImages = async (ids: string[]): Promise<LoadedImages> => {
		const result: LoadedImages = { loaded: new Map(), failed: 0, unsupported: 0 };
		for (const id of ids) {
			const path = docx.images.get(id);
			if (!path) continue;
			// Gambar EMF/WMF/TIFF dari Word tidak bisa ditampilkan di halaman ujian.
			if (!isSupportedDocxImage(path)) {
				result.unsupported++;
				continue;
			}
			const bytes = await docx.zip.file(path)?.async("uint8array");
			if (bytes) result.loaded.set(id, { bytes, fileName: path.split("/").pop()! });
			else result.failed++;
		}
		return result;
	};

	return assembleNaskahImport({ format: "docx", lines: docx.lines, kunciFile, loadImages });
};
