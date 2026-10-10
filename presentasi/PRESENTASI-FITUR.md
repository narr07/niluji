# Presentasi Fitur Niluji

Checklist screenshot untuk bahan presentasi, disusun mengikuti alur menu aplikasi. Pakai data dummy yang sudah ada sekarang (kelas, siswa, bank soal, sesi ujian yang sudah dikerjakan) — beberapa halaman butuh data tertentu supaya tidak kosong saat di-screenshot, sudah ditandai di catatan tiap bagian.

Centang `[x]` tiap sudah di-screenshot. Nama file screenshot bebas, urutkan saja sesuai nomor di bawah biar gampang disusun ulang nanti.

---

## 1. Dashboard

**Halaman:** `/` (Dashboard)

- [ ] 1.1 Tampilan dashboard lengkap — nama sekolah di navbar, tombol "Akses Ujian" (QR/URL join CBT), status IP:port server, 4 kartu statistik (Siswa, Mata Pelajaran, Jenis Ujian, Ujian Aktif), dan info sekolah di bawahnya.

> Catatan: supaya kartu "Ujian Aktif" tidak nol, pastikan ada 1 ujian di Kelola Ujian yang jadwalnya mencakup waktu sekarang.

---

## 2. Siswa

**Halaman:** `/siswa` → `/siswa/[kelas]`

- [ ] 2.1 Halaman pilih kelas (`/siswa`) — kartu tiap kelas.
- [ ] 2.2 Tabel siswa dalam satu kelas (`/siswa/[kelas]`) — kolom NISN, nama, status login (live, kalau ada siswa yang sedang login CBT tampilkan juga).
- [ ] 2.3 Mode pilih banyak (select) + tombol hapus massal di bawah tabel.
- [ ] 2.4 Modal tambah/edit siswa manual.
- [ ] 2.5 Modal "Import Siswa" — area drag-and-drop file CSV/Excel, plus tabel preview hasil parsing (yang valid vs "cek manual") sebelum disimpan.
- [ ] 2.6 (Pengaturan → Tarik Data Siswa) Fitur tarik data siswa dari Google Sheets sekolah — lihat bagian 7.4.

---

## 3. Bank Soal

**Halaman:** `/soal` → `/soal/[kelas]` → `/soal/[kelas]/[pelajaran]` → `/soal/[kelas]/[pelajaran]/[jenis]`

- [ ] 3.1 Halaman pilih kelas (`/soal`).
- [ ] 3.2 Halaman pilih mata pelajaran dalam kelas (`/soal/[kelas]`).
- [ ] 3.3 Grid "Jenis Ujian" per mata pelajaran (`/soal/[kelas]/[pelajaran]`) — kartu jenis ujian dengan badge jumlah soal, tombol titik-tiga (edit nama/hapus), tombol "Buat Jenis Ujian".
- [ ] 3.4 Tabel soal dalam satu jenis ujian (`/soal/[kelas]/[pelajaran]/[jenis]`) — daftar soal PG & esai.
- [ ] 3.5 Mode pilih banyak + hapus massal soal (sama seperti di halaman Siswa).
- [ ] 3.6 Modal tambah/edit soal — termasuk toolbar format teks (bold/italic/underline/coret), tombol sisip gambar di tengah pertanyaan, dan tiap opsi jawaban (A-E) yang bisa diberi gambar sendiri.
- [ ] 3.7 Modal preview soal (mata cek sebelum dipakai ujian).
- [ ] 3.8 Dropdown "Import Soal" dengan 3 pilihan format (klik tombolnya saja, tidak perlu buka satu-satu dulu):
  - [ ] 3.8a Import dari Naskah Word (.docx) — dua slot file (naskah + kunci jawaban), keduanya bisa drag-and-drop, plus tautan unduh contoh template.
  - [ ] 3.8b Import CSV/Excel — drag-and-drop, plus ringkasan hasil import (berhasil vs dilewati).
  - [ ] 3.8c Import dari file Markdown lokal (.md) — drag-and-drop, plus resolusi gambar opsi dari folder `gambar/` di sebelah file.
  - [ ] 3.8d Tabel preview hasil import (bisa diedit per baris / tambah baris manual) sebelum benar-benar disimpan — screenshot salah satu jalur (Word paling representatif).

> Catatan: siapkan minimal 1 jenis ujian dengan judul cukup panjang supaya terlihat wrap-nya rapi di kartu (3.3), dan minimal beberapa soal PG + esai + 1 soal yang opsi jawabannya pakai gambar (3.4/3.6).

---

## 4. Kelola Ujian

**Halaman:** `/ujian`

- [ ] 4.1 Tabel daftar ujian — kolom judul, kelas, mata pelajaran, tombol "Lihat Hasil"/Edit/Hapus.
- [ ] 4.2 Baris yang di-expand — detail jenis soal, waktu pelaksanaan, durasi, token (dengan tombol salin), badge "Acak Soal PG"/"Acak Soal Esai".
- [ ] 4.3 Modal "Buat Ujian Baru" — token yang otomatis terisi 4 huruf acak (dengan tombol acak ulang di sebelahnya), toggle acak PG & Esai terpisah, form jadwal.
- [ ] 4.4 Modal konfirmasi hapus ujian.

---

## 5. Hasil Ujian

**Halaman:** `/hasil` → `/hasil/[kelas]` → `/hasil/[kelas]/[pelajaran]` → `/hasil/[kelas]/[pelajaran]/[jenis]` → `.../analisis` → `/hasil/sesi/[sessionId]`

