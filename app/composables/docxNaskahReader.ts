import type { ReadingLine } from "./naskahSoalParser";
import JSZip from "jszip";

// Mengubah .docx menjadi baris teks seperti yang terlihat di Word, supaya bisa diurai oleh
// pengurai naskah yang sama dengan PDF. Yang ditangani:
// - nomor dan huruf otomatis dari daftar Word (numbering.xml), yang tidak tersimpan di teks;
// - tabel: sel dalam satu baris disusun berdampingan, seperti tampilannya;
// - tab, pindah baris manual, gambar sisipan, dan konten kotak teks.

export interface DocxNaskah {
	lines: ReadingLine[]
	// id gambar di `lines` -> path media di dalam zip.
	images: Map<string, string>
	zip: JSZip
}

interface LevelDef { start: number, format: string, text: string }
interface NumDef { abstractId: string, overrides: Map<number, number> }

interface Numbering {
	abstracts: Map<string, Map<number, LevelDef>>
	nums: Map<string, NumDef>
}

const decodeXml = (s: string) =>
	s.replace(/&(lt|gt|amp|quot|apos|#\d+|#x[\da-f]+);/gi, (_, e: string) => {
		const named: Record<string, string> = { lt: "<", gt: ">", amp: "&", quot: "\"", apos: "'" };
		if (named[e.toLowerCase()]) return named[e.toLowerCase()]!;
		return String.fromCodePoint(e[1]?.toLowerCase() === "x" ? Number.parseInt(e.slice(2), 16) : Number(e.slice(1)));
	});

const attr = (attrs: string, name: string) => {
	const match = new RegExp(`${name}="([^"]*)"`).exec(attrs);
	return match ? decodeXml(match[1]!) : undefined;
};

const parseNumbering = (xml: string | undefined): Numbering => {
	const abstracts = new Map<string, Map<number, LevelDef>>();
	const nums = new Map<string, NumDef>();
	if (!xml) return { abstracts, nums };
	for (const m of xml.matchAll(/<w:abstractNum\b([^>]*)>([\s\S]*?)<\/w:abstractNum>/g)) {
		const id = attr(m[1]!, "w:abstractNumId");
		if (!id) continue;
		const levels = new Map<number, LevelDef>();
		for (const lvl of m[2]!.matchAll(/<w:lvl\b([^>]*)>([\s\S]*?)<\/w:lvl>/g)) {
			const body = lvl[2]!;
			levels.set(Number(attr(lvl[1]!, "w:ilvl") ?? 0), {
				start: Number(/<w:start w:val="(\d+)"/.exec(body)?.[1] ?? 1),
				format: /<w:numFmt w:val="([^"]+)"/.exec(body)?.[1] ?? "decimal",
				text: decodeXml(/<w:lvlText w:val="([^"]*)"/.exec(body)?.[1] ?? "")
			});
		}
		abstracts.set(id, levels);
	}
	for (const m of xml.matchAll(/<w:num\b([^>]*)>([\s\S]*?)<\/w:num>/g)) {
		const id = attr(m[1]!, "w:numId");
		const abstractId = /<w:abstractNumId w:val="([^"]+)"/.exec(m[2]!)?.[1];
		if (!id || !abstractId) continue;
		const overrides = new Map<number, number>();
		for (const o of m[2]!.matchAll(/<w:lvlOverride w:ilvl="(\d+)"[^>]*>([\s\S]*?)<\/w:lvlOverride>/g)) {
			const start = /<w:startOverride w:val="(\d+)"/.exec(o[2]!)?.[1];
			if (start) overrides.set(Number(o[1]), Number(start));
		}
		nums.set(id, { abstractId, overrides });
	}
	return { abstracts, nums };
};

// Gaya paragraf (mis. "List Number") bisa membawa penomoran sendiri.
const parseStyleNumbering = (xml: string | undefined) => {
	const styles = new Map<string, { numId: string, ilvl: number }>();
	if (!xml) return styles;
	for (const m of xml.matchAll(/<w:style\b([^>]*)>([\s\S]*?)<\/w:style>/g)) {
		const id = attr(m[1]!, "w:styleId");
		const numPr = /<w:numPr>([\s\S]*?)<\/w:numPr>/.exec(m[2]!)?.[1];
		const numId = numPr ? /<w:numId w:val="(\d+)"/.exec(numPr)?.[1] : undefined;
		if (id && numId) styles.set(id, { numId, ilvl: Number(/<w:ilvl w:val="(\d+)"/.exec(numPr!)?.[1] ?? 0) });
	}
	return styles;
};

