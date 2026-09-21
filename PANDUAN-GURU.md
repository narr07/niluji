# Panduan Guru — Dari Nol Sampai Lihat Hasil Ujian

Ini alur lengkap buat guru yang baru pertama kali pakai Niluji. Ikuti urutannya dari atas
ke bawah — tiap langkah butuh langkah sebelumnya selesai dulu (misalnya, siswa baru bisa
ikut ujian kalau kelasnya sudah ada, dan soal baru bisa dipakai kalau mata pelajarannya
sudah terdaftar).

Anggap saja ini seperti nyiapin kelas baru di awal semester: kenalan dulu sama aplikasinya,
masukin data anak-anak, siapin soal, baru buka ujiannya.

---

## 1. Kenalan dulu — ubah nama sekolah

**Menu: Pengaturan > Data**

Begitu buka aplikasi pertama kali, ganti dulu nama sekolah supaya nggak nyangkut nama
default. Klik **Pengaturan** di sidebar kiri, masuk ke **Data**, isi nama sekolah dan
info lain yang diminta, lalu **Simpan**.

Kenapa ini duluan? Karena nama sekolah ini yang bakal muncul di halaman login ujian siswa
— jadi anak-anak tahu mereka buka aplikasi yang benar.

## 2. Siapin data kelas dan mata pelajaran

**Menu: Pengaturan > Data Kelas** dan **Pengaturan > Data Pelajaran**

Sebelum masukin siswa atau bikin soal, aplikasi perlu tahu kelas apa saja yang ada
(misalnya Kelas 4, 5, 6) dan mata pelajaran apa saja yang diajarkan (Matematika, IPAS,
Bahasa Indonesia, dst).

- Di **Data Kelas**, klik **Tambah** dan ketik nama kelasnya.
- Di **Data Pelajaran**, klik **Tambah**, isi kode singkat (contoh: `MTK` untuk
  Matematika) dan nama lengkapnya. Kode ini dipakai di banyak tempat lain, jadi bikin
  yang gampang diingat.

Sudah ada beberapa kelas/pelajaran bawaan? Lewati saja langkah ini dan tinggal
sesuaikan kalau ada yang kurang.

## 3. Masukin data siswa

**Menu: Siswa**

Ada dua cara, pilih yang paling cocok:

- **Satu-satu**: klik **Tambah Siswa**, isi NISN, nama, dan kelasnya. Cocok kalau
  jumlah siswa sedikit atau cuma nambah 1-2 anak baru.
- **Sekaligus banyak (impor)**: klik tombol impor, unduh dulu template CSV yang
  disediakan, isi kolomnya (NISN, nama, kelas) di Excel/Spreadsheet, simpan lagi
  sebagai CSV, lalu unggah filenya. Ini jauh lebih cepat kalau siswanya satu kelas
  penuh atau lebih.

Nama yang diketik siswa saat login nanti nggak perlu persis sama besar-kecil hurufnya
(BUDI, Budi, budi — semua dianggap sama), jadi nggak perlu khawatir soal itu.

## 4. Isi Bank Soal

**Menu: Bank Soal**

Ini bagian yang paling makan waktu, tapi begitu selesai bisa dipakai berkali-kali untuk
ujian berikutnya.

1. Pilih **kelas**, lalu pilih **mata pelajaran**.
2. Klik **Jenis Ujian** untuk bikin paket soal baru (contoh: "UTS Semester 1",
   "Latihan Harian"). Satu jenis = satu kumpulan soal yang nanti dipasangkan ke satu
   ujian.
3. Buka jenis yang baru dibuat, lalu isi soalnya. Ada tiga cara:
   - **Tambah manual**: klik **Tambah**, isi soal, pilihan jawaban, kunci jawaban, dan
     skornya satu per satu.
   - **Impor dari file**: siapkan soal di template Excel/CSV yang disediakan, lalu
     unggah — lebih cepat untuk soal dalam jumlah banyak.
   - **Tarik Soal Online**: kalau sekolah sudah punya bank soal online (lewat
     Pengaturan > Tarik Soal Online), ambil soal dari sana, periksa hasilnya di
     tabel preview, centang yang mau dipakai, lalu klik **Impor ke Bank Soal**.

