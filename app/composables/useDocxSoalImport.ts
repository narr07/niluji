import JSZip from "jszip";

export interface DocxImportRow {
	id: string
	number: number
	tipe: "pg" | "esai"
	soal: string
	pilihan_a: string
	pilihan_b: string
	pilihan_c: string
	pilihan_d: string
	kunci_jawaban: string
	catatanJawabanEsai?: string
	imageFileName?: string
	imageBytes?: Uint8Array
	valid: boolean
	alasanInvalid?: string
}

interface Para { text: string, listKey: string | null, imageRelIds: string[] }

const decodeXmlEntities = (s: string) =>
	s.replace(/&lt;/g, "<").replace(/&gt;/g, ">").replace(/&amp;/g, "&").replace(/&quot;/g, "\"").replace(/&apos;/g, "'");

// Word list numbering is multi-level: <w:numId> ganjil dipakai bareng oleh banyak level
// (<w:ilvl>), masing-masing level punya format/counter SENDIRI (mis. ilvl 0 = "I, II, III",
// ilvl 3 = "a, b, c" di bawah numId yang sama). Jadi kuncinya numId+ilvl DIGABUNG, bukan numId
// saja — dan formatnya (decimal/lowerLetter/dst di numbering.xml) sengaja tidak dicek sama
// sekali; yang menentukan daftar mana yang "nomor soal" adalah cocok-tidaknya jumlah pemakaian
// dengan total soal dari kunci jawaban (lihat pickQuestionListKey).
const parseParagraphs = (documentXml: string): Para[] => {
	const paras: Para[] = [];
	for (const m of documentXml.matchAll(/<w:p\b[^>]*>([\s\S]*?)<\/w:p>/g)) {
		const p = m[1]!;
		const numPrMatch = /<w:numPr>([\s\S]*?)<\/w:numPr>/.exec(p);
		let listKey: string | null = null;
		if (numPrMatch) {
			const numId = /<w:numId w:val="(\d+)"\/>/.exec(numPrMatch[1]!)?.[1];
			const ilvl = /<w:ilvl w:val="(\d+)"\/>/.exec(numPrMatch[1]!)?.[1] ?? "0";
			if (numId) listKey = `${numId}:${ilvl}`;
		}
		const imageRelIds = [...p.matchAll(/<a:blip r:embed="(rId\d+)"/g)].map((x) => x[1]!);
		const text = decodeXmlEntities([...p.matchAll(/<w:t[^>]*>([^<]*)<\/w:t>/g)].map((x) => x[1]).join(""));
		paras.push({ text, listKey, imageRelIds });
	}
	return paras;
};

// Dokumen kadang punya beberapa daftar bernomor sekaligus (nomor soal, tapi bisa juga daftar
// lain yang tidak berhubungan, mis. sub-poin di paragraf bacaan) — kalau semuanya dianggap
// penanda soal baru, hasilnya soal bisa terpecah jadi ratusan "blok" palsu. Jadi yang dipakai
// cuma SATU daftar (numId+ilvl): yang jumlah pemakaiannya di dokumen paling dekat dengan total
// soal yang sudah diketahui dari kunci jawaban.
const pickQuestionListKey = (paras: Para[], expectedTotal: number): string | null => {
	const counts = new Map<string, number>();
	for (const p of paras) {
		if (p.listKey) counts.set(p.listKey, (counts.get(p.listKey) ?? 0) + 1);
	}
	let best: string | null = null;
	let bestDiff = Infinity;
	for (const [key, count] of counts) {
		const diff = Math.abs(count - expectedTotal);
		if (diff < bestDiff) {
			best = key;
			bestDiff = diff;
		}
	}
	return best;
};

const isQuestionStart = (p: Para, questionListKey: string | null) =>
	/^\s*\d+[.)]\s/.test(p.text) || (p.listKey !== null && p.listKey === questionListKey);
const stripLeadingNumber = (text: string) => text.replace(/^\s*\d+[.)]\s*/, "").trim();

