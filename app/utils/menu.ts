import type { NavigationMenuItem } from "@nuxt/ui";

// Menu ala niluji: sedikit item di atas + kategori yang bisa dilipat. Tiap item mencatat
// padanannya di e-Rapor SD 2025.1 (`lama`) supaya guru yang pindah tetap bisa menemukan
// fiturnya — ditampilkan di halaman "sedang disiapkan" dan di dokumen analisis.

export interface MenuItem extends NavigationMenuItem {
	lama?: string
	description?: string
	children?: MenuItem[]
}

const item = (label: string, icon: string, to: string, lama?: string, description?: string): MenuItem =>
	({ label, icon, to, lama, description });

export function adminMenu(hasData: boolean): MenuItem[] {
	const top: MenuItem[] = [
		item("Dashboard", "lucide:layout-dashboard", "/e-rapor"),
		item("Sinkron Dapodik", "lucide:refresh-cw", "/e-rapor/dapodik", "Web Service Dapodik + Ambil Data Dapodik"),
		item("Sesi Online", "lucide:globe", "/e-rapor/sesi-online", "(baru) e-Rapor harus selalu online", "Buka akses sementara supaya guru bisa tarik data dan kirim nilai dari rumah")
	];
	if (!hasData)
		return top;
	return [
		...top,
		{
			label: "Data Referensi",
			icon: "lucide:database",
			type: "trigger",
			children: [
				item("Sekolah", "lucide:school", "/e-rapor/referensi/sekolah", "Data Referensi → Data Sekolah"),
				item("Guru", "lucide:user-round", "/e-rapor/referensi/guru", "Data Referensi → Data Guru"),
				item("Siswa", "lucide:users", "/e-rapor/referensi/siswa", "Data Referensi → Data Siswa"),
				item("Kelas", "lucide:layers", "/e-rapor/referensi/kelas", "Data Referensi → Data Kelas"),
				item("Pembelajaran", "lucide:book-open", "/e-rapor/referensi/pembelajaran", "Data Referensi → Data Pembelajaran"),
				item("Mata Pelajaran", "lucide:library", "/e-rapor/referensi/mapel", "Data Referensi → Data Mata Pelajaran"),
				item("Ekstrakurikuler", "lucide:trophy", "/e-rapor/referensi/ekskul", "Data Referensi → Data Ekstrakurikuler"),
				item("Pengguna", "lucide:key-round", "/e-rapor/pengguna", "Data Pengguna")
			]
		},
		{
			label: "Penilaian",
			icon: "lucide:graduation-cap",
			type: "trigger",
			children: [
				item("Tujuan Pembelajaran", "lucide:list-plus", "/e-rapor/tujuan-pembelajaran", "Tujuan Pembelajaran (di e-Rapor hanya guru)", "Isi TP semua mapel dan kelas, termasuk import satu file Excel"),
				item("Nilai Ekstrakurikuler", "lucide:trophy", "/e-rapor/guru/ekskul", "Input Nilai Ekstrakurikuler (di e-Rapor hanya pembina)", "Isi predikat ekskul semua siswa, termasuk kalau pembina berhalangan"),
				item("Status Penilaian", "lucide:list-checks", "/e-rapor/status-penilaian", "Status Penilaian → Status Penilaian / Statistik", "Pantau mapel dan kelas mana yang nilainya belum masuk"),
				item("Perkembangan Nilai", "lucide:trending-up", "/e-rapor/perkembangan-nilai", "Perkembangan Nilai", "Riwayat dan grafik nilai rapor siswa antar semester"),
				item("Kegiatan Kokurikuler", "lucide:notebook", "/e-rapor/kokurikuler", "Data Kokurikuler → Tema / Kegiatan / Kelompok", "Kegiatan, tema, dan dimensi profil lulusan; kelompok = kelas, koordinator = wali kelas"),
				item("Nilai Kokurikuler", "lucide:notebook-pen", "/e-rapor/walas/kokurikuler", "Nilai Kokurikuler + Deskripsi Kokurikuler (di e-Rapor koordinator)", "Capaian dimensi dan deskripsi kokurikuler semua kelas")
			]
		},
		{
			label: "Rapor",
			icon: "lucide:file-badge",
			type: "trigger",
			children: [
				item("Kelengkapan Rapor", "lucide:clipboard-check", "/e-rapor/walas", "Input Kelengkapan → Kehadiran / Catatan / Kenaikan", "Kehadiran, catatan wali kelas, dan kenaikan semua kelas"),
				item("Cetak Rapor", "lucide:printer", "/e-rapor/cetak", "Cetak Nilai → Leger / Pelengkap / Nilai Rapor", "Leger, pelengkap, dan rapor per kelas sekaligus"),
				item("Transkrip Ijazah", "lucide:scroll-text", "/e-rapor/transkrip", "Transkrip Ijazah → Nomor Ijazah / Setting / Mapping / Input / Import / Cetak", "Nomor ijazah, nilai transkrip kelas 6, dan cetak dalam satu halaman"),
				item("Kirim ke Dapodik", "lucide:upload", "/e-rapor/kirim-nilai", "Kirim Nilai Ke Dapodik", "Kirim matev dan nilai rapor dalam satu langkah")
			]
		}
	];
}

