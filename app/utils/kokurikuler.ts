// Kokurikuler (rapor PPA 2025) versi ringkas: kegiatan → capaian per dimensi profil lulusan →
// deskripsi rapor otomatis. Tabel: kokurikuler_kegiatan, nilai_kokurikuler, rapor_siswa.kokurikuler.

// 8 dimensi profil lulusan (Permendikdasmen 2025).
export const DIMENSI: { kode: string, nama: string, singkat: string }[] = [
	{ kode: "keimanan", nama: "Keimanan dan Ketakwaan terhadap Tuhan YME", singkat: "Keimanan" },
	{ kode: "kewargaan", nama: "Kewargaan", singkat: "Kewargaan" },
	{ kode: "penalaran", nama: "Penalaran Kritis", singkat: "Penalaran" },
	{ kode: "kreativitas", nama: "Kreativitas", singkat: "Kreativitas" },
	{ kode: "kolaborasi", nama: "Kolaborasi", singkat: "Kolaborasi" },
	{ kode: "kemandirian", nama: "Kemandirian", singkat: "Kemandirian" },
	{ kode: "kesehatan", nama: "Kesehatan", singkat: "Kesehatan" },
	{ kode: "komunikasi", nama: "Komunikasi", singkat: "Komunikasi" }
];
export const namaDimensi = (kode: string) => DIMENSI.find(d => d.kode === kode)?.nama ?? kode;

// Sama dengan e-Rapor: 1 Berkembang, 2 Cakap, 3 Mahir (juga angka di format import e-Rapor).
export const CAPAIAN: { value: 1 | 2 | 3, label: string, singkat: string, warna: "warning" | "primary" | "success" }[] = [
	{ value: 1, label: "Berkembang", singkat: "B", warna: "warning" },
	{ value: 2, label: "Cakap", singkat: "C", warna: "primary" },
	{ value: 3, label: "Mahir", singkat: "M", warna: "success" }
];

export interface KegiatanKoku {
	id: number
	nama: string
	tema: string | null
	tujuan: string | null
	tingkat: number[]
	dimensi: string[]
	subdimensi: Record<string, string[]> // opsional; dimensi tanpa subdimensi dinilai langsung
}

// Kolom penilaian sebuah kegiatan: tiap subdimensi (kunci "dimensi::urutan"), atau dimensinya
// langsung kalau subdimensinya tidak diisi.
export interface KolomKoku { kunci: string, dimensi: string, label: string, judul: string }
export function kolomKegiatan(k: Pick<KegiatanKoku, "dimensi" | "subdimensi">): KolomKoku[] {
	return k.dimensi.flatMap((d) => {
		const nama = DIMENSI.find(x => x.kode === d);
		const subs = (k.subdimensi?.[d] ?? []).filter(Boolean);
		return subs.length
			? subs.map((sub, i) => ({ kunci: `${d}::${i}`, dimensi: d, label: sub, judul: `${nama?.nama ?? d} — ${sub}` }))
			: [{ kunci: d, dimensi: d, label: nama?.singkat ?? d, judul: nama?.nama ?? d }];
	});
}
export const dimensiDariKunci = (kunci: string) => kunci.split("::")[0]!;

export const DESKRIPSI_KOKU_MAX = 600;

const gabung = (xs: string[]) => (xs.length <= 1 ? (xs[0] ?? "") : `${xs.slice(0, -1).join(", ")} dan ${xs.at(-1)}`);

// "Dani menunjukkan capaian mahir dalam kreativitas dan kolaborasi, cakap dalam kemandirian, serta
// berkembang dalam komunikasi melalui kegiatan Pasar Sekolah." Satu dimensi dinilai di beberapa
// kegiatan → capaian tertinggi yang dipakai.
export function deskripsiKokurikuler(nama: string, items: { kegiatan: string, capaian: Record<string, number> }[]): string {
	const terbaik = new Map<string, number>();
	const ikut: string[] = [];
	for (const it of items) {
		const ada = Object.entries(it.capaian).filter(([, v]) => v >= 1 && v <= 3);
		if (!ada.length)
			continue;
		ikut.push(it.kegiatan);
		for (const [k, v] of ada) {
			const d = dimensiDariKunci(k); // subdimensi dirangkum ke dimensinya
			terbaik.set(d, Math.max(terbaik.get(d) ?? 0, v));
		}
	}
	if (!terbaik.size)
		return "";
	const bagian = [...CAPAIAN].reverse()
		.map(c => ({ c, dims: DIMENSI.filter(d => terbaik.get(d.kode) === c.value).map(d => d.nama.charAt(0).toLowerCase() + d.nama.slice(1)) }))
		.filter(x => x.dims.length)
		.map(x => `${x.c.label.toLowerCase()} dalam ${gabung(x.dims)}`);
	const isi = bagian.length > 1 ? `${bagian.slice(0, -1).join(", ")}, serta ${bagian.at(-1)}` : bagian[0];
	return `${panggilan(nama)} menunjukkan capaian ${isi} melalui kegiatan ${gabung(ikut)}.`;
}
