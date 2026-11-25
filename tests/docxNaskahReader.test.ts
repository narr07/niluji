import { describe, expect, it } from "bun:test";
import JSZip from "jszip";
import { readDocxNaskah } from "../app/composables/docxNaskahReader";
import { importSoalFromDocx } from "../app/composables/useDocxSoalImport";

const W = "xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"";

// Paragraf dengan penomoran otomatis Word: nomornya tidak ada di teks, hanya di numbering.xml.
const p = (text: string, numId?: number, ilvl = 0) =>
	`<w:p><w:pPr>${numId ? `<w:numPr><w:ilvl w:val="${ilvl}"/><w:numId w:val="${numId}"/></w:numPr>` : ""}<w:tabs><w:tab w:val="left" w:pos="720"/></w:tabs></w:pPr>${
		text.split("\t").map((part) => `<w:r><w:t xml:space="preserve">${part}</w:t></w:r>`).join("<w:r><w:tab/></w:r>")
	}</w:p>`;
const cell = (text: string) => `<w:tc>${p(text)}</w:tc>`;
const row = (...cells: string[]) => `<w:tr>${cells.map(cell).join("")}</w:tr>`;

const level = (ilvl: number, format: string, text: string) =>
	`<w:lvl w:ilvl="${ilvl}"><w:start w:val="1"/><w:numFmt w:val="${format}"/><w:lvlText w:val="${text}"/></w:lvl>`;

// abstractNum 1: nomor soal "1."; abstractNum 2: opsi "a."; abstractNum 3: judul bagian "I.".
const NUMBERING = `<w:numbering ${W}>`
	+ `<w:abstractNum w:abstractNumId="1">${level(0, "decimal", "%1.")}</w:abstractNum>`
	+ `<w:abstractNum w:abstractNumId="2">${level(0, "lowerLetter", "%1.")}</w:abstractNum>`
	+ `<w:abstractNum w:abstractNumId="3">${level(0, "upperRoman", "%1.")}</w:abstractNum>`
	+ "<w:num w:numId=\"1\"><w:abstractNumId w:val=\"1\"/></w:num>"
	// Daftar lain yang memakai abstractNum yang sama melanjutkan hitungan, seperti di Word.
	+ "<w:num w:numId=\"2\"><w:abstractNumId w:val=\"1\"/></w:num>"
	+ "<w:num w:numId=\"10\"><w:abstractNumId w:val=\"2\"/><w:lvlOverride w:ilvl=\"0\"><w:startOverride w:val=\"1\"/></w:lvlOverride></w:num>"
	+ "<w:num w:numId=\"11\"><w:abstractNumId w:val=\"2\"/><w:lvlOverride w:ilvl=\"0\"><w:startOverride w:val=\"1\"/></w:lvlOverride></w:num>"
	+ "<w:num w:numId=\"20\"><w:abstractNumId w:val=\"3\"/></w:num>"
	+ "</w:numbering>";

const makeDocx = async (body: string, numbering = NUMBERING) => {
	const zip = new JSZip();
	zip.file("word/document.xml", `<?xml version="1.0"?><w:document ${W}><w:body>${body}</w:body></w:document>`);
	zip.file("word/numbering.xml", numbering);
	return zip.generateAsync({ type: "uint8array" });
};

const texts = async (body: string) => (await readDocxNaskah(await makeDocx(body))).lines.flatMap((l) => (l.kind === "text" ? [l.text] : []));

describe("readDocxNaskah", () => {
	it("menuliskan nomor dan huruf dari penomoran otomatis Word", async () => {
		expect(await texts(
			p("Berilah tanda silang pada jawaban yang benar!", 20)
			+ p("Soal satu", 1) + p("x", 10) + p("y", 10) + p("z", 10)
			+ p("Soal dua", 2) + p("x", 11) + p("y", 11) + p("z", 11)
		)).toEqual([
			"I. Berilah tanda silang pada jawaban yang benar!",
			"1. Soal satu",
			"a. x",
			"b. y",
			"c. z",
			"2. Soal dua",
			"a. x",
			"b. y",
			"c. z"
		]);
	});

	it("membaca tabel opsi berisi sel huruf dan sel jawaban berdampingan", async () => {
		const lines = await texts(`${p("1. Joshua and his friend . . . going to school.")}<w:tbl>${row("A", "is", "B", "are")}${row("C", "was", "D", "were")}</w:tbl>`);
		expect(lines).toEqual(["1. Joshua and his friend . . . going to school.", "A. is B. are", "C. was D. were"]);
	});

	it("mengubah tab menjadi spasi dan melewati isi cadangan kotak teks", async () => {
		const textBox = "<w:r><mc:AlternateContent><mc:Choice><w:txbxContent>"
			+ p("Isi kotak")
			+ "</w:txbxContent></mc:Choice><mc:Fallback><w:txbxContent>"
			+ p("Isi kotak")
			+ "</w:txbxContent></mc:Fallback></mc:AlternateContent></w:r>";
		const lines = await texts(`${p("a. 7.952\tb. 7.529")}<w:p>${textBox}</w:p>`);
		expect(lines).toEqual(["a. 7.952 b. 7.529", "Isi kotak"]);
	});
});

describe("importSoalFromDocx", () => {
	it("mengurai naskah bernomor otomatis lengkap dengan kunci dari file terpisah", async () => {
		const naskah = await makeDocx(
			p("Pilihan Ganda", 20)
			+ p("Ibu kota Indonesia adalah ....", 1) + p("Bandung", 10) + p("Jakarta", 10) + p("Medan", 10)
			+ p("5 + 3 = ....", 1) + p("7", 11) + p("8", 11) + p("9", 11)
			+ p("Uraian", 20)
			+ p("Sebutkan nama hari!", 1)
		);
		const kunci = await makeDocx(`${p("Jumlah Soal: 2 Soal Pilihan Ganda, 1 Soal Uraian")}<w:tbl>${row("No", "Jawaban")}${row("1", "B")}${row("2", "B")}</w:tbl>${p("II. Uraian")}${p("3. Senin sampai Minggu")}`);
		const { rows, summary } = await importSoalFromDocx(naskah, kunci);
		expect(rows.map((r) => [r.number, r.tipe, r.kunci_jawaban, r.valid])).toEqual([[1, "pg", "B", true], [2, "pg", "B", true], [3, "esai", "", true]]);
		expect(rows[1]!.pilihan_b).toBe("8");
		expect(rows[2]!.catatanJawabanEsai).toBe("Senin sampai Minggu");
		expect([summary.expectedPg, summary.expectedEssay]).toEqual([2, 1]);
	});
});
