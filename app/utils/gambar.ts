// Gambar untuk rapor (logo, tanda tangan, stempel, foto siswa) disimpan sebagai data URL di database,
// jadi dikecilkan dulu di sini supaya database tetap ringan dan ikut backup.

export interface GambarRapor { logo?: string, ttdKepsek?: string, stempel?: string }

export const KUNCI_GAMBAR = { logo: "gambar_logo", ttdKepsek: "gambar_ttd_kepsek", stempel: "gambar_stempel" } as const;

function bacaGambar(file: Blob): Promise<HTMLImageElement> {
	return new Promise((resolve, reject) => {
		const url = URL.createObjectURL(file);
		const img = new Image();
		img.onload = () => {
			URL.revokeObjectURL(url);
			resolve(img);
		};
		img.onerror = () => {
			URL.revokeObjectURL(url);
			reject(new Error("File bukan gambar yang bisa dibaca (pakai JPG atau PNG)."));
		};
		img.src = url;
	});
}

// Kecilkan sampai muat di maxW × maxH. PNG mempertahankan latar transparan (tanda tangan & stempel).
export async function kecilkanGambar(file: Blob, maxW: number, maxH: number, jenis: "image/png" | "image/jpeg" = "image/png"): Promise<string> {
	const img = await bacaGambar(file);
	const skala = Math.min(1, maxW / img.naturalWidth, maxH / img.naturalHeight);
	const canvas = document.createElement("canvas");
	canvas.width = Math.round(img.naturalWidth * skala);
	canvas.height = Math.round(img.naturalHeight * skala);
	const ctx = canvas.getContext("2d")!;
	if (jenis === "image/jpeg") {
		ctx.fillStyle = "#fff";
		ctx.fillRect(0, 0, canvas.width, canvas.height);
	}
	ctx.drawImage(img, 0, 0, canvas.width, canvas.height);
	return canvas.toDataURL(jenis, 0.85);
}

// Foto siswa: dipotong ke rasio 3:4 dari tengah, lalu 300 × 400 JPEG.
export async function fotoSiswa(file: Blob): Promise<string> {
	const img = await bacaGambar(file);
	const rasio = 3 / 4;
	let sw = img.naturalWidth;
	let sh = img.naturalHeight;
	if (sw / sh > rasio)
		sw = sh * rasio;
	else
		sh = sw / rasio;
	const canvas = document.createElement("canvas");
	canvas.width = 300;
	canvas.height = 400;
	canvas.getContext("2d")!.drawImage(img, (img.naturalWidth - sw) / 2, (img.naturalHeight - sh) / 2, sw, sh, 0, 0, 300, 400);
	return canvas.toDataURL("image/jpeg", 0.85);
}

export async function muatGambarRapor(db: ReturnType<typeof useDb>): Promise<GambarRapor> {
	const [logo, ttdKepsek, stempel] = await Promise.all(Object.values(KUNCI_GAMBAR).map(k => db.getSetting(k)));
	return { logo: logo || undefined, ttdKepsek: ttdKepsek || undefined, stempel: stempel || undefined };
}

// ===== Kop surat: logo diunggah, teksnya diketik sendiri (bawaan dari data sekolah) =====
export interface Kop { kiri?: string, kanan?: string, baris: string[] }
export const KOP_BARIS = 4;

export function kopBawaan(s: { nama?: string | null, alamat?: string | null, kabupaten?: string | null, kecamatan?: string | null }): string[] {
	const kab = (s.kabupaten ?? "").replace(/^kab(upaten)?\.?\s*/i, "");
	return [
		kab ? `PEMERINTAH KABUPATEN ${kab.toUpperCase()}` : "PEMERINTAH KABUPATEN",
		"DINAS PENDIDIKAN",
		(s.nama ?? "").toUpperCase(),
		[s.alamat, s.kecamatan].filter(Boolean).join(", ")
	];
}

export async function muatKop(db: ReturnType<typeof useDb>): Promise<Kop> {
	const [kiri, kanan, baris, sekolah] = await Promise.all([
		db.getSetting("gambar_kop_kiri"),
		db.getSetting("gambar_kop_kanan"),
		db.getSetting("kop_baris"),
		db.first<{ nama: string, alamat: string | null, kabupaten_kota: string | null, kecamatan: string | null }>("SELECT nama, alamat, kabupaten_kota, kecamatan FROM sekolah LIMIT 1")
	]);
	return {
		kiri: kiri || undefined,
		kanan: kanan || undefined,
		baris: baris ? JSON.parse(baris) : kopBawaan({ nama: sekolah?.nama, alamat: sekolah?.alamat, kabupaten: sekolah?.kabupaten_kota, kecamatan: sekolah?.kecamatan })
	};
}

// Tanda tangan wali kelas berupa gambar, per guru.
export const kunciTtdGuru = (ptkId: string) => `gambar_ttd_ptk:${ptkId}`;
