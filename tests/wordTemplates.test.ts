import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "bun:test";
import { importSoalFromDocx } from "../app/composables/useDocxSoalImport";

// File contoh yang diunduh guru dari modal Import Word harus ada di repo dan bisa diimpor utuh.
// Kalau gagal karena filenya hilang, jalankan: bun run templates:word
const DIR = join(import.meta.dirname, "..", "public", "templates", "naskah-soal-word");
const read = (name: string) => new Uint8Array(readFileSync(join(DIR, name)));

describe("template Import Word", () => {
	it("contoh naskah dan kunci terbaca lengkap tanpa baris yang perlu dicek manual", async () => {
		const { rows, summary } = await importSoalFromDocx(read("contoh-naskah-soal.docx"), read("contoh-kunci-jawaban.docx"));
		expect([summary.pgCount, summary.essayCount]).toEqual([10, 3]);
		expect([summary.expectedPg, summary.expectedEssay]).toEqual([10, 3]);
		expect(summary.kunciFromFile).toBe(10);
		expect(rows.filter((r) => !r.valid).map((r) => `${r.number}: ${r.alasanInvalid}`)).toEqual([]);
		expect(rows[2]!.pilihan_b).toBe("lima ribu tujuh ratus tiga puluh empat");
		expect(rows[9]!.pilihan_d).toBe("3.854");
		expect(rows[11]!.catatanJawabanEsai).toBe("3 x 12 = 36 pensil");
		expect(rows[12]!.soal).toBe("Tuliskan bilangan 9.021 dengan kata-kata!");
	});
});
