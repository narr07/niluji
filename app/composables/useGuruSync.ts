import type { SqlStatement } from "./useDb";
import { invoke } from "@tauri-apps/api/core";

// Sisi laptop guru: TARIK data dari laptop admin, isi nilai offline, lalu KIRIM nilai per kelas.
// Pasangannya di Rust: src-tauri/src/sesi.rs (api_tarik, api_kirim).

export interface KelasSiap {
	rombongan_belajar_id: string
	rombel: string
	mapel: { id: number, singkat: string, siswa: number, lengkap: number, terkunci: number, dikirim_at: string | null, tp: number }[]
	siap: boolean
	terkirim: boolean
}

// Kolom yang boleh ditulis dari paket admin (dicocokkan supaya isi paket tidak bisa menyelipkan SQL).
const KOLOM: Record<string, { cols: string[], key: string, update?: string[] }> = {
	sekolah: { cols: ["sekolah_id", "npsn", "nama", "alamat", "kecamatan", "kabupaten_kota", "provinsi", "email", "kepsek_ptk_id"], key: "sekolah_id" },
	semester: { cols: ["semester_id", "tahun_ajaran", "nama", "aktif", "synced_at"], key: "semester_id" },
	ptk: { cols: ["ptk_id", "nama", "nip", "nuptk", "jenis_kelamin", "gelar_depan", "gelar_belakang"], key: "ptk_id" },
	rombel: { cols: ["rombongan_belajar_id", "semester_id", "nama", "tingkat", "jenis_rombel", "jenis_rombel_str", "kurikulum", "ptk_id"], key: "rombongan_belajar_id" },
	anggota_rombel: { cols: ["anggota_rombel_id", "rombongan_belajar_id", "peserta_didik_id", "semester_id", "jenis_pendaftaran"], key: "anggota_rombel_id" },
	peserta_didik: { cols: ["peserta_didik_id", "nama", "nisn", "nipd", "jenis_kelamin", "tempat_lahir", "tanggal_lahir", "agama"], key: "peserta_didik_id" },
	mapel_rapor: { cols: ["kode", "nama", "singkat", "kelompok", "urutan"], key: "kode" },
	pembelajaran_rapor: { cols: ["id", "semester_id", "rombongan_belajar_id", "kode_mapel", "ptk_id", "sumber", "terkunci", "dikirim_at"], key: "id" },
	tujuan_pembelajaran: { cols: ["uid", "kode_mapel", "tingkat", "semester", "deskripsi", "urutan", "sumber"], key: "uid" },
	kokurikuler_kegiatan: { cols: ["id", "semester_id", "nama", "tema", "tujuan", "tingkat", "dimensi", "subdimensi", "urutan"], key: "id" }
};

function upsert(table: string, row: Record<string, unknown>): SqlStatement {
	const { cols, key } = KOLOM[table]!;
	const set = cols.filter(c => c !== key).map(c => `${c} = excluded.${c}`).join(", ");
	return {
		sql: `INSERT INTO ${table} (${cols.join(", ")}) VALUES (${cols.map(() => "?").join(", ")}) ON CONFLICT (${key}) DO UPDATE SET ${set}`,
		params: cols.map(c => row[c] ?? null)
	};
}

export const normalisasiLink = (url: string) => {
	const u = url.trim().replace(/\/+$/, "");
	return /^https?:\/\//.test(u) ? u : `https://${u}`;
};

