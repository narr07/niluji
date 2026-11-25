// Pengurai naskah soal yang dipakai import PDF dan import Word. Kedua format lebih dulu diubah
// menjadi baris teks seperti yang terlihat di halaman (PDF: disusun ulang dari posisi teks,
// termasuk tata letak dua kolom; Word: lihat docxNaskahReader.ts). Lalu soal, opsi, bacaan,
// dan kunci dibaca dari pola teksnya. Semua fungsi di sini murni supaya bisa diuji langsung.

export interface PdfTextItem {
	str: string
	x: number
	// Garis dasar teks, diukur dari atas halaman.
	y: number
	width: number
	height: number
}

export interface PdfImageBox {
	id: string
	x: number
	// Tepi atas gambar, diukur dari atas halaman.
	y: number
	width: number
	height: number
}

export interface PdfPageData {
	width: number
	height: number
	items: PdfTextItem[]
	images: PdfImageBox[]
}

export type ReadingLine =
	| { kind: "text", text: string, page: number }
	| { kind: "image", imageId: string, page: number };

export interface ReadingResult {
	lines: ReadingLine[]
	twoColumnPages: number
}

const FOOTER_BAND = 0.08;
const QUESTION_LIKE = /^\d{1,3}\s*[.)]/;

interface TextRow { y: number, text: string }

const groupRows = (items: PdfTextItem[]): TextRow[] => {
	const sorted = [...items].sort((a, b) => a.y - b.y || a.x - b.x);
	const rows: { y: number, h: number, items: PdfTextItem[] }[] = [];
	for (const item of sorted) {
		// Subskrip/superskrip sedikit bergeser dari garis dasarnya, jadi beberapa baris terakhir ikut dicek.
		const row = rows.slice(-3).find((r) => Math.abs(r.y - item.y) <= Math.max(r.h, item.height, 1) * 0.45);
		if (row) {
			row.items.push(item);
			row.h = Math.max(row.h, item.height);
		} else {
			rows.push({ y: item.y, h: item.height, items: [item] });
		}
	}
	return rows.map((row) => {
		let text = "";
		let prevEnd = -Infinity;
		for (const item of row.items.sort((a, b) => a.x - b.x)) {
			const gap = item.x - prevEnd;
			if (text && gap > Math.max(item.height, 1) * 0.12 && !/\s$/.test(text) && !/^\s/.test(item.str)) text += " ";
			text += item.str;
			prevEnd = Math.max(prevEnd, item.x + item.width);
		}
		return { y: row.y, text: text.replace(/\s+/g, " ").trim() };
	}).filter((row) => row.text.length > 0);
};

interface ColumnSplit { x: number, headerBottom: number, footerTop: number }

// Halaman dua kolom dikenali dari lorong kosong vertikal yang tidak dilintasi teks apa pun di
// bawah judul. Opsi berjajar "a. ... c. ..." juga membentuk lorong, jadi kolom kanan wajib
// memuat nomor soal sendiri agar tidak keliru dianggap kolom.
const findColumnSplit = (page: PdfPageData): ColumnSplit | null => {
	const items = page.items.filter((it) => it.str.trim());
	if (items.length < 10) return null;
	const footerTop = page.height * (1 - FOOTER_BAND);
	const valid: { x: number, headerBottom: number }[] = [];
	for (let x = page.width * 0.3; x <= page.width * 0.7; x += 2) {
		let headerBottom = -Infinity;
		for (const it of items) {
			if (it.y < footerTop && it.x < x - 1 && it.x + it.width > x + 1) headerBottom = Math.max(headerBottom, it.y);
		}
		const body = items.filter((it) => it.y > headerBottom + 0.5 && it.y < footerTop);
		if (body.length < items.length * 0.5) continue;
		const left = body.filter((it) => it.x + it.width <= x + 1);
		const right = body.filter((it) => it.x >= x - 1);
		if (left.length < 4 || right.length < 4) continue;
		const extent = (arr: PdfTextItem[]) => Math.max(...arr.map((i) => i.y)) - Math.min(...arr.map((i) => i.y));
		const bodyExtent = extent(body);
		if (extent(left) < bodyExtent * 0.5 || extent(right) < bodyExtent * 0.5) continue;
		valid.push({ x, headerBottom });
	}
	if (!valid.length) return null;

	// Titik belah diambil dari tengah rentangan lorong terpanjang.
	let bestStart = 0;
	let bestLen = 0;
	for (let i = 0; i < valid.length;) {
		let j = i;
		while (j + 1 < valid.length && valid[j + 1]!.x - valid[j]!.x <= 2.5) j++;
		if (j - i + 1 > bestLen) {
			bestStart = i;
			bestLen = j - i + 1;
		}
		i = j + 1;
	}
	const chosen = valid[bestStart + Math.floor(bestLen / 2)]!;
	const rightRows = groupRows(items.filter((it) => it.y > chosen.headerBottom + 0.5 && it.y < footerTop && it.x >= chosen.x - 1));
	const leftRows = groupRows(items.filter((it) => it.y > chosen.headerBottom + 0.5 && it.y < footerTop && it.x + it.width <= chosen.x + 1));
	if (!rightRows.some((r) => QUESTION_LIKE.test(r.text)) || !leftRows.some((r) => QUESTION_LIKE.test(r.text))) return null;
	return { x: chosen.x, headerBottom: chosen.headerBottom, footerTop };
};

