# Laporan Analisis Teknis & Evaluasi Arsitektur Platform NILUJI

Dokumen ini memuat audit teknis komprehensif terhadap platform **NILUJI** (v1.0.1), mencakup arsitektur sistem, efisiensi tumpukan teknologi (*tech stack*), evaluasi fitur, analisis kelebihan dan kekurangan, serta rekomendasi mitigasi risiko dan pengembangan lanjutan.

---

## 1. Ringkasan Eksekutif

**NILUJI** adalah platform Computer-Based Testing (CBT) dan e-Rapor sekolah berbasis *offline-first*. Aplikasi ini dirancang untuk mengatasi hambatan infrastruktur sekolah di Indonesia—khususnya ketidakstabilan koneksi internet publik dan kerumitan konfigurasi server konvensional (seperti XAMPP / Apache / MySQL / PHP).

Model operasional Niluji menggabungkan:
1. **Sisi Proktor/Guru:** Aplikasi desktop native berbasis **Tauri 2** (Rust) dan antarmuka **Nuxt 4 + Nuxt UI**.
2. **Sisi Siswa/Klien:** Server web ringan terintegrasi (**Axum**) yang langsung melayani peserta ujian di jaringan area lokal (LAN/WiFi) tanpa memerlukan server eksternal atau koneksi internet luar.
3. **Penyimpanan:** Database relasional **SQLite** lokal dengan mode *Write-Ahead Logging* (WAL).

---

## 2. Struktur Arsitektur & Spesifikasi Teknologi

```
+--------------------------------------------------------------------------+
|                        APLIKASI DESKTOP PROKTOR                          |
|                                                                          |
|  +---------------------------+       +--------------------------------+  |
|  |       Frontend Nuxt 4     |       |          Backend Rust          |  |
|  |  (Nuxt UI, Vue 3, Tiptap) | <===> |            (Tauri 2)           |  |
|  +---------------------------+  IPC  +---------------+----------------+  |
|                                                      |                   |
|                                      +---------------+----------------+  |
|                                      |      Server HTTP (Axum)        |  |
|                                      |      Port 80 / 8080+           |  |
|                                      +---------------+----------------+  |
|                                                      |                   |
|                                            +---------+----------+        |
|                                            | SQLite (cbt.sqlite)|        |
|                                            |   Mode: WAL + Norm |        |
|                                            +--------------------+        |
+------------------------------------------------------|-------------------+
                                                       |
                         Jaringan Area Lokal (WiFi/LAN)| HTTP:80
                                                       v
                               +--------------------------------+
                               |     PERANGKAT SISWA (/cbt)     |
                               | (Browser HP / Tablet / Laptop) |
                               +--------------------------------+
```

### Detail Tumpukan Teknologi (Tech Stack)

| Lapisan | Komponen / Pustaka | Peran Teknis |
| :--- | :--- | :--- |
| **Desktop Shell** | Tauri v2.9.5 | Runtime desktop berbasis WebView2 (Windows), konsumsi memori rendah, IPC aman |
| **Frontend Framework** | Nuxt v4.3.0 + Vue v3.5 | SPA client-side (`ssr: false`) dengan arsitektur modular modern |
| **Komponen UI** | `@nuxt/ui` v4.11 + Tailwind CSS | Komponen berbasis Radix/Reka UI primitives, mendukung dark/light mode dan aksesibilitas |
| **Editor Konten** | Tiptap v3.31 (StarterKit, Image, Code) | Penyuntingan kaya untuk butir soal dan formula/teks format |
| **Parsing Dokumen** | JSZip, XLSX, Calamine (Rust) | Parsing naskah soal Word (`.docx`) di frontend & berkas siswa Excel/CSV di backend |
| **Backend & Web Server** | Axum v0.8 + Tokio (Multi-thread) | Server HTTP embedded melayani portal ujian siswa pada port 80/8080 |
| **Basis Data** | Rusqlite v0.32 (Bundled) | SQLite lokal dengan dukungan WAL dan transaksi terisolasi |

---

## 3. Evaluasi Fitur Utama

### 3.1. Ujian Berbasis Komputer (CBT Engine)
* **Pemisahan Sesi Pilihan Ganda & Esai (Checkpoint):** Alur ujian memisahkan sesi PG dan esai secara terstruktur. Siswa menyelesaikan dan mengunci jawaban PG sebelum melangkah ke esai.
* **Resiliensi Jaringan Lemah (`saveStatus`):** Sistem melacak status setiap jawaban (`saved`, `saving`, `error`) secara visual di sisi siswa. Jika koneksi WiFi terputus sesaat, jawaban tidak hilang dan sistem otomatis mencoba mengirim ulang (*retry*).
* **Randomisasi Soal & Opsi:** Fitur acak nomor soal dan acak opsi pilihan ganda per sesi siswa meminimalkan peluang kecurangan antar peserta.
* **Token Akses & Waktu Jendela:** Ujian dilindungi token acak dan rentang waktu akses (*scheduled window*).

