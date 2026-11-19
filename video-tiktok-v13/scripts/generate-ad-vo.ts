import { mkdirSync, unlinkSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { execFileSync } from "node:child_process";
import { MsEdgeTTS, OUTPUT_FORMAT } from "msedge-tts";

const BEATS = [
  {
    id: "beat-01",
    file: "ad-01.wav",
    text: "Ujian sekolah tanpa internet? Pakai NILUJI! Cukup satu laptop guru jadi server lokal."
  },
  {
    id: "beat-02",
    file: "ad-02.wav",
    text: "Cetak kartu QR dan PIN otomatis, lalu impor soal dari Word, Excel, atau Markdown."
  },
  {
    id: "beat-03",
    file: "ad-03.wav",
    text: "Siswa kerjakan dari HP lewat WiFi sekolah. Jawaban tersimpan otomatis dan aman."
  },
  {
    id: "beat-04",
    file: "ad-04.wav",
    text: "Pengawas pantau langsung: bisa buka kunci perangkat atau tambah waktu sepuluh menit."
  },
  {
    id: "beat-05",
    file: "ad-05.wav",
    text: "Koreksi esai, analisis soal, dan ekspor nilai Excel dalam hitungan detik!"
  },
  {
    id: "beat-06",
    file: "ad-06.wav",
    text: "NILUJI versi satu titik tiga. Unduh gratis di niluji titik permadi titik dev."
  }
];

const AUDIO_DIR = join(process.cwd(), "assets", "audio");
mkdirSync(AUDIO_DIR, { recursive: true });

async function synthesize(text: string, outMp3: string) {
  const tts = new MsEdgeTTS();
  await tts.setMetadata("id-ID-GadisNeural", OUTPUT_FORMAT.AUDIO_24KHZ_96KBITRATE_MONO_MP3);
  const { audioStream } = tts.toStream(text, { rate: "+10%", pitch: "+0Hz" });
  const chunks: Buffer[] = [];
  await new Promise<void>((resolve, reject) => {
    audioStream.on("data", (c: Buffer) => chunks.push(Buffer.from(c)));
    audioStream.on("end", () => resolve());
    audioStream.on("error", (e: Error) => reject(e));
  });
  writeFileSync(outMp3, Buffer.concat(chunks));
}

function normalize(inPath: string, outWav: string): number {
  execFileSync(
    "ffmpeg",
    ["-y", "-i", inPath, "-af", "adelay=120|120,apad=pad_dur=0.25,loudnorm=I=-15:TP=-1.0:LRA=11", "-ar", "44100", "-ac", "1", "-c:a", "pcm_s16le", outWav],
    { stdio: "ignore" }
  );
  const dur = execFileSync("ffprobe", ["-v", "error", "-show_entries", "format=duration", "-of", "default=noprint_wrappers=1:nokey=1", outWav], { encoding: "utf-8" }).trim();
  return Math.round(parseFloat(dur) * 100) / 100;
}

async function main() {
  let cursor = 0;
  const manifest = [];
  for (const b of BEATS) {
    const tmp = join(AUDIO_DIR, `_raw_${b.id}.mp3`);
    const out = join(AUDIO_DIR, b.file);
    await synthesize(b.text, tmp);
    const duration = normalize(tmp, out);
    try { unlinkSync(tmp); } catch {}
    const start = Math.round(cursor * 100) / 100;
    cursor = Math.round((start + duration) * 100) / 100;
    manifest.push({ id: b.id, file: `assets/audio/${b.file}`, start, duration, end: cursor, text: b.text });
    console.log(`✓ ${b.id}: ${duration.toFixed(2)}s [${start.toFixed(2)}s -> ${cursor.toFixed(2)}s]`);
  }
  writeFileSync(join(AUDIO_DIR, "timings.json"), JSON.stringify({ totalDuration: cursor, beats: manifest }, null, 2));
  console.log(`Total ad duration: ${cursor.toFixed(2)}s`);
}

main().catch((e) => { console.error(e); process.exit(1); });