interface Placed { y: number, line: ReadingLine }

const orderRegion = (rows: TextRow[], images: PdfImageBox[], page: number): ReadingLine[] => {
	const placed: Placed[] = [
		...rows.map((r) => ({ y: r.y, line: { kind: "text", text: r.text, page } as ReadingLine })),
		// Gambar diurutkan memakai titik tengahnya, supaya gambar di samping teks soal tetap ikut soal itu.
		...images.map((img) => ({ y: img.y + img.height / 2, line: { kind: "image", imageId: img.id, page } as ReadingLine }))
	];
	return placed.sort((a, b) => a.y - b.y).map((p) => p.line);
};

const pageLines = (page: PdfPageData, pageIndex: number): { lines: ReadingLine[], twoColumn: boolean } => {
	const split = findColumnSplit(page);
	if (!split) return { lines: orderRegion(groupRows(page.items), page.images, pageIndex), twoColumn: false };

	const { x, headerBottom, footerTop } = split;
	const isHeader = (y: number) => y <= headerBottom + 0.5;
	const isFooter = (y: number) => y >= footerTop;
	const header = page.items.filter((it) => isHeader(it.y) && !isFooter(it.y));
	const footer = page.items.filter((it) => isFooter(it.y));
	const body = page.items.filter((it) => !isHeader(it.y) && !isFooter(it.y));
	const left = body.filter((it) => it.x + it.width / 2 < x);
	const right = body.filter((it) => it.x + it.width / 2 >= x);

	const imgCenterY = (img: PdfImageBox) => img.y + img.height / 2;
	const headerImages = page.images.filter((img) => isHeader(img.y));
	const bodyImages = page.images.filter((img) => !isHeader(img.y));
	const leftImages = bodyImages.filter((img) => img.x + img.width / 2 < x && imgCenterY(img) < footerTop);
	const rightImages = bodyImages.filter((img) => img.x + img.width / 2 >= x && imgCenterY(img) < footerTop);

	return {
		lines: [
			...orderRegion(groupRows(header), headerImages, pageIndex),
			...orderRegion(groupRows(left), leftImages, pageIndex),
			...orderRegion(groupRows(right), rightImages, pageIndex),
			...orderRegion(groupRows(footer), [], pageIndex)
		],
		twoColumn: true
	};
};

const PAGE_NUMBER = /^(?:halaman|hal\.?|page)?\s*[-–—]?\s*\d{1,3}\s*[-–—]?\s*(?:(?:dari|of|\/)\s*\d{1,3})?$/i;

// Kepala dan kaki halaman (nama sekolah, "Halaman 2 dari 4") berulang di setiap halaman dan
// akan menempel ke soal terakhir halaman itu kalau tidak dibuang.
const dropRepeatedHeaders = (perPage: ReadingLine[][]): ReadingLine[] => {
	const edgeIndexes = (lines: ReadingLine[]) => {
		const textIdx = lines.map((l, i) => (l.kind === "text" ? i : -1)).filter((i) => i >= 0);
		return new Set([...textIdx.slice(0, 2), ...textIdx.slice(-2)]);
	};
	const keyOf = (text: string) => text.toLowerCase().replace(/\d+/g, "#").replace(/\s+/g, " ").trim();
	const counts = new Map<string, number>();
	for (const lines of perPage) {
		const keys = new Set<string>();
		for (const i of edgeIndexes(lines)) keys.add(keyOf((lines[i] as { text: string }).text));
		for (const key of keys) counts.set(key, (counts.get(key) ?? 0) + 1);
	}
	const threshold = Math.max(2, Math.ceil(perPage.length * 0.6));
	return perPage.flatMap((lines) => {
		const edges = edgeIndexes(lines);
		return lines.filter((line, i) => {
			if (line.kind !== "text" || !edges.has(i)) return true;
			if (PAGE_NUMBER.test(line.text)) return false;
			// Baris soal atau opsi yang kebetulan sama di dua halaman bukan kepala halaman.
			if (QUESTION_LIKE.test(line.text) || /^\(?[a-e]\s*[.)]\s/i.test(line.text)) return true;
			return !(perPage.length >= 2 && (counts.get(keyOf(line.text)) ?? 0) >= threshold);
		});
	});
};

