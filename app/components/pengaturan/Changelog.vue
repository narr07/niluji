<script lang="ts" setup>
	interface ChangelogEntry {
		title: string
		date: string
		badge: string
		perubahan: string[]
	}

	const riwayat: ChangelogEntry[] = [
		{
			title: "Rilis Awal Niluji",
			date: "2026-09-22",
			badge: "v1.0.0",
			perubahan: [
				"Rebrand dari Nuxtor ke Niluji, termasuk ikon aplikasi.",
				"Pemilih tema warna (primary & neutral) dan mode terang/gelap dipindah ke satu menu Pengaturan di bawah sidebar.",
				"Halaman Panduan langkah-demi-langkah untuk guru, plus halaman Tentang dengan tab About/Fitur/Changelog.",
				"Import soal dari naskah Word (.docx) + kunci jawaban terpisah, dengan preview yang bisa diedit per baris dan baris manual tambahan.",
				"Editor teks soal mendukung format tebal/miring/garis bawah/coret, tampil konsisten di form soal, tabel Bank Soal, halaman ujian siswa, dan halaman penilaian.",
				"Tarik Data Siswa dari Google Sheets, dengan pencarian nama sekolah dan tautan langsung untuk membetulkan data sumber.",
				"Jenis Ujian bisa dibatasi cakupannya (kelas ini saja / semua kelas / mapel ini saja / semua) saat dibuat, dan dihapus dengan pilihan cakupan yang sama supaya tidak ikut terhapus di kelas/mapel lain.",
				"Export Bank Soal ke file Markdown + folder gambar, dibatasi PIN, untuk diunggah ke repo GitHub.",
				"Tarik Soal Online diperbarui membaca format Markdown hasil Export (bukan lagi Excel), dengan pemilihan Jenis Ujian sebelum menarik.",
				"Kelola Ujian dirombak: form pembuatan ujian jadi modal (bukan selalu terbuka), tabel bisa di-expand per baris, tanggal ujian otomatis terisi hari ini, judul ujian otomatis dari jenis soal.",
				"Hasil Ujian sekarang berjenjang sampai ke Jenis Ujian (Kelas > Mata Pelajaran > Jenis Ujian), jadi hasil Ulangan Harian dan STS (misalnya) tidak tercampur.",
				"Export Hasil Ujian ke file Excel (.xlsx), dengan pilihan Semua / per Siswa / Bulanan.",
				"Tombol \"Lihat Hasil\" langsung dari Kelola Ujian ke halaman Hasil Ujian yang sesuai.",
				"Kartu-kartu kelas/pelajaran/jenis dirapikan jadi lebih lebar dan konsisten di seluruh aplikasi.",
				"Halaman login CBT siswa dibuat lebih kontras (warna, ukuran teks, placeholder) supaya lebih gampang dibaca.",
				"Menu E-Rapor ditambahkan (belum aktif, masih \"Coming Soon\")."
			]
		}
	];
</script>

<template>
	<UChangelogVersions>
		<UChangelogVersion
			v-for="entry in riwayat"
			:key="entry.title"
			:title="entry.title"
			:date="entry.date"
			:badge="entry.badge">
			<template #body>
				<ul class="text-sm space-y-2 list-disc list-inside marker:text-muted">
					<li v-for="(p, i) in entry.perubahan" :key="i">
						{{ p }}
					</li>
				</ul>
			</template>
		</UChangelogVersion>
	</UChangelogVersions>
</template>
