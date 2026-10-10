// Nilai ekstrakurikuler (tabel nilai_ekskul). Predikat sama dengan e-Rapor SD.

export type Predikat = "SB" | "B" | "C" | "K";

export const PREDIKAT: { value: Predikat, label: string, warna: "success" | "primary" | "warning" | "error" }[] = [
	{ value: "SB", label: "Sangat Baik", warna: "success" },
	{ value: "B", label: "Baik", warna: "primary" },
	{ value: "C", label: "Cukup", warna: "warning" },
	{ value: "K", label: "Kurang", warna: "error" }
];

export const KETERANGAN_MAX = 200;

// Kalimat otomatis kalau pembina tidak menulis keterangan sendiri.
export function keteranganEkskul(predikat: string | null | undefined, ekskul: string): string {
	switch (predikat) {
		case "SB": return `Sangat baik dalam mengikuti kegiatan ${ekskul}.`;
		case "B": return `Baik dalam mengikuti kegiatan ${ekskul}.`;
		case "C": return `Cukup baik dalam mengikuti kegiatan ${ekskul}.`;
		case "K": return `Perlu meningkatkan keaktifan dalam kegiatan ${ekskul}.`;
		default: return "";
	}
}