const toLetters = (n: number) => {
	let s = "";
	for (let v = n; v > 0; v = Math.floor((v - 1) / 26)) s = String.fromCharCode(97 + ((v - 1) % 26)) + s;
	return s;
};
const toRoman = (n: number) => {
	const table: [number, string][] = [[1000, "m"], [900, "cm"], [500, "d"], [400, "cd"], [100, "c"], [90, "xc"], [50, "l"], [40, "xl"], [10, "x"], [9, "ix"], [5, "v"], [4, "iv"], [1, "i"]];
	let rest = n;
	let s = "";
	for (const [value, numeral] of table) {
		while (rest >= value) {
			s += numeral;
			rest -= value;
		}
	}
	return s;
};
const formatNumber = (n: number, format: string) => {
	switch (format) {
		case "lowerLetter": return toLetters(n);
		case "upperLetter": return toLetters(n).toUpperCase();
		case "lowerRoman": return toRoman(n);
		case "upperRoman": return toRoman(n).toUpperCase();
		case "decimalZero": return n < 10 ? `0${n}` : String(n);
		case "none": return "";
		default: return String(n);
	}
};

// Daftar yang memakai abstractNum yang sama berbagi hitungan (seperti di Word), kecuali
// daftarnya memulai ulang lewat startOverride.
const createNumberer = (numbering: Numbering) => {
	const counters = new Map<string, (number | undefined)[]>();
	const startedNums = new Set<string>();
	return (numId: string, ilvl: number) => {
		const num = numbering.nums.get(numId);
		const levels = num ? numbering.abstracts.get(num.abstractId) : undefined;
		const level = levels?.get(ilvl);
		if (!num || !levels || !level) return "";
		const values = counters.get(num.abstractId) ?? [];
		counters.set(num.abstractId, values);
		if (!startedNums.has(numId)) {
			startedNums.add(numId);
			for (const lvl of num.overrides.keys()) values[lvl] = undefined;
		}
		const startOf = (l: number) => num.overrides.get(l) ?? levels.get(l)?.start ?? 1;
		values[ilvl] = values[ilvl] === undefined ? startOf(ilvl) : values[ilvl]! + 1;
		for (let l = ilvl + 1; l < 9; l++) values[l] = undefined;
		if (level.format === "bullet") return "•";
		return level.text.replace(/%(\d)/g, (_, k: string) => {
			const l = Number(k) - 1;
			return formatNumber(values[l] ?? startOf(l), levels.get(l)?.format ?? "decimal");
		});
	};
};

interface Paragraph { lines: string[], text: string, numId?: string, ilvl: number, style?: string, images: string[] }
interface Cell { lines: string[], images: string[] }
interface Table { rows: Cell[][], row: Cell[] | null, cell: Cell | null }

const SKIPPED = new Set(["mc:Fallback", "w:instrText", "w:delText", "w:del", "w:footnoteReference", "w:endnoteReference"]);
const OPTION_LETTER_CELL = /^[A-Ea-e]$/;

// Baris tabel: sel berdampingan digabung per baris paragraf. Sel yang berisi banyak paragraf
// (tabel dipakai sebagai kolom halaman) dibaca sel demi sel.
const rowLines = (cells: Cell[]) => {
	if (cells.some((c) => c.lines.length > 3)) return cells.flatMap((c) => c.lines);
	const height = Math.max(0, ...cells.map((c) => c.lines.length));
	const result: string[] = [];
	for (let i = 0; i < height; i++) {
		// Sel "A" | "is" pada tabel opsi dibaca sebagai "A. is".
		const parts = cells.map((c) => c.lines[i] ?? "").map((t) => (OPTION_LETTER_CELL.test(t.trim()) ? `${t.trim()}.` : t)).filter((t) => t.trim());
		if (parts.length) result.push(parts.join(" "));
	}
	return result;
};

