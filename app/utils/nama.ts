// Format tampilan bersama (nama, tanggal, pesan error). Auto-import Nuxt dari folder utils/.

// Nama guru dengan gelar, format rapor: "Drs. Budi Santoso, M.Pd." (dipakai Data Guru, Sekolah, cetak rapor).
export function namaLengkap(p: { nama: string, gelar_depan?: string | null, gelar_belakang?: string | null }): string {
	const nama = [p.gelar_depan?.trim(), p.nama].filter(Boolean).join(" ");
	return p.gelar_belakang?.trim() ? `${nama}, ${p.gelar_belakang.trim()}` : nama;
}

const BULAN = ["Januari", "Februari", "Maret", "April", "Mei", "Juni", "Juli", "Agustus", "September", "Oktober", "November", "Desember"];

// "2014-03-26" → "26 Maret 2014" (format tanggal di rapor). Teks lain dikembalikan apa adanya.
export function tanggalIndo(iso?: string | null): string {
	const m = iso?.match(/^(\d{4})-(\d{2})-(\d{2})/);
	if (!m)
		return iso ?? "";
	return `${Number(m[3])} ${BULAN[Number(m[2]) - 1]} ${m[1]}`;
}

// "2026-10-07 10:20:31" (datetime SQLite, jam lokal) → "7 Oktober 2026, 10.20".
export function waktuIndo(dt?: string | null): string {
	const m = dt?.match(/^(\d{4}-\d{2}-\d{2})[ T](\d{2}):(\d{2})/);
	return m ? `${tanggalIndo(m[1])}, ${m[2]}.${m[3]}` : (dt ?? "");
}

// "Bandung, 26 Maret 2014" tanpa "-, -" kalau salah satu kosong.
export function tempatTanggal(tempat?: string | null, tanggal?: string | null): string {
	return [tempat?.trim(), tanggalIndo(tanggal)].filter(Boolean).join(", ") || "-";
}

export const atauStrip = (v?: string | number | null) => (v === null || v === undefined || v === "" ? "-" : String(v));

// Pesan error tanpa awalan "Error: " (error dari Rust datang sebagai string biasa).
export const pesanError = (e: unknown) => (e instanceof Error ? e.message : String(e));
