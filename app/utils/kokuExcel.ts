import * as XLSX from "xlsx";

// Nilai kokurikuler satu kelas: satu sheet per kegiatan, baris 2 = kunci kolom (jangan diubah),
// isi 1 = Berkembang, 2 = Cakap, 3 = Mahir (sama dengan format import e-Rapor). Kolom terakhir
// "Deskripsi" opsional: kalau diisi, menggantikan deskripsi otomatis di rapor.

interface Kegiatan { id: number, nama: string, kolom: KolomKoku[] }
interface Siswa { id: string, nisn: string | null, nama: string, deskripsi: string }

export function buildKokuWorkbook(kegiatan: Kegiatan[], siswa: Siswa[], nilai: Record<string, Record<string, number>>): Uint8Array {
	const wb = XLSX.utils.book_new();
	for (const k of kegiatan) {
		const judul = ["No", "NISN", "Nama", ...k.kolom.map(c => c.judul), "Deskripsi (opsional)"];
		const kunci = ["", "", "", ...k.kolom.map(c => c.kunci), "deskripsi"];
		const rows = siswa.map((s, i) => [i + 1, s.nisn ?? "", s.nama, ...k.kolom.map(c => nilai[`${k.id}:${s.id}`]?.[c.kunci] ?? ""), s.deskripsi]);
		const ws = XLSX.utils.aoa_to_sheet([judul, kunci, ...rows]);
		ws["!cols"] = [{ wch: 4 }, { wch: 12 }, { wch: 30 }, ...k.kolom.map(() => ({ wch: 14 })), { wch: 60 }];
		XLSX.utils.book_append_sheet(wb, ws, sheetName(k.nama));
	}
	const petunjuk = XLSX.utils.aoa_to_sheet([
		["Isi capaian: 1 = Berkembang, 2 = Cakap, 3 = Mahir. Kosong = belum dinilai."],
		["Jangan ubah nama sheet, baris ke-2 (kunci kolom), dan kolom NISN."],
		["Kolom Deskripsi boleh dikosongkan: rapor memakai kalimat otomatis dari capaian."]
	]);
	XLSX.utils.book_append_sheet(wb, petunjuk, "Petunjuk");
	return new Uint8Array(XLSX.write(wb, { type: "array", bookType: "xlsx" }));
}

export function readKokuWorkbook(data: ArrayBuffer, kegiatan: Kegiatan[], siswa: Siswa[]) {
	const wb = XLSX.read(data, { type: "array" });
	const nilai: { kegiatanId: number, siswaId: string, capaian: Record<string, number> }[] = [];
	const deskripsi = new Map<string, string>();
	const ditolak: string[] = [];
	for (const k of kegiatan) {
		const ws = wb.Sheets[sheetName(k.nama)];
		if (!ws)
			continue;
		const rows = XLSX.utils.sheet_to_json<unknown[]>(ws, { header: 1, blankrows: false });
		const kunci = (rows[1] ?? []).map(v => String(v ?? "").trim());
		const sah = new Set(k.kolom.map(c => c.kunci));
		for (const r of rows.slice(2)) {
			const nisn = String(r[1] ?? "").trim();
			const nama = String(r[2] ?? "").trim();
			const s = siswa.find(x => (nisn && x.nisn === nisn) || (!nisn && x.nama === nama));
			if (!s) {
				if (nisn || nama)
					ditolak.push(`${k.nama}: ${nama || nisn} bukan siswa kelas ini`);
				continue;
			}
			const capaian: Record<string, number> = {};
			kunci.forEach((key, i) => {
				if (!sah.has(key))
					return;
				const v = r[i];
				if (v === undefined || v === null || v === "")
					return;
				const n = Number(v);
				if ([1, 2, 3].includes(n))
					capaian[key] = n;
				else
					ditolak.push(`${k.nama}: ${s.nama} nilai "${v}" (harus 1, 2, atau 3)`);
			});
			nilai.push({ kegiatanId: k.id, siswaId: s.id, capaian });
			const iDesk = kunci.indexOf("deskripsi");
			const d = iDesk >= 0 ? String(r[iDesk] ?? "").trim() : "";
			if (d)
				deskripsi.set(s.id, d);
		}
	}
	return { nilai, deskripsi, ditolak };
}
