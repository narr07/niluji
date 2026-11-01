# SPEC — NILUJI Video Tutorial Lengkap

- **GOAL**: Guru, operator, dan pengawas sekolah memahami alur penggunaan NILUJI secara utuh dari persiapan server lokal, manajemen siswa & kartu PIN/QR, bank soal bergambar, jadwal & bobot ujian, pengerjaan di Portal CBT siswa, pantauan pengawas real-time, hingga **pemeriksaan jawaban PG/Esai, kemunculan nilai akhir ujian tertimbang, dan analisis pengecoh**.
- **OUTPUT**: HyperFrames `1920 × 1080` (`landscape`), `30 fps`, H.264 MP4 (`niluji-tutorial-lengkap.mp4`).
- **LOOK**:
  - **Ground**: `#0B1120` (Deep Slate Night) and `#0F172A` (Slate 900) studio canvas with a subtle `#1E293B` (`1px` at `28px`) technical grid, framing real light/dark NILUJI UI cards and screenshots with high contrast.
  - **Surface**: `#FFFFFF` (Light UI Surface) and `#1E293B` (Dark HUD Callout Panel) with `1px solid rgba(16, 185, 129, 0.35)` borders.
  - **Ink**: `#F8FAFC` on dark studio backgrounds; `#0F172A` primary and `#334155` secondary on light UI cards. All text pairs meet WCAG AA (`>= 4.5:1` body, `>= 3.0:1` large display).
  - **Accent**: `#10B981` (Emerald 500 — *Verified N-Check*) and `#059669` (Emerald 600) used exclusively for active callout rings, step badges, cursor click rings, and final score highlights. Secondary semantic highlights: `#F59E0B` (Amber — Ragu-ragu / Token) and `#3B82F6` (Blue — PG Badge).
  - **Typography**:
    - Display / Scene Headers & Numeric Scores: `'Plus Jakarta Sans', 'Segoe UI', system-ui, sans-serif`, `700–800`, `44px–64px`, `letter-spacing: -0.02em`, `line-height: 1.15`.
    - UI Labels & Callouts: `'Inter', 'Segoe UI', system-ui, sans-serif`, `600`, `22px–28px`, `line-height: 1.35`.
    - Subtitle / Narration Captions (80% Title-Safe Bottom Bar): `'Inter', 'Segoe UI', system-ui, sans-serif`, `600`, `26px`, `#F8FAFC` on `#0F172A` (`opacity: 0.94`), centered within `left: 192px; right: 192px; bottom: 58px`.
    - Monospace (IP LAN, Token, PIN, Formula): `'JetBrains Mono', 'Consolas', monospace`, `700`, `22px–30px`.
- **SCENES** (synced to Voiceover WAV durations):
  1. `scene-01` — **Persiapan Server Lokal & Identitas Sekolah**
  2. `scene-02` — **Data Siswa, Impor CSV & Cetak Kartu Peserta (QR + PIN)**
  3. `scene-03` — **Bank Soal & Impor Naskah Bergambar (PG + Esai)**
  4. `scene-04` — **Jadwal Ujian, Bobot PG/Esai (70:30) & Token Keamanan**
  5. `scene-05` — **Simulasi Siswa Mengerjakan Ujian di Portal CBT (`/cbt`)**
  6. `scene-06` — **Meja Pantau Pengawas Real-Time (Buka Kunci & +10m)**
  7. `scene-07` — **Pemeriksaan Soal PG, Koreksi Esai Manual & Munculnya Nilai Akhir (88,5)**
  8. `scene-08` — **Analisis Butir Soal (Pengecoh A–D) & Ekspor Nilai Excel (.xlsx)**
- **MOTION**:
  - Scene entrance & camera push-in: `gsap.fromTo` `0.5s` (`15 frames`), `expo.out`.
  - Spotlight callout & badge pop: `gsap.fromTo` `0.35s` (`10 frames`), `back.out(1.4)`.
  - Guided cursor movement: `gsap.fromTo` `0.45s` (`14 frames`), `power2.inOut`, followed by click pulse (`scale: 0.88` for `0.12s`).
- **SOUND**:
  - Voiceover (`assets/audio/vo-scene-01.wav` .. `vo-scene-08.wav`): Generated via `debpalash/VoiceStudio` pipeline (`scripts/generate-voiceover.ts` targeting `VOICESTUDIO_URL` / `http://localhost:3900/v1/audio/speech` with automatic Indonesian studio neural fallback `id-ID-ArdiNeural`), normalized to `-16 LUFS` (`-1 dBTP`).
  - SFX (`assets/sfx/`): Offline curated sound effects (`click-soft.mp3`, `pop.mp3`, `whoosh-short.mp3`, `chime.mp3`, `notification.mp3`) placed `+0.04s` after visual action beats at `data-volume="0.28"`.
- **BANNED**:
  - No neon glows, floating particles, or purple/cyan AI gradients.
  - No generic 3-card feature grids without real NILUJI interface workflows.
  - No CSS `transform` rules on elements animated by GSAP.
  - No `Math.random()`, `Date.now()`, or `requestAnimationFrame` loops.

