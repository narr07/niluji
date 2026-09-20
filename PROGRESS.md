# Progress Log — CBT Offline (Nuxtor)

Catatan ini dibuat supaya kerjaan bisa lanjut walau sesi Claude keputus (kuota habis dll).
Update terakhir: fitur "Jenis Soal" SUDAH SELESAI (backend + frontend), lint bersih, build
(`bun run generate`) sukses. Belum di-build ulang jadi .exe — lakukan `bun run tauri:build`
kalau mau exe/installer terbaru yang sudah termasuk fitur ini.

## Fitur "Jenis Soal" — SELESAI

**Latar belakang:** guru butuh cara bikin ujian baru (misal semester berikutnya) tanpa
harus mengganti/menghapus soal yang sudah ada untuk kelas+pelajaran yang sama. Solusinya:
setiap soal punya label "jenis" (misal "UTS Ganjil 2026", "Latihan Harian"), dan satu ujian
di Kelola Ujian menempel ke satu jenis tertentu — jadi ujian baru = jenis baru, soal lama
tidak diganggu.

### Backend — SUDAH SELESAI (compile, test masih perlu diverifikasi ulang)

- `src-tauri/src/db.rs`: tabel baru `question_types (id, name UNIQUE, description)` buat
  menyimpan daftar nama jenis (dikelola dari Pengaturan, sama seperti tabel `classes`).
  Kolom baru: `questions.jenis TEXT`, `exams.jenis TEXT` (via `ensure_column`).
- `src-tauri/src/settings.rs`: `QuestionTypeRecord` + `list_question_types`,
  `create_question_type`, `update_question_type`, `delete_question_type` (pola sama persis
  dengan `ClassRecord`/`list_classes_full` dkk). Sudah ada test-nya.
- `src-tauri/src/import.rs`: `QuestionSummary`, `QuestionInput`, `QuestionDetail` semua
  ditambah field `jenis: Option<String>`. `ImportOptions` ditambah `default_jenis`.
  `JENIS_COLS = ["jenis", "jenis_soal", "paket", "paket_soal", "tipe"]` buat baca kolom
  jenis dari file import (fallback ke `default_jenis` kalau kolom kosong/gak ada).
  `create_question`/`update_question`/`get_question`/`list_questions` semua sudah include
  jenis. Ada test baru `imports_questions_with_jenis_column_and_default`.
- `src-tauri/src/exams.rs`: `ExamSummary` + `create_exam`/`update_exam` ditambah param
  `jenis: Option<String>` (masuk di URUTAN setelah `class`, sebelum `title` — lihat
  signature-nya kalau mau connect dari frontend).
- `src-tauri/src/exam_session.rs`: `join()` sekarang filter soal yang diambil pakai
  `(?2 IS NULL OR class = ?2) AND (?3 IS NULL OR jenis = ?3)` — jadi kalau exam.jenis diisi,
  cuma soal dengan jenis yang SAMA yang dipakai buat ujian itu. Kalau exam.jenis kosong
  (ujian lama/legacy), tetap ambil semua soal kelas+pelajaran seperti sebelumnya (backward
  compatible, tidak ada migrasi data yang perlu dijalankan).
- `src-tauri/src/lib.rs`: command baru `list_question_types`, `create_question_type`,
  `update_question_type`, `delete_question_type` sudah didaftarkan. `create_exam`/
  `update_exam` command signature ikut berubah (tambah param `jenis`).

**Yang masih perlu dicek:** `cargo test` terakhir dijalankan di background (task belum
sempat dikonfirmasi selesai sebelum sesi terputus) — jalankan ulang
`cd src-tauri && cargo test --no-default-features --features desktop` untuk pastikan semua
hijau sebelum lanjut ke frontend, siapa tahu ada typo/signature mismatch yang kelewat.

### Frontend — SUDAH SELESAI

