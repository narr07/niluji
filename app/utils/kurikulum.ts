// Struktur mapel rapor SD Kurikulum Merdeka. kode = mata_pelajaran_id di referensi Dapodik
// (dicocokkan dengan getMataPelajaran), supaya nilainya bisa dikirim balik ke Dapodik.

export type Fase = "A" | "B" | "C";

// Satu daftar ini = sumber tunggal pengaturan mapel rapor (halaman Data Referensi → Mata Pelajaran).
// Urutan array = urutan mapel di rapor (dalam kelompoknya). Padanan e-Rapor: Data Mata Pelajaran
// + Kelompok Mapel + Mapping Rapor, tapi diatur sekali untuk semua tingkat (fase menentukan kelasnya).
export interface TemplateMapel {
	kode: string
	nama: string
	singkat: string
	kelompok: string
	fase: Fase[]
	aktif: boolean
	transkrip?: boolean // ikut transkrip nilai ijazah (kelas 6)
	lokal?: boolean // ditambahkan admin dari referensi Dapodik (boleh dihapus dari daftar)
}

export interface KelompokMapel {
	nama: string
	aktif: boolean
}

// Batas nama singkat sama dengan e-Rapor (dipakai sebagai judul kolom leger & grafik).
export const SINGKAT_MAX = 20;

export const DEFAULT_KELOMPOK: KelompokMapel[] = [
	{ nama: "Mata Pelajaran Wajib", aktif: true },
	{ nama: "Mata Pelajaran Pilihan", aktif: true },
	{ nama: "Muatan Lokal", aktif: true }
];

export const KODE_GURU_KELAS = "400200000";

export const DEFAULT_TEMPLATE: TemplateMapel[] = [
	{ kode: "100011070", nama: "Pendidikan Agama Islam dan Budi Pekerti", singkat: "PAI", kelompok: "Mata Pelajaran Wajib", fase: ["A", "B", "C"], aktif: true },
	{ kode: "200010300", nama: "Pendidikan Pancasila", singkat: "Pancasila", kelompok: "Mata Pelajaran Wajib", fase: ["A", "B", "C"], aktif: true },
	{ kode: "300110000", nama: "Bahasa Indonesia", singkat: "B. Indonesia", kelompok: "Mata Pelajaran Wajib", fase: ["A", "B", "C"], aktif: true },
	{ kode: "401000000", nama: "Matematika", singkat: "Matematika", kelompok: "Mata Pelajaran Wajib", fase: ["A", "B", "C"], aktif: true },
	{ kode: "401900000", nama: "Ilmu Pengetahuan Alam dan Sosial", singkat: "IPAS", kelompok: "Mata Pelajaran Wajib", fase: ["B", "C"], aktif: true },
	{ kode: "500010000", nama: "Pendidikan Jasmani, Olahraga, dan Kesehatan", singkat: "PJOK", kelompok: "Mata Pelajaran Wajib", fase: ["A", "B", "C"], aktif: true },
	{ kode: "700121000", nama: "Seni Rupa", singkat: "Seni Rupa", kelompok: "Mata Pelajaran Wajib", fase: ["A", "B", "C"], aktif: true },
	{ kode: "700109000", nama: "Seni Musik", singkat: "Seni Musik", kelompok: "Mata Pelajaran Wajib", fase: ["A", "B", "C"], aktif: false },
	{ kode: "700110000", nama: "Seni Tari", singkat: "Seni Tari", kelompok: "Mata Pelajaran Wajib", fase: ["A", "B", "C"], aktif: false },
	{ kode: "700122000", nama: "Seni Teater", singkat: "Seni Teater", kelompok: "Mata Pelajaran Wajib", fase: ["A", "B", "C"], aktif: false },
	{ kode: "300210000", nama: "Bahasa Inggris", singkat: "B. Inggris", kelompok: "Mata Pelajaran Pilihan", fase: ["A", "B", "C"], aktif: false },
	{ kode: "402001000", nama: "Koding dan Kecerdasan Artifisial", singkat: "KKA", kelompok: "Mata Pelajaran Pilihan", fase: ["C"], aktif: false },
	{ kode: "300311900", nama: "Bahasa Sunda", singkat: "B. Sunda", kelompok: "Muatan Lokal", fase: ["A", "B", "C"], aktif: true }
];

export function faseOf(tingkat: number | string | null | undefined): Fase | null {
	const t = Number(tingkat);
	if (t >= 1 && t <= 2)
		return "A";
	if (t >= 3 && t <= 4)
		return "B";
	if (t >= 5 && t <= 6)
		return "C";
	return null;
}

// 20251 → 1 (ganjil), 20252 → 2 (genap)
export const semesterKe = (semesterId?: string | null) => (semesterId?.endsWith("2") ? 2 : 1);

// Deskripsi capaian kompetensi dari centang TP, mengikuti pola e-Rapor SD:
// TP optimal → "Menunjukkan penguasaan yang baik dalam …", TP perlu pendampingan → "Perlu bantuan dalam …".
function joinTp(list: string[]) {
	const items = list.map(t => t.trim().replace(/\.$/, ""));
	if (items.length <= 1)
		return items[0] ?? "";
	return `${items.slice(0, -1).join(", ")} dan ${items.at(-1)}`;
}

// Aturan sekolah: deskripsi tiap siswa per mapel wajib memuat minimal 1 TP tercapai optimal (✓)
// DAN minimal 1 TP perlu bantuan (!). Hanya ✓ saja atau hanya ! saja belum boleh.
export const ATURAN_DESKRIPSI = "minimal 1 TP tercapai (✓) dan 1 TP perlu bantuan (!)";
export const deskripsiSah = (optimal: readonly unknown[], perlu: readonly unknown[]) => optimal.length >= 1 && perlu.length >= 1;

export function deskripsiCapaian(optimal: string[], perlu: string[]): { capai: string, perlu: string } {
	return {
		capai: optimal.length ? `Menunjukkan penguasaan yang baik dalam ${joinTp(optimal)}.` : "",
		perlu: perlu.length ? `Perlu bantuan dalam ${joinTp(perlu)}.` : ""
	};
}
