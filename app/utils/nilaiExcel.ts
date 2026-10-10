import * as XLSX from "xlsx";

// Satu file Excel per kelas, satu sheet per mapel. Kolom TP diisi O (optimal) / P (perlu bantuan).
// Di e-Rapor formatnya per kelas per mapel, jadi guru kelas harus bolak-balik 7 file.

export interface SheetSiswa {
	pesertaDidikId: string
	nisn: string | null
	nama: string
	nilai: number | null
	optimal: number[]
	perlu: number[]
}

export interface SheetMapel {
	prId: number
	singkat: string
	mapel: string
	tps: { id: number, deskripsi: string }[]
	siswa: SheetSiswa[]
}

export const sheetName = (s: string) => s.replace(/[[\]:*?/\\]/g, " ").slice(0, 31);

export function buildWorkbook(rombel: string, mapel: SheetMapel[]): Uint8Array {
	const wb = XLSX.utils.book_new();
	const petunjuk: (string | number)[][] = [
		[`Nilai Rapor ${rombel}`],
		["Isi kolom Nilai (0–100). Kolom TP: O = tercapai optimal, P = perlu bantuan, kosong = tidak disebut di deskripsi."],
		["Jangan ubah nama sheet, kolom NISN, dan urutan kolom TP."],
		[]
	];
	for (const m of mapel) {
		const header = ["No", "NISN", "Nama", "Nilai", ...m.tps.map((_, i) => `TP${i + 1}`)];
		const rows = m.siswa.map((s, i) => [
			i + 1,
			s.nisn ?? "",
			s.nama,
			s.nilai ?? "",
			...m.tps.map(tp => (s.optimal.includes(tp.id) ? "O" : s.perlu.includes(tp.id) ? "P" : ""))
		]);
		const ws = XLSX.utils.aoa_to_sheet([header, ...rows]);
		ws["!cols"] = [{ wch: 4 }, { wch: 12 }, { wch: 32 }, { wch: 7 }, ...m.tps.map(() => ({ wch: 5 }))];
		XLSX.utils.book_append_sheet(wb, ws, sheetName(m.singkat));
		petunjuk.push([m.mapel]);
		m.tps.forEach((tp, i) => petunjuk.push([`TP${i + 1}`, tp.deskripsi]));
		petunjuk.push([]);
	}
	const ws = XLSX.utils.aoa_to_sheet(petunjuk);
	ws["!cols"] = [{ wch: 8 }, { wch: 100 }];
	XLSX.utils.book_append_sheet(wb, ws, "Petunjuk");
	return new Uint8Array(XLSX.write(wb, { type: "array", bookType: "xlsx" }));
}

// Membaca balik file hasil buildWorkbook. Siswa dicocokkan lewat NISN, lalu nama.
export function readWorkbook(data: ArrayBuffer, mapel: SheetMapel[]): { prId: number, siswa: SheetSiswa[] }[] {
	const wb = XLSX.read(data, { type: "array" });
	// Sheet Petunjuk mencatat "TPn → teks TP" saat file dibuat. Kolom TP dicocokkan lewat teks itu,
	// jadi urutan TP boleh diubah (drag & drop) setelah file diunduh tanpa centang pindah ke TP lain.
	const tpFile = new Map<string, Map<string, string>>();
	const pet = wb.Sheets.Petunjuk;
	if (pet) {
		let aktif: Map<string, string> | undefined;
		for (const r of XLSX.utils.sheet_to_json<string[]>(pet, { header: 1, blankrows: false })) {
			const [a, b] = [String(r[0] ?? "").trim(), String(r[1] ?? "").trim()];
			if (/^TP\d+$/.test(a) && b && aktif)
				aktif.set(a, b);
			else if (a && !b && mapel.some(m => m.mapel === a))
				tpFile.set(a, aktif = new Map());
		}
	}
	const out: { prId: number, siswa: SheetSiswa[] }[] = [];
	for (const m of mapel) {
		const ws = wb.Sheets[sheetName(m.singkat)];
		if (!ws)
			continue;
		const semua = XLSX.utils.sheet_to_json<(string | number)[]>(ws, { header: 1, blankrows: false });
		const header = (semua[0] ?? []).map(h => String(h ?? "").trim());
		const teksFile = tpFile.get(m.mapel);
		// Kolom ke-i di file → id TP sekarang. Teks cocok dipakai dulu; file lama tanpa petunjuk = urutan kolom.
		const kolomTp = header.map((h, i) => {
			if (i < 4 || !/^TP\d+$/.test(h))
				return undefined;
			const teks = teksFile?.get(h);
			return teks ? m.tps.find(t => t.deskripsi === teks)?.id : m.tps[i - 4]?.id;
		});
		const rows = semua.slice(1);
		const siswa: SheetSiswa[] = [];
		for (const r of rows) {
			const nisn = String(r[1] ?? "").trim();
			const nama = String(r[2] ?? "").trim().toLowerCase();
			const s = m.siswa.find(x => (nisn && x.nisn === nisn) || x.nama.toLowerCase() === nama);
			if (!s)
				continue;
			const raw = r[3];
			const n = raw === "" || raw === undefined ? null : Math.round(Number(raw));
			const optimal: number[] = [];
			const perlu: number[] = [];
			kolomTp.forEach((id, i) => {
				if (id === undefined)
					return;
				const v = String(r[i] ?? "").trim().toUpperCase();
				if (v === "O")
					optimal.push(id);
				else if (v === "P")
					perlu.push(id);
			});
			siswa.push({ ...s, nilai: n !== null && Number.isFinite(n) ? Math.min(100, Math.max(0, n)) : null, optimal, perlu });
		}
		out.push({ prId: m.prId, siswa });
	}
	return out;
}
