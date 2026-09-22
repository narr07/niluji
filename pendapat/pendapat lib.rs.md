
# 3. Satu hal yang perlu diperhatikan: lib.rs terlalu besar

Sekarang sudah:

±460 baris.



Untuk sekarang masih aman.

Tetapi kalau aplikasi berkembang:

* login guru
* hak akses
* laporan
* ranking
* analisis soal
* dashboard grafik
* backup
* audit

file ini bisa menjadi 1000+ baris.

Saran saya nanti pecah:

```
commands/
 |
 |-- exam_commands.rs
 |-- student_commands.rs
 |-- question_commands.rs
 |-- setting_commands.rs
```

Lalu:

```rust
mod commands;
```

---
# 8. Import file cukup lengkap

Sudah ada:

```rust
read_file_bytes
write_file_bytes
```



Ini mendukung:

* docx
* xlsx
* export

Namun perlu validasi:

Contoh:

Upload soal:

```
MAX SIZE 10 MB
Allowed:
.docx
.xlsx
.csv
```

Jangan menerima semua file.

---

# 10. Kekurangan terbesar: belum terlihat audit log

Untuk aplikasi ujian, saya sangat menyarankan:

Tambahkan:

```
audit_logs
```

Contoh:

| waktu | user      | aksi        |
| ----- | --------- | ----------- |
| 10:01 | Guru A    | buat ujian  |
| 10:05 | Siswa 001 | mulai ujian |
| 10:40 | Siswa 001 | submit      |

Karena jika terjadi komplain ujian:

"nilai saya berubah"

Anda punya bukti.

---
