# NILUJI Siswa (Android)

Aplikasi Android untuk siswa. Aplikasi ini **bukan** server ujian. Server tetap berjalan di laptop pengawas yang menjalankan NILUJI Desktop, dan aplikasi ini hanya menyambung ke sana lalu membuka portal ujian `/cbt`.

## Fitur

- **Pindai QR kartu peserta** atau ketik alamat server (misalnya `192.168.1.10` atau `192.168.1.10:8080`).
- **Cek server sebelum masuk.** Aplikasi memanggil `/api/health` dan hanya melanjutkan bila jawabannya berasal dari server NILUJI.
- **Hanya alamat jaringan lokal** (192.168.x.x, 10.x.x.x, 172.16–31.x.x, 100.64–127.x.x, nama `.local`) yang diterima.
- **Terkunci ke satu server.** Selama ujian, WebView hanya boleh memuat halaman dari server yang dipilih. Situs lain diblokir.
- **Portal tidak bisa mengendalikan aplikasi.** Command aplikasi hanya bisa dipanggil dari layar awal bawaan, bukan dari halaman server.
- **Mode ujian di Android:**
  - tangkapan layar dan rekam layar diblokir (`FLAG_SECURE`),
  - layar tetap menyala,
  - bilah sistem disembunyikan dan aplikasi meminta disematkan (screen pinning),
  - tombol kembali meminta konfirmasi sebelum keluar.

## Batasan yang perlu diketahui

- Penyematan memakai fitur bawaan Android. Di HP biasa Android menampilkan konfirmasi, dan siswa bisa menolaknya atau melepasnya dengan kombinasi tombol sistem. Aplikasi ini mempersulit, tetapi tidak menggantikan pengawasan.
- Portal ujian memakai HTTP di jaringan lokal. Gunakan WiFi ujian yang memakai password.
- Belum ada versi iOS.

## Pengembangan

Kebutuhan: Bun 1.3.6, Rust, JDK 17, Android SDK + NDK, dan target Rust Android (`rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android`).

```bash
bun install
bun run lint
bun run typecheck
bun run test:rust
bun run clippy
bun run desktop:dev        # uji layar awal dan alur di Windows (tanpa pemindai QR)
bun run android:dev        # emulator atau HP via USB
```

## Build APK rilis

APK rilis ditandatangani dengan keystore yang **tidak** disimpan di repositori. Buat sekali:

```bash
keytool -genkeypair -keystore niluji-siswa.jks -alias niluji-siswa -keyalg RSA -keysize 4096 -validity 10000
```

Lalu buat berkas `keystore.properties` di luar repositori:

```properties
storeFile=E:/lokasi/niluji-siswa.jks
keyAlias=niluji-siswa
password=<password keystore>
```

Arahkan build ke berkas itu dan jalankan:

```bash
NILUJI_SISWA_KEYSTORE_PROPERTIES=E:/lokasi/keystore.properties bun run tauri android build --apk --target aarch64 --target armv7 --target x86_64
```

Hasilnya ada di `src-tauri/gen/android/app/build/outputs/apk/universal/release/`.

Simpan keystore dan password-nya baik-baik. Pembaruan APK harus ditandatangani dengan keystore yang sama, kalau tidak siswa harus menghapus aplikasi lama sebelum memasang yang baru.
