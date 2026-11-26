// Lembar cetak kartu peserta: A4 tegak, 2 kolom × 5 baris = 10 kartu per halaman. Kartu dibatasi
// garis tipis yang dipakai bersama oleh kartu bersebelahan, jadi satu guntingan di garis itu
// memisahkan dua kartu sekaligus. Semua kartu berukuran sama persis (97 × 56 mm).

export interface KartuPesertaData {
	name: string
	nisn: string
	class: string | null
	rombel?: string | null
	pin?: string
}

export const KARTU_PER_HALAMAN = 10;

const escapeHtml = (value: string) =>
	value.replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", "\"": "&quot;", "'": "&#39;" })[c]!);

const kelasLabel = (kelas: string | null, rombel?: string | null) => {
	const k = kelas?.trim();
	const r = rombel?.trim();
	if (k && r) return `${k} · ${r}`;
	return k || r || "-";
};

const STYLE = `
@page { size: A4 portrait; margin: 8mm; }
* { box-sizing: border-box; margin: 0; padding: 0; }
body { font-family: Arial, Helvetica, sans-serif; color: #111; background: #fff; }
.page { display: grid; grid-template-columns: 97mm 97mm; grid-template-rows: repeat(5, 56mm); width: 194mm; break-after: page; align-content: start; }
.page:last-child { break-after: auto; }
.kartu { border: 0.3mm solid #777; margin: 0 -0.3mm -0.3mm 0; padding: 5mm 6mm; display: flex; flex-direction: column; justify-content: space-between; overflow: hidden; break-inside: avoid; }
.judul { font-size: 8pt; font-weight: 700; letter-spacing: 1.2pt; text-transform: uppercase; color: #555; }
.sekolah { font-size: 10pt; font-weight: 700; margin-top: 0.5mm; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.nama { font-size: 15pt; font-weight: 700; line-height: 1.15; display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden; }
.baris { display: flex; gap: 6mm; font-size: 11pt; }
.label { font-size: 8pt; color: #555; text-transform: uppercase; letter-spacing: 0.8pt; }
.nisn { font-family: "Courier New", monospace; font-weight: 700; }
.pin { font-family: "Courier New", monospace; font-size: 22pt; font-weight: 700; letter-spacing: 3pt; line-height: 1; }
`;

const kartuHtml = (student: KartuPesertaData, schoolName: string) => `
<div class="kartu">
	<div>
		<p class="judul">Kartu Peserta Ujian</p>
		<p class="sekolah">${escapeHtml(schoolName)}</p>
	</div>
	<p class="nama">${escapeHtml(student.name)}</p>
	<div class="baris">
		<div><p class="label">NISN</p><p class="nisn">${escapeHtml(student.nisn)}</p></div>
		<div><p class="label">Kelas</p><p>${escapeHtml(kelasLabel(student.class, student.rombel))}</p></div>
	</div>
	<div><p class="label">PIN</p><p class="pin">${escapeHtml(student.pin || "-")}</p></div>
</div>`;

export const buildKartuPesertaHtml = (students: KartuPesertaData[], schoolName: string) => {
	const pages: string[] = [];
	for (let i = 0; i < students.length; i += KARTU_PER_HALAMAN) {
		const cards = students.slice(i, i + KARTU_PER_HALAMAN).map((s) => kartuHtml(s, schoolName)).join("");
		pages.push(`<section class="page">${cards}</section>`);
	}
	return `<!doctype html><html lang="id"><head><meta charset="utf-8"><title>Kartu Peserta Ujian</title><style>${STYLE}</style></head><body>${pages.join("")}</body></html>`;
};
