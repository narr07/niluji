import { deskripsiCapaian, semesterKe, KODE_GURU_KELAS } from "~/utils/kurikulum";
import { dapodikGet, dapodikPost, useDapodikSync } from "./useDapodikSync";

export interface DapodikMatev {
	id_evaluasi: string
	nm_mata_evaluasi: string
	a_dari_template: string
	no_urut: string
	kkm_kognitif?: string
	kkm_psikomotorik?: string
	rombongan_belajar_id: string
	mata_pelajaran_id: string | number
	pembelajaran_id: string
	updater_id: string
}

export interface DapodikNilai {
	nilai_id: string
	id_evaluasi: string
	anggota_rombel_id: string
	nilai_kognitif_angka: string
	nilai_kognitif_huruf?: string | null
	ket_kognitif: string | null
	a_beku?: string
	rapor_ke?: string
}

export interface HasilKirimMatev {
	totalMapel: number
	sudahAda: number
	berhasilDibuat: number
	gagal: number
	rincian: { mapel: string, kode: string, status: "sudah_ada" | "sukses" | "gagal", pesan?: string }[]
	warnings: string[]
}

export interface HasilKirimNilai {
	totalNilai: number
	terkirimBaru: number
	diperbarui: number
	sama: number
	gagal: number
	rincian: { mapel: string, siswa: string, nilai: number, status: "baru" | "update" | "sama" | "gagal", pesan?: string }[]
	warnings: string[]
}

