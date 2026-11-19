# Teks Video Tutorial NILUJI v1.3.0 (Siap Salin & Rekam)

Durasi total video: **7 menit 33 detik** (`453,00` detik), 13 adegan.
Setiap blok teks di bawah sudah disamakan dengan subtitle di video dan audio narasi (`adegan-01.wav` s.d. `adegan-13.wav`).

---

## Adegan 1 · Pembuka (`00:00 – 00:26`, 26,49 detik)

Selamat datang di tutorial NILUJI. NILUJI adalah aplikasi ujian sekolah berbasis komputer yang berjalan tanpa internet. Cukup satu laptop guru sebagai server, dan siswa mengerjakan ujian dari HP atau laptop melalui WiFi sekolah. Di video ini, kita akan menyiapkan satu ujian dari awal sampai nilai akhirnya keluar.

---

## Adegan 2 · Instalasi dan jaringan (`00:26 – 01:01`, 34,84 detik)

Pertama, unduh installer NILUJI dari situs niluji titik permadi titik dev, lalu pasang di laptop pengawas. Saat pertama dibuka, Windows akan meminta izin firewall. Pilih izinkan, supaya HP siswa bisa terhubung. Nyalakan WiFi khusus ujian yang memakai password, lalu sambungkan laptop ke WiFi tersebut. Di pojok kanan atas Dashboard, pastikan alamat IP yang dipilih adalah alamat dari WiFi ujian.

---

## Adegan 3 · Identitas sekolah (`01:01 – 01:29`, 28,02 detik)

Berikutnya, klik Isi Identitas Sekolah. Lengkapi nama sekolah, NPSN, alamat, dan nama kepala sekolah. Nama ini akan tercetak di kartu peserta. Di halaman yang sama, buat juga PIN Export, minimal empat karakter. PIN ini diminta setiap kali Anda mengekspor bank soal, jadi simpan baik-baik.

---

## Adegan 4 · Kelas dan mata pelajaran (`01:29 – 01:46`, 17,20 detik)

Lalu buka Data Kelas dan Data Pelajaran. Pastikan daftar kelas dan mata pelajaran sudah sesuai dengan sekolah Anda. Daftar ini dipakai di semua menu, mulai dari data siswa sampai hasil ujian.

---

## Adegan 5 · Data siswa (`01:46 – 02:29`, 43,07 detik)

Sekarang kita masukkan data siswa. Di menu Siswa, Anda bisa menambah siswa satu per satu, atau klik Import untuk mengunggah file Excel atau CSV. Unduh dulu templatenya supaya kolomnya sesuai. Bila data siswa sekolah Anda sudah ada di Google Sheets, buka Tarik Data Siswa, pilih nama sekolah, lalu klik Lihat Pratinjau. Data belum disimpan. Periksa dulu siswa yang baru dan yang datanya berubah, hapus centang bila perlu, lalu klik Simpan. NILUJI membuat cadangan database sebelum menyimpan.

---

## Adegan 6 · Kartu peserta (`02:29 – 02:54`, 24,76 detik)

Setiap siswa otomatis mendapat PIN enam digit. Klik Kartu Peserta untuk melihat kartu ujian lengkap dengan nama sekolah, NISN, PIN, dan kode QR alamat server. Klik Cetak Kartu untuk mencetak atau menyimpannya sebagai PDF. Bagikan kartu ini kepada siswa sebelum ujian dimulai.

---

## Adegan 7 · Bank soal (`02:54 – 03:36`, 42,40 detik)

Selanjutnya, siapkan soal di menu Bank Soal. Pilih kelas dan mata pelajaran, lalu buat jenis ujian, misalnya Sumatif Akhir Semester. Di dalamnya, klik Tambah Soal untuk menulis soal pilihan ganda dengan opsi A sampai E, atau soal esai. Anda juga bisa menyisipkan gambar di soal maupun di setiap pilihan jawaban. Kalau soal sudah diketik di Word, klik Import Soal. NILUJI bisa membaca file Word, Excel atau CSV, dan Markdown, lalu menampilkan pratinjaunya sebelum disimpan.

