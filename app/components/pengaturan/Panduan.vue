<script lang="ts" setup>
	import type { AccordionItem } from "@nuxt/ui";

	interface PanduanLangkah {
		nomor: number;
		judul: string;
		menu: string;
		isi: string[];
	}

	const langkah: PanduanLangkah[] = [
		{
			nomor: 1,
			judul: "Kenalan dulu — ubah nama sekolah",
			menu: "Pengaturan > Data",
			isi: [
				"Begitu buka aplikasi pertama kali, ganti dulu nama sekolah supaya nggak nyangkut nama default. Klik Pengaturan di sidebar kiri, masuk ke Data, isi nama sekolah dan info lain yang diminta, lalu Simpan.",
				"Kenapa ini duluan? Karena nama sekolah ini yang bakal muncul di halaman login ujian siswa — jadi anak-anak tahu mereka buka aplikasi yang benar.",
			],
		},
		{
			nomor: 2,
			judul: "Siapin data kelas dan mata pelajaran",
			menu: "Pengaturan > Data Kelas & Data Pelajaran",
			isi: [
				"Sebelum masukin siswa atau bikin soal, aplikasi perlu tahu kelas apa saja yang ada (misalnya Kelas 4, 5, 6) dan mata pelajaran apa saja yang diajarkan (Matematika, IPAS, Bahasa Indonesia, dst).",
				"Di Data Kelas, klik Tambah dan ketik nama kelasnya. Di Data Pelajaran, klik Tambah, isi kode singkat (contoh: MTK untuk Matematika) dan nama lengkapnya. Kode ini dipakai di banyak tempat lain, jadi bikin yang gampang diingat.",
				"Sudah ada beberapa kelas/pelajaran bawaan? Lewati saja langkah ini dan tinggal sesuaikan kalau ada yang kurang.",
			],
		},
		{
			nomor: 3,
			judul: "Masukin data siswa",
			menu: "Siswa",
			isi: [
				"Ada dua cara, pilih yang paling cocok. Satu-satu: klik Tambah Siswa, isi NISN, nama, dan kelasnya — cocok kalau jumlah siswa sedikit atau cuma nambah 1-2 anak baru.",
				"Sekaligus banyak (impor): klik tombol impor, unduh dulu template CSV yang disediakan, isi kolomnya (NISN, nama, kelas) di Excel/Spreadsheet, simpan lagi sebagai CSV, lalu unggah filenya. Jauh lebih cepat kalau siswanya satu kelas penuh atau lebih.",
				"Nama yang diketik siswa saat login nanti nggak perlu persis sama besar-kecil hurufnya (BUDI, Budi, budi — semua dianggap sama), jadi nggak perlu khawatir soal itu.",
			],
		},
		{
			nomor: 4,
			judul: "Isi Bank Soal",
			menu: "Bank Soal",
			isi: [
				"Ini bagian yang paling makan waktu, tapi begitu selesai bisa dipakai berkali-kali untuk ujian berikutnya. Pilih kelas, lalu pilih mata pelajaran.",
				"Klik Jenis Ujian untuk bikin paket soal baru (contoh: \"UTS Semester 1\", \"Latihan Harian\"). Satu jenis = satu kumpulan soal yang nanti dipasangkan ke satu ujian.",
				"Buka jenis yang baru dibuat, lalu isi soalnya — tambah manual satu-satu, impor dari file Excel/CSV, atau Tarik Soal Online kalau sekolah sudah punya bank soal online.",
				"Soal pilihan ganda dan esai bisa dicampur dalam satu jenis yang sama — nanti siswa otomatis mengerjakan pilihan ganda dulu, baru lanjut ke esai.",
			],
		},
		{
			nomor: 5,
			judul: "Buat ujiannya",
			menu: "Kelola Ujian",
			isi: [
				"Klik Buat Ujian. Pilih kelas, mata pelajaran, dan jenis soal yang mau dipakai (yang sudah diisi di langkah 4).",
				"Tentukan jendela waktu ujian — kapan ujian mulai bisa diakses dan kapan ditutup. Tentukan juga durasi pengerjaan — berapa lama tiap siswa punya waktu SETELAH dia mulai (terpisah dari jendela di atas, jadi siswa yang mulai belakangan tetap dapat durasi penuh).",
				"Isi token ujian — kode ini yang nanti diketik siswa untuk masuk. Bikin yang gampang diingat tapi nggak gampang ditebak orang lain.",
				"Yang perlu dikasih tahu ke siswa cuma dua hal: alamat website ujian (bisa dilihat/dibagikan lewat kode QR di Dashboard) dan token yang barusan dibuat.",
			],
		},
		{
			nomor: 6,
			judul: "Siswa mengerjakan ujian",
			menu: "Otomatis — tinggal pantau",
			isi: [
				"Siswa buka alamat ujian di HP/laptop mereka (masih dalam WiFi/hotspot yang sama dengan laptop guru, nggak perlu internet), login pakai NISN dan nama, masukkan token, lalu mulai mengerjakan.",
				"Mau lihat siapa saja yang sedang online? Buka menu Siswa — di pojok kanan atas ada badge \"X online\" plus tabelnya, update sendiri tiap 3 detik.",
				"Mau lihat progres pengerjaan soal secara live? Buka Hasil Ujian, pilih Kelas, Mata Pelajaran, Jenis Ujian, lalu buka tab Daftar Siswa — halaman ini juga update sendiri tiap 3 detik.",
			],
		},
		{
			nomor: 7,
			judul: "Lihat dan nilai hasilnya",
			menu: "Hasil Ujian",
			isi: [
				"Pilih kelas, mata pelajaran, lalu jenis ujiannya. Tab Daftar Siswa menampilkan progres dan nilai tiap siswa (nilai yang muncul di sini cuma dari soal pilihan ganda — otomatis dihitung sistem).",
				"Kalau ada soal esai, klik Detail di nama siswa untuk buka halaman penilaiannya. Baca jawaban esainya, isi nilai per soal, klik Simpan Nilai. Nilai esai sengaja dipisah dan nggak otomatis digabung ke nilai pilihan ganda.",
				"Tab Analisis Soal menunjukkan soal mana yang paling banyak dijawab salah dan siapa yang nilainya paling tinggi/rendah.",
				"Butuh rekapnya di luar aplikasi? Klik Export (Semua, per Siswa, atau Bulanan) untuk unduh nilai sebagai file Excel.",
			],
		},
	];

	const accordionItems = computed<AccordionItem[]>(() =>
		langkah.map((l) => ({
			label: l.judul,
			value: String(l.nomor),
			slot: "step" as const,
			step: l,
		}))
	);

	const faq = [
		{ tanya: "Ujian belum bisa dibuat?", jawab: "Cek lagi apakah mata pelajaran dan jenis soalnya sudah ada isi soalnya (langkah 4)." },
		{ tanya: "Siswa nggak bisa login?", jawab: "Cek NISN-nya sudah terdaftar di menu Siswa dan sudah masuk ke kelas yang benar." },
		{ tanya: "Token ditolak?", jawab: "Cek jendela waktu ujian — mungkin belum mulai atau sudah lewat." },
		{ tanya: "Nilai esai nggak muncul di rekap?", jawab: "Memang sengaja terpisah — buka Detail siswa untuk lihat/isi nilai esainya." },
	];
