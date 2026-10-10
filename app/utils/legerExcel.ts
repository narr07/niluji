import type { RaporKelas } from "~/composables/useRapor";
import * as XLSX from "xlsx";

// Leger nilai rapor (padanan Cetak Nilai → Leger e-Rapor): satu sheet per kelas, satu baris per siswa.
// Diambil dari data yang sama dengan cetak rapor, jadi angkanya pasti sama dengan rapor.

const rata = (xs: number[]) => (xs.length ? Math.round(xs.reduce((a, b) => a + b, 0) / xs.length * 100) / 100 : "");

export function buildLeger(kelas: RaporKelas[]): Uint8Array {
	const wb = XLSX.utils.book_new();
	for (const k of kelas) {
		// Kolom mapel & ekskul = gabungan semua yang muncul di kelas ini, urut seperti di rapor.
		const mapel = new Map<string, string>();
		const ekskul = new Set<string>();
		for (const s of k.siswa) {
			s.kelompok.forEach(g => g.mapel.forEach(m => mapel.set(m.kode, m.singkat || m.nama)));
			s.ekskul.forEach(x => ekskul.add(x.nama));
		}
		const kodeMapel = [...mapel.keys()];
		const namaEkskul = [...ekskul].sort((a, b) => a.localeCompare(b, "id"));

		const judul: (string | number)[][] = [
			["LEGER NILAI RAPOR"],
			[`${k.sekolah.nama} · ${k.kelas.nama} · Fase ${k.kelas.fase} · Semester ${k.semesterKe} · Tahun Ajaran ${k.tahunAjaran}`],
			[]
		];
		const header = ["No", "NISN", "NIS", "Nama", ...kodeMapel.map(x => mapel.get(x)!), "Jumlah", "Rata-rata", "S", "I", "A", ...namaEkskul];

		const rows = k.siswa.map((s, i) => {
			const nilaiOf = new Map(s.kelompok.flatMap(g => g.mapel).map(m => [m.kode, m.nilai]));
			const nilai = kodeMapel.map(x => nilaiOf.get(x) ?? null);
			const ada = nilai.filter((v): v is number => v !== null);
			const ekskulOf = new Map(s.ekskul.map(x => [x.nama, x.predikat]));
			return [
				i + 1,
				s.nisn ?? "",
				s.nipd ?? "",
				s.nama,
				...nilai.map(v => v ?? ""),
				ada.length ? ada.reduce((a, b) => a + b, 0) : "",
				rata(ada),
				s.sakit ?? "",
				s.izin ?? "",
				s.alpa ?? "",
				...namaEkskul.map(n => ekskulOf.get(n) ?? "")
			];
		});

		// Baris rata-rata kelas per mapel.
		const rataMapel = kodeMapel.map((_, j) => rata(rows.map(r => r[4 + j]).filter((v): v is number => typeof v === "number")));
		const kaki = ["", "", "", "Rata-rata kelas", ...rataMapel];

		const ws = XLSX.utils.aoa_to_sheet([...judul, header, ...rows, kaki]);
		ws["!cols"] = [{ wch: 4 }, { wch: 12 }, { wch: 10 }, { wch: 32 }, ...kodeMapel.map(() => ({ wch: 9 })), { wch: 8 }, { wch: 9 }, { wch: 4 }, { wch: 4 }, { wch: 4 }, ...namaEkskul.map(() => ({ wch: 10 }))];
		ws["!merges"] = [{ s: { r: 0, c: 0 }, e: { r: 0, c: header.length - 1 } }, { s: { r: 1, c: 0 }, e: { r: 1, c: header.length - 1 } }];
		XLSX.utils.book_append_sheet(wb, ws, sheetName(k.kelas.nama));
	}
	return new Uint8Array(XLSX.write(wb, { type: "array", bookType: "xlsx" }));
}
