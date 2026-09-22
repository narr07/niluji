# 8. Load soal

Setelah dapat ID:

```rust
for qid in &ids
```

mengambil:

* text
* type
* image
* option

Bagus.

Tetapi ada masalah performa:

Misal:

50 siswa

100 soal

maka query:

```
5000 query
```

karena loop.

Lebih optimal:

gunakan:

```sql
WHERE id IN (...)
```

sekali query.

---

# 9. Penyimpanan jawaban

Jawaban disimpan:

```
session_id
question_id
option_key
```

Ini sesuai db.rs.

Bagus.

Namun:

Belum terlihat:

* waktu menjawab
* perubahan jawaban
* history

Untuk ujian penting:

Tambahkan:

```
answered_at
```

Contoh:

```
Siswa A
10:01 pilih A
10:03 ubah B
10:10 submit
```

---

# 10. Essay handling

Ini cukup menarik.

Ada:

```rust
grade_essay_answer()
```

Nilai essay:

* tidak masuk auto score
* guru memberi nilai

Ini desain yang benar.

---

Namun saya melihat:

Saat essay:

```sql
option_key=''
```

dipakai untuk menyimpan.

Kurang ideal.

Lebih baik:

Tambah:

```
answer_text TEXT
```

Karena sekarang:

jawaban essay siswa tidak terlihat.

Yang disimpan hanya skor.

---