export const readDocxNaskah = async (bytes: Uint8Array): Promise<DocxNaskah> => {
	let zip: JSZip;
	try {
		zip = await JSZip.loadAsync(bytes);
	} catch (error) {
		throw new Error("File ini bukan dokumen Word (.docx) yang valid.", { cause: error });
	}
	const documentXml = await zip.file("word/document.xml")?.async("string");
	if (!documentXml) throw new Error("File Word tidak valid (word/document.xml tidak ditemukan).");

	const numbering = parseNumbering(await zip.file("word/numbering.xml")?.async("string"));
	const styleNumbering = parseStyleNumbering(await zip.file("word/styles.xml")?.async("string"));
	const relsXml = await zip.file("word/_rels/document.xml.rels")?.async("string");
	const rels = new Map<string, string>();
	for (const m of (relsXml ?? "").matchAll(/<Relationship\b([^>]*)\/?>/g)) {
		const id = attr(m[1]!, "Id");
		const target = attr(m[1]!, "Target");
		if (id && target && attr(m[1]!, "TargetMode") !== "External") rels.set(id, `word/${target.replace(/^\.?\//, "").replace(/^word\//, "")}`);
	}

	const nextLabel = createNumberer(numbering);
	const lines: ReadingLine[] = [];
	const images = new Map<string, string>();
	const paragraphs: Paragraph[] = [];
	const tables: Table[] = [];
	let skipDepth = 0;
	let inText = false;
	let inPPr = false;
	let imageCount = 0;

	const emit = (texts: string[], imageIds: string[]) => {
		const table = tables.at(-1);
		if (table?.cell) {
			table.cell.lines.push(...texts);
			table.cell.images.push(...imageIds);
			return;
		}
		for (const text of texts) lines.push({ kind: "text", text, page: 0 });
		for (const imageId of imageIds) lines.push({ kind: "image", imageId, page: 0 });
	};

	const addImage = (relId: string | undefined) => {
		const p = paragraphs.at(-1);
		const target = relId ? rels.get(relId) : undefined;
		if (!p || !target) return;
		const id = `docx-${imageCount++}`;
		images.set(id, target);
		p.images.push(id);
	};

	for (const m of documentXml.matchAll(/<(\/?)([\w:]+)([^>]*?)(\/?)>|([^<]+)/g)) {
		const [, closing, name, attrs = "", selfClosing, text] = m;
		if (text !== undefined) {
			if (inText && !skipDepth) {
				const p = paragraphs.at(-1);
				if (p) p.text += decodeXml(text);
			}
			continue;
		}
		if (SKIPPED.has(name!)) {
			if (!selfClosing) skipDepth += closing ? -1 : 1;
			continue;
		}
		if (skipDepth) continue;

		const p = paragraphs.at(-1);
		switch (name) {
			case "w:p":
				if (selfClosing) {
					emit([], []);
				} else if (!closing) {
					paragraphs.push({ lines: [], text: "", ilvl: 0, images: [] });
				} else if (p) {
					paragraphs.pop();
					const fromStyle = p.style ? styleNumbering.get(p.style) : undefined;
					const numId = p.numId ?? fromStyle?.numId;
					const label = numId && numId !== "0" ? nextLabel(numId, p.numId ? p.ilvl : (fromStyle?.ilvl ?? 0)) : "";
					const texts = [...p.lines, p.text].map((t) => t.replace(/\s+/g, " ").trim());
					if (label && texts.length) texts[0] = `${label} ${texts[0]}`.trim();
					emit(texts.filter(Boolean), p.images);
				}
				break;
			case "w:pPr":
				if (!selfClosing) inPPr = !closing;
				break;
			case "w:t":
				if (!selfClosing) inText = !closing;
				break;
			case "w:tab":
				if (!inPPr && p) p.text += " ";
				break;
			case "w:br":
			case "w:cr":
				if (p) {
					p.lines.push(p.text);
					p.text = "";
				}
				break;
			case "w:noBreakHyphen":
				if (p) p.text += "-";
				break;
			case "w:numId":
				if (inPPr && p) p.numId = attr(attrs, "w:val");
				break;
			case "w:ilvl":
				if (inPPr && p) p.ilvl = Number(attr(attrs, "w:val") ?? 0);
				break;
			case "w:pStyle":
				if (inPPr && p) p.style = attr(attrs, "w:val");
				break;
			case "a:blip":
				addImage(attr(attrs, "r:embed"));
				break;
			case "v:imagedata":
				addImage(attr(attrs, "r:id"));
				break;
			case "w:tbl":
				if (selfClosing) break;
				if (!closing) {
					tables.push({ rows: [], row: null, cell: null });
				} else {
					const table = tables.pop();
					if (table) emit([], []);
				}
				break;
			case "w:tr": {
				const table = tables.at(-1);
				if (!table || selfClosing) break;
				if (!closing) {
					table.row = [];
				} else if (table.row) {
					const cells = table.row;
					table.row = null;
					// Baris selesai dikirim ke luar tabel (atau ke sel induk bila tabel bersarang).
					const saved = table.cell;
					table.cell = null;
					const parent = tables.length > 1 ? tables[tables.length - 2] : undefined;
					const texts = rowLines(cells);
					const rowImages = cells.flatMap((c) => c.images);
					if (parent?.cell) {
						parent.cell.lines.push(...texts);
						parent.cell.images.push(...rowImages);
					} else {
						for (const t of texts) lines.push({ kind: "text", text: t, page: 0 });
						for (const id of rowImages) lines.push({ kind: "image", imageId: id, page: 0 });
					}
					table.cell = saved;
				}
				break;
			}
			case "w:tc": {
				const table = tables.at(-1);
				if (!table || selfClosing) break;
				if (!closing) {
					table.cell = { lines: [], images: [] };
				} else if (table.cell) {
					table.row?.push(table.cell);
					table.cell = null;
				}
				break;
			}
		}
	}

	return { lines, images, zip };
};

const IMAGE_TYPES = new Set(["png", "jpg", "jpeg", "gif", "webp"]);

export const isSupportedDocxImage = (path: string) => IMAGE_TYPES.has(path.split(".").pop()?.toLowerCase() ?? "");

// Teks polos dokumen Word, misalnya untuk file kunci jawaban.
export const readDocxTextLines = async (bytes: Uint8Array) =>
	(await readDocxNaskah(bytes)).lines.flatMap((l) => (l.kind === "text" ? [l.text] : []));