export function useKirimDapodik() {
	const db = useDb();
	const { session } = useAuth();
	const { requireWebService } = useDapodikSync();

	// Ambil semua daftar mata evaluasi yang ada di Dapodik untuk semester aktif
	async function ambilMatevDapodik(): Promise<DapodikMatev[]> {
		const ws = await requireWebService();
		const sem = session.value?.semesterId;
		if (!sem) throw new Error("Semester belum aktif.");
		const rows = await dapodikGet(ws, "getMatevNilai?a_dari_template=1", sem);
		return rows as DapodikMatev[];
	}

	// Kirim Matev untuk satu rombel/kelas
	async function kirimMatevKelas(
		rombelId: string,
		onProgress?: (msg: string) => void
	): Promise<HasilKirimMatev> {
		const ws = await requireWebService();
		const sem = session.value?.semesterId;
		if (!sem) throw new Error("Semester belum aktif.");

		onProgress?.("Memeriksa mata evaluasi di Dapodik...");
		const existingMatev = await ambilMatevDapodik();
		const rombelMatev = existingMatev.filter(m => m.rombongan_belajar_id === rombelId);

		// Ambil updater_id acuan
		const updaterId = existingMatev.find(m => m.updater_id)?.updater_id || "1e5239f0-6b0e-11e5-8e67-8bcee073aaec";

		// Ambil referensi pembelajaran Dapodik untuk rombel ini dari database lokal
		const pembRows = await db.query<{ pembelajaran_id: string, mata_pelajaran_id: string }>(
			"SELECT pembelajaran_id, mata_pelajaran_id FROM pembelajaran WHERE rombongan_belajar_id = ?",
			[rombelId]
		);
		const pembMap = new Map<string, string>();
		let guruKelasPembId: string | undefined;
		for (const p of pembRows) {
			pembMap.set(String(p.mata_pelajaran_id), p.pembelajaran_id);
			if (String(p.mata_pelajaran_id) === KODE_GURU_KELAS) {
				guruKelasPembId = p.pembelajaran_id;
			}
		}

		// Ambil mapel rapor untuk rombel ini
		const mapelRows = await db.query<{ id: number, kode_mapel: string, nama: string, urutan: number }>(
			`SELECT pr.id, pr.kode_mapel, m.nama, m.urutan
			FROM pembelajaran_rapor pr
			JOIN mapel_rapor m ON m.kode = pr.kode_mapel
			WHERE pr.rombongan_belajar_id = ? AND pr.semester_id = ?
			ORDER BY m.urutan`,
			[rombelId, sem]
		);

		const hasil: HasilKirimMatev = {
			totalMapel: mapelRows.length,
			sudahAda: 0,
			berhasilDibuat: 0,
			gagal: 0,
			rincian: [],
			warnings: []
		};

		const nowStr = new Date().toLocaleString("sv-SE");

		for (const m of mapelRows) {
			const kode = String(m.kode_mapel);
			const exists = rombelMatev.some(em =>
				String(em.mata_pelajaran_id).trim() === kode.trim()
				|| em.nm_mata_evaluasi.trim().toLowerCase() === m.nama.trim().toLowerCase()
			);

			if (exists) {
				hasil.sudahAda++;
				hasil.rincian.push({ mapel: m.nama, kode, status: "sudah_ada", pesan: "Sudah terdaftar di Dapodik" });
				continue;
			}

			// Tentukan pembelajaran_id di Dapodik:
			// 1. Cocokkan kode mapel spesifik
			// 2. Jika tidak ada, pakai pembelajaran Guru Kelas (400200000)
			// 3. Jika kelas tidak punya Guru Kelas di Dapodik (seperti Kelas 1), fallback ke pembelajaran yang ada di rombel tersebut
			const pembId = pembMap.get(kode) ?? guruKelasPembId ?? (pembRows.length > 0 ? pembRows[0].pembelajaran_id : undefined);
			if (!pembId) {
				hasil.gagal++;
				const warn = `Mapel ${m.nama} (${kode}) gagal dibuat: Tidak ada data pembelajaran di Dapodik untuk rombel ini.`;
				hasil.warnings.push(warn);
				hasil.rincian.push({ mapel: m.nama, kode, status: "gagal", pesan: warn });
				continue;
			}

			onProgress?.(`Membuat matev ${m.nama}...`);

			const payload = {
				id_evaluasi: crypto.randomUUID(),
				nm_mata_evaluasi: m.nama,
				a_dari_template: "1",
				no_urut: String(m.urutan || 1),
				kkm_kognitif: "0.00",
				kkm_psikomotorik: "0.00",
				rombongan_belajar_id: rombelId,
				mata_pelajaran_id: kode,
				pembelajaran_id: pembId,
				create_date: nowStr,
				last_update: nowStr,
				soft_delete: "0",
				last_sync: "1901-01-01 00:00:00",
				updater_id: updaterId
			};

			try {
				const res = await dapodikPost(ws, "postMatevRapor", sem, payload);
				if (res.success) {
					hasil.berhasilDibuat++;
					hasil.rincian.push({ mapel: m.nama, kode, status: "sukses", pesan: "Berhasil didaftarkan ke Dapodik" });
				}
				else {
					hasil.gagal++;
					hasil.rincian.push({ mapel: m.nama, kode, status: "gagal", pesan: res.message });
				}
			}
			catch (err) {
				hasil.gagal++;
				hasil.rincian.push({ mapel: m.nama, kode, status: "gagal", pesan: pesanError(err) });
			}
		}

		return hasil;
	}

	// Kirim Matev untuk semua kelas
	async function kirimMatevSemua(
		onProgress?: (msg: string, current: number, total: number) => void
	): Promise<HasilKirimMatev> {
		const sem = session.value?.semesterId;
		if (!sem) throw new Error("Semester belum aktif.");

		const rombels = await db.query<{ rombongan_belajar_id: string, nama: string }>(
			"SELECT rombongan_belajar_id, nama FROM rombel WHERE CAST(tingkat AS INTEGER) >= 1 ORDER BY CAST(tingkat AS INTEGER), nama"
		);

		const gabungan: HasilKirimMatev = {
			totalMapel: 0,
			sudahAda: 0,
			berhasilDibuat: 0,
			gagal: 0,
			rincian: [],
			warnings: []
		};

		for (let i = 0; i < rombels.length; i++) {
			const r = rombels[i];
			onProgress?.(`Memproses matev ${r.nama} (${i + 1}/${rombels.length})...`, i + 1, rombels.length);
			const h = await kirimMatevKelas(r.rombongan_belajar_id, msg => {
				onProgress?.(`${r.nama}: ${msg}`, i + 1, rombels.length);
			});
			gabungan.totalMapel += h.totalMapel;
			gabungan.sudahAda += h.sudahAda;
			gabungan.berhasilDibuat += h.berhasilDibuat;
			gabungan.gagal += h.gagal;
			gabungan.rincian.push(...h.rincian);
			gabungan.warnings.push(...h.warnings);
		}

		return gabungan;
	}

	// Kirim Nilai Rapor untuk satu rombel/kelas
	async function kirimNilaiKelas(
		rombelId: string,
		onProgress?: (msg: string, current: number, total: number) => void
	): Promise<HasilKirimNilai> {
		const ws = await requireWebService();
		const sem = session.value?.semesterId;
		if (!sem) throw new Error("Semester belum aktif.");

		onProgress?.("Memeriksa mata evaluasi di Dapodik...", 0, 1);
		const allMatev = await ambilMatevDapodik();
		const rombelMatev = allMatev.filter(m => m.rombongan_belajar_id === rombelId);

		const hasil: HasilKirimNilai = {
			totalNilai: 0,
			terkirimBaru: 0,
			diperbarui: 0,
			sama: 0,
			gagal: 0,
			rincian: [],
			warnings: []
		};

		if (rombelMatev.length === 0) {
			hasil.warnings.push("Belum ada mata evaluasi di Dapodik untuk kelas ini. Klik \"Kirim Matev\" terlebih dahulu.");
			return hasil;
		}

		const defaultUpdaterId = allMatev.find(m => m.updater_id)?.updater_id || "1e5239f0-6b0e-11e5-8e67-8bcee073aaec";

		// Ambil semua TP untuk konversi ID ke deskripsi teks
		const tpRows = await db.query<{ id: number, deskripsi: string }>("SELECT id, deskripsi FROM tujuan_pembelajaran");
		const tpMap = new Map<number, string>();
		for (const t of tpRows) {
			tpMap.set(t.id, t.deskripsi);
		}

		// Ambil seluruh nilai siswa di rombel ini
		const nilaiRows = await db.query<{
			pr_id: number
			kode_mapel: string
			mapel_nama: string
			anggota_rombel_id: string | null
			peserta_didik_id: string
			siswa_nama: string
			nilai: number | null
			tp_optimal: string | null
			tp_perlu: string | null
		}>(
			`SELECT pr.id AS pr_id, pr.kode_mapel, m.nama AS mapel_nama,
				a.anggota_rombel_id, s.peserta_didik_id, s.nama AS siswa_nama,
				n.nilai, n.tp_optimal, n.tp_perlu
			FROM pembelajaran_rapor pr
			JOIN mapel_rapor m ON m.kode = pr.kode_mapel
			JOIN anggota_rombel a ON a.rombongan_belajar_id = pr.rombongan_belajar_id
			JOIN siswa_rapor s ON s.peserta_didik_id = a.peserta_didik_id
			LEFT JOIN nilai_rapor n ON n.pembelajaran_rapor_id = pr.id AND n.peserta_didik_id = s.peserta_didik_id
			WHERE pr.rombongan_belajar_id = ? AND pr.semester_id = ?
			ORDER BY m.urutan, s.nama COLLATE NOCASE`,
			[rombelId, sem]
		);

		// Filter hanya yang sudah dinilai
		const siapKirim = nilaiRows.filter(r => r.nilai !== null && r.nilai !== undefined);

		hasil.totalNilai = siapKirim.length;

		if (siapKirim.length === 0) {
			hasil.warnings.push("Belum ada nilai yang terisi untuk kelas ini.");
			return hasil;
		}

		// Kelompokkan per mapel untuk mengambil nilai Dapodik efisien per mata evaluasi
		const perMapel = new Map<string, typeof siapKirim>();
		for (const row of siapKirim) {
			const k = String(row.kode_mapel);
			if (!perMapel.has(k)) perMapel.set(k, []);
			perMapel.get(k)!.push(row);
		}

		let processed = 0;
		const total = siapKirim.length;
		const nowStr = new Date().toLocaleString("sv-SE");
		const raporKe = String(semesterKe(sem));

		for (const [kodeMapel, rows] of perMapel.entries()) {
			const mapelNama = rows[0].mapel_nama;
			const matev = rombelMatev.find(m =>
				String(m.mata_pelajaran_id).trim() === kodeMapel.trim()
				|| m.nm_mata_evaluasi.trim().toLowerCase() === mapelNama.trim().toLowerCase()
			);

			if (!matev) {
				const warn = `Matev untuk ${mapelNama} (${kodeMapel}) belum ada di Dapodik. Kirim Matev terlebih dahulu.`;
				hasil.warnings.push(warn);
				for (const r of rows) {
					hasil.gagal++;
					hasil.rincian.push({ mapel: mapelNama, siswa: r.siswa_nama, nilai: r.nilai!, status: "gagal", pesan: warn });
					processed++;
					onProgress?.(`Memproses ${mapelNama}: ${r.siswa_nama}...`, processed, total);
				}
				continue;
			}

			// Ambil nilai yang sudah ada di Dapodik untuk id_evaluasi ini
			let dapodikRows: DapodikNilai[] = [];
			try {
				dapodikRows = (await dapodikGet(ws, `getNilai?table=rapor&id_evaluasi=${matev.id_evaluasi}`, sem)) as DapodikNilai[];
			}
			catch {
				// Bila getNilai gagal, anggap kosong
				dapodikRows = [];
			}

			const dapodikMap = new Map<string, DapodikNilai>();
			for (const dr of dapodikRows) {
				if (dr.anggota_rombel_id) {
					dapodikMap.set(dr.anggota_rombel_id, dr);
				}
			}

			for (const r of rows) {
				processed++;
				onProgress?.(`Mengirim ${mapelNama}: ${r.siswa_nama} (${processed}/${total})...`, processed, total);

				if (!r.anggota_rombel_id) {
					hasil.gagal++;
					const warn = `Siswa ${r.siswa_nama} tidak memiliki anggota_rombel_id di Dapodik.`;
					hasil.rincian.push({ mapel: mapelNama, siswa: r.siswa_nama, nilai: r.nilai!, status: "gagal", pesan: warn });
					continue;
				}

				// Bangun ket_kognitif dari TP
				let optIds: number[] = [];
				let perIds: number[] = [];
				try { optIds = JSON.parse(r.tp_optimal || "[]"); } catch {}
				try { perIds = JSON.parse(r.tp_perlu || "[]"); } catch {}
				const optTexts = optIds.map(id => tpMap.get(id)).filter((t): t is string => Boolean(t));
				const perTexts = perIds.map(id => tpMap.get(id)).filter((t): t is string => Boolean(t));
				const desc = deskripsiCapaian(optTexts, perTexts);
				const ketKognitif = [desc.capai, desc.perlu].filter(Boolean).join(" ");

				const existing = dapodikMap.get(r.anggota_rombel_id);

				// Periksa apakah nilai dan keterangan sudah sama persis
				if (existing) {
					const nilaiSama = Math.round(Number(existing.nilai_kognitif_angka)) === Math.round(r.nilai!);
					const ketSama = (existing.ket_kognitif || "").trim() === ketKognitif.trim();
					if (nilaiSama && ketSama) {
						hasil.sama++;
						hasil.rincian.push({ mapel: mapelNama, siswa: r.siswa_nama, nilai: r.nilai!, status: "sama", pesan: "Sudah sama di Dapodik" });
						continue;
					}
				}

				const isUpdate = Boolean(existing);
				const nilaiId = existing ? existing.nilai_id : crypto.randomUUID();

				const payload = {
					nilai_id: nilaiId,
					id_evaluasi: matev.id_evaluasi,
					anggota_rombel_id: r.anggota_rombel_id,
					nilai_kognitif_angka: String(r.nilai!),
					ket_kognitif: ketKognitif || null,
					a_beku: "0",
					rapor_ke: raporKe,
					create_date: nowStr,
					last_update: nowStr,
					soft_delete: "0",
					last_sync: "1901-01-01 00:00:00",
					updater_id: matev.updater_id || defaultUpdaterId
				};

				try {
					const res = await dapodikPost(ws, "postNilai?table=rapor", sem, payload);
					if (res.success) {
						if (isUpdate) {
							hasil.diperbarui++;
							hasil.rincian.push({ mapel: mapelNama, siswa: r.siswa_nama, nilai: r.nilai!, status: "update", pesan: "Diperbarui di Dapodik" });
						}
						else {
							hasil.terkirimBaru++;
							hasil.rincian.push({ mapel: mapelNama, siswa: r.siswa_nama, nilai: r.nilai!, status: "baru", pesan: "Berhasil dikirim ke Dapodik" });
						}
					}
					else {
						hasil.gagal++;
						hasil.rincian.push({ mapel: mapelNama, siswa: r.siswa_nama, nilai: r.nilai!, status: "gagal", pesan: res.message });
					}
				}
				catch (err) {
					hasil.gagal++;
					hasil.rincian.push({ mapel: mapelNama, siswa: r.siswa_nama, nilai: r.nilai!, status: "gagal", pesan: pesanError(err) });
				}
			}
		}

		return hasil;
	}

	// Kirim Nilai untuk semua kelas
	async function kirimNilaiSemua(
		onProgress?: (msg: string, current: number, total: number) => void
	): Promise<HasilKirimNilai> {
		const sem = session.value?.semesterId;
		if (!sem) throw new Error("Semester belum aktif.");

		const rombels = await db.query<{ rombongan_belajar_id: string, nama: string }>(
			"SELECT rombongan_belajar_id, nama FROM rombel WHERE CAST(tingkat AS INTEGER) >= 1 ORDER BY CAST(tingkat AS INTEGER), nama"
		);

		const gabungan: HasilKirimNilai = {
			totalNilai: 0,
			terkirimBaru: 0,
			diperbarui: 0,
			sama: 0,
			gagal: 0,
			rincian: [],
			warnings: []
		};

		for (let i = 0; i < rombels.length; i++) {
			const r = rombels[i];
			onProgress?.(`Memproses ${r.nama} (${i + 1}/${rombels.length})...`, i + 1, rombels.length);
			const h = await kirimNilaiKelas(r.rombongan_belajar_id, (msg, cur, tot) => {
				onProgress?.(`${r.nama}: ${msg}`, cur, tot);
			});
			gabungan.totalNilai += h.totalNilai;
			gabungan.terkirimBaru += h.terkirimBaru;
			gabungan.diperbarui += h.diperbarui;
			gabungan.sama += h.sama;
			gabungan.gagal += h.gagal;
			gabungan.rincian.push(...h.rincian);
			gabungan.warnings.push(...h.warnings);
		}

		return gabungan;
	}

	return {
		ambilMatevDapodik,
		kirimMatevKelas,
		kirimMatevSemua,
		kirimNilaiKelas,
		kirimNilaiSemua
	};
}
