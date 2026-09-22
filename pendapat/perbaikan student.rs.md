
# 11. CRUD siswa

Sudah lengkap:

```rust
create_student()
update_student()
delete_student()
```

Namun:

## Delete langsung

Saat ini:

```sql
DELETE FROM students
```

Risiko:

Guru tidak sengaja menghapus siswa yang punya:

* nilai ujian
* histori ujian

Saran:

Gunakan:

```text
soft delete
```

misalnya:

```
active = 0
```

---
