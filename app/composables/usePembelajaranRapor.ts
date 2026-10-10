import type { SqlStatement } from "./useDb";

const TEMPLATE_KEY = "template_mapel_sd";
const KELOMPOK_KEY = "kelompok_mapel";

export interface SusunResult {
	kelas: number
	pembelajaran: number
	tanpaGuruKelas: string[]
	mapelBaru: string[]
}

// Menyusun pembelajaran rapor per rombel: pembelajaran "Guru Kelas" dari Dapodik dipecah jadi
// mapel sesuai template + fase, pembelajaran Dapodik lain (PAI, PJOK, dst.) dipakai apa adanya.
// Aman dijalankan berulang: pengampu yang diubah manual dan mapel yang sudah ada nilainya tidak disentuh.
export function usePembelajaranRapor() {
	const db = useDb();

	async function readJson<T>(key: string, fallback: T): Promise<T> {
		const raw = await db.getSetting(key);
		if (!raw)
			return structuredClone(fallback);
		try {
			return JSON.parse(raw);
		}
		catch {
			return structuredClone(fallback);
		}
	}

	const getTemplate = () => readJson<TemplateMapel[]>(TEMPLATE_KEY, DEFAULT_TEMPLATE);
	const getKelompok = () => readJson<KelompokMapel[]>(KELOMPOK_KEY, DEFAULT_KELOMPOK);

	async function saveTemplate(t: TemplateMapel[]) {
		await db.setSetting(TEMPLATE_KEY, JSON.stringify(t));
	}

	async function saveKelompok(k: KelompokMapel[]) {
		await db.setSetting(KELOMPOK_KEY, JSON.stringify(k));
	}

	// Urutan di rapor: kelompok dulu (urutan daftar kelompok), lalu urutan mapel di dalamnya.
	function urutanRapor(template: TemplateMapel[], kelompok: KelompokMapel[]) {
		const posKelompok = (nama: string) => {
			const i = kelompok.findIndex(k => k.nama === nama);
			return i === -1 ? kelompok.length : i;
		};
		const urut = new Map<string, number>();
		template.forEach((m, i) => urut.set(m.kode, posKelompok(m.kelompok) * 1000 + i));
		return urut;
	}

	async function susun(semesterId: string): Promise<SusunResult> {
		const [semua, kelompok, rombel, pemb] = await Promise.all([
			getTemplate(),
			getKelompok(),
			db.query<{ rombongan_belajar_id: string, nama: string, tingkat: string, ptk_id: string | null }>(
				"SELECT rombongan_belajar_id, nama, tingkat, ptk_id FROM rombel WHERE semester_id = ? AND jenis_rombel = '1' ORDER BY CAST(tingkat AS INTEGER), nama",
				[semesterId]
			),
			db.query<{ pembelajaran_id: string, rombongan_belajar_id: string, mata_pelajaran_id: string, nama_mata_pelajaran: string, ptk_id: string | null }>(
				"SELECT pembelajaran_id, rombongan_belajar_id, mata_pelajaran_id, nama_mata_pelajaran, ptk_id FROM pembelajaran WHERE semester_id = ?",
				[semesterId]
			)
		]);

		// Pembelajaran Dapodik di luar daftar (mis. agama lain) dimasukkan ke daftar mapel supaya
		// nama, singkatan, dan urutannya bisa diatur di halaman Mata Pelajaran.
		const mapelBaru: string[] = [];
		for (const p of pemb) {
			if (p.mata_pelajaran_id === KODE_GURU_KELAS || semua.some(m => m.kode === p.mata_pelajaran_id))
				continue;
			semua.push({
				kode: p.mata_pelajaran_id,
				nama: p.nama_mata_pelajaran,
				singkat: p.nama_mata_pelajaran.slice(0, SINGKAT_MAX),
				kelompok: kelompok[0]?.nama ?? DEFAULT_KELOMPOK[0]!.nama,
				fase: ["A", "B", "C"],
				aktif: true,
				transkrip: true
			});
			mapelBaru.push(p.nama_mata_pelajaran);
		}
		if (mapelBaru.length)
			await saveTemplate(semua);

		const template = semua.filter(m => m.aktif);
		const urut = urutanRapor(semua, kelompok);

		const st: SqlStatement[] = template.map(m => ({
			sql: `INSERT INTO mapel_rapor (kode, nama, singkat, kelompok, urutan, transkrip) VALUES (?, ?, ?, ?, ?, ?)
				ON CONFLICT (kode) DO UPDATE SET nama = excluded.nama, singkat = excluded.singkat, kelompok = excluded.kelompok,
					urutan = excluded.urutan, transkrip = excluded.transkrip`,
			params: [m.kode, m.nama, m.singkat, m.kelompok, urut.get(m.kode) ?? 99999, m.transkrip === false ? 0 : 1]
		}));

		const tanpaGuruKelas: string[] = [];
		let total = 0;

		for (const r of rombel) {
			const fase = faseOf(r.tingkat);
			const milik = pemb.filter(p => p.rombongan_belajar_id === r.rombongan_belajar_id);
			const guruKelas = milik.find(p => p.mata_pelajaran_id === KODE_GURU_KELAS);
			if (!guruKelas)
				tanpaGuruKelas.push(r.nama);
			const pengampuKelas = guruKelas?.ptk_id ?? r.ptk_id;

			const rows = new Map<string, { ptk: string | null, sumber: string, pembelajaranId: string | null }>();
			for (const m of template) {
				if (!fase || !m.fase.includes(fase))
					continue;
				const dap = milik.find(p => p.mata_pelajaran_id === m.kode);
				rows.set(m.kode, dap
					? { ptk: dap.ptk_id, sumber: "dapodik", pembelajaranId: dap.pembelajaran_id }
					: { ptk: pengampuKelas, sumber: "sub", pembelajaranId: guruKelas?.pembelajaran_id ?? null });
			}

			for (const [kode, v] of rows) {
				total++;
				st.push({
					sql: `INSERT INTO pembelajaran_rapor (semester_id, rombongan_belajar_id, kode_mapel, ptk_id, sumber, pembelajaran_id)
						VALUES (?, ?, ?, ?, ?, ?)
						ON CONFLICT (semester_id, rombongan_belajar_id, kode_mapel) DO UPDATE SET
							ptk_id = CASE WHEN pembelajaran_rapor.sumber = 'manual' THEN pembelajaran_rapor.ptk_id ELSE excluded.ptk_id END,
							sumber = CASE WHEN pembelajaran_rapor.sumber = 'manual' THEN 'manual' ELSE excluded.sumber END,
							pembelajaran_id = excluded.pembelajaran_id`,
					params: [semesterId, r.rombongan_belajar_id, kode, v.ptk, v.sumber, v.pembelajaranId]
				});
			}
			// Mapel yang dimatikan / fasenya dicabut ikut dibuang, kecuali sudah ada nilainya.
			const keep = [...rows.keys()];
			st.push({
				sql: `DELETE FROM pembelajaran_rapor WHERE semester_id = ? AND rombongan_belajar_id = ? AND sumber <> 'manual'
					AND kode_mapel NOT IN (${keep.map(() => "?").join(",") || "''"})
					AND id NOT IN (SELECT pembelajaran_rapor_id FROM nilai_rapor)`,
				params: [semesterId, r.rombongan_belajar_id, ...keep]
			});
		}

		await db.batch(st);
		return { kelas: rombel.length, pembelajaran: total, tanpaGuruKelas, mapelBaru };
	}

	return { getTemplate, saveTemplate, getKelompok, saveKelompok, susun };
}
