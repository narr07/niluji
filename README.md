<p align="center">
  <img width="128" src="./public/logo.svg" alt="NILUJI Logo">
</p>

<h1 align="center">NILUJI</h1>

<p align="center">
  <strong>Platform Ujian Berbasis Komputer (CBT) 100% Offline untuk Sekolah</strong>
  <br />
  Dibangun dengan <a href="https://nuxt.com">Nuxt 4</a>, <a href="https://ui.nuxt.com">Nuxt UI v4</a>, dan <a href="https://v2.tauri.app">Tauri 2 (Rust + Axum + SQLite)</a>.
</p>

<p align="center">
  <a href="https://narr07.github.io/niluji/"><strong>📘 Dokumentasi Resmi</strong></a> ·
  <a href="https://narr07.github.io/niluji/rilis"><strong>⬇️ Unduh Installer</strong></a> ·
  <a href="https://permadi.dev/id/projek/niluji"><strong>🌐 Studi Kasus (permadi.dev)</strong></a> ·
  <a href="https://github.com/narr07/niluji/issues"><strong>🐛 Laporkan Masalah</strong></a>
</p>

<p align="center">
  <img src="https://img.shields.io/github/package-json/v/narr07/niluji" alt="Version" />
  <img src="https://img.shields.io/github/license/narr07/niluji" alt="License" />
</p>

---

## Ringkasan Arsitektur

- **Satu Laptop Proktor = Server Ujian Lengkap**: Aplikasi desktop Tauri menjalankan server HTTP tertanam (`axum`) di jaringan lokal (`http://IP-LOKAL/cbt`).
- **Tanpa Internet & Tanpa Instalasi di HP Siswa**: Siswa cukup terhubung ke WiFi/hotspot lokal yang sama dan membuka alamat ujian melalui browser HP atau laptop.
- **Database SQLite Lokal Tahan Gangguan**: Seluruh data siswa, bank soal, gambar, sesi ujian, dan nilai disimpan di SQLite lokal dengan mode `WAL`, `busy_timeout`, serta **Auto-Backup 14 Hari**.

## Fitur Utama

- **Keamanan Ujian Berlapis**:
  - **PIN Peserta 6 Digit** otomatis untuk setiap siswa.
  - **Token Ujian Dinamis** yang dapat diputar pengawas kapan saja.
  - **Kunci Perangkat (*Single-Device Lock*)** mencegah satu akun dibuka di dua HP bersamaan.
  - **Server-Side Timer** memastikan sisa waktu ujian dihitung dari server dan tidak bisa dimanipulasi dari jam HP siswa.
- **Kontrol Pengawas Real-Time**:
  - Pantau status pengerjaan PG dan Esai tiap siswa secara langsung.
  - **Buka Kunci Perangkat** (*Unlock Device*) bila HP siswa kehabisan baterai.
  - **Tambah Waktu (`+10 Menit`)** atau **Paksa Kumpul** langsung dari layar proktor.
- **Bank Soal Multi-Format & Bergambar**:
  - Impor soal dari **Microsoft Word (`.docx`)**, **Excel (`.xlsx`)**, **CSV**, maupun **Bundel Markdown (`.zip`)** lengkap dengan gambar di tengah soal dan gambar pada opsi jawaban A–E.
  - **Tarik & Bagikan Bank Soal Online** terproteksi PIN lewat repositori GitHub.
- **Cetak Kartu Peserta QR + PIN**:
  - Cetak kartu peserta ujian satu kelas sekaligus siap gunting lengkap dengan QR Code server lokal.
- **Portal CBT Siswa Ergonomis**:
  - Penanda soal **Ragu-ragu** (*Bookmark*), pengatur ukuran huruf (`A-` / `A` / `A+`), pintasan keyboard `A–D`, bilah navigasi bawah khusus HP, dan antrean simpan ulang otomatis (*Offline Retry Queue*).
- **Bobot Nilai & Analisis Butir Soal**:
  - Pengaturan bobot persentase Pilihan Ganda vs Esai (mis. `70% PG + 30% Esai`), grafik tingkat kesulitan soal, dan **Analisis Pengecoh** (distribusi pemilih opsi A/B/C/D/Kosong).

---

## Dokumentasi Lengkap

Panduan lengkap untuk Guru, Pengawas/Proktor, dan Siswa tersedia di situs dokumentasi resmi:

👉 **[https://narr07.github.io/niluji/](https://narr07.github.io/niluji/)**

- [Panduan Instalasi (Windows, Linux, macOS)](https://narr07.github.io/niluji/memulai/instalasi)
- [Persiapan Jaringan WiFi Lokal](https://narr07.github.io/niluji/memulai/persiapan-jaringan)
- [Impor & Ekspor Soal (Word, Excel, Markdown ZIP)](https://narr07.github.io/niluji/panduan-guru/impor-dan-ekspor-soal)
- [Meja Pantau Pengawas & Aksi Sesi](https://narr07.github.io/niluji/panduan-pengawas/pantauan-dan-aksi)
- [Troubleshooting Ruang Ujian](https://narr07.github.io/niluji/referensi/troubleshooting)

---

## Pengembangan Lokal

> Bun digunakan sebagai package manager utama proyek ini.

```sh
# Instal dependensi aplikasi utama
bun install

# Jalankan mode pengembangan (Nuxt + Tauri)
bun run tauri:dev

# Jalankan situs dokumentasi (http://localhost:3005/niluji/)
cd docs
bun install
bun run dev
```

### Build Produksi & Rilis Otomatis

```sh
# Build aplikasi desktop lokal
bun run tauri:build

# Rilis versi baru otomatis ke GitHub Releases & GitHub Pages
bun run release
```

---

## Lisensi & Kredit

Dirancang dan dikembangkan oleh **[Dinar Permadi Yusup (permadi.dev)](https://permadi.dev)** — baca studi kasus lengkapnya di **[permadi.dev/id/projek/niluji](https://permadi.dev/id/projek/niluji)**.

Lisensi MIT © 2026–Sekarang [narr07](https://github.com/narr07).
