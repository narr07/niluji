// Membuat file contoh untuk fitur Import dari Naskah Soal Word:
//   public/templates/naskah-soal-word/contoh-naskah-soal.docx
//   public/templates/naskah-soal-word/contoh-kunci-jawaban.docx
// Isinya fiktif (bukan dokumen sekolah asli) dan memperlihatkan format yang dikenali importer.
// Jalankan ulang dengan: bun run templates:word

import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import JSZip from "jszip";

const OUT_DIR = join(import.meta.dirname, "..", "public", "templates", "naskah-soal-word");

const escapeXml = (s: string) => s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");

interface RunOptions { bold?: boolean }

const run = (text: string, { bold }: RunOptions = {}) =>
	`<w:r><w:rPr><w:rFonts w:ascii="Arial" w:hAnsi="Arial" w:cs="Arial"/>${bold ? "<w:b/>" : ""}<w:sz w:val="22"/></w:rPr><w:t xml:space="preserve">${escapeXml(text)}</w:t></w:r>`;
const tab = "<w:r><w:tab/></w:r>";

interface ParaOptions extends RunOptions { center?: boolean, indent?: boolean, tabs?: number[] }

// Teks dengan "\t" dipecah menjadi beberapa run yang dipisah tab, seperti opsi berjajar di naskah cetak.
const para = (text: string, options: ParaOptions = {}) => {
	const tabStops = options.tabs?.length ? `<w:tabs>${options.tabs.map((pos) => `<w:tab w:val="left" w:pos="${pos}"/>`).join("")}</w:tabs>` : "";
	const pPr = `<w:pPr>${tabStops}<w:spacing w:after="60"/>${options.center ? "<w:jc w:val=\"center\"/>" : ""}${options.indent ? "<w:ind w:left=\"426\"/>" : ""}</w:pPr>`;
	return `<w:p>${pPr}${text.split("\t").map((part) => run(part, options)).join(tab)}</w:p>`;
};

const cell = (text: string, bold = false) =>
	`<w:tc><w:tcPr><w:tcW w:w="1400" w:type="dxa"/></w:tcPr>${para(text, { bold, center: true })}</w:tc>`;
const table = (rows: string[][], headerRows = 1) => {
	const border = (side: string) => `<w:${side} w:val="single" w:sz="4" w:space="0" w:color="000000"/>`;
	const borders = `<w:tblBorders>${["top", "left", "bottom", "right", "insideH", "insideV"].map(border).join("")}</w:tblBorders>`;
	const body = rows.map((row, i) => `<w:tr>${row.map((text) => cell(text, i < headerRows)).join("")}</w:tr>`).join("");
	return `<w:tbl><w:tblPr><w:tblW w:w="0" w:type="auto"/>${borders}</w:tblPr>${body}</w:tbl>`;
};

const buildDocx = async (body: string) => {
	const zip = new JSZip();
	zip.file(
		"[Content_Types].xml",
		"<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>"
		+ "<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">"
		+ "<Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>"
		+ "<Default Extension=\"xml\" ContentType=\"application/xml\"/>"
		+ "<Override PartName=\"/word/document.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml\"/>"
		+ "</Types>"
	);
	zip.file(
		"_rels/.rels",
		"<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>"
		+ "<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">"
		+ "<Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" Target=\"word/document.xml\"/>"
		+ "</Relationships>"
	);
	zip.file(
		"word/document.xml",
		"<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>"
		+ "<w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"><w:body>"
		+ body
		+ "<w:sectPr><w:pgSz w:w=\"11906\" w:h=\"16838\"/><w:pgMar w:top=\"1134\" w:right=\"1134\" w:bottom=\"1134\" w:left=\"1134\" w:header=\"709\" w:footer=\"709\" w:gutter=\"0\"/></w:sectPr>"
		+ "</w:body></w:document>"
	);
	// Tanggal tetap supaya file hasil generate tidak berubah setiap kali skrip dijalankan.
	for (const file of Object.values(zip.files)) file.date = new Date("2026-01-01T00:00:00Z");
	return zip.generateAsync({ type: "uint8array", compression: "DEFLATE" });
};

const OPTION_TABS = [2600, 5200, 7800];
const GRID_TABS = [4600];

// Tiga gaya penulisan opsi yang dikenali importer: satu per baris, berjajar satu baris, dan 2x2.
type Options = { style: "lines" | "row" | "grid", items: [string, string, string, string] };
interface PgSoal { text: string, options: Options, kunci: "A" | "B" | "C" | "D" }

