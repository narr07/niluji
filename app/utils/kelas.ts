// Kelas (tingkat) dan rombel disimpan terpisah: kelas "4" dipakai bank soal dan ujian, rombel
// "IVA" mengelompokkan siswa. Label gabungannya dipakai di tabel dan kartu peserta.
export const labelKelas = (kelas: string | null | undefined, rombel?: string | null) => {
	const k = kelas?.trim();
	const r = rombel?.trim();
	if (k && r) return `${k} · ${r}`;
	return k || r || "-";
};

// Halaman /siswa/lainnya menampung siswa yang kelasnya kosong atau tidak ada di Data Kelas.
export const KELAS_LAINNYA = "lainnya";