export const buildReadingLines = (pages: PdfPageData[]): ReadingResult => {
	let twoColumnPages = 0;
	const perPage = pages.map((page, index) => {
		const result = pageLines(page, index);
		if (result.twoColumn) twoColumnPages++;
		return result.lines;
	});
	return { lines: dropRepeatedHeaders(perPage), twoColumnPages };
};

// ---------------------------------------------------------------------------------------------
// Soal, opsi, dan kunci
// ---------------------------------------------------------------------------------------------

export type Section = "pg" | "esai";

export interface PdfStimulus {
	lines: string[]
	imageIds: string[]
}

export interface SoalBlock {
	number: number
	section: Section | null
	lines: string[]
	imageIds: string[]
	inlineKunci?: string
	// Nomor yang tertulis di naskah bila berbeda dari urutannya (salah ketik).
	writtenNumber?: number
	// Bacaan atau gambar bersama ("Bacalah teks berikut untuk soal no. 1-5") yang ikut ditampilkan di soal ini.
	stimulus?: PdfStimulus
}

export interface NaskahParseResult {
	blocks: SoalBlock[]
	kunciLines: string[]
	missingNumbers: number[]
}

const QUESTION_START = /^(\d{1,3})\s*[.)]\s*(?!\d)(.*)$/;
const PG_KEYWORD = /pilihan\s+ganda|pilihlah|berilah\s+tanda\s+silang|beri\s+tanda\s+silang|silanglah|multiple\s+choice|choose\s+the\s+(?:correct|best|right)/i;
// Kata utuh saja: "Kepolisian" tidak boleh terbaca sebagai judul bagian "Isian".
const ESAI_KEYWORD = /\b(?:uraian|esai|essay|isian|isilah|jawablah)\b|\bkerjakan\s+soal|\banswer\s+the\s+(?:following\s+)?questions?\b|\bfill\s+in\s+the\s+blanks?\b/i;
// Judul bagian yang diawali huruf ("B. Esai") mirip baris opsi, jadi wajib menyebut nama bagiannya.
const STRONG_SECTION = /\b(?:pilihan\s+ganda|uraian|esai|essay|isian)\b/i;
const OPTION_PREFIX = /^\(?[a-eA-E]\s*[.)]\s/;
// Nomor tanpa titik ("4 Perhatikan ...") hanya diterima bila tepat melanjutkan nomor sebelumnya.
const LOOSE_QUESTION_START = /^(\d{1,3})\s+([A-Z“"‘(].*)$/;
const ROMAN_HEADER = /^[IVX]{1,4}\s*[.)]\s+/;
const KUNCI_HEADING = /^(?:kunci(?:\s+jawaban)?|kunci\s+dan\s+pembahasan|jawaban\s+soal)\b/i;
const INLINE_KUNCI = /^(?:kunci(?:\s+jawaban)?|jawaban(?:\s+(?:yang\s+)?benar)?)\s*[:=]\s*\(?([a-e])\)?\s*\.?$/i;
const KUNCI_PAIR = /(?<![\w.])(\d{1,3})\s*[.):=-]?\s*([a-e])(?!\w)/gi;
const PETUNJUK = /^petunjuk\b/i;
// Garis tempat menjawab di soal uraian: "Jawab", "Jawaban :", atau baris titik-titik.
const ANSWER_BLANK = /^(?:(?:jawab(?:an)?|answer)\s*:?)?[\s.…_-]*$/i;
const STIMULUS_INTRO = /^(?:bacalah|perhatikan|cermatilah|simaklah|amatilah|read|look\s+at)\b/i;
const STIMULUS_OBJECT = /\b(?:teks|text|bacaan|cerita|paragraf|puisi|dongeng|wacana|dialog|percakapan|gambar|tabel|story|passage|picture)\b/i;
// Rentang nomor di kalimat pengantar: "soal no. 1-5", "nomor 8 sampai 14", "number 31 to 32".
const STIMULUS_RANGE = /\b(\d{1,3})\s*(?:-|–|—|s\.?\s*d\.?|sampai(?:\s+dengan)?|hingga|dan|to|and)\s*(\d{1,3})\b/i;

