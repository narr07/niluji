import * as XLSX from "xlsx";

// Satu file per kelas 6: nomor ijazah, nomor transkrip, tanggal lulus, dan nilai tiap mapel transkrip.
// Padanan e-Rapor: Import Nomor Ijazah + Import Nilai Transkrip (di e-Rapor dua format & per mapel).

export interface TranskripBaris {
	id: string
	nisn: string | null
	nama: string
	noIjazah: string
	noTranskrip: string
	tanggalLulus: string
	nilai: Record<string, number | null>
}

const KOLOM_TETAP = ["No", "NISN", "Nama", "Nomor Ijazah", "Nomor Transkrip", "Tanggal Lulus (YYYY-MM-DD)"];

export function buildTranskripWorkbook(kelas: string, mapel: { kode: string, singkat: string }[], siswa: TranskripBaris[]): Uint8Array {
	const header = [...KOLOM_TETAP, ...mapel.map(m => m.singkat)];
	const rows = siswa.map((s, i) => [i + 1, s.nisn ?? "", s.nama, s.noIjazah, s.noTranskrip, s.tanggalLulus, ...mapel.map(m => s.nilai[m.kode] ?? "")]);
	const ws = XLSX.utils.aoa_to_sheet([header, ...rows]);
	ws["!cols"] = [{ wch: 4 }, { wch: 12 }, { wch: 32 }, { wch: 22 }, { wch: 18 }, { wch: 14 }, ...mapel.map(() => ({ wch: 10 }))];
	const wb = XLSX.utils.book_new();
	XLSX.utils.book_append_sheet(wb, ws, sheetName(`Transkrip ${kelas}`));
	const petunjuk = XLSX.utils.aoa_to_sheet([
		["Isi nomor ijazah, nomor transkrip, tanggal lulus, dan nilai (0–100, boleh desimal)."],
		["Ketik angka langsung, jangan pakai rumus. Jangan ubah kolom NISN dan judul kolom mapel."]
	]);
	XLSX.utils.book_append_sheet(wb, petunjuk, "Petunjuk");
	return new Uint8Array(XLSX.write(wb, { type: "array", bookType: "xlsx" }));
}

// Excel menyimpan tanggal sebagai angka seri; ubah ke YYYY-MM-DD.
function tanggal(v: unknown): string {
	if (typeof v === "number") {
		const d = XLSX.SSF.parse_date_code(v);
		return d ? `${d.y}-${String(d.m).padStart(2, "0")}-${String(d.d).padStart(2, "0")}` : "";
	}
	return String(v ?? "").trim();
}

export function readTranskripWorkbook(data: ArrayBuffer, mapel: { kode: string, singkat: string }[], siswa: { id: string, nisn: string | null, nama: string }[]) {
	const wb = XLSX.read(data, { type: "array" });
	const ws = wb.Sheets[wb.SheetNames[0]!];
	if (!ws)
		throw new Error("File kosong");
	const rows = XLSX.utils.sheet_to_json<unknown[]>(ws, { header: 1, blankrows: false });
	const header = (rows[0] ?? []).map(h => String(h ?? "").trim());
	if (header[1] !== "NISN")
		throw new Error("Format tidak dikenali. Pakai file dari tombol Export.");
	const kolomMapel = mapel.map(m => ({ kode: m.kode, i: header.indexOf(m.singkat) })).filter(x => x.i >= 0);
	const hasil: { id: string, noIjazah: string, noTranskrip: string, tanggalLulus: string, nilai: Record<string, number | null> }[] = [];
	const ditolak: string[] = [];
	for (const r of rows.slice(1)) {
		const nisn = String(r[1] ?? "").trim();
		const nama = String(r[2] ?? "").trim();
		const s = siswa.find(x => (nisn && x.nisn === nisn) || (!nisn && x.nama === nama));
		if (!s) {
			if (nisn || nama)
				ditolak.push(`${nama || nisn}: bukan siswa kelas ini`);
			continue;
		}
		const nilai: Record<string, number | null> = {};
		for (const k of kolomMapel) {
			const v = r[k.i];
			if (v === undefined || v === null || v === "") {
				nilai[k.kode] = null;
				continue;
			}
			const n = Number(String(v).replace(",", "."));
			if (!Number.isFinite(n) || n < 0 || n > 100) {
				ditolak.push(`${s.nama}: nilai "${v}" tidak valid`);
				continue;
			}
			nilai[k.kode] = Math.round(n * 100) / 100;
		}
		hasil.push({ id: s.id, noIjazah: String(r[3] ?? "").trim(), noTranskrip: String(r[4] ?? "").trim(), tanggalLulus: tanggal(r[5]), nilai });
	}
	return { hasil, ditolak };
}
