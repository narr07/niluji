# Catatan Perubahan

## Belum dirilis

### Tarik Data Siswa

- Data dari Google Sheets kini ditampilkan sebagai pratinjau dulu dan tidak langsung disimpan.
- Setiap siswa diberi status Baru, Berubah, Sama, atau Tidak valid. Untuk siswa yang berubah, data lamanya ikut ditampilkan.
- Guru memilih siswa yang disimpan. Baris tidak valid (NISN atau nama kosong, NISN dobel) tidak bisa disimpan.
- Backup database dibuat otomatis sebelum menyimpan.

### Aplikasi Android untuk siswa

- Aplikasi baru **NILUJI Siswa** di folder `apk-siswa/`: masuk ujian dengan memindai QR kartu peserta, hanya membuka server ujian yang dipilih, dan memakai mode ujian Android (blokir tangkapan layar, layar tetap menyala, penyematan aplikasi, konfirmasi tombol kembali).

### Pengembangan

- Test server tidak lagi sesekali gagal karena dua test berbagi folder database yang sama.

## v1.3.0 (2026-11-19)

Perubahan ini berfokus pada keamanan ujian, ketepatan nilai, dan keselamatan data. Nomor v1.1.0 dan v1.2.0 dilewati karena tag tersebut sudah ada di repositori tanpa rilis yang terbit.

### Yang perlu dilakukan setelah memperbarui

- **Cetak ulang kartu bila diminta.** Buka **Siswa > Kartu Peserta**. Bila muncul peringatan PIN versi lama, klik **Acak Ulang PIN** lalu cetak ulang kartunya. PIN lama tidak diganti otomatis supaya kartu yang sudah dibagikan tidak mendadak gagal.
- **Periksa data kelas siswa.** Siswa kini hanya bisa masuk ke ujian kelasnya. Form ujian memberi peringatan bila kelas yang dipilih belum punya siswa.
- **Periksa ujian lama yang berbobot PG 100% dan punya esai.** Dulu ujian seperti ini diam-diam dihitung 70% PG dan 30% esai. Sekarang 100% berarti nilai akhir hanya dari PG.
- **Atur PIN Export** di **Pengaturan > Data Sekolah** sebelum memakai fitur ekspor. PIN `1234` lama tidak berlaku lagi.
- **Muat ulang halaman ujian siswa** bila aplikasi diperbarui saat ada sesi berjalan, karena sesi lama harus masuk ulang untuk mendapat token baru.

### Keamanan ujian

- Server memberi token rahasia per sesi saat siswa masuk. Jawaban, kumpul PG, dan kumpul akhir ditolak tanpa token itu, jadi siswa tidak bisa lagi mengubah atau mengumpulkan ujian siswa lain.
- Token sesi diganti saat sesi pindah perangkat, dan perangkat lama langsung kehilangan akses setelah pengawas membuka kunci.
- Jawaban hanya diterima untuk soal milik sesi itu dan pilihan yang benar-benar ada.
- Siswa tidak bisa masuk ke ujian kelas lain. Siswa tanpa data kelas tetap diizinkan.
- Ujian baru mewajibkan PIN secara bawaan, dan siswa tanpa PIN tidak lolos pada ujian yang mewajibkannya.
- Percobaan login dan PIN yang gagal dibatasi: 8 kali per siswa dari satu perangkat dalam 5 menit, dan 30 kali per perangkat dalam 1 menit.
- PIN, token sesi, dan token ujian otomatis memakai generator acak yang aman. Token otomatis kini 6 huruf, token manual minimal 4 karakter dan disimpan dalam huruf besar.
- Sesi yang sudah selesai tidak lagi mengirim soal, jawaban, nilai, atau token perangkat.
- Status online siswa hanya bisa dihapus oleh perangkat yang login.
- Sesi yang ditinggalkan siswa dikumpulkan otomatis 3 menit setelah waktunya habis, dan pengawas bisa membukanya lagi dengan tambahan 10 menit.

### Ketepatan nilai

- Bobot PG dipakai persis seperti yang diisi guru.
- Nilai akhir tetap kosong dengan status **Menunggu nilai esai** sampai semua jawaban esai dinilai. Esai yang tidak dijawab otomatis bernilai 0.
- Mengumpulkan ulang tidak lagi mengubah waktu atau nilai yang sudah tersimpan.

### Jawaban siswa

- Tiap soal hanya punya satu pengiriman yang berjalan, sehingga teks esai lama tidak bisa menimpa teks baru. Ketikan esai dikirim setelah jeda singkat.
- Saat waktu habis, jawaban yang tertunda dikirim dulu sebelum ujian dikumpulkan.
- Pengumpulan dicoba ulang otomatis saat koneksi terganggu, dan layar siswa menampilkan statusnya.
- Jawaban yang ditolak server ditampilkan dan tidak diulang terus-menerus.
- Pintasan keyboard tidak lagi aktif bersama Ctrl, Alt, atau Command.