export function guruMenu(isWali: boolean, guruMode = false): MenuItem[] {
	const items: MenuItem[] = [
		item("Dashboard", "lucide:layout-dashboard", "/e-rapor"),
		// Laptop guru: tarik data & kirim nilai ke laptop admin.
		...(guruMode ? [item("Sinkron ke Admin", "lucide:arrow-left-right", "/e-rapor/sinkron")] : []),
		{
			label: "Penilaian",
			icon: "lucide:square-pen",
			type: "trigger",
			children: [
				item("Tujuan Pembelajaran", "lucide:list-plus", "/e-rapor/tujuan-pembelajaran", "Tujuan Pembelajaran", "Bank TP per mapel dan fase, bisa dipakai ulang tiap tahun"),
				item("Nilai Rapor", "lucide:table", "/e-rapor/guru/nilai", "Input Nilai Rapor + Import + Nilai Tersimpan + Deskripsi Tersimpan", "Input, import, dan cek nilai beserta deskripsinya di satu tabel per kelas"),
				item("Ekstrakurikuler", "lucide:trophy", "/e-rapor/guru/ekskul", "Input Nilai Ekstrakurikuler (pembina)"),
				item("Perkembangan Nilai", "lucide:trending-up", "/e-rapor/guru/perkembangan", "Cek Capaian Nilai Rapor Siswa + Grafik Perkembangan", "Riwayat nilai dan deskripsi siswa antar semester")
			]
		}
	];
	if (!isWali)
		return items;
	return [
		...items,
		{
			label: "Wali Kelas",
			icon: "lucide:users",
			type: "trigger",
			children: [
				item("Data Siswa", "lucide:users", "/e-rapor/walas/siswa", "Input Kelengkapan → Data Siswa", "Betulkan data siswa kelas Anda untuk rapor (Dapodik tidak berubah)"),
				item("Kelas Saya", "lucide:clipboard-check", "/e-rapor/walas", "Input Kelengkapan → Kehadiran / Catatan / Kenaikan", "Kehadiran, catatan wali kelas, dan kenaikan kelas dalam satu tabel"),
				item("Kokurikuler", "lucide:notebook-pen", "/e-rapor/walas/kokurikuler", "Nilai Kokurikuler + Deskripsi (di e-Rapor koordinator projek)", "Capaian dimensi profil lulusan dan deskripsi kokurikuler kelas Anda"),
				item("Status Penilaian", "lucide:list-checks", "/e-rapor/walas/status", "Cek Penilaian Kelas"),
				item("Cetak Rapor", "lucide:printer", "/e-rapor/walas/cetak", "Cetak Nilai → Leger / Pelengkap / Nilai Rapor"),
				item("Transkrip Ijazah", "lucide:scroll-text", "/e-rapor/walas/transkrip", "Input dan Cetak Transkrip Nilai Ijazah (kelas 6)")
			]
		}
	];
}

export function siswaMenu(): MenuItem[] {
	return [
		item("Dashboard", "lucide:layout-dashboard", "/e-rapor"),
		item("Nilai Rapor", "lucide:graduation-cap", "/e-rapor/siswa/nilai-rapor", "Rekap Capaian → Nilai Rapor"),
		item("Download Rapor", "lucide:download", "/e-rapor/siswa/download-rapor", "Download Rapor")
	];
}

// Menu di footer sidebar (seperti tombol Pengaturan niluji).
export function settingsMenu(level: UserLevel | undefined): MenuItem[] {
	if (level !== "admin")
		return [];
	return [
		{
			label: "Pengaturan Rapor",
			icon: "lucide:settings",
			children: [
				item("Logo & Tanda Tangan", "lucide:image", "/e-rapor/pengaturan/logo-ttd", "Data Referensi → Data Logo dan TTD"),
				item("Foto Siswa", "lucide:camera", "/e-rapor/pengaturan/foto-siswa", "Data Referensi → Foto Siswa")
			]
		},
		item("Backup & Restore", "lucide:hard-drive", "/e-rapor/backup", "Backup & Restore")
	];
}

// Info menu untuk path tertentu (dipakai halaman "sedang disiapkan").
export function findMenuItem(path: string): (MenuItem & { parent?: string }) | undefined {
	const all = [...adminMenu(true), ...guruMenu(true), ...siswaMenu(), ...settingsMenu("admin")];
	for (const m of all) {
		if (m.to === path)
			return m;
		for (const c of m.children ?? []) {
			if (c.to === path)
				return { ...c, parent: String(m.label) };
		}
	}
	return undefined;
}
