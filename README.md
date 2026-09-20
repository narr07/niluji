<p align="center">
  <img width="150" src="./public/logo.png" alt="NILUJI Logo">
</p>

<h1 align="center">NILUJI</h1>

<p align="center">
  Platform CBT (Computer-Based Test) offline untuk sekolah dasar, dibangun dengan
  <a href="https://nuxt.com">Nuxt 4</a> dan <a href="https://v2.tauri.app">Tauri 2</a>.
  <br />
  Satu aplikasi desktop untuk guru/admin, satu halaman ujian yang bisa dibuka siswa
  dari browser mana pun di jaringan lokal (WiFi/hotspot) yang sama — sepenuhnya
  tanpa internet.
</p>

<p align="center">
  <img src="https://img.shields.io/github/package-json/v/narr07/niluji" />
  <img src="https://img.shields.io/github/license/narr07/niluji" />
</p>

---

## Arsitektur Singkat

- Aplikasi Tauri (guru/admin) menjalankan server HTTP (axum) tertanam di dalamnya.
- Server itu melayani dashboard admin sekaligus halaman ujian siswa lewat IP lokal
  (mis. `http://192.168.x.x`), jadi siswa cukup buka browser di HP/laptop mereka —
  tidak perlu install apa pun.
- Data (soal, siswa, hasil ujian) disimpan di SQLite lokal, tidak ada dependensi
  cloud untuk menjalankan ujian.

## Fitur Utama

- Bank Soal: kelola soal per kelas, mata pelajaran, dan "jenis" (paket soal, mis.
  UTS/UAS/Latihan) — pilihan ganda dan esai.
- **Tarik Soal Online**: opsional, mengambil template soal `.xlsx` dari repo GitHub
  publik (lewat `raw.githubusercontent.com`) untuk diperiksa & diimpor ke bank soal
  lokal — tetap berjalan sepenuhnya offline setelah diimpor.
- Kelola Ujian: jadwalkan ujian per kelas/mata pelajaran, atur jendela akses dan
  durasi pengerjaan per siswa secara terpisah.
- Sesi ujian PG dan Esai terpisah: siswa menyelesaikan Pilihan Ganda dulu, lalu
  esai; nilai esai digrading manual oleh guru dan tidak masuk kalkulasi otomatis.
- Hasil Ujian: progres real-time per siswa, grading esai inline, serta dashboard
  analitik (tingkat kesulitan soal & peringkat siswa) dengan grafik.
- Login siswa via QR code, cocok untuk perangkat siswa yang berbeda-beda.

## Tech Stack

- Nuxt v4 + Nuxt UI v4 + Tailwind CSS v4
- Tauri v2 (Rust) + SQLite (`rusqlite`) + axum (server HTTP tertanam)
- TypeScript, ESLint
- Auto imports (termasuk Tauri API)

---

## Setup

> 🚀 Bun adalah package manager utama proyek ini.

```sh
bun install

# Mode pengembangan (Nuxt + Tauri sekaligus)
bun run tauri:dev
```

> ⚠️ Nuxt SSR dimatikan (`ssr: false`) karena Tauri berperan sebagai backend.
> Routing, layout, middleware, dan composable Nuxt lain tetap berfungsi normal.

---

## Build

```sh
bun run tauri:build
```

Output ada di `src-tauri/target/release/bundle/`.

### Debug Build

```sh
bun run tauri:build:debug
```

Mengaktifkan akses console di dalam aplikasi yang sudah di-bundle.

---

## Catatan Data

Database SQLite dan file upload TIDAK ikut ter-bundle ke dalam installer/exe.
Lokasinya ada di `%APPDATA%\<identifier>\` (Windows) — perlu disalin manual kalau
memindahkan instalasi ke perangkat lain.

## Catatan Pengembangan

- Permission Tauri harus dideklarasikan di `src-tauri/capabilities/main.json`.
- Tauri API di-auto-import lewat `app/modules/tauri.ts`; menambah plugin Tauri baru
  perlu update modul ini.
- Progres pengembangan fitur dicatat di `PROGRESS.md`.

---

## License

MIT © 2026–Present [narr07](https://github.com/narr07)

<sub>Berbasis starter template <a href="https://github.com/NicolaSpadari/nuxtor">Nuxtor</a> oleh Nicola Spadari.</sub>
