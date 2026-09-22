
# 9. Potensi masalah gambar

Tetapi:

Saat ini image disimpan:

```text
uploads/xxx.jpg
```

Kemungkinan.

Tidak terlihat ada:

* validasi ukuran
* validasi tipe file

Risiko:

Guru upload:

```text
virus.exe
```

atau:

```text
besar 500MB
```

Saran:

Batasi:

```
jpg
jpeg
png
webp
```

maks:

```
5 MB
```

---

# 10. Validasi soal masih kurang

Ini temuan penting.

Misalnya import:

```text
Soal:
Apa ibu kota Indonesia?

A:
B:
C:
D:
```

tanpa kunci.

Sekarang kemungkinan tetap masuk.

Karena:

```rust
unwrap_or_default()
```

Saran:

Untuk pilihan ganda:

wajib:

* minimal 2 opsi
* wajib kunci jawaban
* kunci harus ada dalam opsi

---

# 11. Masalah duplicate soal

Saat import:

langsung:

```sql
INSERT INTO questions
```

Jika guru import file sama dua kali:

hasil:

```
Soal 1
Soal 1
Soal 1
```

duplikat.

Saran:

Tambahkan:

```sql
question_hash
```

contoh:

```
sha256(question_text)
```

Saat import:

cek dulu.

---
