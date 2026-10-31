<script lang="ts" setup>
	import type { AccordionItem } from "@nuxt/ui";

	interface PanduanLangkah {
		nomor: number
		judul: string
		menu: string
		isi: string[]
	}

	const langkah: PanduanLangkah[] = [
		{
			nomor: 1,
			judul: "Kenalan dulu: ubah nama sekolah",
			menu: "Pengaturan > Data",
			isi: [
				"Begitu buka aplikasi pertama kali, ganti dulu nama sekolah supaya tidak memakai nama bawaan. Klik Pengaturan di sidebar kiri, masuk ke Data, isi nama sekolah dan info lain yang diminta, lalu Simpan.",
				"Nama sekolah ini akan muncul di header aplikasi dan kartu peserta ujian siswa, sehingga siswa tahu mereka membuka aplikasi yang benar."
			]
		},
		{
			nomor: 2,
			judul: "Siapkan data kelas dan mata pelajaran",
			menu: "Pengaturan > Data Kelas & Data Pelajaran",
			isi: [
				"Sebelum memasukkan siswa atau membuat soal, aplikasi perlu tahu kelas apa saja yang ada (misalnya Kelas 4, 5, 6) dan mata pelajaran apa saja yang diajarkan (Matematika, IPAS, Bahasa Indonesia, dst).",
				"Di Data Kelas, klik Tambah dan ketik nama kelasnya. Di Data Pelajaran, klik Tambah, isi kode singkat (contoh: MTK untuk Matematika) dan nama lengkapnya. Kode ini dipakai di banyak tempat lain, jadi buat yang mudah diingat.",
				"Sudah ada beberapa kelas/pelajaran bawaan? Lewati saja langkah ini dan sesuaikan jika ada yang kurang."
			]
		},
		{
			nomor: 3,
			judul: "Masukkan data siswa & cetak kartu peserta",
			menu: "Siswa",
			isi: [
				"Ada dua cara, pilih yang paling cocok. Satu per satu: klik Tambah Siswa, isi NISN, nama, dan kelasnya. Cocok jika jumlah siswa sedikit atau hanya menambah 1-2 anak baru.",
				"Sekaligus banyak (impor): klik tombol Import, unduh template CSV/Excel yang disediakan, isi kolomnya (NISN, nama, kelas), lalu unggah filenya.",
				"Setiap siswa otomatis dibuatkan 6 digit PIN Kartu Peserta. Klik tombol Kartu Peserta untuk melihat atau mencetak kartu ujian lengkap dengan QR Code akses lokal."
			]
		},
		{
			nomor: 4,
			judul: "Isi Bank Soal",
			menu: "Bank Soal",
			isi: [
				"Pilih kelas, lalu pilih mata pelajaran, atau gunakan pencarian Akses Cepat Paket Soal di bagian bawah halaman Bank Soal.",
				"Klik Jenis Ujian untuk membuat paket soal baru (contoh: \"UTS Semester 1\", \"Latihan Harian\"). Satu jenis sama dengan satu kumpulan soal yang nantinya dipasangkan ke satu ujian.",
				"Buka jenis yang baru dibuat, lalu isi soalnya: tambah manual, impor dari Word (.docx), CSV/Excel, Markdown (.md), atau Tarik Soal Online.",
				"Soal pilihan ganda dan esai bisa dicampur dalam satu jenis yang sama. Siswa otomatis mengerjakan pilihan ganda dulu, baru lanjut ke sesi esai."
			]
		},
		{
			nomor: 5,
			judul: "Buat jadwal dan pengaturan ujian",
			menu: "Kelola Ujian",
			isi: [
				"Klik Buat Ujian Baru. Pilih kelas, mata pelajaran, dan jenis soal yang sudah disiapkan.",
				"Tentukan jendela waktu ujian (jam mulai dan selesai) serta durasi pengerjaan dalam menit.",
				"Atur opsi pengacakan soal PG, pengacakan opsi A-D, pengacakan esai, bobot nilai PG (%), serta apakah siswa wajib memasukkan PIN Kartu Peserta.",
				"Bagikan alamat ujian (lewat tombol QR Code di Dashboard) dan token ujian kepada siswa di ruang ujian."
			]
		},
		{
			nomor: 6,
			judul: "Pantau siswa saat mengerjakan ujian",
			menu: "Dashboard & Hasil Ujian",
			isi: [
				"Siswa membuka alamat ujian di HP atau laptop dalam jaringan WiFi/LAN yang sama tanpa internet, login memakai NISN dan nama, lalu memasukkan token ujian.",
				"Sesi ujian siswa otomatis dikunci pada perangkat pertama yang dipakai. Jika HP siswa mati dan harus pindah perangkat, pengawas dapat menekan tombol Buka Kunci Perangkat di tabel Hasil Ujian.",
				"Pengawas juga dapat memberikan tambahan waktu (+10 menit) atau memaksa selesai (Force Submit) langsung dari tabel Hasil Ujian."
			]
		},
		{
			nomor: 7,
			judul: "Koreksi esai, analisis pengecoh, dan ekspor nilai",
			menu: "Hasil Ujian",
			isi: [
				"Pilih kelas, mata pelajaran, lalu jenis ujiannya. Tab Daftar Siswa menampilkan Nilai PG, Nilai Esai, dan Nilai Akhir gabungan sesuai bobot yang diatur.",
				"Jika ada soal esai, klik ikon Detail pada baris siswa untuk menilai jawaban uraian per soal.",
				"Tab Analisis Soal menampilkan tingkat kesulitan, peringkat siswa, dan Analisis Pengecoh (Distractor Analysis) untuk memantau sebaran pemilih opsi A, B, C, dan D.",
				"Klik tombol Export untuk mengunduh rekap nilai lengkap dalam format Excel (.xlsx) atau bundel per siswa (.zip)."
			]
		}
	];

	const accordionItems = computed<AccordionItem[]>(() =>
		langkah.map((l) => ({
			label: l.judul,
			value: String(l.nomor),
			slot: "step" as const,
			step: l
		}))
	);

	const faq = [
		{ tanya: "Ujian belum bisa dibuat?", jawab: "Periksa kembali apakah mata pelajaran dan jenis soalnya sudah memiliki butir soal di Bank Soal (langkah 4)." },
		{ tanya: "Siswa tidak bisa login?", jawab: "Pastikan NISN dan nama siswa sudah terdaftar di menu Siswa pada kelas yang sesuai." },
		{ tanya: "Siswa pindah HP karena baterai habis?", jawab: "Buka menu Hasil Ujian pada paket ujian tersebut, lalu klik ikon Buka Kunci Perangkat pada baris nama siswa." },
		{ tanya: "Bagaimana cara menggabungkan nilai PG dan Esai?", jawab: "Atur persentase Bobot Nilai PG saat membuat/mengedit ujian. Begitu esai dinilai di halaman Detail Siswa, Nilai Akhir terhitung otomatis." }
	];