const sectionOf = (text: string): Section | null => {
	if (text.length > 140 || QUESTION_START.test(text)) return null;
	if (OPTION_PREFIX.test(text) && !STRONG_SECTION.test(text)) return null;
	const esai = ESAI_KEYWORD.test(text);
	const pg = PG_KEYWORD.test(text);
	if (esai === pg) return null;
	return esai ? "esai" : "pg";
};

const isKunciPairLine = (line: string) => {
	const pairs = [...line.matchAll(KUNCI_PAIR)];
	if (!pairs.length) return false;
	// Hanya huruf dan angka yang dihitung, supaya "1 B." dari sel tabel tetap terbaca sebagai pasangan kunci.
	const covered = pairs.reduce((sum, m) => sum + m[0].replace(/[^\p{L}\p{N}]/gu, "").length, 0);
	return covered >= line.replace(/[^\p{L}\p{N}]/gu, "").length * 0.7;
};

interface StimulusDraft extends PdfStimulus {
	from: number | null
	to: number | null
}

export const parseNaskahLines = (lines: ReadingLine[]): NaskahParseResult => {
	const texts = lines.map((l) => (l.kind === "text" ? l.text : ""));
	const nextTextIndexes = (from: number, count: number) => {
		const found: number[] = [];
		for (let i = from + 1; i < lines.length && found.length < count; i++) {
			if (lines[i]!.kind === "text") found.push(i);
		}
		return found;
	};

	// Bagian kunci di akhir naskah: judul "Kunci Jawaban" yang benar-benar diikuti daftar jawaban.
	let kunciStart = -1;
	for (let i = 0; i < lines.length; i++) {
		const text = texts[i]!;
		if (text.length > 80 || !KUNCI_HEADING.test(text) || INLINE_KUNCI.test(text)) continue;
		const rest = text.replace(KUNCI_HEADING, "").replace(/^[\s:.-]+/, "");
		if (isKunciPairLine(rest)) {
			kunciStart = i;
			break;
		}
		const ahead = nextTextIndexes(i, 12).map((j) => texts[j]!);
		const hasAnswers = ahead.some(isKunciPairLine) || ahead.filter((t) => QUESTION_START.test(t)).length >= 2;
		// Kalimat petunjuk seperti "Kunci jawaban ditulis di lembar jawaban" bukan judul bagian kunci.
		const looksLikeHeading = !rest || rest === rest.toUpperCase() || i > lines.length * 0.5;
		if (hasAnswers && looksLikeHeading) {
			kunciStart = i;
			break;
		}
	}
	const end = kunciStart === -1 ? lines.length : kunciStart;
	const kunciLines = kunciStart === -1
		? []
		: [texts[kunciStart]!.replace(KUNCI_HEADING, "").replace(/^\s*[:.-]\s*/, ""), ...texts.slice(kunciStart + 1)].filter(Boolean);

	const blocks: SoalBlock[] = [];
	const missingNumbers: number[] = [];
	let current: SoalBlock | null = null;
	let section: Section | null = null;
	let lastNumber = 0;
	let inPetunjuk = false;
	let headerSinceLast = false;
	let draft: StimulusDraft | null = null;
	let stimulus: StimulusDraft | null = null;

	const closeCurrent = () => {
		if (current) blocks.push(current);
		current = null;
	};

	const startsAhead = (from: number, number: number) =>
		nextTextIndexes(from, 150).some((j) => j < end && Number(QUESTION_START.exec(texts[j]!)?.[1]) === number);

	// Baris teks sesudah `from` sampai sebelum nomor soal berikutnya.
	const linesUntilNextStart = (from: number) => {
		const result: string[] = [];
		for (let j = from + 1; j < end; j++) {
			if (lines[j]!.kind !== "text") continue;
			if (QUESTION_START.test(texts[j]!)) return { lines: result, nextStart: j };
			result.push(texts[j]!);
		}
		return { lines: result, nextStart: -1 };
	};

	const restartAhead = (from: number) => {
		for (let j = from + 1; j < end; j++) {
			if (Number(QUESTION_START.exec(texts[j]!)?.[1]) === 1 && startsAhead(j, 2)) return true;
		}
		return false;
	};

	const acceptNumber = (n: number, index: number) => {
		if (lastNumber === 0) return startsAhead(index, n + 1) || n === 1;
		if (n > lastNumber && n <= lastNumber + 3) return true;
		// Penomoran yang mulai lagi dari 1 hanya sah sesudah judul bagian baru (misalnya bagian
		// uraian). Daftar "1. ... 2. ..." di dalam soal tidak boleh memecah soal itu.
		return n === 1 && headerSinceLast && startsAhead(index, 2);
	};

	for (let i = 0; i < end; i++) {
		const line = lines[i]!;
		if (line.kind === "image") {
			if (current) current.imageIds.push(line.imageId);
			else if (draft) draft.imageIds.push(line.imageId);
			continue;
		}
		const text = line.text;
		if (ANSWER_BLANK.test(text)) continue;

		if (PETUNJUK.test(text)) {
			inPetunjuk = true;
			closeCurrent();
			continue;
		}

		if (!QUESTION_START.test(text) && STIMULUS_INTRO.test(text)) {
			const range = STIMULUS_RANGE.exec(text);
			const open = current as SoalBlock | null;
			if (range || (STIMULUS_OBJECT.test(text) && (!open || extractOptions(open.lines).options))) {
				closeCurrent();
				inPetunjuk = false;
				// Kalimat pengantar yang menyebut rentang nomor tidak perlu ikut ditampilkan di setiap soal.
				draft = { lines: range ? [] : [text], imageIds: [], from: range ? Number(range[1]) : null, to: range ? Number(range[2]) : null };
				continue;
			}
		}

		const header = sectionOf(text);
		// Judul bagian hanya diakui kalau sesudahnya ada nomor soal dan di antaranya tidak ada opsi
		// jawaban, supaya kalimat "Pilihlah ..." di dalam soal tidak memotong soal itu.
		const following = header ? linesUntilNextStart(i) : null;
		if (
			header && following && following.nextStart !== -1
			&& (ROMAN_HEADER.test(text) || following.lines.length <= 3)
			&& !extractOptions(following.lines).options
		) {
			section = header;
			headerSinceLast = true;
			inPetunjuk = false;
			closeCurrent();
			continue;
		}

		const loose = LOOSE_QUESTION_START.exec(text);
		const start = QUESTION_START.exec(text) ?? (loose && lastNumber && Number(loose[1]) === lastNumber + 1 ? loose : null);
		if (start) {
			let n = Number(start[1]);
			if (inPetunjuk) {
				// Petunjuk umum bisa bernomor 1, 2, 3 sebelum soal nomor 1. Baris bernomor dianggap
				// petunjuk kalau tidak punya opsi jawaban dan nanti penomorannya mulai lagi dari 1.
				const isInstruction = !extractOptions(linesUntilNextStart(i).lines).options && restartAhead(i);
				if (isInstruction) continue;
				inPetunjuk = false;
				lastNumber = 0;
				headerSinceLast = true;
			}
			// Nomor yang salah ketik ("1." padahal seharusnya 3) dikenali dari soal sebelumnya yang
			// opsinya sudah lengkap: baris bernomor sesudahnya pasti soal baru, jadi dinomori ulang.
			const open = current as SoalBlock | null;
			const accepted = acceptNumber(n, i);
			const recovered = !accepted && !inPetunjuk && start[2]!.trim().length >= 10 && Boolean(open && extractOptions(open.lines).options);
			const writtenNumber = recovered ? n : undefined;
			if (recovered) n = lastNumber + 1;
			if (accepted || recovered) {
				if (lastNumber && n > lastNumber + 1) {
					for (let m = lastNumber + 1; m < n; m++) missingNumbers.push(m);
				}
				closeCurrent();
				if (draft) {
					stimulus = draft;
					draft = null;
				}
				const firstLine = start[2]!.replace(/^[\s.)]+/, "").trim();
				const block: SoalBlock = { number: n, section, lines: firstLine ? [firstLine] : [], imageIds: [], writtenNumber };
				if (stimulus) {
					const { from, to } = stimulus;
					const inRange = from === null || to === null || (n >= from && n <= to);
					if (inRange) block.stimulus = { lines: stimulus.lines, imageIds: stimulus.imageIds };
					if (!inRange || from === null || to === null || n >= to) stimulus = null;
				}
				current = block;
				lastNumber = n;
				headerSinceLast = false;
				continue;
			}
		}

		const open = current as SoalBlock | null;
		if (!open) {
			if (draft) draft.lines.push(text);
			continue;
		}
		if (inPetunjuk) continue;
		const inline = INLINE_KUNCI.exec(text);
		if (inline) {
			open.inlineKunci = inline[1]!.toUpperCase();
			continue;
		}
		open.lines.push(text);
	}
	closeCurrent();

	return { blocks, kunciLines, missingNumbers };
};

