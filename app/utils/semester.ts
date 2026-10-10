// semester_id Dapodik: <tahun><1|2>, mis. 20251 = 2025/2026 Ganjil, 20252 = 2025/2026 Genap.
export function semesterLabel(id?: string | null): string {
	if (!id || !/^\d{5}$/.test(id))
		return id || "-";
	const y = Number(id.slice(0, 4));
	return `${y}/${y + 1} ${id.endsWith("1") ? "Ganjil" : "Genap"}`;
}

export function tahunAjaran(id: string): string {
	const y = Number(id.slice(0, 4));
	return `${y}/${y + 1}`;
}

// Semester yang sedang berjalan menurut tanggal: Juli–Desember = Ganjil, Januari–Juni = Genap.
export function currentSemesterId(date = new Date()): string {
	const y = date.getFullYear();
	return date.getMonth() >= 6 ? `${y}1` : `${y - 1}2`;
}

// Pilihan semester di halaman Ambil Data Dapodik (Dapodik tidak punya endpoint daftar semester).
export function semesterOptions(yearsBack = 3): { label: string, value: string }[] {
	const cur = currentSemesterId();
	const startYear = Number(cur.slice(0, 4)) - yearsBack;
	const out: { label: string, value: string }[] = [];
	for (let y = Number(cur.slice(0, 4)); y >= startYear; y--) {
		for (const s of ["2", "1"]) {
			const id = `${y}${s}`;
			if (id <= cur)
				out.push({ label: semesterLabel(id), value: id });
		}
	}
	return out;
}