- [ ] 5.1 Halaman pilih kelas (`/hasil`).
- [ ] 5.2 Halaman pilih mata pelajaran dalam kelas (`/hasil/[kelas]`) — termasuk tombol "Export Kelas" (dropdown: semua sesi jadi zip, atau per-siswa-detail jadi zip).
- [ ] 5.3 Halaman pilih jenis ujian dalam mata pelajaran (`/hasil/[kelas]/[pelajaran]`).
- [ ] 5.4 Tabel hasil sesi ujian per jenis (`/hasil/[kelas]/[pelajaran]/[jenis]`) — nama siswa, skor, status submit, tombol reset sesi (dengan modal konfirmasi).
- [ ] 5.5 Dropdown export di halaman ini — 3 pilihan: Export Semua (ringkasan semua siswa), Export per Siswa (satu siswa, isi detail tiap soal), Export per Siswa (semua siswa jadi zip, isi detail tiap soal juga).
- [ ] 5.6 Halaman Analisis (`.../[jenis]/analisis`) — grafik "Tingkat Kesulitan Soal" & "Peringkat Siswa", plus 4 kartu: 5 Soal Termudah, 5 Soal Tersulit, 5 Siswa Nilai Tertinggi, 5 Siswa Nilai Terendah.
- [ ] 5.7 Detail satu sesi ujian (`/hasil/sesi/[sessionId]`) — jawaban siswa per soal, benar/salah, nilai esai yang bisa diberi skor manual oleh guru.

> Catatan: kartu "Termudah/Tersulit" dan "Tertinggi/Terendah" di 5.6 akan terlihat lebih meyakinkan kalau jumlah soal >5 dan jumlah siswa yang sudah submit >5, supaya kedua kartu tidak menampilkan data yang sama persis (dengan data dummy sekarang yang cuma beberapa soal/siswa, dua kartu itu saling berkebalikan isinya — itu bukan bug, cuma karena datanya sedikit).

---

## 6. E-Rapor

**Halaman:** `/e-rapor`

- [ ] 6.1 Halaman placeholder "Coming Soon" (fitur belum aktif, cukup 1 screenshot saja untuk menunjukkan roadmap).

---

## 7. Pengaturan

Menu ini ada di pojok kiri bawah sidebar (bukan menu utama), klik "Pengaturan" untuk membuka sub-menunya.

**Halaman:** `/pengaturan` dan turunannya

- [ ] 7.1 Data Sekolah (`/pengaturan`) — nama sekolah, NPSN, alamat, kepala sekolah, PIN export.
- [ ] 7.2 Data Kelas (`/pengaturan/kelas`) — daftar kelas, tambah/edit/hapus.
- [ ] 7.3 Data Pelajaran (`/pengaturan/pelajaran`) — daftar mata pelajaran, tambah/edit/hapus.
- [ ] 7.4 Tarik Data Siswa (`/pengaturan/tarik-data-siswa`) — cari nama sekolah dari spreadsheet online, tarik data siswa langsung tanpa input manual, plus tautan "Buka & Edit Data Sumber".
- [ ] 7.5 Tarik Soal Online (`/pengaturan/tarik-soal-online`) — pilih kelas/mapel/jenis ujian dari repo GitHub sekolah lain, preview, lalu impor ke Bank Soal.

### Panduan & Tentang (juga di menu Pengaturan)

- [ ] 7.6 Panduan (`/panduan`) — langkah-demi-langkah pakai aplikasi untuk guru.
- [ ] 7.7 Tentang → About (`/tentang`).
- [ ] 7.8 Tentang → Fitur (`/tentang/fitur`) — termasuk kartu "Export Bank Soal" (unduh contoh template Markdown) dan kartu "Backup Database" (status backup terakhir + tombol backup manual).
- [ ] 7.9 Tentang → Changelog (`/tentang/changelog`) — riwayat versi v1.0.0 dan v1.0.1.

### Tema & Tampilan (dropdown di menu Pengaturan, bukan halaman terpisah)

- [ ] 7.10 Dropdown "Tema" terbuka — pilihan warna primary & neutral (grid swatch warna).
- [ ] 7.11 Dropdown "Tampilan" terbuka — pilihan mode Terang/Gelap.
- [ ] 7.12 Satu halaman yang sama (mis. Dashboard) di-screenshot 2x — sekali mode terang, sekali mode gelap — biar kontrasnya kelihatan jelas di presentasi.
- [ ] 7.13 (opsional) Halaman yang sama dengan warna primary berbeda dari default — untuk menunjukkan kustomisasi warna tema.

---

## 8. Halaman CBT (sisi siswa)

**Halaman:** `/cbt` (dibuka lewat browser di perangkat siswa, bukan lewat sidebar admin)

- [ ] 8.1 Halaman login siswa — input NISN & token ujian.
- [ ] 8.2 Halaman soal Pilihan Ganda — mode satu-soal-per-layar atau mode scroll semua soal (pilih salah satu untuk screenshot), termasuk tampilan gambar pada soal maupun pada opsi jawaban.
- [ ] 8.3 Indikator status simpan jawaban (banner "Menyimpan jawaban..." kalau lagi retry, tombol Selesai yang terkunci sampai semua tersimpan) — kalau sempat kepakai, screenshot state ini juga jadi nilai tambah (bukti ketahanan saat koneksi terputus).
- [ ] 8.4 Halaman soal Esai.
- [ ] 8.5 Halaman konfirmasi/selesai setelah submit.

---

Setelah semua screenshot terkumpul, kirim ke saya berurutan sesuai nomor di atas — saya bantu susun jadi draf presentasinya.