### Hasil ujian

- Setiap sesi menyimpan salinan soal, kunci, dan skornya. Mengubah Bank Soal tidak lagi mengubah ujian yang sudah dimulai atau hasilnya.
- Tombol **Nilai Ulang** di halaman hasil menerapkan kunci dan skor terbaru bila guru membetulkan kunci yang salah.
- Soal, ujian, dan mata pelajaran yang sudah punya hasil tidak bisa dihapus.
- Database otomatis dicadangkan sebelum reset sesi, nilai ulang, hapus ujian, hapus mata pelajaran, dan pemulihan.
- Daftar hasil kini dihitung dengan satu query, sehingga lebih ringan saat dipantau selama ujian.

### Backup

- Backup harian dibuat saat aplikasi dibuka, diperbarui tiap jam, dan disimpan untuk 14 hari pemakaian.
- Backup manual dan cadangan sebelum perubahan punya batas sendiri, dan gambar soal ikut dicadangkan.
- Menu **Tentang > Fitur** kini bisa memulihkan backup. Pemulihan ditolak selama masih ada siswa yang sedang ujian.
- Backup tidak lagi menahan database lebih lama dari yang diperlukan.

### Impor dan ekspor

- Impor Markdown dan Tarik Soal Online kini mempertahankan opsi E beserta gambarnya.
- Soal Markdown dengan kunci yang tidak cocok dengan pilihannya ditandai untuk dicek manual.
- Halaman ekspor mengingatkan bahwa file memuat kunci jawaban.
- PIN Export bawaan `1234` dihapus. Sekolah harus menentukan PIN sendiri di **Pengaturan > Data Sekolah** sebelum bisa mengekspor.
- Impor Word kini membaca opsi E dan kunci jawaban E.

### Aplikasi desktop

- Izin menjalankan perintah `sh`, plugin yang tidak dipakai (fs, os, store, notification), dan modul auto-import Tauri dihapus.
- Pemuat gambar tidak bisa lagi membaca file di luar folder `uploads`.
- CSP aplikasi desktop tidak lagi mengizinkan skrip inline sembarangan.
- Tarik Data Siswa hanya menerima tautan Google Sheets lewat HTTPS.
- Query server berjalan di luar thread async, dan SQLite memakai `busy_timeout` 5 detik.

### Pengembangan dan rilis

- Workflow **CI** baru menjalankan lint, typecheck, test frontend, build, clippy, dan test Rust. Workflow rilis memakainya sebelum publish.
- Script baru: `lint:fix`, `typecheck`, `test`, `test:rust`, dan `clippy`. Script `lint` tidak lagi mengubah file.
- Versi dependensi frontend dikunci. TypeScript resmi 5.9.3 dan `vue-tsc` menggantikan `tslite`, dan 12 kesalahan tipe lama diperbaiki.
- Versi paket Tauri JavaScript disejajarkan dengan crate Rust, karena perbedaan versinya menggagalkan `tauri build`.
- Test baru mencakup akses HTTP antarsiswa, pembatas percobaan, backup dan pemulihan, penilaian, antrean jawaban, serta parser Markdown.

### Perbaikan tampilan

- Langkah **Isi Identitas Sekolah** di dashboard kini membuka halaman Data Sekolah, bukan halaman 404, dan baru dianggap selesai setelah nama sekolah diganti.

### Dokumentasi

- README dan situs dokumentasi disesuaikan dengan perilaku sebenarnya, termasuk format ekspor, backup, pemulihan, bobot nilai, dan cara rilis.
- Situs dokumentasi tidak lagi melebar di HP. Tautan header pindah ke menu, dan tautan footer serta path panjang kini turun ke baris berikutnya.
- Template soal Markdown di halaman Rilis kini memakai format yang dibaca aplikasi (sebelumnya 0 soal terbaca) dan tidak lagi memuat PIN `1234`.
- Panduan impor CSV/Excel kini memakai kolom yang benar-benar dibaca aplikasi, dengan peringatan agar tidak memakai kolom `tipe` atau `jenis`.
- Rumus yang tampil sebagai kode mentah diganti teks biasa.
- Halaman Rilis tidak lagi mencantumkan v1.1.0 yang tidak ada, hanya menawarkan sistem operasi yang installernya tersedia, dan tombol unduhnya langsung mengarah ke installer.

### Belum diubah

- Halaman siswa tetap memakai HTTP di jaringan lokal, karena HTTPS tanpa domain akan memunculkan peringatan sertifikat di setiap HP siswa. Gunakan WiFi ujian yang tertutup dan berpassword.
- Folder `db-soal/` tidak diubah karena soalnya tidak dipakai untuk ujian.
