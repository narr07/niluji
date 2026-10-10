// Data satu kelas untuk cetak rapor (format Laporan Hasil Belajar e-Rapor SD 2025, Kurikulum Merdeka).
// Semua diambil sekali per kelas, lalu dirakit per siswa di sisi tampilan.

export interface RaporMapel { kode: string, nama: string, singkat: string, nilai: number | null, capaian: string }
export interface RaporSiswa {
	id: string
	nama: string
	nisn: string | null
	nipd: string | null
	kelompok: { nama: string, mapel: RaporMapel[] }[]
	ekskul: { nama: string, predikat: string, keterangan: string }[]
	kokurikuler: string
	sakit: number | null
	izin: number | null
	alpa: number | null
	catatan: string
	naik: number | null
	kurang: string[] // yang belum lengkap, untuk peringatan sebelum cetak
}
export interface Penanda { nama: string, nip: string | null }
export interface RaporKelas {
	sekolah: { nama: string, npsn: string, alamat: string, kecamatan: string | null, kabupaten: string | null }
	kepsek: Penanda | null
	wali: Penanda | null
	kelas: { nama: string, tingkat: number, fase: string }
	semesterKe: number
	tahunAjaran: string
	gambar: GambarRapor
	ttdWali?: string
	siswa: RaporSiswa[]
}

