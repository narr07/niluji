import { existsSync, mkdirSync, writeFileSync } from "node:fs";
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
    title: "Persiapan Server Lokal & Identitas Sekolah",
    file: "vo-scene-01.wav",
    text: "Selamat datang di panduan lengkap NILUJI. Hanya dengan satu laptop pengawas dan jaringan WiFi lokal tanpa internet, Anda siap menyelenggarakan ujian berbasis komputer. Pastikan status Server Ujian menyala hijau, lalu simpan identitas sekolah Anda di menu Pengaturan."
  },
  {
    id: "scene-02",
    sceneNumber: 2,
    title: "Data Siswa & Cetak Kartu Peserta (QR + PIN)",
    file: "vo-scene-02.wav",
    text: "Langkah kedua: menyiapkan data peserta. Di menu Siswa, impor daftar siswa sekaligus dari file CSV. NILUJI otomatis membuatkan PIN enam digit unik untuk setiap siswa. Klik Cetak Kartu Peserta untuk mengunduh kartu ujian lengkap dengan QR Code server."
  },
  {
    id: "scene-03",
    sceneNumber: 3,
    title: "Bank Soal & Impor Naskah Bergambar",
    file: "vo-scene-03.wav",
    text: "Langkah ketiga: menyusun Bank Soal. Anda dapat mengetik langsung atau mengimpor naskah dari Word, Excel, maupun bundel Markdown bergambar, baik gambar pada soal maupun pada pilihan jawaban A sampai D. Gunakan Pratinjau untuk memeriksa tampilan soal dan kunci jawaban."
  },
  {
    id: "scene-04",
    sceneNumber: 4,
    title: "Jadwal Ujian, Bobot Nilai & Token Keamanan",
    file: "vo-scene-04.wav",
    text: "Langkah keempat: menjadwalkan ujian. Di menu Kelola Ujian, pilih kelas dan mata pelajaran, atur durasi enam puluh menit, lalu tentukan bobot nilai Pilihan Ganda tujuh puluh persen dan Esai tiga puluh persen. Aktifkan acak soal, lalu bagikan Token Ujian enam karakter kepada siswa."
  },
  {
    id: "scene-05",
    sceneNumber: 5,
    title: "Simulasi Siswa di Portal CBT",
    file: "vo-scene-05.wav",
    text: "Langkah kelima: pengerjaan ujian oleh siswa di Portal CBT. Siswa masuk menggunakan NISN, PIN enam digit, dan Token Ujian. Saat memilih jawaban Pilihan Ganda, progres tersimpan otomatis ke server lokal. Siswa juga dapat menandai soal ragu-ragu, mengisi jawaban uraian Esai, lalu menekan Kumpulkan Ujian."
  },
  {
    id: "scene-06",
    sceneNumber: 6,
    title: "Meja Pantau Pengawas Real-Time",
    file: "vo-scene-06.wav",
    text: "Langkah keenam: pemantauan pengawas secara langsung. Di halaman Hasil Ujian, pantau status pengerjaan setiap siswa secara real-time. Jika perangkat siswa terkunci karena berganti HP, klik Buka Kunci Perangkat. Pengawas juga dapat menambah waktu sepuluh menit atau memaksa kumpul sesi."
  },
  {
    id: "scene-07",
    sceneNumber: 7,
    title: "Pemeriksaan Soal, Koreksi Esai & Munculnya Nilai Akhir",
    file: "vo-scene-07.wav",
    text: "Langkah ketujuh: pemeriksaan soal dan kemunculan nilai ujian. Klik Periksa Sesi pada nama siswa untuk meninjau rincian jawaban Pilihan Ganda yang dinilai otomatis. Selanjutnya pada bagian Penilaian Esai, baca jawaban uraian siswa, ketik skor delapan puluh lima, lalu klik Simpan Nilai Esai. Seketika itu juga, Nilai Akhir tertimbang sebesar delapan puluh delapan koma lima langsung muncul di tabel peringkat kelas!"
  },
  {
    id: "scene-08",
    sceneNumber: 8,
    title: "Analisis Butir Soal (Pengecoh) & Ekspor Excel",
    file: "vo-scene-08.wav",
    text: "Langkah terakhir: evaluasi mutu soal dan ekspor nilai. Buka panel Analisis Pengecoh untuk melihat persentase pemilih opsi A, B, C, D, dan jawaban kosong beserta kategori tingkat kesulitan soal. Klik Ekspor Excel untuk mengunduh rekap nilai lengkap siap masuk rapor. Unduh NILUJI gratis di narr07.github.io garis miring niluji."
  }
];

const AUDIO_DIR = join(process.cwd(), "assets", "audio");
mkdirSync(AUDIO_DIR, { recursive: true });

const VOICESTUDIO_URL = (process.env.VOICESTUDIO_URL || "http://localhost:3900").replace(/\/+$/, "");

