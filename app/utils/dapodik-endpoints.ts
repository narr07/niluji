// Endpoint Web Service Dapodik yang sudah dicek langsung ke Dapodik (NPSN 20246133, semester 20261).
// Semuanya GET <base>/WebService/<nama>?npsn=<npsn>[&semester_id=<id>], header Authorization: Bearer <token>.
// Yang TIDAK ada (balas halaman HTML 404): getSemester, getTahunAjaran, getEkstrakurikuler,
// getPembelajaran, getAnggotaRombel — anggota & pembelajaran ikut nested di getRombonganBelajar,
// ekskul juga ada di sana sebagai rombel dengan jenis_rombel 51.
export interface DapodikEndpoint {
	key: string
	path: string
	label: string
	description: string
}

export const DAPODIK_ENDPOINTS: DapodikEndpoint[] = [
	{ key: "sekolah", path: "WebService/getSekolah", label: "Sekolah", description: "Profil sekolah (rows berupa objek tunggal, bukan array)" },
	{ key: "gtk", path: "WebService/getGtk", label: "GTK / Guru", description: "Nama, NIP, NUPTK, jabatan, riwayat pendidikan & pangkat" },
	{ key: "rombel", path: "WebService/getRombonganBelajar", label: "Rombel", description: "Kelas + ekskul (jenis_rombel 51), dengan anggota_rombel & pembelajaran nested" },
	{ key: "pd", path: "WebService/getPesertaDidik", label: "Peserta Didik", description: "NISN, NIK, TTL, orang tua, rombel aktif" },
	{ key: "pengguna", path: "WebService/getPengguna", label: "Pengguna", description: "Akun Dapodik (password berupa hash, disamarkan)" },
	{ key: "mapel", path: "WebService/getMataPelajaran", label: "Mata Pelajaran", description: "Referensi seluruh mapel nasional — besar (±5.500 baris, ±1,6 MB)" }
];
