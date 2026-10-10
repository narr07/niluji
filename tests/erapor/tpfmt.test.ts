import { expect, test } from "bun:test";
import * as XLSX from "xlsx";
Object.assign(globalThis, await import("../../app/utils/kurikulum.ts"));
const { buildTpWorkbook, readTpWorkbook } = await import("../../app/utils/tpExcel.ts");
const mapel = [
	{ kode: "300110000", nama: "Bahasa Indonesia", singkat: "B. Indonesia" },
	{ kode: "401000000", nama: "Matematika", singkat: "Matematika" }
];

test("file asli e-Rapor (f_tp_Bahasa Indonesia.xlsx)", () => {
	const ws = XLSX.utils.aoa_to_sheet([
		["FORMAT IMPORT TP  BAHASA INDONESIA , JENJANG SD"], [],
		["NO", "TINGKAT", "FASE", "SEMESTER", "TUJUAN PEMBELAJARAN (Maximal 100 Karakter)"],
		[1, 1, "A", 1, "Mengenal huruf vokal dan konsonan melalui lagu atau permainan."],
		[2, 2, "A", 1, "Membaca kalimat sederhana dengan lafal dan intonasi yang tepat."],
		[3, 9, "A", 1, "tingkat salah"],
		[4, 3, "B", 1, ""]
	]);
	const wb = XLSX.utils.book_new(); XLSX.utils.book_append_sheet(wb, ws, "Worksheet");
	const buf = XLSX.write(wb, { type: "array", bookType: "xlsx" });
	const r = readTpWorkbook(buf, mapel, "401000000");
	console.log(r);
	expect(r.rows.length).toBe(2);
	expect(r.rows[0].kode_mapel).toBe("300110000"); // dari judul, bukan fallback
	expect(r.issues.length).toBe(1);
});

test("template sendiri bolak-balik", () => {
	const bytes = buildTpWorkbook([
		{ kode: "401000000", nama: "Matematika", singkat: "Matematika", rows: [{ tingkat: 4, semester: 1, deskripsi: "menjumlahkan pecahan" }, { tingkat: 4, semester: 1, deskripsi: "" }] },
		{ kode: "300110000", nama: "Bahasa Indonesia", singkat: "B. Indonesia", rows: [{ tingkat: 5, semester: 2, deskripsi: "menulis puisi" }] }
	]);
	const r = readTpWorkbook(bytes.buffer, mapel);
	expect(r.rows).toEqual([
		{ no: 1, kode_mapel: "401000000", tingkat: 4, semester: 1, deskripsi: "menjumlahkan pecahan" },
		{ no: 1, kode_mapel: "300110000", tingkat: 5, semester: 2, deskripsi: "menulis puisi" }
	]);
	const wb = XLSX.read(bytes, { type: "array" });
	console.log(wb.SheetNames, XLSX.utils.sheet_to_json(wb.Sheets.Matematika!, { header: 1 }).slice(0, 4));
});
