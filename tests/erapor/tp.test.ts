import { Database } from "bun:sqlite";
import { expect, test } from "bun:test";
const { buildTpWorkbook, readTpWorkbook } = await import("../../app/utils/tpExcel.ts");

test.skipIf(!process.env.ERAPOR_DB_COPY)("tp excel import dedupe", () => {
	const db = new Database(process.env.ERAPOR_DB_COPY!);
	const mapel = db.query("SELECT kode, nama, singkat FROM mapel_rapor ORDER BY urutan").all() as any[];
	const rows = [
		{ kode_mapel: "", mapel: "Matematika", tingkat: 5, semester: 1, deskripsi: "Peserta didik dapat membandingkan pecahan." },
		{ kode_mapel: "300110000", mapel: "", tingkat: 5, semester: 1, deskripsi: "menyimak teks narasi" },
		{ kode_mapel: "", mapel: "IPAS", tingkat: 5, semester: 2, deskripsi: "mengidentifikasi organ pencernaan" },
		{ kode_mapel: "", mapel: "Tidak Ada", tingkat: 5, semester: 1, deskripsi: "x" },
		{ kode_mapel: "401000000", mapel: "", tingkat: 9, semester: 1, deskripsi: "x" }
	];
	const bytes = buildTpWorkbook(rows as any, mapel);
	const { rows: got, skipped } = readTpWorkbook(bytes.buffer as ArrayBuffer, mapel);
	console.log(got, skipped);
	expect(got.length).toBe(3);
	expect(skipped).toBe(2);
	const ins = db.prepare(`INSERT INTO tujuan_pembelajaran (kode_mapel, tingkat, semester, deskripsi, urutan)
		SELECT ?, ?, ?, ?, COALESCE((SELECT MAX(urutan) FROM tujuan_pembelajaran WHERE kode_mapel = ? AND tingkat = ? AND semester = ?), 0) + 1
		WHERE NOT EXISTS (SELECT 1 FROM tujuan_pembelajaran WHERE kode_mapel = ? AND tingkat = ? AND semester = ? AND deskripsi = ?)`);
	const run = () => got.forEach(r => ins.run(r.kode_mapel, r.tingkat, r.semester, r.deskripsi, r.kode_mapel, r.tingkat, r.semester, r.kode_mapel, r.tingkat, r.semester, r.deskripsi));
	const before = (db.query("SELECT COUNT(*) c FROM tujuan_pembelajaran").get() as any).c;
	run(); run();
	const after = (db.query("SELECT COUNT(*) c FROM tujuan_pembelajaran").get() as any).c;
	expect(after - before).toBe(3);
});