export interface ParsedOptions {
	soal: string
	options: [string, string, string, string, string] | null
	markedKunci?: string
	// Label opsi yang salah ketik ("a, a, b, c") dan sudah dibaca ulang menurut urutannya.
	relabeled?: string
	// Label opsi yang tidak bisa dibaca dengan yakin ("a, c, c, d").
	irregularLabels?: string
}

interface OptionMarker { index: number, end: number, letter: string, upper: boolean, star: boolean, lineStart: boolean }

// Penanda opsi: "a.", "a)", "(a)", "a .", "b..". Penanda boleh menempel ke teksnya ("a.42", "a.Jakarta").
// Huruf kecil yang menempel ("a.kendang") hanya diterima di awal baris, supaya "e.g." dan "a.n." tidak
// terbaca sebagai opsi.
const OPTION_MARKER = /(?<=^|\s)(\*?)\(?([a-eA-E])\s*[.)]{1,2}(?=(.?)(.?))/gm;
const KUNCI_MARK = /^\*+\s*|\s*\*+$|\s*\(\*\)|\s*[✓√]/g;
// Titik-titik panjang tempat menjawab diringkas jadi "....".
const clean = (s: string) => s.replace(/(?:[.…_]\s?){5,}/g, "....").replace(/\s+/g, " ").trim();