### 3.2. Bank Soal & Manajemen Konten
* **Import Langsung dari Microsoft Word (`.docx`):** Memanfaatkan parser XML di frontend berbasis `jszip`. Guru dapat langsung mengunggah naskah soal format Word standar tanpa perlu input manual satu per satu.
* **Penyimpanan Aset Gambar Mandiri:** Gambar yang disematkan dalam soal dikonversi dan disimpan ke folder `uploads` lokal, lalu dilayani via endpoint statis Axum.
* **Ekspor & Backup Terenkripsi:** Bank soal dapat diekspor per kelas dan mata pelajaran dengan enkripsi PIN.

### 3.3. Manajemen Siswa & Integrasi
* **Impor Data Fleksibel:** Mendukung berkas Excel (`.xlsx`) dan CSV dengan deteksi baris data otomatis via engine Rust `calamine`.
* **Sinkronisasi Daring (Tarik Data):** Menyediakan mekanisme penarikan daftar sekolah dan siswa langsung dari sumber URL publik/CSV sekolah.

---

## 4. Kelebihan Sistem (Strengths)

1. **Zero-Configuration Server Deployment**
   * Tidak memerlukan instalasi terpisah dari web server Apache, Nginx, PHP runtime, atau MySQL service.
   * Cukup satu file biner `.exe` yang dijalankan di laptop/PC guru, server langsung aktif di port 80 dan siap menerima koneksi siswa.
2. **Efisiensi Memori & Kecepatan Eksekusi**
   * Backend Rust dengan model asynchronous Tokio + Axum memiliki *overhead* footprint sangat kecil (~20–40 MB RAM), berbanding terbalik dengan stack berbasis Electron atau PHP-FPM yang memakan ratusan megabyte hingga gigabyte RAM.
3. **Penyimpanan Bersih dengan SQLite WAL**
   * Pengaturan `PRAGMA journal_mode = WAL` dan `PRAGMA synchronous = NORMAL` memastikan proses pembacaan oleh dashboard proktor tidak diblokir oleh penulisan simultan dari ratusan jawaban siswa yang masuk bersamaan.
4. **UX/UI Modern & Konsisten**
   * Mematuhi standar desain Nuxt UI: hirarki visual jelas, indikator status menggunakan token semantik, responsif pada berbagai resolusi layar siswa (ponsel hingga desktop).
5. **Mitigasi Kehilangan Data Bawaan**
   * Aplikasi mengeksekusi prosedur *backup* SQLite otomatis setiap kali aplikasi desktop dibuka, meminimalkan risiko integritas data akibat insiden listrik mati (*power failure*).

---

## 5. Kekurangan & Keterbatasan Teknis (Weaknesses)

1. **Deteksi Interface Jaringan Lokal (LAN IP)**
   * **Implementasi Saat Ini:** Fungsi `local_lan_ip()` di `src-tauri/src/server.rs` mengirimkan probe UDP tiruan ke alamat publik `8.8.8.8:80` untuk membaca alamat soket lokal yang aktif.
   * **Risiko:** Jika proktor menjalankan ujian pada jaringan router lokal yang **100% terisolasi tanpa sambungan internet sama sekali**, atau komputer proktor memiliki beberapa adapter jaringan (seperti adapter virtual VMware, VirtualBox, Hyper-V, atau WSL), probe ini berpotensi gagal atau mengembalikan IP adapter virtual yang tidak dapat dijangkau oleh perangkat siswa.
2. **Koneksi Database Single-Mutex (`Arc<Mutex<Connection>>`)**
   * **Implementasi Saat Ini:** Akses ke SQLite dibungkus dalam satu `Mutex<Connection>`.
   * **Risiko:** Untuk skala 1–2 kelas (30–60 siswa) performanya masih memadai. Namun, jika digunakan pada skala 1 angkatan (150–300+ siswa) di mana banyak siswa mengirimkan jawaban di detik yang sama, antrian kunci *mutex* dapat menyebabkan latensi respon HTTP meningkat.
