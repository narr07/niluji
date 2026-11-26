import { describe, expect, it } from "bun:test";
import { buildKartuPesertaHtml, KARTU_PER_HALAMAN } from "../app/utils/kartuPeserta";

const siswa = (n: number) => ({ name: `Siswa ${n}`, nisn: String(1000 + n), class: "4", rombel: "IVA", pin: "123456" });

describe("lembar cetak kartu peserta", () => {
	it("membagi kartu menjadi 10 per halaman A4", () => {
		const html = buildKartuPesertaHtml(Array.from({ length: 23 }, (_, i) => siswa(i + 1)), "SD Contoh");
		expect(KARTU_PER_HALAMAN).toBe(10);
		expect(html.match(/<section class="page">/g)).toHaveLength(3);
		expect(html.match(/class="kartu"/g)).toHaveLength(23);
		expect(html).toContain("size: A4 portrait");
	});

	it("tidak memuat QR atau alamat server, dan menampilkan kelas beserta rombel", () => {
		const html = buildKartuPesertaHtml([siswa(1)], "SD Contoh");
		expect(html).not.toMatch(/http|<img|qr/i);
		expect(html).toContain("4 · IVA");
		expect(html).toContain("123456");
	});

	it("membatasi setiap kartu dengan garis potong tipis", () => {
		const html = buildKartuPesertaHtml([siswa(1)], "SD Contoh");
		expect(html).toMatch(/\.kartu \{ border: 0\.3mm solid/);
	});

	it("meloloskan karakter HTML pada nama", () => {
		const html = buildKartuPesertaHtml([{ ...siswa(1), name: "<b>Andi</b> & \"Budi\"" }], "SD <Contoh>");
		expect(html).toContain("&lt;b&gt;Andi&lt;/b&gt; &amp; &quot;Budi&quot;");
		expect(html).toContain("SD &lt;Contoh&gt;");
		expect(html).not.toContain("<b>Andi</b>");
	});
});