export function useGuruSync() {
	const db = useDb();

	async function serverUrl() {
		return (await db.getSetting("server_url")) ?? "";
	}

	async function tarik(input: { url: string, pin: string, username: string, password: string }) {
		const base = normalisasiLink(input.url);
		const res = await invoke<{ guru: { nama: string, username: string, ptkId: string }, paket: Record<string, any[]> & { semesterId: string } }>(
			"http_post_json",
			{ url: `${base}/api/tarik`, body: { pin: input.pin, username: input.username, password: input.password }, timeoutSecs: 90 }
		);
		const { guru, paket } = res;
		const sem = paket.semesterId;

		// Akun lokal: password sama dengan yang baru dipakai, supaya bisa login offline.
		await invoke("guru_set_local_user", { username: guru.username, password: input.password, nama: guru.nama, ptkId: guru.ptkId });

		const prIds = (paket.pembelajaran_rapor ?? []).map(p => p.id);
		const kegIds = (paket.kokurikuler_kegiatan ?? []).map(k => k.id);
		const st: SqlStatement[] = [
			// Data rombel & pembelajaran semester ini diganti sesuai admin (mis. pengampu diganti).
			{ sql: "DELETE FROM anggota_rombel WHERE semester_id = ?", params: [sem] },
			{ sql: "DELETE FROM rombel WHERE semester_id = ?", params: [sem] },
			{ sql: `DELETE FROM nilai_rapor WHERE pembelajaran_rapor_id IN (SELECT id FROM pembelajaran_rapor WHERE semester_id = ? AND id NOT IN (${prIds.map(() => "?").join(",") || "NULL"}))`, params: [sem, ...prIds] },
			{ sql: `DELETE FROM pembelajaran_rapor WHERE semester_id = ? AND id NOT IN (${prIds.map(() => "?").join(",") || "NULL"})`, params: [sem, ...prIds] },
			// Kegiatan kokurikuler yang dihapus admin ikut hilang (beserta capaiannya).
			{ sql: `DELETE FROM nilai_kokurikuler WHERE kegiatan_id IN (SELECT id FROM kokurikuler_kegiatan WHERE semester_id = ? AND id NOT IN (${kegIds.map(() => "?").join(",") || "NULL"}))`, params: [sem, ...kegIds] },
			{ sql: `DELETE FROM kokurikuler_kegiatan WHERE semester_id = ? AND id NOT IN (${kegIds.map(() => "?").join(",") || "NULL"})`, params: [sem, ...kegIds] }
		];
		for (const table of Object.keys(KOLOM)) {
			for (const row of paket[table] ?? [])
				st.push(upsert(table, row));
		}
		await db.batch(st);

		// Nilai dari admin: TP disimpan sebagai uid di paket → ubah ke id lokal.
		const tp = await db.query<{ id: number, uid: string }>("SELECT id, uid FROM tujuan_pembelajaran");
		const idOf = new Map(tp.map(t => [t.uid, t.id]));
		const terkunci = new Set((paket.pembelajaran_rapor ?? []).filter(p => p.terkunci).map(p => p.id));
		const toIds = (json: string) => (JSON.parse(json || "[]") as string[]).map(u => idOf.get(u)).filter(Boolean);
		await db.batch((paket.nilai_rapor ?? []).map(n => ({
			// Sudah terkunci di admin → nilai admin yang berlaku. Belum → pekerjaan lokal guru tidak ditimpa.
			sql: `INSERT ${terkunci.has(n.pembelajaran_rapor_id) ? "OR REPLACE" : "OR IGNORE"} INTO nilai_rapor
				(pembelajaran_rapor_id, peserta_didik_id, nilai, tp_optimal, tp_perlu) VALUES (?, ?, ?, ?, ?)`,
			params: [n.pembelajaran_rapor_id, n.peserta_didik_id, n.nilai, JSON.stringify(toIds(n.tp_optimal)), JSON.stringify(toIds(n.tp_perlu))]
		})));

		// Isian wali kelas dari admin hanya mengisi yang belum ada; isian lokal guru tidak ditimpa.
		await db.batch((paket.rapor_siswa ?? []).map(w => ({
			sql: `INSERT OR IGNORE INTO rapor_siswa (semester_id, peserta_didik_id, rombongan_belajar_id, sakit, izin, alpa, catatan, naik, kokurikuler)
				VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)`,
			params: [w.semester_id, w.peserta_didik_id, w.rombongan_belajar_id, w.sakit, w.izin, w.alpa, w.catatan, w.naik, w.kokurikuler]
		})));
		await db.batch((paket.nilai_kokurikuler ?? []).map(n => ({
			sql: "INSERT OR IGNORE INTO nilai_kokurikuler (kegiatan_id, peserta_didik_id, capaian) VALUES (?, ?, ?)",
			params: [n.kegiatan_id, n.peserta_didik_id, n.capaian]
		})));
		// Nilai ekskul dari admin: sama, hanya mengisi yang belum ada.
		await db.batch((paket.nilai_ekskul ?? []).map(n => ({
			sql: `INSERT OR IGNORE INTO nilai_ekskul (semester_id, rombongan_belajar_id, peserta_didik_id, predikat, keterangan)
				VALUES (?, ?, ?, ?, ?)`,
			params: [n.semester_id, n.rombongan_belajar_id, n.peserta_didik_id, n.predikat, n.keterangan]
		})));

		await db.setSetting("mode", "guru");
		await db.setSetting("server_url", base);
		await db.setSetting("guru_username", guru.username);
		await db.setSetting("last_pull", JSON.stringify({ at: new Date().toLocaleString("sv-SE"), semesterId: sem }));
		useAppMode().mode.value = "guru";
		return { guru, semesterId: sem };
	}

	// Kesiapan kirim per kelas: setiap siswa punya nilai + minimal 1 TP tercapai dan 1 TP perlu bantuan di semua mapel guru di kelas itu.
	async function kesiapan(ptkId: string, semesterId: string): Promise<KelasSiap[]> {
		const smt = semesterKe(semesterId);
		const rows = await db.query<KelasSiap["mapel"][number] & { rombongan_belajar_id: string, rombel: string }>(
			`SELECT pr.id, pr.rombongan_belajar_id, r.nama AS rombel, m.singkat, pr.terkunci, pr.dikirim_at,
				(SELECT COUNT(*) FROM anggota_rombel a WHERE a.rombongan_belajar_id = pr.rombongan_belajar_id) AS siswa,
				(SELECT COUNT(*) FROM nilai_rapor n WHERE n.pembelajaran_rapor_id = pr.id AND n.nilai IS NOT NULL
					AND n.tp_optimal <> '[]' AND n.tp_perlu <> '[]') AS lengkap,
				(SELECT COUNT(*) FROM tujuan_pembelajaran t WHERE t.kode_mapel = pr.kode_mapel AND t.tingkat = CAST(r.tingkat AS INTEGER) AND t.semester = ?) AS tp
			FROM pembelajaran_rapor pr JOIN rombel r USING (rombongan_belajar_id) JOIN mapel_rapor m ON m.kode = pr.kode_mapel
			WHERE pr.ptk_id = ? AND pr.semester_id = ? ORDER BY CAST(r.tingkat AS INTEGER), r.nama, m.urutan`,
			[smt, ptkId, semesterId]
		);
		const map = new Map<string, KelasSiap>();
		for (const r of rows) {
			if (!map.has(r.rombongan_belajar_id))
				map.set(r.rombongan_belajar_id, { rombongan_belajar_id: r.rombongan_belajar_id, rombel: r.rombel, mapel: [], siap: true, terkirim: true });
			const k = map.get(r.rombongan_belajar_id)!;
			k.mapel.push(r);
			if (!r.terkunci) {
				k.terkirim = false;
				if (r.lengkap < r.siswa || !r.siswa)
					k.siap = false;
			}
		}
		return [...map.values()].map(k => ({ ...k, siap: k.siap && !k.terkirim }));
	}

	async function kirim(input: { pin: string, password: string, rombelId: string, ptkId: string, semesterId: string }) {
		const base = await serverUrl();
		const username = (await db.getSetting("guru_username")) ?? "";
		const prs = await db.query<{ id: number }>(
			"SELECT id FROM pembelajaran_rapor WHERE ptk_id = ? AND semester_id = ? AND rombongan_belajar_id = ? AND terkunci = 0",
			[input.ptkId, input.semesterId, input.rombelId]
		);
		const ids = prs.map(p => p.id);
		if (!ids.length)
			throw new Error("Semua mapel kelas ini sudah terkirim.");
		const ph = ids.map(() => "?").join(",");

		const nilai = await db.query<{ pembelajaran_rapor_id: number, peserta_didik_id: string, nilai: number | null, tp_optimal: string, tp_perlu: string }>(
			`SELECT n.pembelajaran_rapor_id, n.peserta_didik_id, n.nilai,
				(SELECT json_group_array(t.uid) FROM json_each(n.tp_optimal) j JOIN tujuan_pembelajaran t ON t.id = j.value) AS tp_optimal,
				(SELECT json_group_array(t.uid) FROM json_each(n.tp_perlu) j JOIN tujuan_pembelajaran t ON t.id = j.value) AS tp_perlu
			FROM nilai_rapor n WHERE n.pembelajaran_rapor_id IN (${ph})`,
			ids
		);
		// TP yang dipakai nilai + semua TP buatan guru untuk mapel/kelas ini (ikut masuk bank TP admin).
		const tp = await db.query(
			`SELECT DISTINCT t.uid, t.kode_mapel, t.tingkat, t.semester, t.deskripsi, t.urutan FROM tujuan_pembelajaran t
			WHERE t.id IN (SELECT j.value FROM nilai_rapor n, json_each(n.tp_optimal) j WHERE n.pembelajaran_rapor_id IN (${ph})
				UNION SELECT j.value FROM nilai_rapor n, json_each(n.tp_perlu) j WHERE n.pembelajaran_rapor_id IN (${ph}))
			OR (t.sumber = 'guru' AND EXISTS (SELECT 1 FROM pembelajaran_rapor pr JOIN rombel r USING (rombongan_belajar_id)
				WHERE pr.id IN (${ph}) AND pr.kode_mapel = t.kode_mapel AND CAST(r.tingkat AS INTEGER) = t.tingkat))`,
			[...ids, ...ids, ...ids]
		);

		const res = await invoke<{ pembelajaran: number, nilai: number, waktu: string }>("http_post_json", {
			url: `${base}/api/kirim`,
			body: {
				pin: input.pin,
				username,
				password: input.password,
				semester_id: input.semesterId,
				pembelajaran_ids: ids,
				tp,
				nilai: nilai.map(n => ({ ...n, tp_optimal: JSON.parse(n.tp_optimal), tp_perlu: JSON.parse(n.tp_perlu) }))
			},
			timeoutSecs: 90
		});
		await db.execute(`UPDATE pembelajaran_rapor SET terkunci = 1, dikirim_at = ? WHERE id IN (${ph})`, [res.waktu, ...ids]);
		return res;
	}

	// Isian wali kelas satu kelas (kehadiran, catatan, kenaikan). Boleh dikirim ulang kapan saja.
	async function kirimWalas(input: { pin: string, password: string, rombelId: string, semesterId: string }) {
		const base = await serverUrl();
		const username = (await db.getSetting("guru_username")) ?? "";
		const siswa = await db.query(
			`SELECT peserta_didik_id, sakit, izin, alpa, catatan, naik, kokurikuler FROM rapor_siswa
			WHERE semester_id = ? AND rombongan_belajar_id = ?`,
			[input.semesterId, input.rombelId]
		);
		const kokurikuler = await db.query(
			`SELECT n.kegiatan_id, n.peserta_didik_id, n.capaian FROM nilai_kokurikuler n JOIN kokurikuler_kegiatan k ON k.id = n.kegiatan_id
			WHERE k.semester_id = ? AND n.peserta_didik_id IN (SELECT peserta_didik_id FROM anggota_rombel WHERE rombongan_belajar_id = ?)`,
			[input.semesterId, input.rombelId]
		);
		// Nilai ekskul siswa kelas ini (wali kelas boleh mengisi kalau pembina belum).
		const ekskul = await db.query(
			`SELECT rombongan_belajar_id, peserta_didik_id, predikat, keterangan FROM nilai_ekskul
			WHERE semester_id = ? AND peserta_didik_id IN (SELECT peserta_didik_id FROM anggota_rombel WHERE rombongan_belajar_id = ?)`,
			[input.semesterId, input.rombelId]
		);
		if (!siswa.length && !kokurikuler.length && !ekskul.length)
			throw new Error("Belum ada isian wali kelas untuk kelas ini.");
		const res = await invoke<{ rombel: string, siswa: number, waktu: string }>("http_post_json", {
			url: `${base}/api/kirim-walas`,
			body: { pin: input.pin, username, password: input.password, semester_id: input.semesterId, rombongan_belajar_id: input.rombelId, siswa, kokurikuler, ekskul },
			timeoutSecs: 60
		});
		await db.setSetting(`walas_terkirim:${input.rombelId}`, res.waktu);
		return res;
	}

	// Nilai satu ekskul yang dibina. Boleh dikirim ulang kapan saja.
	async function kirimEkskul(input: { pin: string, password: string, ekskulId: string, semesterId: string }) {
		const base = await serverUrl();
		const username = (await db.getSetting("guru_username")) ?? "";
		const siswa = await db.query(
			"SELECT peserta_didik_id, predikat, keterangan FROM nilai_ekskul WHERE semester_id = ? AND rombongan_belajar_id = ?",
			[input.semesterId, input.ekskulId]
		);
		if (!siswa.length)
			throw new Error("Belum ada nilai untuk ekskul ini.");
		const res = await invoke<{ ekskul: string, siswa: number, waktu: string }>("http_post_json", {
			url: `${base}/api/kirim-ekskul`,
			body: { pin: input.pin, username, password: input.password, semester_id: input.semesterId, rombongan_belajar_id: input.ekskulId, siswa },
			timeoutSecs: 60
		});
		await db.setSetting(`ekskul_terkirim:${input.ekskulId}`, res.waktu);
		return res;
	}

	return { serverUrl, tarik, kesiapan, kirim, kirimWalas, kirimEkskul };
}
