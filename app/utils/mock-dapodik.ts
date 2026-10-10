// Data tiruan untuk mode simulasi (tanpa PC Dapodik). Bentuknya mengikuti pola response
// Web Service Dapodik: { results, id, start, limit, rows }. Ganti dengan sampel asli
// begitu sudah berhasil tarik data dari Dapodik sungguhan.
const NPSN = "20200000";
const SEMESTER = "20261";

function wrap(rows: unknown[]) {
	return { results: rows.length, id: NPSN, start: 0, limit: rows.length, rows };
}

const gtk = [
	{ ptk_id: "gtk-001", nama: "Siti Aminah, S.Pd.", nip: "198501012010012001", nuptk: "1234567890123456", jenis_kelamin: "P", jenis_ptk_id_str: "Guru Kelas", tahun_ajaran_id: "2025" },
	{ ptk_id: "gtk-002", nama: "Budi Santoso, S.Pd.", nip: null, nuptk: "2234567890123456", jenis_kelamin: "L", jenis_ptk_id_str: "Guru Mapel", tahun_ajaran_id: "2025" },
	{ ptk_id: "gtk-003", nama: "Dewi Lestari, S.Pd.SD", nip: "199002022019032002", nuptk: null, jenis_kelamin: "P", jenis_ptk_id_str: "Guru Kelas", tahun_ajaran_id: "2025" }
];

const pd = [
	{ peserta_didik_id: "pd-001", nama: "Ahmad Fauzi", nisn: "0151234561", nik: "3201010101150001", jenis_kelamin: "L", tempat_lahir: "Bandung", tanggal_lahir: "2015-01-01", nama_ayah: "Fauzan", nama_ibu: "Rina", rombongan_belajar_id: "rb-4a" },
	{ peserta_didik_id: "pd-002", nama: "Nabila Putri", nisn: "0151234562", nik: "3201010202150002", jenis_kelamin: "P", tempat_lahir: "Garut", tanggal_lahir: "2015-02-02", nama_ayah: "Hendra", nama_ibu: "Yuli", rombongan_belajar_id: "rb-4a" },
	{ peserta_didik_id: "pd-003", nama: "Rizky Pratama", nisn: "0131234563", nik: "3201010303130003", jenis_kelamin: "L", tempat_lahir: "Bandung", tanggal_lahir: "2013-03-03", nama_ayah: "Asep", nama_ibu: "Neneng", rombongan_belajar_id: "rb-6a" }
];

const rombel = [
	{
		rombongan_belajar_id: "rb-4a",
		nama: "Kelas 4A",
		tingkat_pendidikan_id: "4",
		jenis_rombel: "1",
		jenis_rombel_str: "Kelas",
		semester_id: SEMESTER,
		ptk_id: "gtk-001",
		ptk_id_str: "Siti Aminah, S.Pd.",
		kurikulum_id_str: "Kurikulum Merdeka",
		anggota_rombel: [
			{ anggota_rombel_id: "ar-001", peserta_didik_id: "pd-001", jenis_pendaftaran_id_str: "Siswa Baru" },
			{ anggota_rombel_id: "ar-002", peserta_didik_id: "pd-002", jenis_pendaftaran_id_str: "Siswa Baru" }
		],
		pembelajaran: [
			{ pembelajaran_id: "pb-001", mata_pelajaran_id: "100014000", mata_pelajaran_id_str: "Matematika", nama_mata_pelajaran: "Matematika", ptk_id: "gtk-001", jam_mengajar_per_minggu: 5 },
			{ pembelajaran_id: "pb-002", mata_pelajaran_id: "300210000", mata_pelajaran_id_str: "Pendidikan Jasmani", nama_mata_pelajaran: "PJOK", ptk_id: "gtk-002", jam_mengajar_per_minggu: 3 }
		]
	},
	{
		rombongan_belajar_id: "rb-6a",
		nama: "Kelas 6A",
		tingkat_pendidikan_id: "6",
		jenis_rombel: "1",
		jenis_rombel_str: "Kelas",
		semester_id: SEMESTER,
		ptk_id: "gtk-003",
		ptk_id_str: "Dewi Lestari, S.Pd.SD",
		kurikulum_id_str: "Kurikulum Merdeka",
		anggota_rombel: [
			{ anggota_rombel_id: "ar-003", peserta_didik_id: "pd-003", jenis_pendaftaran_id_str: "Siswa Baru" }
		],
		pembelajaran: [
			{ pembelajaran_id: "pb-003", mata_pelajaran_id: "100014000", mata_pelajaran_id_str: "Matematika", nama_mata_pelajaran: "Matematika", ptk_id: "gtk-003", jam_mengajar_per_minggu: 5 }
		]
	},
	{
		rombongan_belajar_id: "rb-pramuka",
		nama: "Pramuka",
		tingkat_pendidikan_id: "6",
		jenis_rombel: "51",
		jenis_rombel_str: "Ekstrakurikuler",
		semester_id: SEMESTER,
		ptk_id: "gtk-002",
		ptk_id_str: "Budi Santoso, S.Pd.",
		kurikulum_id_str: "Lainnya",
		anggota_rombel: [
			{ anggota_rombel_id: "ar-101", peserta_didik_id: "pd-001", jenis_pendaftaran_id_str: "Siswa Baru" },
			{ anggota_rombel_id: "ar-102", peserta_didik_id: "pd-003", jenis_pendaftaran_id_str: "Siswa Baru" }
		],
		pembelajaran: []
	}
];

export const MOCK_DAPODIK: Record<string, unknown> = {
	// Dapodik asli: rows getSekolah berupa objek tunggal, bukan array.
	"WebService/getSekolah": { results: 1, id: "sekolah_id", start: 0, limit: 20, rows: {
		sekolah_id: "sek-001",
		nama: "SD Negeri Contoh 1",
		npsn: NPSN,
		bentuk_pendidikan_id_str: "SD",
		status_sekolah_str: "Negeri",
		alamat_jalan: "Jl. Pendidikan No. 1",
		desa_kelurahan: "Sukamaju",
		kecamatan: "Sukasari",
		kabupaten_kota: "Kab. Contoh",
		provinsi: "Jawa Barat",
		kode_pos: "40000",
		email: "sdncontoh1@example.sch.id",
		is_sync_to_server: true
	} },
	"WebService/getGtk": wrap(gtk),
	"WebService/getPesertaDidik": wrap(pd),
	"WebService/getRombonganBelajar": wrap(rombel),
	"WebService/getPengguna": wrap([
		{ pengguna_id: "u-001", username: "operator@sdncontoh1", nama: "Operator Sekolah", peran_id_str: "Operator Sekolah", password: "$2y$10$contohhashbcrypt", ptk_id: null }
	]),
	"WebService/getMataPelajaran": wrap([
		{ mata_pelajaran_id: 100014000, nama: "Matematika", pilihan_sekolah: 1, pilihan_evaluasi: 1, jurusan_id: null },
		{ mata_pelajaran_id: 300210000, nama: "Pendidikan Jasmani, Olahraga, dan Kesehatan", pilihan_sekolah: 1, pilihan_evaluasi: 1, jurusan_id: null }
	])
};
