# Tes nuxt-erapor

Tes logika (tanpa membuka aplikasi). Jalankan dari folder `nuxt-erapor`:

| File | Yang diuji | Butuh |
|---|---|---|
| `sync.test.ts` | Tes koneksi + tarik data Dapodik asli | Dapodik menyala, `DAPODIK_TOKEN`, `DAPODIK_NPSN` |
| `rapor.test.ts` | Susun pembelajaran SD, nilai, Excel nilai | `ERAPOR_DB_COPY` = **salinan** `erapor.sqlite` yang sudah sinkron |
| `tp.test.ts` | Import Excel TP tanpa dobel | `ERAPOR_DB_COPY` |
| `tpfmt.test.ts` | Baca format f_tp e-Rapor + template sendiri | – |
| `guru.test.ts` | Laptop guru: tarik → isi → kirim | `ERAPOR_TEST_DIR` (lihat bawah) |

```bash
DAPODIK_TOKEN=xxxx DAPODIK_NPSN=12345678 bun test tests/sync.test.ts
```

Alur sinkron admin ↔ guru diuji berantai bersama tes Rust di `src-tauri/src/sesi.rs`
(salinan database admin di `$ERAPOR_TEST_DIR/admin/erapor.sqlite`, sudah checkpoint WAL):

```bash
cd src-tauri && ERAPOR_TEST_DIR=... cargo test --lib a_dump_paket -- --ignored   # paket untuk guru
cd .. && ERAPOR_TEST_DIR=... bun test tests/guru.test.ts                          # guru isi & kirim
cd src-tauri && ERAPOR_TEST_DIR=... cargo test --lib b_terima_kiriman -- --ignored # admin terima
CF_EXE=path/cloudflared.exe cargo test --lib c_tunnel_nyata -- --ignored          # tunnel sungguhan (internet)
```

Jangan pernah arahkan tes ke database asli di AppData — selalu pakai salinan.
