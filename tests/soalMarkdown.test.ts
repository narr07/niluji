import { describe, expect, it, mock } from "bun:test";

mock.module("@tauri-apps/plugin-http", () => ({ fetch: async () => new Response("") }));

const { parseSoalMarkdown } = await import("../app/composables/useBankSoalOnline");
const { renderSoalMarkdown } = await import("../app/composables/useSoalMarkdown");

const soal = (body: string) => `---\nkelas: 6\npelajaran: Uji\njenis: STS\n---\n\n## 1. Pilihan Ganda\n\n${body}\n`;

describe("parseSoalMarkdown", () => {
	it("mempertahankan opsi E beserta gambarnya", () => {
		const text = soal("Pilih satu\n\n- A. satu\n- B. dua\n- C. tiga\n- D. empat\n- E. lima ![opsi-E](gambar/soal-1-opsi-e.png)\n\nKunci: E\nSkor: 2");
		const { rows } = parseSoalMarkdown(text, "kelas_6/uji");
		expect(rows[0]!.pilihan_e).toBe("lima");
		expect(rows[0]!.pilihan_e_gambar).toBe("soal-1-opsi-e.png");
		expect(rows[0]!.valid).toBe(true);
		expect(rows[0]!.skor).toBe(2);
	});

	it("menandai kunci yang tidak cocok dengan pilihan", () => {
		const { rows } = parseSoalMarkdown(soal("Pilih\n\n- A. satu\n- B. dua\n\nKunci: E\nSkor: 1"), "");
		expect(rows[0]!.valid).toBe(false);
		expect(rows[0]!.alasan_invalid).toContain("kunci");
	});
});

describe("renderSoalMarkdown", () => {
	it("tidak merender HTML dari teks soal", () => {
		const html = renderSoalMarkdown("<img src=x onerror=alert(1)> **tebal**");
		expect(html).not.toContain("<img");
		expect(html).toContain("&lt;img");
		expect(html).toContain("<strong>tebal</strong>");
	});
});