const PG: PgSoal[] = [
	{ text: "Hasil dari 12.352 + 4.673 adalah ....", options: { style: "lines", items: ["16.925", "17.025", "17.235", "18.355"] }, kunci: "B" },
	{ text: "Hasil dari 250 : 5 = ....", options: { style: "row", items: ["40", "45", "50", "55"] }, kunci: "C" },
	{ text: "Bilangan 5.734 dibaca ....", options: { style: "grid", items: ["lima tujuh tiga empat", "lima ribu tujuh ratus tiga puluh empat", "lima ribu tujuh puluh tiga", "lima ratus tujuh puluh tiga"] }, kunci: "B" },
	{ text: "Faktor dari 12 adalah ....", options: { style: "lines", items: ["1, 2, 3, 4, 6, 12", "1, 2, 3, 4, 5, 12", "2, 4, 6, 8", "1, 3, 6, 9"] }, kunci: "A" },
	{ text: "Kelipatan 4 yang kurang dari 20 adalah ....", options: { style: "row", items: ["4, 8, 12, 16", "4, 8, 10, 16", "8, 12, 16, 20", "4, 6, 8, 10"] }, kunci: "A" },
	{ text: "Ibu membagi 24 kue sama banyak kepada 6 anak. Setiap anak mendapat ... kue.", options: { style: "row", items: ["3", "4", "5", "6"] }, kunci: "B" },
	{ text: "Pecahan 1/2 senilai dengan ....", options: { style: "lines", items: ["2/3", "2/4", "3/4", "1/3"] }, kunci: "B" },
	{ text: "Nilai tempat angka 8 pada bilangan 18.465 adalah ....", options: { style: "grid", items: ["satuan", "puluhan", "ratusan", "ribuan"] }, kunci: "D" },
	{ text: "Hasil dari 15 x 4 = ....", options: { style: "row", items: ["45", "50", "55", "60"] }, kunci: "D" },
	{ text: "Bilangan terkecil dari 3.845, 3.458, 3.584, dan 3.854 adalah ....", options: { style: "lines", items: ["3.845", "3.458", "3.584", "3.854"] }, kunci: "B" }
];

const ESAI = [
	{ text: "Sebutkan tiga bilangan kelipatan 5 yang kurang dari 20!", jawaban: "5, 10, dan 15" },
	{ text: "Ani membeli 3 kotak pensil. Setiap kotak berisi 12 pensil. Berapa jumlah pensil Ani?", jawaban: "3 x 12 = 36 pensil" },
	{ text: "Tuliskan bilangan 9.021 dengan kata-kata!", jawaban: "Sembilan ribu dua puluh satu" }
];

const optionParas = ({ style, items: [a, b, c, d] }: Options) => {
	if (style === "lines") return [`a. ${a}`, `b. ${b}`, `c. ${c}`, `d. ${d}`].map((t) => para(t, { indent: true })).join("");
	if (style === "row") return para(`a. ${a} \tb. ${b} \tc. ${c} \td. ${d}`, { indent: true, tabs: OPTION_TABS });
	return para(`a. ${a} \tc. ${c}`, { indent: true, tabs: GRID_TABS }) + para(`b. ${b} \td. ${d}`, { indent: true, tabs: GRID_TABS });
};

const naskahBody = [
	para("SUMATIF TENGAH SEMESTER (CONTOH)", { bold: true, center: true }),
	para("SD CONTOH NILUJI", { bold: true, center: true }),
	para("Mata Pelajaran : Matematika\tNama : ....................", { tabs: [5600] }),
	para("Kelas : IV (Empat)\tNomor : ....................", { tabs: [5600] }),
	para("I. Berilah tanda silang (X) pada huruf a, b, c, atau d pada jawaban yang paling benar!", { bold: true }),
	...PG.map((soal, i) => para(`${i + 1}. ${soal.text}`) + optionParas(soal.options)),
	para("II. Jawablah pertanyaan di bawah ini dengan benar!", { bold: true }),
	...ESAI.map((soal, i) => para(`${PG.length + i + 1}. ${soal.text}`))
].join("");

const half = Math.ceil(PG.length / 2);
const kunciRows = Array.from({ length: half }, (_, i) => {
	const right = PG[i + half];
	return [String(i + 1), PG[i]!.kunci, right ? String(i + half + 1) : "", right?.kunci ?? ""];
});

const kunciBody = [
	para("KUNCI JAWABAN", { bold: true, center: true }),
	para("SUMATIF TENGAH SEMESTER (CONTOH)", { bold: true, center: true }),
	para("Mata Pelajaran : Matematika"),
	para("Kelas : IV (Empat)"),
	para(`Jumlah Soal: ${PG.length} Soal Pilihan Ganda, ${ESAI.length} Soal Uraian`),
	para("I. Pilihan Ganda", { bold: true }),
	table([["No", "Jawaban", "No", "Jawaban"], ...kunciRows]),
	para("II. Uraian", { bold: true }),
	...ESAI.map((soal, i) => para(`${PG.length + i + 1}. ${soal.jawaban}`))
].join("");

mkdirSync(OUT_DIR, { recursive: true });
writeFileSync(join(OUT_DIR, "contoh-naskah-soal.docx"), await buildDocx(naskahBody));
writeFileSync(join(OUT_DIR, "contoh-kunci-jawaban.docx"), await buildDocx(kunciBody));
console.log(`Template Word dibuat di ${OUT_DIR}`);
