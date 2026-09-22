
# 4. LoginState

Ini:

```rust
pub type LoginState =
Arc<Mutex<HashMap<String, OnlineStudent>>>;
```

Fungsinya:

dashboard guru melihat:

```
Siswa online:
- Ahmad
- Budi
- Siti
```

Bagus.

Namun:

## Kekurangan

Data hanya RAM.

Jika aplikasi restart:

```
server mati
↓
login siswa hilang
```

Komentar kode juga menjelaskan:

> reset pada app restart

Untuk CBT masih acceptable.

Tetapi untuk ujian resmi saya sarankan:

buat tabel:

```
active_sessions
```

di SQLite.

---

# 8. Upload file

Ada:

```
/uploads
```

dengan:

```rust
ServeDir::new(&uploads_dir)
```

Ini perlu perhatian.

Karena upload soal:

misalnya:

```
soal1.png
diagram.png
```

langsung tersedia.

Risiko:

Jika user bisa upload:

```
.html
.js
.svg
```

bisa menjadi XSS.

Saran:

batasi:

```
png
jpg
jpeg
webp
```

dan rename file.

---