export function useRapor() {
	const db = useDb();
	const { getKelompok } = usePembelajaranRapor();

	async function penanda(ptkId?: string | null): Promise<Penanda | null> {
		if (!ptkId)
			return null;
		const p = await db.first<{ nama: string, nip: string | null, gelar_depan: string | null, gelar_belakang: string | null }>(
			"SELECT nama, nip, gelar_depan, gelar_belakang FROM ptk WHERE ptk_id = ?",
			[ptkId]
		);
		return p ? { nama: namaLengkap(p), nip: p.nip?.trim() || null } : null;
	}

	async function muat(rombelId: string, semesterId: string): Promise<RaporKelas> {
		const rombel = await db.first<{ nama: string, tingkat: number, ptk_id: string | null }>(
			"SELECT nama, CAST(tingkat AS INTEGER) AS tingkat, ptk_id FROM rombel WHERE rombongan_belajar_id = ?",
			[rombelId]
		);
		if (!rombel)
			throw new Error("Kelas tidak ditemukan. Jalankan Sinkron Dapodik ulang.");
		const sekolah = await db.first<{ nama: string, npsn: string, alamat: string | null, kecamatan: string | null, kabupaten_kota: string | null, kepsek_ptk_id: string | null }>(
			"SELECT nama, npsn, alamat, kecamatan, kabupaten_kota, kepsek_ptk_id FROM sekolah LIMIT 1"
		);

		// Ekskul yang diikuti siswa kelas ini (anggota rombel ekskul) beserta nilainya.
		const ekskul = await db.query<{ siswa: string, nama: string, predikat: string | null, keterangan: string | null }>(
			`SELECT a.peserta_didik_id AS siswa, e.nama, n.predikat, n.keterangan
			FROM anggota_rombel a JOIN rombel e ON e.rombongan_belajar_id = a.rombongan_belajar_id AND e.jenis_rombel = '51' AND e.semester_id = ?
			LEFT JOIN nilai_ekskul n ON n.semester_id = e.semester_id AND n.rombongan_belajar_id = e.rombongan_belajar_id AND n.peserta_didik_id = a.peserta_didik_id
			WHERE a.peserta_didik_id IN (SELECT peserta_didik_id FROM anggota_rombel WHERE rombongan_belajar_id = ?)
			ORDER BY e.nama COLLATE NOCASE`,
			[semesterId, rombelId]
		);

		const [siswa, mapel, isian, kelompokUrut] = await Promise.all([
			db.query<{ id: string, nama: string, nisn: string | null, nipd: string | null }>(
				`SELECT s.peserta_didik_id AS id, s.nama, s.nisn, s.nipd FROM anggota_rombel a JOIN siswa_rapor s USING (peserta_didik_id)
				WHERE a.rombongan_belajar_id = ? ORDER BY s.nama COLLATE NOCASE`,
				[rombelId]
			),
			db.query<{ id: number, kode: string, nama: string, singkat: string, kelompok: string, urutan: number }>(
				`SELECT pr.id, m.kode, m.nama, m.singkat, m.kelompok, m.urutan FROM pembelajaran_rapor pr JOIN mapel_rapor m ON m.kode = pr.kode_mapel
				WHERE pr.rombongan_belajar_id = ? AND pr.semester_id = ? ORDER BY m.urutan`,
				[rombelId, semesterId]
			),
			db.query<{ peserta_didik_id: string, sakit: number | null, izin: number | null, alpa: number | null, catatan: string | null, naik: number | null, kokurikuler: string | null }>(
				"SELECT peserta_didik_id, sakit, izin, alpa, catatan, naik, kokurikuler FROM rapor_siswa WHERE semester_id = ? AND rombongan_belajar_id = ?",
				[semesterId, rombelId]
			),
			getKelompok()
		]);

		// Kokurikuler: kegiatan untuk tingkat kelas ini + capaian siswanya (deskripsi otomatis kalau tidak ditulis).
		const kegiatan = await db.query<{ id: number, nama: string }>(
			"SELECT id, nama FROM kokurikuler_kegiatan k WHERE semester_id = ? AND EXISTS (SELECT 1 FROM json_each(k.tingkat) WHERE value = ?) ORDER BY urutan, id",
			[semesterId, rombel.tingkat]
		);
		const capaianKoku = kegiatan.length
			? await db.query<{ kegiatan_id: number, peserta_didik_id: string, capaian: string }>(
				`SELECT kegiatan_id, peserta_didik_id, capaian FROM nilai_kokurikuler WHERE kegiatan_id IN (${kegiatan.map(() => "?").join(",")})`,
				kegiatan.map(k => k.id)
			)
			: [];
		const kokuOf = new Map(capaianKoku.map(c => [`${c.kegiatan_id}:${c.peserta_didik_id}`, JSON.parse(c.capaian) as Record<string, number>]));

		const prIds = mapel.map(m => m.id);
		const nilai = prIds.length
			? await db.query<{ pr: number, siswa: string, nilai: number | null, tp_optimal: string, tp_perlu: string }>(
				`SELECT pembelajaran_rapor_id AS pr, peserta_didik_id AS siswa, nilai, tp_optimal, tp_perlu FROM nilai_rapor
				WHERE pembelajaran_rapor_id IN (${prIds.map(() => "?").join(",")})`,
				prIds
			)
			: [];
		const tpIds = [...new Set(nilai.flatMap(n => [...JSON.parse(n.tp_optimal), ...JSON.parse(n.tp_perlu)] as number[]))];
		const tp = tpIds.length
			? await db.query<{ id: number, deskripsi: string }>(
				`SELECT id, deskripsi FROM tujuan_pembelajaran WHERE id IN (${tpIds.map(() => "?").join(",")})`,
				tpIds
			)
			: [];
		const teksTp = new Map(tp.map(t => [t.id, t.deskripsi]));
		const nilaiOf = new Map(nilai.map(n => [`${n.pr}:${n.siswa}`, n]));
		const isianOf = new Map(isian.map(w => [w.peserta_didik_id, w]));

		// Urutan kelompok mengikuti Data Referensi → Mata Pelajaran; kelompok tak dikenal di akhir.
		const urutKelompok = (k: string) => {
			const i = kelompokUrut.findIndex(x => x.nama === k);
			return i < 0 ? 999 : i;
		};
		const namaKelompok = [...new Set(mapel.map(m => m.kelompok))].sort((a, b) => urutKelompok(a) - urutKelompok(b));
		const genap = semesterKe(semesterId) === 2;

		return {
			sekolah: {
				nama: sekolah?.nama ?? "",
				npsn: sekolah?.npsn ?? "",
				alamat: sekolah?.alamat ?? "",
				kecamatan: sekolah?.kecamatan ?? null,
				kabupaten: sekolah?.kabupaten_kota ?? null
			},
			kepsek: await penanda(sekolah?.kepsek_ptk_id),
			wali: await penanda(rombel.ptk_id),
			kelas: { nama: rombel.nama, tingkat: rombel.tingkat, fase: faseOf(rombel.tingkat) ?? "-" },
			semesterKe: semesterKe(semesterId),
			tahunAjaran: tahunAjaran(semesterId),
			gambar: await muatGambarRapor(db),
			ttdWali: rombel.ptk_id ? (await db.getSetting(kunciTtdGuru(rombel.ptk_id))) || undefined : undefined,
			siswa: siswa.map((s) => {
				const kurang: string[] = [];
				const kelompok = namaKelompok.map(k => ({
					nama: k,
					mapel: mapel.filter(m => m.kelompok === k).map((m) => {
						const n = nilaiOf.get(`${m.id}:${s.id}`);
						const teks = (json?: string) => (JSON.parse(json || "[]") as number[]).map(id => teksTp.get(id)).filter((t): t is string => !!t);
						const d = deskripsiCapaian(teks(n?.tp_optimal), teks(n?.tp_perlu));
						const capaian = [d.capai, d.perlu].filter(Boolean).join(" ");
						if (n?.nilai === null || n?.nilai === undefined)
							kurang.push(`nilai ${m.nama}`);
						else if (!deskripsiSah(JSON.parse(n.tp_optimal || "[]"), JSON.parse(n.tp_perlu || "[]")))
							kurang.push(`deskripsi ${m.nama} (perlu min. 1 ✓ & 1 !)`);
						return { kode: m.kode, nama: m.nama, singkat: m.singkat, nilai: n?.nilai ?? null, capaian };
					})
				}));
				const ikut = ekskul.filter(x => x.siswa === s.id);
				ikut.filter(x => !x.predikat).forEach(x => kurang.push(`nilai ekskul ${x.nama}`));
				const w = isianOf.get(s.id);
				const kokurikuler = w?.kokurikuler?.trim()
					|| deskripsiKokurikuler(s.nama, kegiatan.map(k => ({ kegiatan: k.nama, capaian: kokuOf.get(`${k.id}:${s.id}`) ?? {} })));
				if (kegiatan.length && !kokurikuler)
					kurang.push("kokurikuler");
				if (!w || w.sakit === null || w.izin === null || w.alpa === null)
					kurang.push("kehadiran");
				if (!w?.catatan)
					kurang.push("catatan wali kelas");
				if (genap && (w?.naik === null || w?.naik === undefined))
					kurang.push("keputusan kenaikan");
				return {
					...s,
					kelompok,
					// Yang belum dinilai tidak dicetak (seperti e-Rapor).
					ekskul: ikut.filter(x => x.predikat).map(x => ({ nama: x.nama, predikat: x.predikat!, keterangan: x.keterangan?.trim() || keteranganEkskul(x.predikat, x.nama) })),
					kokurikuler,
					sakit: w?.sakit ?? null,
					izin: w?.izin ?? null,
					alpa: w?.alpa ?? null,
					catatan: w?.catatan ?? "",
					naik: w?.naik ?? null,
					kurang
				};
			})
		};
	}

	// Data Pelengkap Rapor (sampul, identitas sekolah, identitas murid). Data koreksi admin
	// (siswa_rapor) didahulukan; sisanya dari data Dapodik mentah.
	async function muatPelengkap(rombelId: string): Promise<PelengkapKelas> {
		const sekolah = await db.first<{ raw: string | null, nama: string, npsn: string, kepsek_ptk_id: string | null }>(
			"SELECT raw, nama, npsn, kepsek_ptk_id FROM sekolah LIMIT 1"
		);
		const sr = sekolah?.raw ? JSON.parse(sekolah.raw) : {};
		const rows = await db.query<{ id: string, nama: string, nisn: string | null, nipd: string | null, jenis_kelamin: string | null, tempat_lahir: string | null,
			tanggal_lahir: string | null, agama: string | null, nama_ayah: string | null, nama_ibu: string | null, raw: string | null, foto: string | null }>(
			`SELECT s.peserta_didik_id AS id, s.nama, s.nisn, s.nipd, s.jenis_kelamin, s.tempat_lahir, s.tanggal_lahir, s.agama, s.nama_ayah, s.nama_ibu, p.raw, f.foto
			FROM anggota_rombel a JOIN siswa_rapor s USING (peserta_didik_id) JOIN peserta_didik p USING (peserta_didik_id)
			LEFT JOIN foto_siswa f ON f.peserta_didik_id = s.peserta_didik_id
			WHERE a.rombongan_belajar_id = ? ORDER BY s.nama COLLATE NOCASE`,
			[rombelId]
		);
		const t = (v: unknown) => (v === null || v === undefined ? "" : String(v).trim());
		const alamatSekolah = [t(sr.alamat_jalan), sr.rt || sr.rw ? `RT ${t(sr.rt)}/RW ${t(sr.rw)}` : "", t(sr.dusun)].filter(Boolean).join(", ");
		return {
			sekolah: {
				nama: sekolah?.nama ?? "",
				npsn: sekolah?.npsn ?? "",
				nss: t(sr.nss),
				alamat: alamatSekolah,
				kodePos: t(sr.kode_pos),
				telepon: t(sr.nomor_telepon),
				desa: t(sr.desa_kelurahan),
				kecamatan: t(sr.kecamatan),
				kabupaten: t(sr.kabupaten_kota),
				provinsi: t(sr.provinsi),
				website: t(sr.website),
				email: t(sr.email)
			},
			kepsek: await penanda(sekolah?.kepsek_ptk_id),
			gambar: await muatGambarRapor(db),
			siswa: rows.map((s) => {
				const r = s.raw ? JSON.parse(s.raw) : {};
				return {
					id: s.id,
					nama: s.nama,
					nisn: s.nisn,
					nipd: s.nipd,
					jenisKelamin: s.jenis_kelamin === "L" ? "Laki-laki" : s.jenis_kelamin === "P" ? "Perempuan" : t(s.jenis_kelamin),
					tempatLahir: t(s.tempat_lahir),
					tanggalLahir: t(s.tanggal_lahir),
					agama: t(s.agama) || t(r.agama_id_str),
					anakKe: t(r.anak_keberapa),
					sekolahAsal: t(r.sekolah_asal),
					tanggalMasuk: t(r.tanggal_masuk_sekolah),
					alamat: t(r.alamat_jalan),
					telepon: t(r.nomor_telepon_seluler) || t(r.nomor_telepon_rumah),
					ayah: t(s.nama_ayah),
					ibu: t(s.nama_ibu),
					kerjaAyah: t(r.pekerjaan_ayah_id_str),
					kerjaIbu: t(r.pekerjaan_ibu_id_str),
					wali: t(r.nama_wali),
					kerjaWali: t(r.nama_wali) ? t(r.pekerjaan_wali_id_str) : "",
					foto: s.foto
				};
			})
		};
	}

	return { muat, muatPelengkap };
}

export interface PelengkapSiswa {
	id: string
	nama: string
	nisn: string | null
	nipd: string | null
	jenisKelamin: string
	tempatLahir: string
	tanggalLahir: string
	agama: string
	anakKe: string
	sekolahAsal: string
	tanggalMasuk: string
	alamat: string
	telepon: string
	ayah: string
	ibu: string
	kerjaAyah: string
	kerjaIbu: string
	wali: string
	kerjaWali: string
	foto: string | null
}
export interface PelengkapKelas {
	sekolah: { nama: string, npsn: string, nss: string, alamat: string, kodePos: string, telepon: string, desa: string, kecamatan: string, kabupaten: string, provinsi: string, website: string, email: string }
	kepsek: Penanda | null
	gambar: GambarRapor
	siswa: PelengkapSiswa[]
}

const ANGKA = ["nol", "satu", "dua", "tiga", "empat", "lima", "enam", "tujuh"];
export const angkaKata = (n: number) => ANGKA[n] ?? String(n);