1. **Pengaturan**: tab ke-4 "Jenis Soal" ditambahkan (`app/components/pengaturan/JenisSoal.vue`,
   dipakai di `app/pages/pengaturan.vue`), CRUD lewat `list_question_types`/
   `create_question_type`/`update_question_type`/`delete_question_type`. Field: "Nama" +
   "Deskripsi" (opsional).

2. **Bank Soal**:
   - `FormModal.vue` & `ImportModal.vue` sekarang punya field "Jenis Soal" (USelectMenu,
     opsional), sumbernya prop `questionTypes: string[]`.
   - `app/pages/soal/[kelas]/[pelajaran].vue` fetch `list_question_types` dan teruskan ke
     kedua modal itu sebagai prop.

3. **Kelola Ujian** (`app/components/ujian/Form.vue`): field baru "Jenis Soal" — dropdown-nya
   difilter dari jenis yang SUDAH ADA soalnya untuk kombinasi kelas+pelajaran terpilih
   (computed `jenisItemsForClassAndSubject`, sumber data dari prop `questions` yang sekarang
   ikut bawa field `jenis`). Pilih jenis → `form.title` auto-terisi nama jenis itu (masih
   bisa diedit manual). `submit()` sudah kirim `jenis` ke `create_exam`/`update_exam`.
   `ujian/Table.vue` juga dapat kolom baru "Jenis Soal".

4. **Bug ekstra yang ketemu & sudah diperbaiki**: di `app/pages/siswa/index.vue`, variabel
   lokal `const open = ref(false)` (buat dropdown template) nge-shadow `import { open } from
   "@tauri-apps/plugin-dialog"` yang dipakai `pickAndImport()` buat buka file picker — kalau
   kepakai bakal crash ("open is not a function"). Sudah di-rename jadi `templateMenuOpen`.

**Verifikasi:** `cargo test` (9/9 pass), `bunx eslint app/pages app/components` (bersih,
cuma 1 error lama tak terkait di `cbt.vue`), `bun run generate` (build sukses). BELUM
di-build ulang jadi .exe — jalankan `bun run tauri:build` untuk exe/installer terbaru.

## Item lain yang sudah kelar sebelumnya (referensi cepat)

- Refactor semua halaman jadi komponen di `app/components/<section>/...`.
- ESLint diganti ke config yang lebih simpel (`withNuxt` + `@stylistic` minimal + tailwind
  lint), `<script>` sekarang selalu di atas `<template>`.
- Server pakai port 80 (fallback 8080), QR code login siswa di Dashboard.
- Breadcrumb ditambahkan di semua halaman drill-down (siswa/soal/hasil).
- Font diganti ke TikTok Sans (self-hosted lewat `@fontsource-variable/tiktok-sans`, tidak
  butuh internet saat build).
- Matching nama file gambar soal saat import sekarang case-insensitive + extension-optional
  (lihat `resolve_image` di `import.rs`).
- Siswa: bisa multi-select + hapus banyak sekaligus (checkbox + tombol "Hapus Terpilih").
- Data sekolah/kelas/pelajaran dikelola terpusat di halaman Pengaturan, bukan hardcode lagi.
- Build exe terakhir yang sukses: `src-tauri/target/release/bundle/nsis/Nuxtor_1.6.0_x64-setup.exe`
  (belum termasuk fitur Jenis Soal — perlu build ulang setelah fitur ini kelar).
- Data aplikasi (SQLite + uploads) TIDAK ikut ter-bundle ke exe, lokasinya di
  `%APPDATA%\com.nicolaspadari.nuxtor\` — perlu di-copy manual kalau pindah ke laptop lain.

## Lanjutan kalau sesi baru mulai lagi

Baca file ini dulu, cek `cargo test` masih hijau, terus lanjut ke bagian "Frontend — BELUM
DIKERJAKAN" di atas secara berurutan (Pengaturan → Bank Soal → Kelola Ujian).
