# Bank Soal Online

Folder ini disiapkan untuk bank soal yang diambil oleh aplikasi NILUJI langsung dari GitHub
(`raw.githubusercontent.com`). jsDelivr sempat dicoba tapi CDN-nya menolak (403 Forbidden)
semua file `.xlsx`/dokumen Office — gambar tetap boleh, hanya dokumennya yang diblokir.

Gunakan kode mata pelajaran dari tabel `subjects` sebagai nama folder, dalam bentuk URL-safe
huruf kecil. Contoh kode `IPAS` menjadi `ipas`, dan `B.Indo` menjadi `b-indo`.

```text
db-soal/
├── kelas_4/
│   └── ipas/
│       ├── 4-ipas.xlsx
│       └── gambar1.jpg
├── kelas_5/
└── kelas_6/
```

Nama file utama mengikuti pola `{nomor-kelas}-{kode-mapel}.xlsx`, misalnya `4-ipas.xlsx`.
Untuk kompatibilitas sementara, fitur juga masih menerima nama lama `soal.xlsx`.

Kolom Excel yang didukung:

`soal`, `jenis`, `pilihan_a`, `pilihan_b`, `pilihan_c`, `pilihan_d`, `kunci_jawaban`, `skor`, `nama_file_gambar`
