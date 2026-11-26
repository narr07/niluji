import { fetch as tauriFetch } from "@tauri-apps/plugin-http";

// Format Markdown hasil Export Bank Soal dan Publikasikan Bank Soal Gugus, beserta pemeriksaan
// koneksi ke GitHub. Dipakai import file Markdown lokal dan Tarik Soal Online (useBankSoalGugus.ts).

const CONNECTION_CHECK_URL = "https://raw.githubusercontent.com/";

export interface OnlineSubject {
	id: number
	name: string
	code: string | null
}

export interface BankSoalOnlineRow {
	id: string
	rowNumber: number
	tipe: "pg" | "esai"
	soal: string
	jenis: string
	pilihan_a: string
	pilihan_b: string
	pilihan_c: string
	pilihan_d: string
	pilihan_e?: string
	pilihan_a_gambar?: string
	pilihan_b_gambar?: string
	pilihan_c_gambar?: string
	pilihan_d_gambar?: string
	pilihan_e_gambar?: string
	kunci_jawaban: string
	skor: number | string
	// Nama file gambar soal di folder "gambar/" sebelah file soal.
	nama_file_gambar: string
	valid: boolean
	alasan_invalid?: string
}

export interface BankSoalConnectionStatus {
	connected: boolean
	checkedAt: Date
	message: string
}

// Frontmatter YAML sederhana (kelas/pelajaran/jenis) diikuti soal bernomor dengan heading
// "## N. Tipe", gambar (opsional) sebagai baris pertama isi blok, pilihan sebagai list "- A. teks",
// lalu baris "Kunci: X" dan "Skor: N". Sengaja regex manual karena strukturnya ditulis exporter
// NILUJI sendiri (src-tauri/src/export.rs).
export const parseSoalMarkdown = (text: string): { jenis: string, rows: BankSoalOnlineRow[] } => {
	const fmMatch = text.match(/^---\r?\n([\s\S]*?)\r?\n---\r?\n/);
	const frontmatter: Record<string, string> = {};
	let body = text;
	if (fmMatch) {
		for (const line of fmMatch[1]!.split(/\r?\n/)) {
			const m = line.match(/^([a-z_]+):\s*(.*)$/i);
			if (m) frontmatter[m[1]!] = m[2]!.trim();
		}
		body = text.slice(fmMatch[0].length);
	}

	const headingRegex = /^##\s*(\d+)\.\s*(Pilihan Ganda|Esai)\s*$/gm;
	const matches = [...body.matchAll(headingRegex)];
	const rows: BankSoalOnlineRow[] = [];

	matches.forEach((m, i) => {
		const number = Number(m[1]);
		const tipeLabel = m[2];
		const start = m.index! + m[0].length;
		const end = i + 1 < matches.length ? matches[i + 1]!.index! : body.length;
		let content = body.slice(start, end).trim();

		let imageFile: string | undefined;
		const imgMatch = content.match(/^!\[gambar]\(gambar\/([^)]+)\)\s*\n*/);
		if (imgMatch) {
			imageFile = imgMatch[1];
			content = content.slice(imgMatch[0].length);
		}

		const optionMatches = [...content.matchAll(/^-\s*([A-E])\.\s*(.*)$/gm)];
		const kunciMatch = content.match(/^Kunci:\s*(.+)$/m);
		const skorMatch = content.match(/^Skor:\s*([\d.]+)$/m);

		let soalEnd = content.length;
		if (optionMatches.length) soalEnd = optionMatches[0]!.index!;
		else if (skorMatch) soalEnd = skorMatch.index!;
		const soal = content.slice(0, soalEnd).trim();

		const tipe: "pg" | "esai" = tipeLabel === "Esai" ? "esai" : "pg";
		const pilihan: Record<string, string> = {};
		const pilihanGambar: Record<string, string> = {};
		for (const om of optionMatches) {
			const key = om[1]!.toLowerCase();
			let rawOpt = om[2]!.trim();
			const optImgMatch = rawOpt.match(/\s*!\[opsi-[A-E]]\(gambar\/([^)]+)\)\s*$/i);
			if (optImgMatch) {
				pilihanGambar[key] = optImgMatch[1]!;
				rawOpt = rawOpt.slice(0, optImgMatch.index).trim();
			}
			pilihan[key] = rawOpt;
		}
		const kunci_jawaban = kunciMatch ? kunciMatch[1]!.trim() : "";
		const skor = skorMatch ? Number(skorMatch[1]) : 1;

		const filledChoices = ["a", "b", "c", "d", "e"].filter((k) => Boolean(pilihan[k] || pilihanGambar[k]));
		const invalidReasons: string[] = [];
		if (!soal) invalidReasons.push("teks soal kosong");
		if (tipe === "pg") {
			if (filledChoices.length < 2) invalidReasons.push("kurang dari 2 pilihan terisi");
			if (!kunci_jawaban) invalidReasons.push("kunci jawaban kosong");
			else if (!filledChoices.includes(kunci_jawaban.toLowerCase())) invalidReasons.push("kunci jawaban tidak cocok dengan pilihan yang terisi");
		}

		const nama_file_gambar = imageFile ?? "";

		rows.push({
			id: `${tipe}-${number}`,
			rowNumber: number,
			tipe,
			soal,
			jenis: frontmatter.jenis ?? "",
			pilihan_a: pilihan.a ?? "",
			pilihan_b: pilihan.b ?? "",
			pilihan_c: pilihan.c ?? "",
			pilihan_d: pilihan.d ?? "",
			pilihan_e: pilihan.e ?? "",
			...(pilihanGambar.a ? { pilihan_a_gambar: pilihanGambar.a } : {}),
			...(pilihanGambar.b ? { pilihan_b_gambar: pilihanGambar.b } : {}),
			...(pilihanGambar.c ? { pilihan_c_gambar: pilihanGambar.c } : {}),
			...(pilihanGambar.d ? { pilihan_d_gambar: pilihanGambar.d } : {}),
			...(pilihanGambar.e ? { pilihan_e_gambar: pilihanGambar.e } : {}),
			kunci_jawaban,
			skor,
			nama_file_gambar,
			valid: invalidReasons.length === 0,
			...(invalidReasons.length ? { alasan_invalid: invalidReasons.join("; ") } : {})
		});
	});

	return { jenis: frontmatter.jenis ?? "", rows };
};

/** Memastikan GitHub benar-benar dapat dijangkau sebelum menarik bank soal. */
export const checkBankSoalConnection = async (): Promise<BankSoalConnectionStatus> => {
	try {
		const response = await tauriFetch(CONNECTION_CHECK_URL, { cache: "no-store" });
		if (!response.ok) throw new Error(`Server merespons status ${response.status}`);
		return {
			connected: true,
			checkedAt: new Date(),
			message: "Internet terhubung dan server bank soal dapat dijangkau."
		};
	} catch {
		return {
			connected: false,
			checkedAt: new Date(),
			message: "Tidak ada koneksi internet atau server bank soal tidak dapat dijangkau."
		};
	}
};
