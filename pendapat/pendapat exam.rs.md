
# 5. Ada masalah kecil: validasi input

Sekarang:

```rust
title.trim()
token.trim()
```

saja.

Belum ada validasi:

## Durasi

Misalnya:

```text
duration = -999
```

masih bisa masuk.

Saran:

```rust
if duration <=0 {
 return Err(...)
}
```

---

## Window waktu

Sekarang:

bisa saja:

```
scheduled_at:
10:00

window_end:
08:00
```

Artinya ujian selesai sebelum dimulai.

Perlu:

```rust
if window_end < scheduled_at
```

---

# 6. Token ujian

Database sebelumnya:

```sql
token UNIQUE
```

Di create:

langsung insert.

Bagus karena SQLite mencegah duplikasi.

Tetapi:

pesan error masih generic:

```rust
e.to_string()
```

Misalnya guru membuat token sama:

hasil:

```
UNIQUE constraint failed
```

Kurang user friendly.

Lebih baik:

```rust
Token ujian sudah digunakan
```

---
