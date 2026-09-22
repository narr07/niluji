<script lang="ts" setup>
	interface ChangelogEntry {
		title: string;
		date: string;
		badge: string;
		badgeColor?: "primary" | "neutral" | "success";
		perubahan: string[];
	}

	const riwayat: ChangelogEntry[] = [
		{
			title: "Perbaikan & Fitur Tambahan",
			date: "2026-09-22",
			badge: "v1.0.1",
			badgeColor: "neutral",
			perubahan: [
				"Perbaikan bug kritis: soal yang sudah dihapus dari Bank Soal tapi masih dirujuk sesi ujian lama sebelumnya bisa bikin daftar Hasil Ujian, export, bahkan halaman ujian siswa gagal total — sekarang soal yang hilang dilewati saja, tidak lagi menggagalkan semuanya.",
				"Jawaban siswa sekarang otomatis dicoba simpan ulang kalau koneksi ke server sempat putus (mis. listrik padam sebentar), dengan peringatan di layar dan tombol Selesai dikunci sampai semua jawaban benar-benar tersimpan.",
				"Gambar soal sekarang bisa ditaruh di posisi mana pun di tengah teks pertanyaan (tombol \"Sisipkan Gambar\" di editor), tidak cuma selalu di bawah semua teks.",
				"Tiap pilihan jawaban (A-E) sekarang bisa punya gambar sendiri, cocok untuk soal yang jawabannya berupa gambar.",
				"Opsi acak soal saat membuat ujian dipisah jadi dua: acak untuk soal Pilihan Ganda dan acak untuk soal Esai, bisa diatur sendiri-sendiri.",
				"Urutan soal di laporan Hasil Ujian (detail per siswa) sekarang selalu mengikuti urutan input guru di Bank Soal, bukan urutan acak yang dilihat siswa saat ujian.",
				"Export Hasil Ujian dirombak: \"Export per Siswa\" sekarang berisi rincian jawaban per soal (bukan cuma skor akhir), ada mode borongan semua siswa sekaligus jadi satu file .zip, dan tambahan level export untuk satu kelas penuh (semua mata pelajaran & jenis ujian).",
				"Import Soal sekarang satu tombol dengan tiga pilihan format: Naskah Word, CSV/Excel, dan file Markdown lokal (baru) — plus contoh template Word & Markdown yang sudah diuji bisa langsung diimpor.",
				"Semua modal import (Word, CSV/Excel, Markdown, Import Siswa) sekarang mendukung seret-lepas (drag-and-drop) file, tidak cuma klik pilih file.",
				"Import Siswa sekarang menampilkan preview dulu (bisa diedit per baris) sebelum benar-benar disimpan, sama seperti alur Import dari Word.",
				"Backup database otomatis tiap aplikasi dibuka, plus tombol backup manual — 14 backup terakhir disimpan di folder data aplikasi.",
				"Semua tombol hapus di aplikasi sekarang pakai dialog konfirmasi yang seragam (bukan popup browser bawaan), dan Bank Soal dapat fitur pilih beberapa soal lalu hapus sekaligus.",
				"Validasi tambahan: soal pilihan ganda yang diimpor wajib minimal 2 opsi dan kunci jawaban yang cocok (baris bermasalah dilewati, bukan bikin import gagal total), jadwal ujian wajib durasi lebih dari 0 dan jam selesai setelah jam mulai, dan ukuran gambar dibatasi maksimal 10 MB.",
			],
		},
		{
			title: "Rilis Awal Niluji",
			date: "2026-09-22",
			badge: "v1.0.0",
			badgeColor: "primary",
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
				"Menu E-Rapor ditambahkan (belum aktif, masih \"Coming Soon\").",
			],
		},
	];
</script>

<template>
	<UChangelogVersions>
		<UChangelogVersion
			v-for="entry in riwayat"
			:key="entry.title"
			:title="entry.title"
			:date="entry.date"
			:badge="{ label: entry.badge, color: entry.badgeColor ?? 'primary', variant: 'subtle' }">
			<template #body>
				<ul class="text-sm space-y-2.5">
					<li
						v-for="(p, i) in entry.perubahan"
						:key="i"
						class="flex items-start gap-2.5">
						<UIcon name="i-lucide-circle-check" class="size-4 mt-0.5 shrink-0 text-primary" />
						<span class="text-toned leading-relaxed">{{ p }}</span>
					</li>
				</ul>
			</template>
		</UChangelogVersion>
	</UChangelogVersions>
</template>