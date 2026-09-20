# Progress Log — CBT Offline (Niluji)

Catatan ini dibuat supaya kerjaan bisa lanjut walau sesi Claude keputus (kuota habis dll).

## Rebrand Nuxtor → Niluji — SELESAI

Proyek ini awalnya dari starter template "Nuxtor" (Nicola Spadari). Sudah di-rebrand penuh:

- `src-tauri/tauri.conf.json`: `productName` dan judul window jadi "Niluji", `version` disamakan
  ke `1.0.0` (sebelumnya `1.6.0`, tidak sinkron dengan `package.json`).
- `src-tauri/Cargo.toml`: `[package] name` jadi `niluji`, `[lib] name` jadi `niluji_lib`,
  `description`/`authors`/`repository` diarahkan ke narr07/niluji. `src-tauri/src/main.rs`
  ikut diupdate (`niluji_lib::run()`). Binary hasil build sekarang `niluji.exe`, bukan
  `nuxtor.exe`. Diverifikasi: `cargo check` dan `cargo test` (13/13) tetap hijau setelah rename.
- **Sengaja TIDAK diubah**: `identifier` di `tauri.conf.json` (masih
  `com.nicolaspadari.nuxtor`). Field ini menentukan folder data aplikasi
  (`%APPDATA%\com.nicolaspadari.nuxtor\`, tempat SQLite + uploads hidup). Mengubahnya akan
  membuat aplikasi menulis ke folder BARU dan terlihat seperti "semua data hilang" padahal
  cuma pindah lokasi. Kalau suatu saat memang mau diubah, perlu salin manual folder data lama
  ke folder baru dulu.
- `README.md` ditulis ulang total dari boilerplate starter template jadi deskripsi NILUJI yang
  sebenarnya (arsitektur, fitur, tech stack, catatan data). `README.zh-CN.md` (terjemahan
  boilerplate lama) belum disentuh — kalau tidak akan dirawat, sebaiknya dihapus.
- Halaman baru **Pengaturan > Tentang** (`app/pages/pengaturan/tentang.vue` +
  `app/components/pengaturan/Tentang.vue`): menampilkan nama & versi aplikasi (dibaca live
  lewat `getName()`/`getVersion()` dari `@tauri-apps/api/app`, bukan di-hardcode), info
  pembuat, dan link repo. Ditaruh di tab kedua Pengaturan, tepat setelah "Data Sekolah".

## Fitur "Tarik Soal Online" — Impor ke Bank Soal SUDAH BISA (branch `feature/tarik-soal-online`)

- **Bug ditemukan & diperbaiki: skema kolom Excel nyata beda total dari rencana awal.** Setelah
  fetch berhasil (lihat fix jsDelivr→raw.githubusercontent.com di bawah), SEMUA baris tampil
  invalid ("teks soal kosong" dll) padahal file ada isinya. Diverifikasi dengan mengunduh &
  membaca langsung file `.xlsx` yang sudah diupload user: header aslinya
  `no, question, image, answer_a..d, correct_answer, score` (PG) dan `no, question, image, score`
  (esai) — BUKAN `soal, jenis, pilihan_a..d, kunci_jawaban, skor, nama_file_gambar` seperti di
  spek awal. `parseRows()` di `useBankSoalOnline.ts` sudah dipetakan ulang ke skema nyata ini
  (bentuk internal `BankSoalOnlineRow` tetap Bahasa Indonesia, jadi komponen UI tidak perlu
  berubah banyak).
- File esai (`{kelas}-{kode}-esai.xlsx`) sebelumnya tidak pernah ikut diambil sama sekali —
  sekarang `fetchBankSoal()` mengambil file PG (wajib) DAN file esai (opsional, boleh tidak
  ada), digabung jadi satu daftar dengan field `tipe: "pg" | "esai"` dan `id` unik per baris
  (perlu field `id` terpisah dari `rowNumber` karena rowNumber PG dan esai bisa sama-sama "2"
  kalau dihitung dari file masing-masing — dulu ini bisa bikin tabrakan `:key`/seleksi di UI).
  Validasi esai hanya mensyaratkan teks soal terisi (tidak butuh pilihan/kunci jawaban).
- **Fitur baru: tombol "Impor ke Bank Soal"** menggantikan tombol lama yang cuma
  `emit("selected", ...)` tanpa efek nyata. Sekarang benar-benar memanggil `create_question`
  per baris terpilih, dengan field "Jenis Soal" (wajib diisi manual, mis. "UTS Semester 1")
  karena sumber online tidak punya kolom jenis. Gambar diunduh sekali per
  `pathRelatifGambar` (bukan per baris — sesuai catatan awal bahwa satu gambar bisa dipakai
  berulang oleh banyak soal) lewat command Rust baru `save_question_image_bytes` di
  `uploads.rs`/`lib.rs` (menyimpan bytes yang sudah difetch di frontend, beda dari
  `save_image` lama yang menyalin dari path lokal). Baris yang sudah diimpor ditandai
  badge "Diimpor" dan checkbox-nya dikunci supaya tidak ke-import dobel.
- Berikutnya: commit semua perubahan branch `feature/tarik-soal-online` (composable, komponen,
  `uploads.rs`, `lib.rs`, capability, package.json, rebrand) lalu push branch-nya untuk dites,
  baru merge ke `main` setelah user konfirmasi impor beneran masuk ke Bank Soal dengan benar.

## Fitur "Tarik Soal Online" — riwayat awal

- Repo remote sudah diarahkan ke `https://github.com/narr07/niluji.git`. Kode aplikasi (commit
  awal) dan folder `db-soal` (bank soal PAI + Bahasa Indonesia kelas 4-6) sudah di-push ke `main`.