---

## Adegan 8 · Membuat jadwal ujian (`03:36 – 04:18`, 41,73 detik)

Sekarang buat jadwal ujian di menu Kelola Ujian. Klik Buat Ujian Baru, lalu pilih kelas, mata pelajaran, dan jenis soal. Atur tanggal, jam mulai, jam selesai, dan durasi pengerjaan. Token ujian dibuat otomatis enam huruf. Isi bobot nilai pilihan ganda, misalnya tujuh puluh persen, sehingga esai bernilai tiga puluh persen. Aktifkan pengacakan soal dan pilihan jawaban, dan biarkan Wajib PIN Kartu tetap menyala supaya siswa tidak bisa masuk memakai akun temannya.

---

## Adegan 9 · Hari ujian di sisi pengawas (`04:18 – 04:39`, 21,35 detik)

Di hari ujian, buka Dashboard. Ujian yang sedang berlangsung muncul di panel ujian aktif. Klik tombol token untuk menampilkannya dalam ukuran besar di proyektor. Klik QR Akses Ujian bila siswa ingin memindai alamat server langsung dari layar.

---

## Adegan 10 · Siswa mengerjakan ujian (`04:39 – 05:34`, 54,37 detik)

Dari sisi siswa, ada dua cara masuk. Cara pertama, buka browser dan ketik alamat yang tertera di kartu peserta. Cara kedua, pasang aplikasi Android NILUJI Siswa, lalu pindai QR pada kartu. Aplikasi ini memblokir tangkapan layar dan meminta HP disematkan selama ujian. Setelah itu, siswa mengisi NISN dan nama lengkap, lalu token ujian dan PIN kartu, dan menekan Mulai Kerjakan Ujian. Jawaban tersimpan otomatis ke server. Soal yang masih ragu bisa ditandai, dan bila WiFi sempat putus, jawaban dikirim ulang dengan sendirinya. Setelah pilihan ganda selesai, siswa lanjut ke esai, lalu menekan Selesai dan Kumpulkan.

---

## Adegan 11 · Pantauan dan bantuan pengawas (`05:34 – 06:13`, 39,18 detik)

Selama ujian, Dashboard menampilkan berapa siswa yang sedang mengerjakan pilihan ganda, esai, dan yang sudah selesai. Klik Pantau Hasil untuk melihat setiap siswa. Bila HP siswa mati, klik buka kunci perangkat, lalu siswa bisa melanjutkan dari HP lain tanpa kehilangan jawaban. Bila ada kendala, beri tambahan waktu sepuluh menit. Siswa yang lupa mengumpulkan akan dikumpulkan otomatis tiga menit setelah waktunya habis, dan pengawas tetap bisa membukanya lagi dengan tambahan waktu.

---

## Adegan 12 · Koreksi esai, analisis, dan ekspor nilai (`06:13 – 06:59`, 46,12 detik)

Setelah ujian, buka Hasil Ujian. Nilai pilihan ganda langsung dihitung, sedangkan nilai akhir menunggu sampai semua esai dinilai. Klik ikon detail pada nama siswa, baca jawabannya, lalu isi nilai esai. Nilai akhir otomatis dihitung sesuai bobot. Bila ternyata ada kunci jawaban yang salah, perbaiki di Bank Soal, lalu klik Nilai Ulang. Buka tab Analisis Soal untuk melihat soal yang paling mudah dan paling sulit, serta pilihan jawaban mana yang paling sering menjebak siswa. Terakhir, klik Export untuk mengunduh rekap nilai dalam format Excel.

---

## Adegan 13 · Cadangan data dan penutup (`06:59 – 07:33`, 33,47 detik)

Data Anda aman. NILUJI membuat cadangan harian yang disimpan selama empat belas hari, dan juga sebelum setiap perubahan penting. Bila perlu, buka Tentang, lalu Fitur, dan pulihkan database dari cadangan. Panduan lengkap, aplikasi Android untuk siswa, dan installer terbaru tersedia di niluji titik permadi titik dev. Selamat mencoba, semoga ujian di sekolah Anda berjalan lancar.
