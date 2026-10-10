// Isian wali kelas per siswa (tabel rapor_siswa). Padanan e-Rapor: Input Kelengkapan →
// Kehadiran / Catatan Wali Kelas / Kenaikan Kelas, digabung jadi satu tabel.

export interface IsianWalas {
	pesertaDidikId: string
	nama: string
	nisn: string | null
	jenisKelamin: string | null
	sakit: number | null
	izin: number | null
	alpa: number | null
	catatan: string
	naik: number | null
}

export const HARI_MAX = 366;
export const CATATAN_MAX = 500;

// Kalimat siap pakai. {nama} diganti nama panggilan siswa (kata pertama nama).
export const TEMPLATE_CATATAN: { label: string, teks: string }[] = [
	{ label: "Prestasi baik", teks: "Selamat, {nama} menunjukkan prestasi yang baik. Pertahankan semangat belajar dan terus tingkatkan." },
	{ label: "Rajin & disiplin", teks: "{nama} rajin, disiplin, dan bertanggung jawab. Terus pertahankan sikap positif ini." },
	{ label: "Aktif di kelas", teks: "{nama} aktif dalam kegiatan pembelajaran dan mampu bekerja sama dengan teman. Teruslah berkembang." },
	{ label: "Perlu fokus", teks: "{nama} perlu lebih fokus saat pembelajaran dan rajin mengulang pelajaran di rumah." },
	{ label: "Perlu kehadiran", teks: "{nama} perlu meningkatkan kehadiran di sekolah agar tidak tertinggal pelajaran." },
	{ label: "Perlu bimbingan", teks: "{nama} perlu bimbingan lebih dalam membaca dan berhitung. Dukungan orang tua di rumah sangat diharapkan." },
	{ label: "Percaya diri", teks: "{nama} perlu lebih percaya diri untuk bertanya dan menyampaikan pendapat di kelas." }
];

const kapital = (s: string) => s.charAt(0).toUpperCase() + s.slice(1).toLowerCase();
export const panggilan = (nama: string) => kapital(nama.trim().split(/\s+/)[0] ?? "");
export const isiTemplate = (teks: string, nama: string) => teks.replaceAll("{nama}", panggilan(nama));

// Keputusan kenaikan hanya di semester genap. Kelas 6 = kelulusan.
export function pilihanNaik(tingkat: number) {
	const akhir = tingkat >= 6;
	return [
		{ value: 1, label: akhir ? "Lulus" : `Naik ke kelas ${tingkat + 1}` },
		{ value: 0, label: akhir ? "Tidak lulus" : `Tinggal di kelas ${tingkat}` }
	];
}
