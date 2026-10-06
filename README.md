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

### Bank Soal

- Soal disusun per kelas → mata pelajaran → **jenis ujian** (paket soal, mis.
  UTS/UAS/Latihan), pilihan ganda dan esai, dengan gambar di soal maupun di tiap opsi.
- Kartu kelas menampilkan ringkasan: berapa mapel yang sudah punya jenis ujian
  (dengan progress bar), jumlah jenis ujian, dan jumlah soal.
- Tombol **Buat Jenis Ujian** ada di setiap level Bank Soal. Kelas lalu mata pelajaran
  wajib dipilih dulu; kalau dibuka dari halaman kelas/mapel, pilihannya sudah terisi.
- Menghapus jenis ujian ikut menghapus soal di dalamnya, hanya pada cakupan yang
  dipilih — soal di kelas/mapel lain yang memakai nama jenis yang sama tetap aman.
- Import soal dari naskah Word (`.docx` + kunci jawaban), CSV/Excel, atau Markdown.

### Tarik Soal Online

Ambil bank soal (format Markdown hasil fitur Export) lalu periksa sebelum diimpor.
Sumbernya bisa dipilih:

| Sumber | Keterangan |
| --- | --- |
| NILUJI (bawaan) | Folder `db-soal/` di repo ini |
| Repo GitHub lain | Repo dari operator kecamatan |
| Link file ZIP | `db-soal.zip` di Google Drive, Dropbox, atau link unduhan lain |
| File ZIP lokal | Dari flashdisk/WhatsApp — **tanpa internet** |

Kelas dan mapel dibaca dari isi sumbernya; mapel dicocokkan otomatis ke mapel di
aplikasi dan bisa dipilih manual kalau namanya berbeda. Ada panduan untuk operator
kecamatan di dalam aplikasi. Struktur foldernya dijelaskan di [db-soal/README.md](db-soal/README.md).

### Siswa

- Tambah manual, import CSV/Excel (dengan preview yang bisa diedit), atau **Tarik
  Data Siswa** dari Google Sheets.
- Tarik Data Siswa memakai sheet bawaan, atau link Google Sheet lain dari operator
  kecamatan — lengkap dengan panduan & template kolom `NISN | NAMA SISWA | KELAS |
  NAMA SEKOLAH`.

### Ujian & Hasil

- Kelola Ujian: jadwalkan ujian per kelas/mata pelajaran, atur jendela akses dan
  durasi pengerjaan per siswa secara terpisah, token otomatis.
- Sesi ujian PG dan Esai terpisah: siswa menyelesaikan Pilihan Ganda dulu, lalu
  esai; nilai esai dinilai manual oleh guru dan tidak masuk nilai otomatis.
- Hasil Ujian hanya menampilkan ujian yang **sudah terlaksana**: ada di Kelola Ujian,
  soalnya ada di Bank Soal, dan waktunya sudah dimulai (status *sedang berlangsung* /
  *selesai*). Kartu kelas menampilkan progres pengerjaan, jumlah ujian, dan rata-rata
  nilai PG.
- Progres real-time per siswa, penilaian esai inline, analitik (tingkat kesulitan soal
  & peringkat siswa), dan export ke Excel/zip.
- Login siswa via QR code, cocok untuk perangkat siswa yang berbeda-beda.

### Lainnya

- Urutan kelas & mapel mengikuti yang diatur di Pengaturan, di semua halaman.
- Unduhan template disimpan ke folder Downloads, dengan tombol **Buka Folder** di notifikasinya.
- Backup database otomatis tiap aplikasi dibuka (14 backup terakhir disimpan).

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

> Build rilis ikut membuat file tanda tangan untuk update otomatis, jadi butuh kunci privat:
> set `TAURI_SIGNING_PRIVATE_KEY` ke isi file `~/.tauri/niluji.key` (dan
> `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` kosong) sebelum menjalankan perintah di atas.

### Debug Build

```sh
bun run tauri:build:debug
```

Mengaktifkan akses console di dalam aplikasi yang sudah di-bundle.

---

## Rilis & Update Otomatis

Aplikasi yang sudah terpasang memeriksa `latest.json` di GitHub Release terbaru setiap dibuka,
lalu menawarkan update (ditahan selama ada ujian berlangsung atau siswa yang login).

Cara merilis versi baru:

1. Naikkan versi: `bun run bump` (mengubah `package.json`, `tauri.conf.json`, `Cargo.toml`).
2. Tambahkan entri di `app/components/pengaturan/Changelog.vue`, lalu tulis ringkasannya di
   `RELEASE_NOTES.md` — teks ini yang tampil di jendela update di komputer sekolah.
3. Commit, lalu push tag dengan versi yang sama: `git tag v1.1.0 && git push origin v1.1.0`.
4. GitHub Actions (`.github/workflows/release.yml`) membangun installer Windows,
   menandatanganinya, dan membuat Release berisi installer + `latest.json`.

Syarat sekali saja: secret `TAURI_SIGNING_PRIVATE_KEY` di repo GitHub berisi kunci privat.

> ⚠️ Simpan cadangan `~/.tauri/niluji.key` di tempat aman. Kalau kunci ini hilang, aplikasi
> yang sudah terpasang tidak bisa menerima update otomatis lagi dan harus dipasang ulang manual.

---

## Catatan Data

Database SQLite dan file upload TIDAK ikut ter-bundle ke dalam installer/exe.
Lokasinya ada di `%APPDATA%\com.narr07.niluji\` (Windows) — perlu disalin manual kalau
memindahkan instalasi ke perangkat lain. Update tidak menyentuh folder ini.

Sampai v1.0.1 foldernya bernama `com.nicolaspadari.nuxtor`; v1.1.0 menyalin isinya otomatis
ke folder baru saat pertama dibuka (folder lama dibiarkan sebagai cadangan).

## Catatan Pengembangan

- Permission Tauri harus dideklarasikan di `src-tauri/capabilities/main.json`.
- Tauri API di-auto-import lewat `app/modules/tauri.ts`; menambah plugin Tauri baru
  perlu update modul ini.

---

## License

MIT © 2026–Present [narr07](https://github.com/narr07)

<sub>Berbasis starter template <a href="https://github.com/NicolaSpadari/nuxtor">Nuxtor</a> oleh Nicola Spadari.</sub>
