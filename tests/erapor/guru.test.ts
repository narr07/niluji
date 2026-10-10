import { Database } from "bun:sqlite";
import { expect, mock, test } from "bun:test";
import { readFileSync, writeFileSync } from "node:fs";

const T = process.env.ERAPOR_TEST_DIR;
const sqlite = new Database(T ? `${T}/guru/erapor.sqlite` : ":memory:");

mock.module("@tauri-apps/api/core", () => ({
	invoke: async (cmd: string, args: any) => {
		if (cmd === "db_batch") { sqlite.transaction((st: any[]) => st.forEach(s => sqlite.prepare(s.sql).run(...s.params)))(args.statements); return 0; }
		if (cmd === "db_query") return sqlite.prepare(args.sql).all(...args.params);
		if (cmd === "db_execute") return sqlite.prepare(args.sql).run(...args.params).changes;
		if (cmd === "guru_set_local_user") return null;
		if (cmd === "http_post_json") {
			if (args.url.endsWith("/api/tarik")) return JSON.parse(readFileSync(`${T}/paket.json`, "utf8"));
			if (args.url.endsWith("/api/kirim")) { writeFileSync(`${T}/kirim.json`, JSON.stringify(args.body)); return { pembelajaran: args.body.pembelajaran_ids.length, nilai: args.body.nilai.length, waktu: "2026-10-06 20:00:00" }; }
		}
		throw new Error(`unmocked ${cmd}`);
	},
	isTauri: () => true
}));
const g = globalThis as any;
Object.assign(g, await import("../../app/utils/kurikulum.ts"));
g.useDb = (await import("../../app/composables/useDb.ts")).useDb;
g.useAppMode = () => ({ mode: { value: "admin" } });
const { useGuruSync } = await import("../../app/composables/useGuruSync.ts");

test.skipIf(!process.env.ERAPOR_TEST_DIR)("guru: tarik → isi → kirim", async () => {
	const s = useGuruSync();
	const r = await s.tarik({ url: "abc.trycloudflare.com/", pin: "123456", username: "maspupah", password: "x" });
	const ptk = r.guru.ptkId, sem = r.semesterId;
	console.log("server_url:", sqlite.query("SELECT value FROM settings WHERE key='server_url'").get());

	let k = await s.kesiapan(ptk, sem);
	console.log(k.map(x => ({ kelas: x.rombel, siap: x.siap, mapel: x.mapel.map(m => `${m.singkat}:${m.lengkap}/${m.siswa} tp=${m.tp}`) })));
	expect(k[0]!.siap).toBe(false);

	// Guru mengisi: mapel tanpa TP admin → guru bikin TP sendiri (sumber guru).
	const prs: any[] = sqlite.query("SELECT pr.id, pr.kode_mapel, CAST(r.tingkat AS INTEGER) t, pr.rombongan_belajar_id rb FROM pembelajaran_rapor pr JOIN rombel r USING(rombongan_belajar_id) WHERE pr.ptk_id=?").all(ptk);
	for (const p of prs) {
		let tp: any = sqlite.query("SELECT id FROM tujuan_pembelajaran WHERE kode_mapel=? AND tingkat=? AND semester=1 LIMIT 1").get(p.kode_mapel, p.t);
		if (!tp) {
			sqlite.prepare("INSERT INTO tujuan_pembelajaran (kode_mapel,tingkat,semester,deskripsi,urutan,sumber) VALUES (?,?,1,?,1,'guru')").run(p.kode_mapel, p.t, `tp guru ${p.kode_mapel}`);
			tp = sqlite.query("SELECT id FROM tujuan_pembelajaran WHERE kode_mapel=? AND tingkat=? LIMIT 1").get(p.kode_mapel, p.t);
		}
		const siswa: any[] = sqlite.query("SELECT peserta_didik_id FROM anggota_rombel WHERE rombongan_belajar_id=?").all(p.rb);
		for (const sw of siswa)
			sqlite.prepare("INSERT OR REPLACE INTO nilai_rapor (pembelajaran_rapor_id,peserta_didik_id,nilai,tp_optimal,tp_perlu) VALUES (?,?,85,?,'[]')").run(p.id, sw.peserta_didik_id, JSON.stringify([tp.id]));
	}
	k = await s.kesiapan(ptk, sem);
	expect(k[0]!.siap).toBe(true);

	const res = await s.kirim({ pin: "123456", password: "x", rombelId: k[0]!.rombongan_belajar_id, ptkId: ptk, semesterId: sem });
	console.log("kirim:", res);
	const body = JSON.parse(readFileSync(`${T}/kirim.json`, "utf8"));
	console.log("tp terkirim:", body.tp.length, "nilai:", body.nilai.length, "contoh tp_optimal:", body.nilai[0].tp_optimal);
	expect(body.nilai[0].tp_optimal[0]).toMatch(/^[0-9a-f]{32}$/);
	k = await s.kesiapan(ptk, sem);
	expect(k[0]!.terkirim).toBe(true);
});