Soal pilihan ganda dan esai bisa dicampur dalam satu jenis yang sama — nanti siswa
otomatis mengerjakan pilihan ganda dulu, baru lanjut ke esai.

## 5. Buat ujiannya

**Menu: Kelola Ujian**

Setelah bank soal siap, saatnya bikin ujian yang bisa diakses siswa.

1. Klik **Buat Ujian**.
2. Pilih **kelas**, **mata pelajaran**, dan **jenis soal** yang mau dipakai (yang sudah
   diisi di langkah 4).
3. Tentukan **jendela waktu ujian** — kapan ujian mulai bisa diakses dan kapan
   ditutup (misalnya, siswa cuma bisa mulai antara jam 07.00–09.00).
4. Tentukan **durasi pengerjaan** — berapa lama tiap siswa punya waktu SETELAH dia
   mulai (ini terpisah dari jendela di atas, jadi siswa yang mulai jam 08.50 tetap
   dapat durasi penuh, bukan cuma sisa 10 menit).
5. Isi **token ujian** — kode ini yang nanti diketik siswa untuk masuk. Bikin yang
   gampang diingat tapi nggak gampang ditebak orang lain (hindari "1234" atau
   "UJIAN").
6. Klik **Simpan**.

Ujian sudah siap. Yang perlu dikasih tahu ke siswa cuma dua hal: **alamat website ujian**
(bisa dilihat/dibagikan lewat kode QR di Dashboard) dan **token** yang barusan dibuat.

## 6. Siswa mengerjakan ujian

Bagian ini otomatis — guru tinggal pantau. Siswa buka alamat ujian di HP/laptop mereka
(masih dalam WiFi/hotspot yang sama dengan laptop guru, nggak perlu internet), login
pakai NISN dan nama, masukkan token, lalu mulai mengerjakan.

Progres siswa (siapa yang sudah mulai, sudah sampai mana, sudah selesai) bisa dipantau
langsung dari **Hasil Ujian** — halaman itu update sendiri tanpa perlu di-refresh.

## 7. Lihat dan nilai hasilnya

**Menu: Hasil Ujian**

1. Pilih **kelas**, lalu pilih **mata pelajaran**.
2. Tab **Daftar Siswa** menampilkan progres dan nilai tiap siswa (nilai yang muncul di
   sini cuma dari soal pilihan ganda — otomatis dihitung sistem).
3. Kalau ada soal esai, klik **Detail** di nama siswa untuk buka halaman penilaiannya.
   Baca jawaban esainya, isi nilai per soal, klik **Simpan Nilai**. Nilai esai ini
   sengaja dipisah dan nggak otomatis digabung ke nilai pilihan ganda — guru yang
   putuskan sendiri nilai akhirnya.
4. Tab **Analisis Soal** menunjukkan soal mana yang paling banyak dijawab salah dan
   siapa yang nilainya paling tinggi/rendah — enak buat lihat pola cepat tanpa
   ngecek satu-satu.
5. Butuh rekapnya di luar aplikasi? Klik **Export Semua** untuk unduh nilai satu
   kelas sekaligus, atau **Export** di halaman detail seorang siswa untuk rekap per
   anak.

Selesai — dari sekolah kosong sampai nilai ujian siap dibagikan ke wali murid, semua
lewat alur di atas.

---

## Kalau nyasar / lupa langkah

- **Ujian belum bisa dibuat?** Cek lagi apakah mata pelajaran dan jenis soalnya sudah
  ada isi soalnya (langkah 4).
- **Siswa nggak bisa login?** Cek NISN-nya sudah terdaftar di menu Siswa dan sudah
  masuk ke kelas yang benar.
- **Token ditolak?** Cek jendela waktu ujian — mungkin belum mulai atau sudah lewat.
- **Nilai esai nggak muncul di rekap?** Memang sengaja terpisah — buka Detail siswa
  untuk lihat/isi nilai esainya.
