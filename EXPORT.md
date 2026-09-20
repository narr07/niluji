# Catatan Export Aplikasi CBT

Ada 2 target export: **Windows (.exe/installer)** untuk laptop guru (server + admin),
dan **Android (.apk)** untuk HP siswa (client ujian). Server (axum + SQLite) tetap
jalan di laptop guru; APK Android hanya menggantikan peran browser di sisi siswa,
tetap konek ke laptop guru lewat WiFi/hotspot yang sama — jadi APK **tidak menjalankan
server sendiri**, hanya membuka halaman `/cbt` yang sudah ada.

## 1. Export ke Windows (.exe)

Perintah:

```
bun run tauri:build          # release (dioptimasi, ukuran kecil, lebih lama build-nya)
bun run tauri:build:debug    # debug (build cepat, buat tes cepat, jangan dipakai buat sekolah)
```

Perintah ini otomatis menjalankan `bun run generate` dulu (build frontend), baru
compile Rust-nya. Hasilnya ada di:

- `src-tauri/target/release/nuxtor.exe` — exe langsung, bisa di-copy & jalankan tanpa install.
- `src-tauri/target/release/bundle/msi/*.msi` dan `bundle/nsis/*.exe` — installer, ini
  yang paling enak dibagikan ke laptop guru lain (tinggal klik, ke-install ke Start Menu).

Prasyarat sudah terpenuhi di komputer ini (Rust toolchain, Bun, dependency ter-install).
Ikon (`lucide:...`) sudah dibikin lokal lewat paket `@iconify-json/lucide`, jadi build
**tidak butuh internet** untuk ambil ikon — penting karena aplikasi ini harus bisa
dibangun ulang di komputer sekolah yang mungkin gak ada koneksi internet, asalkan
`node_modules` dan cache Cargo (`~/.cargo`) sudah pernah terisi sebelumnya (misal
di-copy dari komputer ini, atau install sekali saat masih ada internet).

## 2. Export ke Android (.apk)

### Prasyarat (baru perlu disiapkan sekali di komputer build)

1. Install Android Studio (untuk Android SDK + NDK + emulator).
2. Set environment variable `ANDROID_HOME` / `NDK_HOME` (biasanya otomatis kalau
   install lewat Android Studio, tinggal restart terminal).
3. Tambah target compile Rust untuk Android:
   ```
   rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
   ```
4. Project Android sebenarnya sudah ada (folder `src-tauri/gen/android`), jadi tidak
   perlu `tauri android init` lagi kecuali foldernya dihapus.

### Build

```
bun run tauri:dev:android      # jalanin di emulator/HP buat tes
bun run tauri:build:android    # build APK/AAB
```

Hasil APK ada di:
`src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release.apk`

Untuk APK **release** (bukan debug), Android mewajibkan penandatanganan (signing key).
Kalau belum ada keystore, bikin dulu:

```
keytool -genkey -v -keystore cbt-release.keystore -alias cbt -keyalg RSA -keysize 2048 -validity 10000
```

lalu isi info keystore-nya di `src-tauri/gen/android/keystore.properties` (dibuat manual,
lihat dokumentasi Tauri Mobile untuk formatnya). Tanpa signing, hanya bisa build APK debug
(`app-universal-debug.apk`) — cukup buat testing, jangan dipakai beneran ke HP siswa
banyak-banyak karena tiap install butuh "Sumber tidak dikenal" diaktifkan manual.

## 3. Soal "biar siswa tidak bisa mencontek" (kiosk mode) — CATATAN, BELUM DIKERJAKAN

Install APK saja **belum otomatis** mengunci siswa di dalam aplikasi — siswa tetap bisa
tekan tombol Home / Recent Apps lalu buka browser/aplikasi lain kalau tidak ada
pengaman tambahan. Ada 2 opsi:

**Opsi A — tanpa ubah kode sama sekali (langsung bisa dipakai sekarang):**
Pakai fitur bawaan Android "Pin Aplikasi" (App Pinning / Screen Pinning).
Aktifkan di tiap HP siswa sebelum ujian: *Setelan > Keamanan > Pin Aplikasi* (nama
menu beda-beda tergantung merk HP), lalu saat aplikasi CBT dibuka, tekan tombol
Recent Apps dan pilih "Pin". Layar terkunci ke 1 aplikasi itu saja sampai di-lepas
pakai kombinasi tombol tertentu (biasanya tahan tombol Back + Overview). Guru yang
perlu set ini manual di tiap HP sebelum ujian dimulai.

**Opsi B — kunci otomatis dari dalam aplikasi (butuh coding tambahan, belum dibuat):**
Tambah kode native Android (Kotlin, lewat plugin Tauri custom) yang otomatis
mengaktifkan Lock Task Mode + fullscreen + disable tombol Back begitu halaman ujian
(`/cbt`, stage `exam`) terbuka, dan melepas kuncinya lagi begitu submit selesai. Ini
kerjaan tambahan di luar plugin Tauri standar — bisa saya kerjakan kalau memang mau
dipakai beneran, tapi butuh Android Studio + testing di HP fisik.

## Ringkasan Perintah

| Target | Perintah | Output |
|---|---|---|
| Windows (release) | `bun run tauri:build` | `src-tauri/target/release/bundle/{msi,nsis}/` |
| Windows (cepat, buat tes) | `bun run tauri:build:debug` | `src-tauri/target/debug/nuxtor.exe` |
| Android (tes) | `bun run tauri:dev:android` | jalan langsung di emulator/HP |
| Android (APK) | `bun run tauri:build:android` | `src-tauri/gen/android/app/build/outputs/apk/` |

## Build Terakhir (Windows)

Installer siap pakai, tinggal copy ke laptop lain dan jalankan (tidak butuh internet):

```
E:\website\nuxt ecosystem\cbt ofline\nuxtor-main\src-tauri\target\release\bundle\nsis\Nuxtor_1.6.0_x64-setup.exe
```