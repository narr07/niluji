---
title: NILUJI - Platform CBT Offline Sekolah
description: Aplikasi ujian sekolah berbasis jaringan lokal (LAN/WiFi) tanpa internet. Cepat, aman dengan PIN & Kunci Perangkat, mendukung impor soal Word/Markdown bergambar, serta analisis butir soal otomatis.
---

::u-page-hero{class="dark:bg-gradient-to-b from-neutral-900 to-neutral-950"}
---
orientation: horizontal
---
#top
:hero-background

#title
Ujian Sekolah [100% Offline]{.text-primary} Tanpa Ribet Server.

#description
**NILUJI** mengubah satu laptop guru atau komputer proktor menjadi server ujian lokal siap pakai. Siswa mengerjakan soal lewat browser HP atau laptop melalui WiFi sekolah tanpa kuota internet, lengkap dengan Kartu Peserta QR + PIN, Kunci Perangkat (*Single-Device Lock*), dan Analisis Pengecoh.

#links
  :::u-button
  ---
  to: /memulai/instalasi
  size: xl
  trailing-icon: i-lucide-arrow-right
  ---
  Mulai Panduan
  :::

  :::u-button
  ---
  icon: i-lucide-download
  color: neutral
  variant: outline
  size: xl
  to: /rilis
  ---
  Unduh Installer
  :::

#default
  :::prose-pre
  ---
  code: |
    [PC Proktor: NILUJI Desktop]
      ├── Database Lokal : SQLite (WAL Mode + Auto-Backup 14 Hari)
      ├── Server Ujian   : HTTP Axum LAN (http://192.168.1.10/cbt)
      ├── Keamanan Sesi  : Token Ujian + PIN 6 Digit + Single-Device Lock
      └── Analisis Nilai : Bobot PG/Esai + Sebaran Opsi A/B/C/D/Kosong
  filename: Arsitektur Jaringan Lokal NILUJI
  ---

  ```txt [Arsitektur Jaringan Lokal NILUJI]
  [PC Proktor: NILUJI Desktop]
    ├── Database Lokal : SQLite (WAL Mode + Auto-Backup 14 Hari)
    ├── Server Ujian   : HTTP Axum LAN (http://192.168.1.10/cbt)
    ├── Keamanan Sesi  : Token Ujian + PIN 6 Digit + Single-Device Lock
    └── Analisis Nilai : Bobot PG/Esai + Sebaran Opsi A/B/C/D/Kosong
  ```
  :::
::

::u-page-section{class="dark:bg-neutral-950"}
#title
Dirancang Khusus untuk Kebutuhan Nyata Ruang Ujian Sekolah

#links
  :::u-button
  ---
  color: neutral
  size: lg
  to: /memulai/langkah-awal
  trailingIcon: i-lucide-arrow-right
  variant: subtle
  ---
  Lihat 7 Langkah Persiapan Ujian
  :::

#features
  :::u-page-feature
  ---
  icon: i-lucide-wifi-off
  ---
  #title
  100% Berjalan Tanpa Internet

  #description
  Tidak bergantung pada sinyal seluler maupun bandwidth internet sekolah. Cukup satu router WiFi lokal, puluhan hingga ratusan siswa dapat ujian serentak dengan respons instan.
  :::

  :::u-page-feature
  ---
  icon: i-lucide-shield-check
  ---
  #title
  Kunci Perangkat & PIN Peserta

  #description
  Setiap siswa memiliki PIN 6 digit pada Kartu Peserta. Sesi ujian otomatis terkunci pada perangkat pertama yang login dan waktu dihitung dari server untuk mencegah kecurangan.
  :::

  :::u-page-feature
  ---
  icon: i-lucide-radio
  ---
  #title
  Kontrol Pengawas Real-Time

  #description
  Pantau progres pengerjaan PG dan Esai secara langsung. Pengawas dapat membuka kunci perangkat jika HP siswa mati, menambah waktu +10 menit, atau memaksa kumpul dari layar utama.
  :::

  :::u-page-feature
  ---
  icon: i-lucide-file-text
  ---
  #title
  Impor Soal Word, Excel & Markdown

  #description
  Impor naskah ujian dari Microsoft Word (.docx) beserta kunci jawaban, tabel Excel/CSV, atau bundel Markdown (.zip) lengkap dengan gambar di tengah soal dan gambar pada opsi A-E.
  :::

  :::u-page-feature
  ---
  icon: i-lucide-bar-chart-3
  ---
  #title
  Bobot Nilai & Analisis Pengecoh

  #description
  Gabungkan nilai Pilihan Ganda dan Esai sesuai bobot persentase otomatis. Evaluasi kualitas soal lewat statistik tingkat kesulitan dan sebaran pemilih opsi A, B, C, D, hingga kosong.
  :::

  :::u-page-feature
  ---
  icon: i-lucide-database-backup
  ---
  #title
  Tahan Gangguan & Auto-Backup

  #description
  Jawaban siswa otomatis dicoba simpan ulang saat sinyal WiFi terputus sementara. Database SQLite dicadangkan otomatis setiap aplikasi dibuka hingga 14 salinan terakhir.
  :::
::

::u-page-section{class="dark:bg-neutral-950"}
#title
Lengkap untuk Guru, Pengawas, maupun Siswa

#features
  :::u-page-feature
  ---
  icon: i-lucide-id-card
  ---
  #title
  Cetak Kartu Peserta QR + PIN

  #description
  Cetak kartu peserta ujian satu kelas sekaligus dalam tata letak siap gunting, dilengkapi identitas sekolah, NISN, PIN 6 digit, dan QR Code alamat server lokal.
  :::

  :::u-page-feature
  ---
  icon: i-lucide-smartphone
  ---
  #title
  Portal CBT Siswa Nyaman di HP & Laptop

  #description
  Dilengkapi tombol Ragu-ragu (bookmark), pengatur ukuran teks (A-/A/A+), indikator status simpan real-time, bilah navigasi bawah khusus HP, serta pintasan keyboard A-D.
  :::

  :::u-page-feature
  ---
  icon: i-lucide-cloud-download
  ---
  #title
  Sinkronisasi Google Sheets & Bank Soal Online

  #description
  Tarik data siswa dari Google Sheets dan bagikan paket Bank Soal antar-guru menggunakan arsip Markdown ber-PIN melalui repositori GitHub.
  :::
::

::u-page-section{class="dark:bg-gradient-to-b from-neutral-950 to-neutral-900"}
  :::u-page-c-t-a
  ---
  links:
    - label: Unduh NILUJI Sekarang
      to: '/rilis'
      trailingIcon: i-lucide-download
    - label: Baca Panduan Instalasi
      to: '/memulai/instalasi'
      variant: subtle
      icon: i-lucide-book-open
    - label: Studi Kasus di permadi.dev
      to: 'https://permadi.dev/id/projek/niluji'
      target: '_blank'
      variant: ghost
      icon: i-lucide-globe
  title: Siap menyelenggarakan ujian sekolah tanpa kendala sinyal?
  description: Gratis dan bersumber terbuka (MIT License). Unduh installer untuk Windows, Linux, atau macOS dan mulai ujian pertama Anda dalam 10 menit.
  class: dark:bg-neutral-950
  ---

  :stars-bg
  :::
::