// Tiga pola pilihan yang benar-benar ditemukan di file soal STS asli — file cetak selalu ada
// variasi formatnya, jadi dicoba berurutan dari yang paling ketat ke yang paling longgar.
const splitOptions = (lines: string[]): { soalLines: string[], options: [string, string, string, string] | null } => {
	const last = lines[lines.length - 1] ?? "";
	// Pola 1: satu baris "a. xxx b. xxx c. xxx d. xxx"
	const singleLine = last.match(/^\s*a\s*[.)]\s*(.*?)\s*b\s*[.)]\s*(.*?)\s*c\s*[.)]\s*(.*?)\s*d\s*[.)]\s*(.*?)\s*$/i);
	if (singleLine) {
		return { soalLines: lines.slice(0, -1), options: [singleLine[1]!, singleLine[2]!, singleLine[3]!, singleLine[4]!] };
	}

	// Pola 2: dua baris terakhir "kiri<spasi>c/d. kanan" (label kiri opsional). Jarak antar
	// pilihan bisa berupa banyak spasi (rata kolom) ATAU nol spasi sama sekali (langsung
	// nempel ke label berikutnya, mis. "...sintac. Rani dan sinta") — dua-duanya ditemukan di
	// file asli, jadi jaraknya dibuat longgar (nol atau lebih), bukan wajib 2+.
	if (lines.length >= 2) {
		const [line1, line2] = lines.slice(-2) as [string, string];
		const split2 = (line: string, leftLetter: string, rightLetter: string) => {
			const m = line.match(new RegExp(`^(?:${leftLetter}\\s*[.)]\\s*)?(.*?)\\s*${rightLetter}\\s*[.)]\\s*(.*)$`, "i"));
			return m ? ([m[1]!.trim(), m[2]!.trim()] as const) : null;
		};
		const row1 = split2(line1, "a", "c");
		const row2 = split2(line2, "b", "d");
		if (row1 && row2 && row1[0] && row2[0]) {
			return { soalLines: lines.slice(0, -2), options: [row1[0], row2[0], row1[1], row2[1]] };
		}
	}

	// Pola 3: tanpa label huruf sama sekali — 4 baris terakhir masing-masing satu pilihan polos
	if (lines.length >= 4) {
		const lastFour = lines.slice(-4).map((l) => l.trim());
		if (lastFour.every((l) => l.length > 0 && l.length < 120)) {
			return { soalLines: lines.slice(0, -4), options: lastFour as [string, string, string, string] };
		}
	}

	return { soalLines: lines, options: null };
};

const normalizeAnswer = (v: string) => v.trim().toLowerCase().replace(/[^a-z]/g, "");

const groupIntoBlocks = (paras: Para[], questionListKey: string | null) => {
	const blocks: Para[][] = [];
	let current: Para[] | null = null;
	for (const p of paras) {
		if (!p.text.trim() && p.imageRelIds.length === 0) continue;
		if (isQuestionStart(p, questionListKey)) {
			if (current) blocks.push(current);
			current = [p];
		} else if (current) {
			current.push(p);
		}
	}
	if (current) blocks.push(current);
	return blocks;
};

export interface KunciJawabanResult {
	pg: Map<number, string>
	esai: Map<number, string>
	detectedPgCount?: number
	detectedEssayCount?: number
}

