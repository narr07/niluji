import * as XLSX from "xlsx";

// Format Excel TP disamakan dengan "Format Import TP" e-Rapor SD (Panduan Gambar 175):
//   baris 1 : FORMAT IMPORT TP <MAPEL>, JENJANG SD
//   baris 3 : NO | TINGKAT | FASE | SEMESTER | TUJUAN PEMBELAJARAN (Maximal 100 Karakter)
// Satu sheet = satu mapel. File f_tp_*.xlsx hasil download dari e-Rapor juga bisa langsung di-import.

export const TP_MAX = 100;

export interface TpRow {
	kode_mapel: string
	tingkat: number
	semester: number
	deskripsi: string
	no?: number // kolom NO di file = urutan TP
}

export interface TpSheet {
	kode: string
	nama: string
	singkat: string
	rows: { tingkat: number, semester: number, deskripsi: string }[]
}

export interface TpIssue {
	sheet: string
	baris: number
	alasan: string
}

const HEADER = ["NO", "TINGKAT", "FASE", "SEMESTER", `TUJUAN PEMBELAJARAN (Maximal ${TP_MAX} Karakter)`];

export const tpSheetName = (s: string) => s.replace(/[[\]:*?/\\]/g, " ").slice(0, 31);

export function buildTpWorkbook(sheets: TpSheet[]): Uint8Array {
	const wb = XLSX.utils.book_new();
	for (const s of sheets) {
		const ws = XLSX.utils.aoa_to_sheet([
			[`FORMAT IMPORT TP ${s.nama.toUpperCase()}, JENJANG SD`],
			[],
			HEADER,
			...s.rows.map((r, i) => [i + 1, r.tingkat, faseOf(r.tingkat) ?? "", r.semester, r.deskripsi])
		]);
		ws["!cols"] = [{ wch: 5 }, { wch: 9 }, { wch: 7 }, { wch: 11 }, { wch: 100 }];
		ws["!merges"] = [{ s: { r: 0, c: 0 }, e: { r: 0, c: 4 } }];
		XLSX.utils.book_append_sheet(wb, ws, tpSheetName(s.singkat));
	}
	return new Uint8Array(XLSX.write(wb, { type: "array", bookType: "xlsx" }));
}

const norm = (s: unknown) => String(s ?? "").trim().toLowerCase().replace(/\s+/g, " ");

// Mapel per sheet ditentukan dari: judul "FORMAT IMPORT TP <MAPEL>", lalu nama sheet,
// lalu mapel yang sedang dipilih di layar (fallbackKode) — file e-Rapor nama sheet-nya "Worksheet".
export function readTpWorkbook(
	data: ArrayBuffer,
	mapel: { kode: string, nama: string, singkat: string }[],
	fallbackKode?: string
): { rows: TpRow[], issues: TpIssue[] } {
	const wb = XLSX.read(data, { type: "array" });
	const rows: TpRow[] = [];
	const issues: TpIssue[] = [];

	const cari = (teks: string) => {
		const t = norm(teks);
		if (!t)
			return undefined;
		return mapel.find(m => t === norm(m.nama) || t === norm(m.singkat))
			?? mapel.find(m => t.includes(norm(m.nama)))
			?? mapel.find(m => t.includes(norm(m.singkat)));
	};

	for (const name of wb.SheetNames) {
		const grid = XLSX.utils.sheet_to_json<unknown[]>(wb.Sheets[name]!, { header: 1, blankrows: true, defval: "" });
		const headerIdx = grid.findIndex(r => r.some(c => norm(c) === "tingkat"));
		if (headerIdx === -1)
			continue;
		const header = grid[headerIdx]!.map(norm);
		const col = {
			no: header.indexOf("no"),
			tingkat: header.indexOf("tingkat"),
			semester: header.indexOf("semester"),
			tp: header.findIndex(h => h.startsWith("tujuan pembelajaran"))
		};
		if (col.tingkat === -1 || col.semester === -1 || col.tp === -1) {
			issues.push({ sheet: name, baris: headerIdx + 1, alasan: "Kolom TINGKAT / SEMESTER / TUJUAN PEMBELAJARAN tidak ditemukan" });
			continue;
		}

		const judul = String(grid.slice(0, headerIdx).flat().find(c => norm(c).startsWith("format import tp")) ?? "");
		const dariJudul = judul.replace(/format import tp/i, "").replace(/,?\s*jenjang.*$/i, "");
		const m = cari(dariJudul) ?? cari(name) ?? mapel.find(x => x.kode === fallbackKode);
		if (!m) {
			issues.push({ sheet: name, baris: 1, alasan: `Mata pelajaran "${dariJudul.trim() || name}" tidak dikenal` });
			continue;
		}

		grid.slice(headerIdx + 1).forEach((r, i) => {
			const baris = headerIdx + 2 + i;
			const deskripsi = String(r[col.tp] ?? "").trim();
			const tingkat = Number(r[col.tingkat]);
			const semester = Number(r[col.semester]);
			if (!deskripsi)
				return; // baris kosong di template
			if (!(tingkat >= 1 && tingkat <= 6))
				issues.push({ sheet: name, baris, alasan: "Tingkat harus 1–6" });
			else if (![1, 2].includes(semester))
				issues.push({ sheet: name, baris, alasan: "Semester harus 1 atau 2" });
			else
				rows.push({ kode_mapel: m.kode, tingkat, semester, deskripsi, no: col.no >= 0 && Number(r[col.no]) > 0 ? Number(r[col.no]) : undefined });
		});
	}
	return { rows, issues };
}