async function checkVoiceStudio(): Promise<boolean> {
  try {
    const controller = new AbortController();
    const timeout = setTimeout(() => controller.abort(), 1800);
    const res = await fetch(`${VOICESTUDIO_URL}/health`, { signal: controller.signal });
    clearTimeout(timeout);
    return res.ok;
  } catch {
    return false;
  }
}

async function resolveVoiceStudioVoice(): Promise<string> {
  if (process.env.VOICESTUDIO_VOICE_ID) {
    return process.env.VOICESTUDIO_VOICE_ID;
  }
  try {
    // Check preset or designed voices in debpalash/VoiceStudio
    const voicesRes = await fetch(`${VOICESTUDIO_URL}/v1/audio/voices`);
    if (voicesRes.ok) {
      const data = (await voicesRes.json()) as { voices?: Array<{ id?: string; name?: string }> };
      if (data.voices && data.voices.length > 0 && data.voices[0].id) {
        return data.voices[0].id;
      }
    }
    // Or design an Indonesian studio narrator voice via VoiceStudio Voice Design endpoint
    const designRes = await fetch(`${VOICESTUDIO_URL}/api/voice/design`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        name: "NILUJI Studio Narrator ID",
        description:
          "Warm, articulate Indonesian male educator, medium pitch, calm and clear studio narration tone, crisp diction.",
        language: "id"
      })
    });
    if (designRes.ok) {
      const designed = (await designRes.json()) as { profile_id?: string; id?: string };
      return designed.profile_id || designed.id || "alloy";
    }
  } catch {
    // fallback to default VoiceStudio preset ID
  }
  return "alloy";
}

async function synthesizeWithVoiceStudio(text: string, voiceId: string, outRawPath: string): Promise<boolean> {
  try {
    const res = await fetch(`${VOICESTUDIO_URL}/v1/audio/speech`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        model: "qwen3-tts",
        input: text,
        voice: voiceId,
        response_format: "wav",
        speed: 1.02
      })
    });
    if (!res.ok) return false;
    const buf = Buffer.from(await res.arrayBuffer());
    writeFileSync(outRawPath, buf);
    return buf.byteLength > 1000;
  } catch {
    return false;
  }
}

async function synthesizeWithEdgeNeural(text: string, outRawMp3: string): Promise<void> {
  const tts = new MsEdgeTTS();
  await tts.setMetadata("id-ID-ArdiNeural", OUTPUT_FORMAT.AUDIO_24KHZ_96KBITRATE_MONO_MP3);
  const { audioStream } = tts.toStream(text, { rate: "+4%", pitch: "+0Hz" });
  const chunks: Buffer[] = [];
  await new Promise<void>((resolve, reject) => {
    audioStream.on("data", (chunk: Buffer) => chunks.push(Buffer.from(chunk)));
    audioStream.on("end", () => resolve());
    audioStream.on("error", (err: Error) => reject(err));
  });
  writeFileSync(outRawMp3, Buffer.concat(chunks));
}

function normalizeAndPadWav(inputPath: string, outputWavPath: string): number {
  // Add 0.25s lead-in silence and 0.55s tail silence + EBU R128 loudness normalization (-16 LUFS, -1.0 dBTP)
  execFileSync(
    "ffmpeg",
    [
      "-y",
      "-i",
      inputPath,
      "-af",
      "adelay=250|250,apad=pad_dur=0.55,loudnorm=I=-16:TP=-1.0:LRA=11",
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
  const vsAvailable = await checkVoiceStudio();
  let vsVoiceId = "alloy";
  if (vsAvailable) {
    vsVoiceId = await resolveVoiceStudioVoice();
    console.log(`[VoiceStudio] Connected to ${VOICESTUDIO_URL} using voice profile: ${vsVoiceId}`);
  } else {
    console.log(
      `[VoiceStudio] Server at ${VOICESTUDIO_URL} not active; using studio neural fallback (id-ID-ArdiNeural) with EBU R128 -16 LUFS normalization.`
    );
  }

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
    const tmpRaw = join(AUDIO_DIR, `_raw_${scene.id}.tmp`);
    let engine = "VoiceStudio (Qwen3-TTS)";

    let ok = false;
    if (vsAvailable) {
      ok = await synthesizeWithVoiceStudio(scene.text, vsVoiceId, tmpRaw);
    }
    if (!ok) {
      engine = "Neural Studio ID (id-ID-ArdiNeural)";
      await synthesizeWithEdgeNeural(scene.text, tmpRaw);
    }

    const duration = normalizeAndPadWav(tmpRaw, outWav);
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
      engine,
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
        voiceStudioUrl: VOICESTUDIO_URL,
        scenes: timings
      },
      null,
      2
    ),
    "utf-8"
  );
  console.log(`\nTotal tutorial duration: ${cursor.toFixed(2)}s. Saved manifest to ${summaryPath}`);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});

