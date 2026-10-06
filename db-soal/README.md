# Bank Soal Online

Folder ini adalah sumber bawaan fitur **Tarik Soal Online** di NILUJI. Isinya file Markdown hasil
fitur **Export Bank Soal** (Pengaturan → Fitur), satu file per jenis ujian.

```text
db-soal/
├── kelas_4/
│   ├── mtk/
│   │   ├── sts-matematika-2026-2027.md
│   │   └── gambar/
│   │       └── soal-213.jpg
│   └── ppkn/
│       └── sts-pkn-2026-2027.md
├── kelas_5/
└── kelas_6/
```

- Folder kelas: `kelas_4`, `kelas_5`, dst.
- Folder mapel: kode mapel huruf kecil (MTK → `mtk`, B.Indo → `b-indo`) supaya otomatis cocok
  dengan mapel di aplikasi. Kalau berbeda, sekolah memilih mapel tujuan secara manual saat impor.
- Gambar soal ada di subfolder `gambar/` di sebelah file `.md`.

## Dipakai di kecamatan lain

Operator kecamatan menyiapkan folder `db-soal` dengan struktur yang sama, lalu membagikannya
lewat salah satu cara berikut (sekolah memilih sumber yang sesuai di aplikasi):

| Cara bagikan | Sumber yang dipilih sekolah |
| --- | --- |
| Repo GitHub publik berisi folder `db-soal` | Repo GitHub lain |
| `db-soal.zip` di Google Drive / Dropbox (dibagikan "Siapa saja yang memiliki link") | Link file ZIP |
| `db-soal.zip` lewat flashdisk / WhatsApp | File ZIP lokal (tanpa internet) |

Panduan langkah demi langkah ada di aplikasi: Pengaturan → Tarik Soal Online → Panduan untuk
operator kecamatan.
