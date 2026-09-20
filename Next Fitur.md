<!-- 1. catat di log progres kita,
2. push folder nuxtor-main ini ke repo saya https://github.com/narr07/niluji.git
3. dan saya ingin ada fitur ini 
Buat fitur "Tarik Soal Online" untuk aplikasi CBT Nuxt + Tauri (project "nuxtor"). Fokus HANYA pada proses tarik & parsing data dari GitHub — TIDAK termasuk insert ke database.

SUMBER DATA
Repo GitHub publik "USERNAME/niluji" (ini repo utama aplikasi), bank soal ada di dalam folder "db-soal" di repo tersebut, diakses via jsDelivr CDN. Struktur folder di dalamnya: per kelas lalu per mata pelajaran:
  db-soal/{kelas}/{mata_pelajaran}/soal.xlsx
  db-soal/{kelas}/{mata_pelajaran}/{nama-file-gambar}

Contoh:
  https://cdn.jsdelivr.net/gh/USERNAME/niluji/db-soal/kelas_4/matematika/soal.xlsx
  https://cdn.jsdelivr.net/gh/USERNAME/niluji/db-soal/kelas_4/matematika/gambar1.jpg

Slug {kelas} format: kelas_4, kelas_5, kelas_6.
Slug {mata_pelajaran} format: lowercase, spasi jadi tanda hubung (mis. "bahasa-indonesia", "ipas").

Kolom Excel (soal.xlsx, baris pertama = header):
  soal, jenis, pilihan_a, pilihan_b, pilihan_c, pilihan_d, kunci_jawaban, skor, nama_file_gambar
(mata_pelajaran dan kelas TIDAK perlu jadi kolom karena sudah ditentukan dari folder yang dipilih. nama_file_gambar boleh kosong kalau soal tidak ada gambar. Satu nama_file_gambar boleh dipakai berulang oleh beberapa baris soal dalam folder yang sama — ini normal, bukan error.)

SETUP
- bun add xlsx @tauri-apps/plugin-http
- daftarkan plugin http di src-tauri/src/lib.rs (.plugin(tauri_plugin_http::init()))
- tambahkan izin di src-tauri/capabilities/default.json untuk domain https://cdn.jsdelivr.net/*

YANG PERLU DIBUAT

1. composables/useBankSoalOnline.ts
   - Konstanta CDN_BASE = "https://cdn.jsdelivr.net/gh/USERNAME/niluji/db-soal"
   - Daftar konstan pilihan kelas: ['kelas_4', 'kelas_5', 'kelas_6'] dan daftar mata pelajaran (pakai slug yang konsisten dengan menu pelajaran yang sudah ada di app / tabel subjects).
   - fungsi buildFolderPath(kelas: string, mataPelajaran: string): string -> hasilkan "{kelas}/{mataPelajaran}".
   - fetchBankSoal(kelas: string, mataPelajaran: string): fetch "{CDN_BASE}/{folderPath}/soal.xlsx" via @tauri-apps/plugin-http, parse dengan library "xlsx" (XLSX.read lalu sheet_to_json), return array of objects sesuai kolom di atas.
   - Untuk tiap baris hasil parsing yang nama_file_gambar terisi, sertakan field urlGambar = "{CDN_BASE}/{folderPath}/{nama_file_gambar}" DAN field pathRelatifGambar = "{folderPath}/{nama_file_gambar}" (path ini yang nanti dipakai sebagai identitas unik gambar saat disimpan lokal — jangan dibuat unik per baris soal karena satu gambar bisa dipakai banyak soal).
   - Validasi ringan per baris (tandai `valid: boolean` + `alasan_invalid?: string`): soal kosong, kurang dari 2 pilihan terisi, atau kunci_jawaban tidak cocok pilihan manapun yang terisi. Baris invalid tetap dikembalikan (jangan di-skip diam-diam) supaya bisa ditampilkan di preview.
   - Tangani error kalau soal.xlsx tidak ditemukan di folder itu (kombinasi kelas+mapel belum ada datanya) dengan pesan jelas, bukan silent fail.

2. Komponen UI:
   - Dua dropdown berurutan: pilih Kelas dulu, baru pilih Mata Pelajaran (list mapel ambil dari daftar yang didefinisikan di composable, bukan hardcode di komponen).
   - Tombol "Ambil & Preview" memanggil fetchBankSoal dengan kelas+mapel terpilih, tampilkan loading state.
   - Tampilkan hasil sebagai tabel: nomor baris, cuplikan soal, status (✅ valid / ❌ invalid + alasan), ada gambar atau tidak (ikon + nama file gambarnya, supaya kelihatan kalau beberapa baris memang sengaja pakai gambar yang sama).
   - Baris invalid: warna merah, checkbox tidak tercentang & disabled. Baris valid: checkbox tercentang default.
   - Tombol "Gunakan Soal Terpilih" mengembalikan array baris terpilih (termasuk urlGambar & pathRelatifGambar) ke parent component — belum menyimpan apa pun ke lokal/database.

TEKNIS
- TypeScript, <script setup lang="ts">.
- Semua teks UI Bahasa Indonesia.
- Jangan proses download gambar ke lokal di fitur ini — cukup siapkan urlGambar & pathRelatifGambar yang valid, penyimpanan lokal ditangani di fitur terpisah.

terus buatkan foler ini
niluji/                          ← repo utama aplikasi
├── (source code app kamu...)
└── db-soal/                     ← folder khusus bank soal
    ├── kelas_4/
    │   ├── matematika/
    │   │   ├── soal.xlsx
    │   │   ├── gambar1.jpg
    │   │   └── gambar2.png
    │   └── ipas/
    │       ├── soal.xlsx
    │       └── gambar1.jpg
    └── kelas_5/
        └── ...
tambhakn dulu ke log, penamaan, misakan soal.xlsx nya ganti biar tidak kerlirum, nanti saya pas upload akan jadi 4-ipas.xlsx

jadi nama kode pelajaranya -->
Done
# Baru
fitur baru nanti pasang ini https://nuxt-tour.behonbaker.com/llms-full.txt