// Bagian "I. Pilihan Ganda" selalu tabel No/Jawaban yang bersih; bagian "II. Uraian" bisa dua
// gaya ("31" lalu "jawabannya" di paragraf terpisah, ATAU "31. jawabannya" digabung satu baris)
// — keduanya ditangani karena kedua contoh kunci jawaban asli memakai gaya yang berbeda.
export const parseKunciJawabanDocx = async (bytes: Uint8Array): Promise<KunciJawabanResult> => {
	const zip = await JSZip.loadAsync(bytes);
	const documentXml = await zip.file("word/document.xml")?.async("string");
	if (!documentXml) throw new Error("File kunci jawaban tidak valid (word/document.xml tidak ditemukan).");

	const texts = [...documentXml.matchAll(/<w:t[^>]*>([^<]*)<\/w:t>/g)]
		.map((m) => decodeXmlEntities(m[1]!).trim())
		.filter((t) => t.length > 0);

	// Header "Jumlah Soal: 30 Soal Pilihan Ganda, 10 Soal Uraian" — kalau ada, ini yang paling
	// bisa diandalkan buat tahu berapa soal PG/esai yang harus diambil dari naskah soal.
	const fullText = texts.join(" ");
	const pgCountMatch = fullText.match(/(\d+)\s*Soal Pilihan Ganda/i);
	const essayCountMatch = fullText.match(/(\d+)\s*Soal Uraian/i);
	const detectedPgCount = pgCountMatch ? Number(pgCountMatch[1]) : undefined;
	const detectedEssayCount = essayCountMatch ? Number(essayCountMatch[1]) : undefined;

	// Cari header seksi "II. Uraian" secara spesifik — bukan sekadar kata "uraian" di mana saja,
	// karena baris ringkasan di atas ("Jumlah Soal: ... Soal Uraian") juga mengandung kata itu
	// dan jadi ketemu duluan kalau pencariannya longgar, bikin data PG kepotong jadi kosong.
	const uraianIndex = texts.findIndex((t) => /^i{1,2}\s*[.)]?\s*uraian/i.test(t.trim()));
	const pgTexts = uraianIndex === -1 ? texts : texts.slice(0, uraianIndex);
	const esaiTexts = uraianIndex === -1 ? [] : texts.slice(uraianIndex + 1);

	const pg = new Map<number, string>();
	// Gaya tabel "No" / "Jawaban" berdampingan (paragraf angka, lalu paragraf huruf terpisah).
	for (let i = 0; i < pgTexts.length - 1; i++) {
		const num = Number(pgTexts[i]);
		if (!Number.isInteger(num) || num <= 0) continue;
		const answer = pgTexts[i + 1]!;
		if (/^[a-dA-D]$/.test(answer.trim())) pg.set(num, answer.trim().toUpperCase());
	}
	// Gaya "1. a" (nomor dan huruf digabung satu paragraf, tanpa tabel sama sekali).
	for (const t of pgTexts) {
		const m = t.trim().match(/^(\d+)\s*[.)]\s*([a-dA-D])$/);
		if (m) pg.set(Number(m[1]), m[2]!.toUpperCase());
	}

	const esai = new Map<number, string>();
	// Gaya "31. jawabannya" (satu paragraf)
	for (const t of esaiTexts) {
		const m = t.match(/^(\d+)[.)]\s*(.*)$/);
		if (m) esai.set(Number(m[1]), m[2]!.trim());
	}
	// Gaya "31" lalu "jawabannya" di paragraf terpisah — cuma dipakai untuk nomor yang belum
	// ketemu lewat gaya di atas, supaya tidak menimpa hasil yang sudah benar.
	for (let i = 0; i < esaiTexts.length - 1; i++) {
		const num = Number(esaiTexts[i]);
		if (!Number.isInteger(num) || num <= 0 || esai.has(num)) continue;
		esai.set(num, esaiTexts[i + 1]!.trim());
	}

	return { pg, esai, detectedPgCount, detectedEssayCount };
};

