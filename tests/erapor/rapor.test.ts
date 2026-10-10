import { Database } from "bun:sqlite";
import { expect, mock, test } from "bun:test";
import { readFileSync } from "node:fs";
import { join } from "node:path";

const dbRsPath = join(import.meta.dir, "..", "..", "src-tauri", "src", "erapor", "db.rs");
const rs = readFileSync(dbRsPath, "utf8");
const schema = rs.match(/execute_batch\(\s*"([\s\S]*?)",\s*\)/)![1];
const sqlite = new Database(process.env.ERAPOR_DB_COPY || ":memory:");
if (process.env.ERAPOR_DB_COPY) {
	sqlite.exec(schema);
}

mock.module("@tauri-apps/api/core", () => ({
	invoke: async (cmd: string, args: any) => {
		if (cmd === "db_batch") { sqlite.transaction((st: any[]) => st.forEach(s => sqlite.prepare(s.sql).run(...s.params)))(args.statements); return 0; }
		if (cmd === "db_query") return sqlite.prepare(args.sql).all(...args.params);
		if (cmd === "db_execute") return sqlite.prepare(args.sql).run(...args.params).changes;
		throw new Error(`unmocked ${cmd}`);
	},
	isTauri: () => true
}));
const g = globalThis as any;
Object.assign(g, await import("../../app/utils/kurikulum.ts"));
g.useDb = (await import("../../app/composables/useDb.ts")).useDb;
const { usePembelajaranRapor } = await import("../../app/composables/usePembelajaranRapor.ts");
const xl = await import("../../app/utils/nilaiExcel.ts");

test.skipIf(!process.env.ERAPOR_DB_COPY)("susun pembelajaran + nilai + excel", async () => {
	const p = usePembelajaranRapor();
	const r = await p.susun("20261");
	console.log(r);
	const r2 = await p.susun("20261"); // idempoten
	expect(r2.pembelajaran).toBe(r.pembelajaran);
	console.log(sqlite.query(`SELECT r.nama, group_concat(m.singkat || ':' || pr.sumber || ':' || substr(coalesce(g.nama,'-'),1,10), ' | ') x
		FROM pembelajaran_rapor pr JOIN rombel r USING(rombongan_belajar_id) JOIN mapel_rapor m ON m.kode=pr.kode_mapel LEFT JOIN ptk g ON g.ptk_id=pr.ptk_id
		WHERE pr.semester_id='20261' GROUP BY r.nama ORDER BY r.tingkat`).all());

	// TP + nilai untuk Matematika kelas 4
	const pr: any = sqlite.query("SELECT pr.id, pr.rombongan_belajar_id FROM pembelajaran_rapor pr JOIN rombel r USING(rombongan_belajar_id) WHERE r.nama='Kelas 4' AND pr.kode_mapel='401000000'").get();
	sqlite.run("INSERT INTO tujuan_pembelajaran (kode_mapel,tingkat,semester,deskripsi,urutan) VALUES ('401000000',4,1,'menjumlahkan pecahan',1),('401000000',4,1,'mengukur luas bangun datar',2)");
	const tps: any[] = sqlite.query("SELECT id, deskripsi FROM tujuan_pembelajaran WHERE kode_mapel='401000000' AND tingkat=4 ORDER BY urutan").all();
	const siswa: any[] = sqlite.query("SELECT p.peserta_didik_id, p.nisn, p.nama FROM anggota_rombel a JOIN peserta_didik p USING(peserta_didik_id) WHERE a.rombongan_belajar_id=? ORDER BY p.nama").all(pr.rombongan_belajar_id);
	const sheet = { prId: pr.id, singkat: "Matematika", mapel: "Matematika", tps, siswa: siswa.map((s, i) => ({ pesertaDidikId: s.peserta_didik_id, nisn: s.nisn, nama: s.nama, nilai: 70 + i, optimal: [tps[0].id], perlu: [tps[1].id] })) };
	const bytes = xl.buildWorkbook("Kelas 4", [sheet]);
	const back = xl.readWorkbook(bytes.buffer as ArrayBuffer, [{ ...sheet, siswa: sheet.siswa.map(s => ({ ...s, nilai: null, optimal: [], perlu: [] })) }]);
	expect(back[0]!.siswa.length).toBe(siswa.length);
	expect(back[0]!.siswa[3]!.nilai).toBe(73);
	expect(back[0]!.siswa[0]!.optimal).toEqual([tps[0].id]);
	console.log("siswa kelas 4:", siswa.length, "contoh deskripsi:", deskripsiCapaian(["menjumlahkan pecahan"], ["mengukur luas bangun datar"]));
	console.log("json_each:", sqlite.query("SELECT COUNT(*) c FROM (SELECT 1 FROM json_each('[1,11]') WHERE value = 1)").get());
}, 60000);
