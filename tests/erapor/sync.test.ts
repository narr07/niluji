import { Database } from "bun:sqlite";
import { expect, mock, test } from "bun:test";
import { readFileSync } from "node:fs";
import { join } from "node:path";

const dbRsPath = join(import.meta.dir, "..", "..", "src-tauri", "src", "erapor", "db.rs");
const rs = readFileSync(dbRsPath, "utf8");
const schema = rs.match(/execute_batch\(\s*"([\s\S]*?)",\s*\)/)![1];
const sqlite = new Database(":memory:");
sqlite.exec(schema);

mock.module("@tauri-apps/api/core", () => ({
	invoke: async (cmd: string, args: any) => {
		if (cmd === "dapodik_request") {
			const r = args.req;
			const u = new URL(`${r.baseUrl}/${r.path}`);
			u.searchParams.set("npsn", r.npsn);
			if (r.semesterId) u.searchParams.set("semester_id", r.semesterId);
			const res = await fetch(u, { headers: { Authorization: `Bearer ${r.token}` } });
			return { status: res.status, body: await res.text() };
		}
		if (cmd === "db_batch") {
			const tx = sqlite.transaction((st: any[]) => st.forEach(s => sqlite.prepare(s.sql).run(...s.params)));
			tx(args.statements);
			return args.statements.length;
		}
		if (cmd === "db_query") return sqlite.prepare(args.sql).all(...args.params);
		if (cmd === "db_execute") return sqlite.prepare(args.sql).run(...args.params).changes;
		throw new Error(`unmocked ${cmd}`);
	},
	isTauri: () => true
}));

const g = globalThis as any;
const sem = await import("../../app/utils/semester.ts");
Object.assign(g, sem);
const _dap = await import("../../app/composables/useDapodik.ts").catch(() => null);
g.unwrapDapodikBody = (status: number, body: string) => {
	const m = body.match(/^HTTP\/\d(?:\.\d)?\s+(\d{3})/);
	if (!m) return { status, body };
	const i = body.search(/\r?\n\r?\n/);
	return { status: Number(m[1]), body: i === -1 ? "" : body.slice(i).trimStart() };
};
g.explainStatus = (s: number) => `HTTP ${s}`;
const { useDb } = await import("../../app/composables/useDb.ts");
g.useDb = useDb;
const { useDapodikSync } = await import("../../app/composables/useDapodikSync.ts");

test.skipIf(!process.env.DAPODIK_TOKEN)("tes koneksi + ambil semester + ambil semua", async () => {
	sqlite.prepare("INSERT INTO webservice (nama_aplikasi, ip_dapodik, port, token, npsn) VALUES ('tes','localhost',5774,?,?)").run(process.env.DAPODIK_TOKEN!, process.env.DAPODIK_NPSN!);
	const s = useDapodikSync();
	console.log(await s.testKoneksi());
	console.log(await s.ambilSemester("20261"));
	const r = await s.ambilSemua("20261", m => console.log(" ", m));
	console.log(r);
	// jalankan dua kali: harus idempoten
	await s.ambilSemua("20261", () => {});
	for (const t of ["sekolah", "semester", "ptk", "peserta_didik", "rombel", "anggota_rombel", "pembelajaran", "mata_pelajaran"])
		console.log(t, sqlite.query(`SELECT COUNT(*) c FROM ${t}`).get());
	console.log(sqlite.query("SELECT nama, tingkat, jenis_rombel, ptk_id IS NOT NULL wali FROM rombel").all());
	expect(r.siswa).toBeGreaterThan(0);
	// token salah → pesan Dapodik
	sqlite.run("UPDATE webservice SET token = 'salah'");
	await expect(s.testKoneksi()).rejects.toThrow("Aplikasi tidak terdaftar");
}, 120000);