</script>

<template>
	<div class="space-y-6 w-full">
		<UAlert
			icon="i-lucide-map"
			color="primary"
			variant="subtle"
			title="Dari nol sampai lihat hasil ujian"
			description="Ikuti urutannya dari atas ke bawah — tiap langkah butuh langkah sebelumnya selesai dulu. Anggap saja seperti nyiapin kelas baru di awal semester." />

		<UCard :ui="{ body: 'p-0 sm:p-0' }">
			<UAccordion
				:items="accordionItems"
				type="multiple"
				:default-value="['1']"
				:ui="{ label: 'font-semibold' }"
			>
				<template #leading="{ item }">
					<span class="flex items-center justify-center size-7 rounded-full bg-primary text-inverted text-sm font-bold shrink-0">
						{{ (item as any).step.nomor }}
					</span>
				</template>

				<template #step="{ item }">
					<div class="space-y-2 text-sm pl-10">
						<UBadge
							color="neutral"
							variant="subtle"
							size="sm"
							class="mb-1">
							{{ (item as any).step.menu }}
						</UBadge>
						<p v-for="(p, i) in (item as any).step.isi" :key="i" class="text-toned leading-relaxed">
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
			title="Fitur tambahan"
		>
			<template #description>
				Import dari Word, Tarik Soal Online, Tarik Data Siswa, Export Bank Soal — lihat di
				<NuxtLink to="/tentang/fitur" class="text-primary hover:underline">Tentang > Fitur</NuxtLink>.
			</template>
		</UAlert>

		<UCard>
			<template #header>
				<span class="font-semibold">Kalau nyasar / lupa langkah</span>
			</template>
			<dl class="space-y-3 text-sm">
				<div v-for="item in faq" :key="item.tanya">
					<dt class="font-medium">{{ item.tanya }}</dt>
					<dd class="text-muted">{{ item.jawab }}</dd>
				</div>
			</dl>
		</UCard>
	</div>
</template>