- Dependensi `xlsx` dan `@tauri-apps/plugin-http` sudah ditambahkan.
- Tauri HTTP plugin sudah didaftarkan dan capability `http:default` sudah diaktifkan, di-scope ke
  `https://raw.githubusercontent.com/*`.
- **Bug ditemukan & diperbaiki: jsDelivr memblokir `.xlsx`.** Rencana awal pakai
  `cdn.jsdelivr.net/gh/...` sebagai sumber, tapi terverifikasi CDN itu mengembalikan
  `403 Forbidden` untuk SEMUA file `.xlsx` (dicoba 2 file berbeda, konsisten 403), sementara
  `.jpg` di folder yang sama dan `README.md` sama-sama 200 OK — ini pembatasan jsDelivr terhadap
  dokumen Office, bukan soal nama file/case/ukuran. `raw.githubusercontent.com` melayani file
  `.xlsx` yang sama dengan 200 OK, jadi `CDN_BASE` di `useBankSoalOnline.ts` dan scope capability
  dipindah ke situ. Kalau nanti mau balik ke jsDelivr untuk alasan performa, gambar tetap aman
  lewat jsDelivr — cukup dokumen Excel-nya yang wajib lewat raw.githubusercontent.com (atau host
  lain yang tidak memblokir `.xlsx`).
- Bug lain yang ditemukan & diperbaiki: halaman `app/pages/pengaturan/tarik-soal-online.vue`
  memanggil komponennya sebagai `<TarikSoalOnline>`, padahal Nuxt me-prefix komponen di
  `app/components/pengaturan/` jadi `PengaturanTarikSoalOnline` (lihat pola di
  `PengaturanSekolah`/`PengaturanKelas`/`PengaturanPelajaran`). Nama yang salah bikin komponennya
  tidak ter-resolve, dan render awal crash `RangeError: Maximum call stack size exceeded` yang
  akhirnya muncul ke user sebagai halaman 500 ("useHead() was called without provide context") —
  pesan itu cuma efek samping dari crash-nya, bukan sebab aslinya. Sudah diganti ke
  `<PengaturanTarikSoalOnline>` dan dikonfirmasi `bun run generate` sukses tanpa error.
- Composable `app/composables/useBankSoalOnline.ts` mengambil Excel dari `narr07/niluji`,
  mem-parsing kolom bank soal, menambahkan URL gambar, dan menandai baris invalid tanpa
  memasukkannya ke database.
- Halaman baru Pengaturan > Tarik Soal Online sudah dibuat dengan dropdown kelas 4–6,
  dropdown mapel dari `subjects.code`, preview tabel, checkbox soal valid, dan fallback
  nama file `soal.xlsx` untuk kompatibilitas dengan nama lama.
- Sebelum mengambil file, halaman mengecek koneksi ke sumbernya; status terlihat di UI, dapat
  di-refresh manual, dan tombol tarik soal terkunci saat aplikasi offline atau host tidak dapat
  dijangkau.
- Konvensi nama file final: `{nomor-kelas}-{kode-mapel}.xlsx` untuk soal PG dan
  `{nomor-kelas}-{kode-mapel}-esai.xlsx` untuk soal esai (dua file terpisah per kelas+mapel).
  Nama file gambar TIDAK diubah/dinormalisasi karena harus tetap sama persis dengan nilai
  `nama_file_gambar` yang dirujuk di Excel.
- Struktur `db-soal/kelas_4`, `db-soal/kelas_5`, `db-soal/kelas_6` sudah disiapkan dan sudah ada
  isi nyata untuk PAI dan Bahasa Indonesia (kelas 4-6), sudah di-push ke `main`.
- Verifikasi lokal: lint file fitur bersih, `cargo check --no-default-features --features desktop`
  sukses, dan `bun run generate` sukses. Build menampilkan peringatan chunk besar bawaan bundle
  serta warning Browserslist lama, bukan error fitur.
- `bunx tauri build --debug --no-bundle` sukses; executable uji ada di
  `src-tauri/target/debug/nuxtor.exe`.
- Berikutnya: commit sisa perubahan kode fitur ini (composable, komponen, halaman, Cargo.toml,
  capability, package.json — masih uncommitted di branch `feature/tarik-soal-online` saat catatan
  ini ditulis) lalu push branch-nya, retest fetch dengan host baru (raw.githubusercontent.com)
  sampai preview tabel benar-benar tampil, baru merge ke `main`.

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