3. **Monitoring Proktor Masih Manual / Polling**
   * Dashboard proktor belum memanfaatkan kanal komunikasi dua arah (*duplex*) seperti WebSocket atau Server-Sent Events (SSE). Proktor harus menekan tombol refresh secara berkala untuk memperbarui status daftar kehadiran dan progres pengerjaan siswa.
4. **Keamanan Autentikasi Siswa di Tingkat Jaringan**
   * Siswa melakukan autentikasi ke server Axum hanya bermodalkan Nomor Induk (NISN) dan Nama. Mengingat nomor NISN siswa dalam satu sekolah sering kali mudah diketahui rekan sekelas, terdapat celah di mana peserta dapat membuka sesi menggunakan identitas siswa lain jika tidak ada proteksi PIN individu per peserta.
5. **Protokol HTTP Polos (Unencrypted LAN)**
   * Transmisi data siswa berjalan melalui protokol HTTP standar di port 80. Dalam jaringan WiFi sekolah yang terbuka tanpa enkripsi WPA2/WPA3 enterprise, paket data dapat diinspeksi oleh perangkat lain dalam satu segmen jaringan.

---

## 6. Rekomendasi Peningkatan (Actionable Roadmap)

### Rekomendasi Jangka Pendek (Arsitektur & Keandalan)

1. **Deteksi IP Antarmuka Menggunakan Crate Fisik**
   * Ganti mekanisme probe dummy UDP dengan crate pembaca antarmuka jaringan seperti `network-interface`.
   * Berikan prioritas pada adapter Wi-Fi dan Ethernet fisik yang memiliki subnet private (`192.168.x.x` atau `10.x.x.x`) dan abaikan adapter virtual (`vEthernet`, `vmnet`, `wsl`).
   * Sediakan *selector dropdown* di menu Pengaturan aplikasi desktop agar proktor dapat memilih antarmuka jaringan secara manual jika terdapat multi-NIC.

2. **Implementasi Connection Pooling (`r2d2_sqlite`)**
   * Tingkatkan concurrency basis data dari `Arc<Mutex<Connection>>` ke connection pool dengan pustaka `r2d2_sqlite`.
   * Pisahkan koneksi *read-only* (untuk dashboard pemantauan dan penyajian soal) dengan koneksi *write* (untuk penyimpanan jawaban siswa).

3. **Live Monitoring Menggunakan Server-Sent Events (SSE)**
   * Sediakan rute `/api/exam/stream/:exam_id` pada server Axum menggunakan fitur `axum::response::sse::Sse`.
   * Kirim event ringan setiap kali ada event penting (misal: siswa login, submit PG, submit akhir) sehingga dashboard proktor selalu *real-time* tanpa membebani server dengan polling berulang.

### Rekomendasi Jangka Menengah (Fitur & Keamanan)

1. **Kartu Peserta Berbasis PIN Unik Sesi**
   * Tambahkan kolom PIN acak (4–6 karakter alfanumerik) pada setiap sesi siswa yang digenerate bersamaan dengan kartu peserta ujian.
   * Siswa wajib memasukkan NISN + PIN Sesi untuk mencegah penggunaan identitas ganda.

2. **Fitur Kiosk / Pengawasan Layar Ringan (Client-Side Integrity)**
   * Tambahkan event listener `visibilitychange` dan `window.blur` pada portal `cbt.vue`.
   * Catat rekam jejak pelanggaran (*switch tab count*) ke backend jika siswa berpindah aplikasi atau membuka tab baru saat ujian berlangsung.

3. **Penyempurnaan Modul e-Rapor**
   * Lengkapi integrasi penilaian ujian CBT langsung ke dalam kalkulasi nilai formatif/sumatif e-Rapor.
   * Sediakan generator berkas rapor PDF siap cetak dengan tata letak standar Kurikulum Merdeka.

---

## 7. Kesimpulan

Platform **NILUJI** memiliki pondasi arsitektur perangkat lunak yang sangat kokoh, modern, dan tepat guna. Pilihan menggunakan **Tauri 2 + Nuxt 4 + Axum + SQLite** berhasil memecahkan masalah klasik aplikasi CBT sekolah: menghilangkan kebutuhan konfigurasi server web pihak ketiga yang rumit, menjamin portabilitas, dan menghasilkan efisiensi komputasi yang tinggi.

Dengan menerapkan penyempurnaan pada deteksi jaringan multi-adapter, *connection pooling* basis data, dan penambahan kanal *real-time streaming* untuk proktor, Niluji memiliki kesiapan penuh untuk diterapkan dalam skala ujian sekolah yang lebih besar secara mandiri dan aman.