</script>

<template>
	<div class="space-y-6 w-full">
		<UAlert
			icon="i-lucide-map"
			color="primary"
			variant="subtle"
			title="Dari nol sampai lihat hasil ujian"
			description="Ikuti urutannya dari atas ke bawah. Setiap langkah menyiapkan data yang dibutuhkan oleh langkah berikutnya." />

		<UCard :ui="{ body: 'p-0 sm:p-0' }">
			<UAccordion
				:items="accordionItems"
				type="multiple"
				:default-value="['1']"
				:ui="{ label: 'font-semibold' }">
				<template #leading="{ item }">
					<span class="flex items-center justify-center size-7 rounded-full bg-primary text-inverted text-sm font-bold shrink-0">
						{{ (item as unknown as { step: PanduanLangkah }).step.nomor }}
					</span>
				</template>

				<template #step="{ item }">
					<div class="space-y-2 text-sm pl-10">
						<UBadge
							color="neutral"
							variant="subtle"
							size="sm"
							class="mb-1">
							{{ (item as unknown as { step: PanduanLangkah }).step.menu }}
						</UBadge>
						<p v-for="(p, i) in (item as unknown as { step: PanduanLangkah }).step.isi" :key="i" class="text-toned leading-relaxed">
							{{ p }}
						</p>
					</div>
				</template>
			</UAccordion>
		</UCard>

		<UAlert
			icon="i-lucide-layout-grid"
			color="neutral"
			variant="subtle"
			title="Fitur tambahan">
			<template #description>
				Import dari Word, Tarik Soal Online, Tarik Data Siswa, Export Bank Soal dapat dilihat di
				<NuxtLink to="/tentang/fitur" class="text-primary hover:underline">
					Tentang &gt; Fitur
				</NuxtLink>.
			</template>
		</UAlert>

		<UCard>
			<template #header>
				<span class="font-semibold">Tanya Jawab Singkat</span>
			</template>
			<dl class="space-y-3 text-sm">
				<div v-for="item in faq" :key="item.tanya">
					<dt class="font-medium">
						{{ item.tanya }}
					</dt>
					<dd class="text-muted">
						{{ item.jawab }}
					</dd>
				</div>
			</dl>
		</UCard>
	</div>
</template>
