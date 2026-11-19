import { mkdirSync, unlinkSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { execFileSync } from "node:child_process";
import { MsEdgeTTS, OUTPUT_FORMAT } from "msedge-tts";

interface SceneScript {
  id: string;
  sceneNumber: number;
  title: string;
  file: string;
  text: string;
}

const SCENES: SceneScript[] = [
  {
    id: "scene-01",
    sceneNumber: 1,
    title: "Pembuka",
    file: "adegan-01.wav",
    text: "Selamat datang di tutorial NILUJI. NILUJI adalah aplikasi ujian sekolah berbasis komputer yang berjalan tanpa internet. Cukup satu laptop guru sebagai server, dan siswa mengerjakan ujian dari HP atau laptop melalui WiFi sekolah. Di video ini, kita akan menyiapkan satu ujian dari awal sampai nilai akhirnya keluar."
  },
  {
    id: "scene-02",
    sceneNumber: 2,
    title: "Instalasi dan jaringan",
    file: "adegan-02.wav",
    text: "Pertama, unduh installer NILUJI dari situs niluji titik permadi titik dev, lalu pasang di laptop pengawas. Saat pertama dibuka, Windows akan meminta izin firewall. Pilih izinkan, supaya HP siswa bisa terhubung. Nyalakan WiFi khusus ujian yang memakai password, lalu sambungkan laptop ke WiFi tersebut. Di pojok kanan atas Dashboard, pastikan alamat IP yang dipilih adalah alamat dari WiFi ujian."
  },
  {
    id: "scene-03",
    sceneNumber: 3,
    title: "Identitas sekolah",
    file: "adegan-03.wav",
    text: "Berikutnya, klik Isi Identitas Sekolah. Lengkapi nama sekolah, NPSN, alamat, dan nama kepala sekolah. Nama ini akan tercetak di kartu peserta. Di halaman yang sama, buat juga PIN Export, minimal empat karakter. PIN ini diminta setiap kali Anda mengekspor bank soal, jadi simpan baik-baik."
  },
  {
    id: "scene-04",
    sceneNumber: 4,
    title: "Kelas dan mata pelajaran",
    file: "adegan-04.wav",
    text: "Lalu buka Data Kelas dan Data Pelajaran. Pastikan daftar kelas dan mata pelajaran sudah sesuai dengan sekolah Anda. Daftar ini dipakai di semua menu, mulai dari data siswa sampai hasil ujian."
  },
  {
    id: "scene-05",
    sceneNumber: 5,
    title: "Data siswa",
    file: "adegan-05.wav",
    text: "Sekarang kita masukkan data siswa. Di menu Siswa, Anda bisa menambah siswa satu per satu, atau klik Import untuk mengunggah file Excel atau CSV. Unduh dulu templatenya supaya kolomnya sesuai. Bila data siswa sekolah Anda sudah ada di Google Sheets, buka Tarik Data Siswa, pilih nama sekolah, lalu klik Lihat Pratinjau. Data belum disimpan. Periksa dulu siswa yang baru dan yang datanya berubah, hapus centang bila perlu, lalu klik Simpan. NILUJI membuat cadangan database sebelum menyimpan."
  },
  {
    id: "scene-06",
    sceneNumber: 6,
    title: "Kartu peserta",
    file: "adegan-06.wav",
    text: "Setiap siswa otomatis mendapat PIN enam digit. Klik Kartu Peserta untuk melihat kartu ujian lengkap dengan nama sekolah, NISN, PIN, dan kode QR alamat server. Klik Cetak Kartu untuk mencetak atau menyimpannya sebagai PDF. Bagikan kartu ini kepada siswa sebelum ujian dimulai."
  },
  {
    id: "scene-07",
    sceneNumber: 7,
    title: "Bank soal",
    file: "adegan-07.wav",
    text: "Selanjutnya, siapkan soal di menu Bank Soal. Pilih kelas dan mata pelajaran, lalu buat jenis ujian, misalnya Sumatif Akhir Semester. Di dalamnya, klik Tambah Soal untuk menulis soal pilihan ganda dengan opsi A sampai E, atau soal esai. Anda juga bisa menyisipkan gambar di soal maupun di setiap pilihan jawaban. Kalau soal sudah diketik di Word, klik Import Soal. NILUJI bisa membaca file Word, Excel atau CSV, dan Markdown, lalu menampilkan pratinjaunya sebelum disimpan."
  },
  {
    id: "scene-08",
    sceneNumber: 8,
    title: "Membuat jadwal ujian",
    file: "adegan-08.wav",
    text: "Sekarang buat jadwal ujian di menu Kelola Ujian. Klik Buat Ujian Baru, lalu pilih kelas, mata pelajaran, dan jenis soal. Atur tanggal, jam mulai, jam selesai, dan durasi pengerjaan. Token ujian dibuat otomatis enam huruf. Isi bobot nilai pilihan ganda, misalnya tujuh puluh persen, sehingga esai bernilai tiga puluh persen. Aktifkan pengacakan soal dan pilihan jawaban, dan biarkan Wajib PIN Kartu tetap menyala supaya siswa tidak bisa masuk memakai akun temannya."
  },
  {
    id: "scene-09",
    sceneNumber: 9,
    title: "Hari ujian di sisi pengawas",
    file: "adegan-09.wav",
    text: "Di hari ujian, buka Dashboard. Ujian yang sedang berlangsung muncul di panel ujian aktif. Klik tombol token untuk menampilkannya dalam ukuran besar di proyektor. Klik QR Akses Ujian bila siswa ingin memindai alamat server langsung dari layar."
  },
  {
    id: "scene-10",
    sceneNumber: 10,
    title: "Siswa mengerjakan ujian",
    file: "adegan-10.wav",
    text: "Dari sisi siswa, ada dua cara masuk. Cara pertama, buka browser dan ketik alamat yang tertera di kartu peserta. Cara kedua, pasang aplikasi Android NILUJI Siswa, lalu pindai QR pada kartu. Aplikasi ini memblokir tangkapan layar dan meminta HP disematkan selama ujian. Setelah itu, siswa mengisi NISN dan nama lengkap, lalu token ujian dan PIN kartu, dan menekan Mulai Kerjakan Ujian. Jawaban tersimpan otomatis ke server. Soal yang masih ragu bisa ditandai, dan bila WiFi sempat putus, jawaban dikirim ulang dengan sendirinya. Setelah pilihan ganda selesai, siswa lanjut ke esai, lalu menekan Selesai dan Kumpulkan."
  },
  {
    id: "scene-11",
    sceneNumber: 11,
    title: "Pantauan dan bantuan pengawas",
    file: "adegan-11.wav",
    text: "Selama ujian, Dashboard menampilkan berapa siswa yang sedang mengerjakan pilihan ganda, esai, dan yang sudah selesai. Klik Pantau Hasil untuk melihat setiap siswa. Bila HP siswa mati, klik buka kunci perangkat, lalu siswa bisa melanjutkan dari HP lain tanpa kehilangan jawaban. Bila ada kendala, beri tambahan waktu sepuluh menit. Siswa yang lupa mengumpulkan akan dikumpulkan otomatis tiga menit setelah waktunya habis, dan pengawas tetap bisa membukanya lagi dengan tambahan waktu."
  },
  {
    id: "scene-12",
    sceneNumber: 12,
    title: "Koreksi esai, analisis, dan ekspor nilai",
    file: "adegan-12.wav",
    text: "Setelah ujian, buka Hasil Ujian. Nilai pilihan ganda langsung dihitung, sedangkan nilai akhir menunggu sampai semua esai dinilai. Klik ikon detail pada nama siswa, baca jawabannya, lalu isi nilai esai. Nilai akhir otomatis dihitung sesuai bobot. Bila ternyata ada kunci jawaban yang salah, perbaiki di Bank Soal, lalu klik Nilai Ulang. Buka tab Analisis Soal untuk melihat soal yang paling mudah dan paling sulit, serta pilihan jawaban mana yang paling sering menjebak siswa. Terakhir, klik Export untuk mengunduh rekap nilai dalam format Excel."
  },
  {
    id: "scene-13",
    sceneNumber: 13,
    title: "Cadangan data dan penutup",
    file: "adegan-13.wav",
    text: "Data Anda aman. NILUJI membuat cadangan harian yang disimpan selama empat belas hari, dan juga sebelum setiap perubahan penting. Bila perlu, buka Tentang, lalu Fitur, dan pulihkan database dari cadangan. Panduan lengkap, aplikasi Android untuk siswa, dan installer terbaru tersedia di niluji titik permadi titik dev. Selamat mencoba, semoga ujian di sekolah Anda berjalan lancar."
  }
];

const AUDIO_DIR = join(process.cwd(), "assets", "audio");
mkdirSync(AUDIO_DIR, { recursive: true });

const VOICE_NAME = process.env.EDGE_TTS_VOICE || "id-ID-GadisNeural";

async function synthesizeWithEdgeNeural(text: string, outRawMp3: string): Promise<void> {
  const tts = new MsEdgeTTS();
  await tts.setMetadata(VOICE_NAME, OUTPUT_FORMAT.AUDIO_24KHZ_96KBITRATE_MONO_MP3);
  const { audioStream } = tts.toStream(text, { rate: "+0%", pitch: "+0Hz" });
  const chunks: Buffer[] = [];
  await new Promise<void>((resolve, reject) => {
    audioStream.on("data", (chunk: Buffer) => chunks.push(Buffer.from(chunk)));
    audioStream.on("end", () => resolve());
    audioStream.on("error", (err: Error) => reject(err));
  });
  writeFileSync(outRawMp3, Buffer.concat(chunks));
}

function normalizeAndPadWav(inputPath: string, outputWavPath: string): number {
  execFileSync(
    "ffmpeg",
    [
      "-y",
      "-i",
      inputPath,
      "-af",
      "adelay=300|300,apad=pad_dur=0.65,loudnorm=I=-16:TP=-1.0:LRA=11",
      "-ar",
      "44100",
      "-ac",
      "1",
      "-c:a",
      "pcm_s16le",
      outputWavPath
    ],
    { stdio: "ignore" }
  );

  const probeOut = execFileSync(
    "ffprobe",
    [
      "-v",
      "error",
      "-show_entries",
      "format=duration",
      "-of",
      "default=noprint_wrappers=1:nokey=1",
      outputWavPath
    ],
    { encoding: "utf-8" }
  ).trim();

  return Math.round(parseFloat(probeOut) * 100) / 100;
}

async function main() {
  console.log(`Generating 13 Indonesian voiceover tracks using ${VOICE_NAME}...`);
  let cursor = 0;
  const timings: Array<{
    id: string;
    sceneNumber: number;
    title: string;
    file: string;
    start: number;
    duration: number;
    end: number;
    engine: string;
    text: string;
  }> = [];

  for (const scene of SCENES) {
    const outWav = join(AUDIO_DIR, scene.file);
    const tmpRaw = join(AUDIO_DIR, `_raw_${scene.id}.mp3`);
    await synthesizeWithEdgeNeural(scene.text, tmpRaw);
    const duration = normalizeAndPadWav(tmpRaw, outWav);
    try {
      unlinkSync(tmpRaw);
    } catch {
      // ignore
    }
    const start = Math.round(cursor * 100) / 100;
    const end = Math.round((start + duration) * 100) / 100;
    cursor = end;

    timings.push({
      id: scene.id,
      sceneNumber: scene.sceneNumber,
      title: scene.title,
      file: `assets/audio/${scene.file}`,
      start,
      duration,
      end,
      engine: VOICE_NAME,
      text: scene.text
    });

    console.log(`  ✓ ${scene.id} (${scene.file}): ${duration.toFixed(2)}s [${start.toFixed(2)}s -> ${end.toFixed(2)}s]`);
  }

  const summaryPath = join(AUDIO_DIR, "timings.json");
  writeFileSync(
    summaryPath,
    JSON.stringify(
      {
        totalDuration: Math.round(cursor * 100) / 100,
        voice: VOICE_NAME,
        scenes: timings
      },
      null,
      2
    ),
    "utf-8"
  );
  console.log(`\nTotal duration: ${cursor.toFixed(2)}s (${(cursor / 60).toFixed(2)} min). Saved to ${summaryPath}`);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