const readRels = async (zip: JSZip): Promise<Map<string, string>> => {
	const relsXml = await zip.file("word/_rels/document.xml.rels")?.async("string");
	const rels = new Map<string, string>();
	if (!relsXml) return rels;
	for (const m of relsXml.matchAll(/<Relationship Id="(rId\d+)"[^>]*Target="([^"]+)"/g)) {
		rels.set(m[1]!, m[2]!.replace(/^\.?\//, ""));
	}
	return rels;
};

export const parseNaskahSoalDocx = async (
	bytes: Uint8Array,
	pgCount: number,
	essayCount: number,
	kunci: KunciJawabanResult
): Promise<DocxImportRow[]> => {
	const zip = await JSZip.loadAsync(bytes);
	const documentXml = await zip.file("word/document.xml")?.async("string");
	if (!documentXml) throw new Error("File naskah soal tidak valid (word/document.xml tidak ditemukan).");
	const rels = await readRels(zip);

	const paras = parseParagraphs(documentXml);
	const totalExpected = pgCount + essayCount;
	const questionListKey = pickQuestionListKey(paras, totalExpected);
	const blocks = groupIntoBlocks(paras, questionListKey);

	// Tidak ada penanda nomor soal (baik teks "1." literal maupun satu daftar bernomor otomatis
	// yang konsisten) yang jumlahnya cukup dekat dengan total soal dari kunci jawaban — daripada
	// diam-diam menghasilkan pemisahan yang salah, lebih baik berhenti dengan pesan jelas supaya
	// soal ini ditambahkan manual saja.
	if (Math.abs(blocks.length - totalExpected) > totalExpected * 0.3) {
		throw new Error(
			`Penomoran soal di file ini tidak terdeteksi dengan jelas (ketemu ${blocks.length} kemungkinan soal, ` +
			`padahal kunci jawaban menyebutkan ${totalExpected}). Formatnya kemungkinan beda dari yang sudah didukung — ` +
			"soal untuk file ini perlu ditambahkan manual dulu."
		);
	}

	const questionBlocks = blocks.slice(-totalExpected);

	const rows: DocxImportRow[] = [];
	for (let i = 0; i < questionBlocks.length; i++) {
		const block = questionBlocks[i]!;
		const number = i + 1;
		const isPg = i < pgCount;
		const lines = block.map((p) => p.text);
		lines[0] = stripLeadingNumber(lines[0]!);

		const relId = block.flatMap((p) => p.imageRelIds)[0];
		const mediaPath = relId ? rels.get(relId) : undefined;
		let imageBytes: Uint8Array | undefined;
		let imageFileName: string | undefined;
		if (mediaPath) {
			const file = zip.file(`word/${mediaPath}`);
			if (file) {
				imageBytes = await file.async("uint8array");
				imageFileName = mediaPath.split("/").pop();
			}
		}

		const invalidReasons: string[] = [];
		let soal: string;
		let options: [string, string, string, string] | null = null;

		if (isPg) {
			const split = splitOptions(lines);
			soal = split.soalLines.join(" ").trim();
			options = split.options;
			if (!options) invalidReasons.push("pilihan A-D tidak terbaca otomatis, isi manual");
		} else {
			soal = lines.join(" ").trim();
		}
		if (!soal) invalidReasons.push("teks soal kosong");

		const kunciJawaban = isPg ? (kunci.pg.get(number) ?? "") : "";
		if (isPg && options) {
			if (!kunciJawaban) {
				invalidReasons.push("kunci jawaban tidak ditemukan di file kunci untuk nomor ini");
			} else {
				const idx = ["a", "b", "c", "d"].indexOf(normalizeAnswer(kunciJawaban));
				if (idx === -1 || !options[idx]) invalidReasons.push("kunci jawaban tidak cocok dengan pilihan yang ada");
			}
		}

		rows.push({
			id: `${isPg ? "pg" : "esai"}-${number}`,
			number,
			tipe: isPg ? "pg" : "esai",
			soal,
			pilihan_a: options?.[0] ?? "",
			pilihan_b: options?.[1] ?? "",
			pilihan_c: options?.[2] ?? "",
			pilihan_d: options?.[3] ?? "",
			kunci_jawaban: kunciJawaban,
			catatanJawabanEsai: !isPg ? kunci.esai.get(number) : undefined,
			imageFileName,
			imageBytes,
			valid: invalidReasons.length === 0,
			alasanInvalid: invalidReasons.length ? invalidReasons.join("; ") : undefined
		});
	}

	return rows;
};