// Butir daftar yang selalu dimulai di baris baru: "1.", "a)", "(1)", "(a)", "•", "-".
const LIST_ITEM = /^(?:(?:\d{1,2}|[a-eA-E])\s*[.)]|\((?:\d{1,2}|[a-eA-E])\)|[•▪◦\-–])\s*\S/;

// Baris PDF adalah baris tampilan, jadi baris yang terpotong karena lebar halaman disambung lagi.
// Pindah baris dipertahankan untuk butir daftar, kutipan, dan awal kalimat baru.
export const joinSoalLines = (lines: string[]) => {
	const cleaned = lines.map(clean).filter(Boolean);
	const widest = Math.max(0, ...cleaned.map((l) => l.length));
	let out = "";
	let previous = "";
	for (const line of cleaned) {
		if (!out) {
			out = line;
			previous = line;
			continue;
		}
		const startsSentence = /^[A-Z0-9“"‘']/.test(line);
		// Baris yang jauh lebih pendek dari baris terlebar sengaja diakhiri penulisnya (judul, akhir paragraf).
		const endedEarly = previous.length < widest * 0.6;
		const newLine = LIST_ITEM.test(line) || /^[“"‘']/.test(line) || (startsSentence && (/[.!?:”"’']$/.test(previous) || endedEarly));
		out += (newLine ? "\n" : " ") + line;
		previous = line;
	}
	return out;
};

const findMarkers = (text: string): OptionMarker[] =>
	[...text.matchAll(OPTION_MARKER)].flatMap((m) => {
		const index = m.index!;
		const lineStart = index === 0 || text[index - 1] === "\n";
		const next = m[3] ?? "";
		const after = m[4] ?? "";
		const glued = next !== "" && !/\s/.test(next);
		if (glued && !/[0-9A-Z(“"'‘-]/.test(next) && !(lineStart && /[a-z]/.test(next) && after !== ".")) return [];
		return [{
			index,
			end: index + m[0].length,
			letter: m[2]!.toLowerCase(),
			upper: m[2] === m[2]!.toUpperCase(),
			star: Boolean(m[1]),
			lineStart
		}];
	});

// Opsi dicari sebagai rantai penanda a, b, c, ... yang sama jenis hurufnya. Rantai yang dipakai
// adalah yang terakhir, karena opsi selalu berada di akhir soal.
export const extractOptions = (lines: string[]): ParsedOptions => {
	const text = lines.join("\n");
	const markers = findMarkers(text);

	let best: OptionMarker[] | null = null;
	for (let i = 0; i < markers.length; i++) {
		if (markers[i]!.letter !== "a") continue;
		// Opsi bisa tersusun berjajar ("a. ... c. ..." lalu "b. ... d. ..."), jadi urutan huruf di teks
		// tidak harus a, b, c, d. Yang diambil adalah huruf-huruf berbeda sesudah "a" sampai ada yang berulang.
		const found = [markers[i]!];
		const seen = new Set(["a"]);
		for (let j = i + 1; j < markers.length; j++) {
			const m = markers[j]!;
			if (m.upper !== found[0]!.upper) continue;
			if (seen.has(m.letter)) break;
			seen.add(m.letter);
			found.push(m);
		}
		let prefix = 0;
		while (prefix < 5 && seen.has("abcde"[prefix]!)) prefix++;
		const chain = found.filter((m) => "abcde".indexOf(m.letter) < prefix);
		if (chain.length >= 3 || (chain.length === 2 && chain.every((m) => m.lineStart))) best = chain;
	}

	if (!best) {
		// Tiga penanda atau lebih yang diawali "a" tetapi tidak berurutan hampir pasti opsi yang salah ketik.
		const irregular = markers.length >= 3 && markers[0]!.letter === "a";
		return { soal: joinSoalLines(lines), options: null, irregularLabels: irregular ? markers.map((m) => m.letter).join(", ") : undefined };
	}

	let chain = best;
	let relabeled: string | undefined;
	// Label pertama dobel ("a. 1-2-3", "a. 3-5-6", "b. ...", "c. ...") adalah salah ketik: opsi yang
	// masing-masing satu baris dibaca ulang menurut urutannya menjadi a, b, c, d.
	const before = markers[markers.indexOf(best[0]!) - 1];
	if (
		before && before.letter === "a" && before.upper === best[0]!.upper && before.lineStart
		&& best.length < 5 && best.every((m) => m.lineStart)
		&& !text.slice(before.end, best[0]!.index).trim().includes("\n")
	) {
		chain = [before, ...best];
		relabeled = chain.map((m) => m.letter).join(", ");
	}

	const options: [string, string, string, string, string] = ["", "", "", "", ""];
	let markedKunci: string | undefined;
	chain.forEach((marker, k) => {
		const position = relabeled ? k : "abcde".indexOf(marker.letter);
		const raw = text.slice(marker.end, chain[k + 1]?.index ?? text.length);
		const marked = marker.star || /^\s*\*|\*\s*$|\(\*\)|[✓√]/.test(raw);
		if (marked) markedKunci = "ABCDE"[position];
		options[position] = clean(clean(raw).replace(KUNCI_MARK, ""));
	});
	return { soal: joinSoalLines(text.slice(0, chain[0]!.index).split("\n")), options, markedKunci, relabeled };
};

export interface KunciMaps {
	pg: Map<number, string>
	esai: Map<number, string>
	// Jumlah soal yang disebut di kunci ("Jumlah Soal: 30 Soal Pilihan Ganda, 10 Soal Uraian").
	expectedPg?: number
	expectedEssay?: number
}

export const parseKunciLines = (lines: string[]): KunciMaps => {
	const pg = new Map<number, string>();
	const esai = new Map<number, string>();
	let mode: Section = "pg";
	let currentEsai: number | null = null;

	for (const raw of lines) {
		const line = clean(raw);
		if (!line) continue;
		if (line.length <= 80 && !/^\d/.test(line)) {
			const esaiHeading = ESAI_KEYWORD.test(line);
			const pgHeading = PG_KEYWORD.test(line);
			// Baris ringkasan "30 Soal Pilihan Ganda, 10 Soal Uraian" menyebut keduanya, jadi bukan judul bagian.
			if (esaiHeading !== pgHeading) {
				mode = esaiHeading ? "esai" : "pg";
				currentEsai = null;
				continue;
			}
		}
		if (isKunciPairLine(line)) {
			for (const m of line.matchAll(KUNCI_PAIR)) pg.set(Number(m[1]), m[2]!.toUpperCase());
			currentEsai = null;
			continue;
		}
		// Tabel kunci uraian sering tanpa titik sesudah nomor ("31 2/8").
		const numbered = (mode === "esai" ? /^(\d{1,3})\s*[.)]?\s+(.+)$/ : /^(\d{1,3})\s*[.)]\s*(.+)$/).exec(line);
		if (numbered) {
			esai.set(Number(numbered[1]), numbered[2]!.trim());
			currentEsai = Number(numbered[1]);
			continue;
		}
		if (currentEsai !== null && mode === "esai") esai.set(currentEsai, `${esai.get(currentEsai)} ${line}`);
	}
	const summary = lines.join(" ");
	const expectedPg = /(\d{1,3})\s*soal\s+pilihan\s+ganda/i.exec(summary)?.[1];
	const expectedEssay = /(\d{1,3})\s*soal\s+(?:uraian|esai|essay|isian)/i.exec(summary)?.[1];
	return {
		pg,
		esai,
		expectedPg: expectedPg ? Number(expectedPg) : undefined,
		expectedEssay: expectedEssay ? Number(expectedEssay) : undefined
	};
};

export interface SoalRowDraft {
	id: string
	number: number
	tipe: "pg" | "esai"
	soal: string
	options: [string, string, string, string, string]
	kunci: string
	kunciSource: "tanda" | "file" | "naskah" | null
	catatanJawabanEsai?: string
	imageId?: string
	reasons: string[]
	// Salah ketik di naskah yang sudah dibetulkan otomatis, untuk dilaporkan ke guru.
	fixes: string[]
}

export interface BuildRowsInput {
	blocks: SoalBlock[]
	kunciFile?: KunciMaps
	kunciNaskah?: KunciMaps
}

export const buildSoalRows = ({ blocks, kunciFile, kunciNaskah }: BuildRowsInput): SoalRowDraft[] => {
	// Di bagian uraian, butir "a. ... b. ..." adalah sub-soal, bukan opsi pilihan ganda.
	const parsed = blocks.map((block) => ({
		block,
		...(block.section === "esai" ? { soal: joinSoalLines(block.lines), options: null } as ParsedOptions : extractOptions(block.lines))
	}));
	const essayBlocks = parsed.filter((p) => !p.options && p.block.section !== "pg");

	const esaiAnswer = (number: number, ordinal: number) => {
		for (const maps of [kunciFile, kunciNaskah]) {
			if (!maps?.esai.size) continue;
			if (maps.esai.has(number)) return maps.esai.get(number);
			// Nomor esai di kunci bisa berlanjut (31, 32, ...) padahal di naskah mulai dari 1, atau sebaliknya.
			if (maps.esai.size === essayBlocks.length) return [...maps.esai.entries()].sort((a, b) => a[0] - b[0])[ordinal]?.[1];
		}
		return undefined;
	};

	let essayOrdinal = 0;
	return parsed.map(({ block, soal: ownSoal, options, markedKunci, relabeled, irregularLabels }, index) => {
		const stimulusText = block.stimulus ? joinSoalLines(block.stimulus.lines) : "";
		const soal = stimulusText && ownSoal ? `${stimulusText}

${ownSoal}` : ownSoal;
		const isPg = Boolean(options) || block.section === "pg" || Boolean(irregularLabels);
		const reasons: string[] = [];
		const fixes: string[] = [];
		if (block.writtenNumber !== undefined) fixes.push(`nomor tertulis ${block.writtenNumber}, dibaca sebagai nomor ${block.number}`);
		if (relabeled) fixes.push(`label opsi ${relabeled} dibaca sebagai ${"abcde".slice(0, relabeled.split(", ").length).split("").join(", ")}`);
		let kunci = "";
		let kunciSource: SoalRowDraft["kunciSource"] = null;
		let catatanJawabanEsai: string | undefined;

		if (isPg) {
			const fromMarks = markedKunci ?? block.inlineKunci;
			const fromFile = kunciFile?.pg.get(block.number);
			const fromNaskah = kunciNaskah?.pg.get(block.number);
			if (fromMarks) [kunci, kunciSource] = [fromMarks, "tanda"];
			else if (fromFile) [kunci, kunciSource] = [fromFile, "file"];
			else if (fromNaskah) [kunci, kunciSource] = [fromNaskah, "naskah"];

			if (!options && irregularLabels) reasons.push(`label opsi tidak berurutan (${irregularLabels}), periksa naskah lalu isi lewat Edit`);
			else if (!options) reasons.push("pilihan jawaban tidak terbaca otomatis, isi manual");
			else if (!options[0] || !options[1]) reasons.push("teks pilihan A/B kosong, kemungkinan berupa gambar");
			if (!kunci) reasons.push("kunci jawaban belum ada, pilih di kolom Kunci");
			else if (options && !options["ABCDE".indexOf(kunci)]) reasons.push("kunci jawaban tidak cocok dengan pilihan yang ada");
		} else {
			catatanJawabanEsai = esaiAnswer(block.number, essayOrdinal);
			essayOrdinal++;
		}
		if (!ownSoal) reasons.push("teks soal kosong");

		return {
			id: `soal-${index + 1}`,
			number: block.number,
			tipe: isPg ? "pg" : "esai",
			soal,
			options: options ?? ["", "", "", "", ""],
			kunci,
			kunciSource,
			catatanJawabanEsai,
			imageId: block.imageIds[0] ?? block.stimulus?.imageIds[0],
			reasons,
			fixes
		};
	});
};
