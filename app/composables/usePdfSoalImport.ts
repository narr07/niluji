import type { PDFDocumentProxy, PDFPageProxy } from "pdfjs-dist";
import type { KunciMaps, PdfImageBox, PdfPageData, PdfTextItem } from "./naskahSoalParser";
import type { LoadedImages, NaskahImportResult } from "./useNaskahImport";
import { buildReadingLines, parseKunciLines } from "./naskahSoalParser";
import { assembleNaskahImport } from "./useNaskahImport";

export type PdfJs = typeof import("pdfjs-dist");
type Matrix = [number, number, number, number, number, number];

// Gambar lebih kecil dari ini biasanya garis, ikon, atau titik dekorasi, bukan gambar soal.
const MIN_IMAGE_SIZE = 24;
const RENDER_SCALE = 2;
const SCAN_MESSAGE =
	"PDF ini tidak berisi teks yang bisa dibaca, kemungkinan hasil scan atau foto. " +
	"Pakai PDF asli hasil \"Save as PDF\" dari Word, atau ketik ulang soalnya di Word lalu gunakan Import Word.";

let pdfjsPromise: Promise<PdfJs> | null = null;

// pdf.js cukup besar, jadi baru dimuat saat guru benar-benar memproses PDF.
const loadPdfJs = () => {
	pdfjsPromise ??= (async () => {
		const [pdfjs, worker] = await Promise.all([import("pdfjs-dist"), import("pdfjs-dist/build/pdf.worker.min.mjs?url")]);
		pdfjs.GlobalWorkerOptions.workerSrc = worker.default;
		return pdfjs;
	})();
	return pdfjsPromise;
};

const multiply = (m1: Matrix, m2: Matrix): Matrix => [
	m1[0] * m2[0] + m1[2] * m2[1],
	m1[1] * m2[0] + m1[3] * m2[1],
	m1[0] * m2[2] + m1[2] * m2[3],
	m1[1] * m2[2] + m1[3] * m2[3],
	m1[0] * m2[4] + m1[2] * m2[5] + m1[4],
	m1[1] * m2[4] + m1[3] * m2[5] + m1[5]
];
const applyPoint = (m: Matrix, x: number, y: number): [number, number] => [m[0] * x + m[2] * y + m[4], m[1] * x + m[3] * y + m[5]];

export const openPdfDocument = async (pdfjs: PdfJs, bytes: Uint8Array): Promise<PDFDocumentProxy> => {
	try {
		// Salinan dikirim karena pdf.js memindahkan buffer ke worker.
		return await pdfjs.getDocument({ data: bytes.slice(), verbosity: pdfjs.VerbosityLevel.ERRORS }).promise;
	} catch (error) {
		if (error instanceof Error && error.name === "PasswordException") {
			throw new Error("PDF ini dikunci kata sandi. Buka kuncinya dulu, lalu coba lagi.", { cause: error });
		}
		throw new Error(`File PDF tidak bisa dibuka: ${error instanceof Error ? error.message : String(error)}`, { cause: error });
	}
};

const readText = async (pdfjs: PdfJs, page: PDFPageProxy): Promise<PdfTextItem[]> => {
	const viewport = page.getViewport({ scale: 1 });
	const content = await page.getTextContent();
	const items: PdfTextItem[] = [];
	for (const item of content.items) {
		if (!("str" in item) || !item.str) continue;
		const tx = pdfjs.Util.transform(viewport.transform, item.transform) as Matrix;
		items.push({ str: item.str, x: tx[4], y: tx[5], width: item.width, height: Math.hypot(tx[2], tx[3]) || item.height });
	}
	return items;
};

// Posisi gambar dihitung dengan menelusuri matriks transformasi di daftar operasi gambar halaman.
const readImages = async (pdfjs: PdfJs, page: PDFPageProxy, pageIndex: number): Promise<PdfImageBox[]> => {
	const { OPS } = pdfjs;
	const viewport = page.getViewport({ scale: 1 });
	const ops = await page.getOperatorList();
	const images: PdfImageBox[] = [];
	const stack: Matrix[] = [];
	let ctm: Matrix = [1, 0, 0, 1, 0, 0];

	for (let i = 0; i < ops.fnArray.length; i++) {
		const fn = ops.fnArray[i];
		const args = ops.argsArray[i] as unknown[] | null;
		switch (fn) {
			case OPS.save:
				stack.push(ctm);
				break;
			case OPS.restore:
			case OPS.paintFormXObjectEnd:
			case OPS.endGroup:
				ctm = stack.pop() ?? ctm;
				break;
			case OPS.transform:
				ctm = multiply(ctm, args as Matrix);
				break;
			case OPS.paintFormXObjectBegin: {
				stack.push(ctm);
				const matrix = args?.[0] as Matrix | null | undefined;
				if (matrix) ctm = multiply(ctm, matrix);
				break;
			}
			case OPS.beginGroup: {
				stack.push(ctm);
				const matrix = (args?.[0] as { matrix?: Matrix } | undefined)?.matrix;
				if (matrix) ctm = multiply(ctm, matrix);
				break;
			}
			case OPS.paintImageXObject:
			case OPS.paintInlineImageXObject:
			case OPS.paintImageXObjectRepeat:
			case OPS.paintImageMaskXObject: {
				const m = multiply(viewport.transform as Matrix, ctm);
				const corners = [applyPoint(m, 0, 0), applyPoint(m, 1, 0), applyPoint(m, 0, 1), applyPoint(m, 1, 1)];
				const xs = corners.map((p) => p[0]);
				const ys = corners.map((p) => p[1]);
				const x = Math.max(0, Math.min(...xs));
				const y = Math.max(0, Math.min(...ys));
				const width = Math.min(viewport.width, Math.max(...xs)) - x;
				const height = Math.min(viewport.height, Math.max(...ys)) - y;
				if (width < MIN_IMAGE_SIZE || height < MIN_IMAGE_SIZE) break;
				// Gambar seukuran halaman adalah latar atau hasil scan, bukan gambar soal.
				if (width * height > viewport.width * viewport.height * 0.85) break;
				images.push({ id: `p${pageIndex}-${images.length}`, x, y, width, height });
				break;
			}
		}
	}
	return images;
};

export const readPdfPages = async (pdfjs: PdfJs, doc: PDFDocumentProxy, withImages: boolean) => {
	const pages: PdfPageData[] = [];
	const proxies: PDFPageProxy[] = [];
	for (let i = 0; i < doc.numPages; i++) {
		const page = await doc.getPage(i + 1);
		const viewport = page.getViewport({ scale: 1 });
		proxies.push(page);
		pages.push({
			width: viewport.width,
			height: viewport.height,
			items: await readText(pdfjs, page),
			images: withImages ? await readImages(pdfjs, page, i) : []
		});
	}
	return { pages, proxies };
};

const hasText = (pages: PdfPageData[]) => pages.reduce((sum, p) => sum + p.items.reduce((s, it) => s + it.str.trim().length, 0), 0) >= 30;

const canvasToPng = (canvas: HTMLCanvasElement) =>
	new Promise<Uint8Array>((resolve, reject) => {
		canvas.toBlob(async (blob) => {
			if (!blob) return reject(new Error("Gagal membuat gambar"));
			resolve(new Uint8Array(await blob.arrayBuffer()));
		}, "image/png");
	});

// Gambar diambil dengan merender halaman lalu memotong area gambarnya, supaya hasilnya sama
// persis dengan yang terlihat di PDF (termasuk gambar bertopeng atau terputar).
const cropImages = async (page: PDFPageProxy, boxes: PdfImageBox[]) => {
	const viewport = page.getViewport({ scale: RENDER_SCALE });
	const canvas = document.createElement("canvas");
	canvas.width = Math.ceil(viewport.width);
	canvas.height = Math.ceil(viewport.height);
	const result = new Map<string, Uint8Array>();
	try {
		await page.render({ canvas, viewport }).promise;
		for (const box of boxes) {
			const crop = document.createElement("canvas");
			crop.width = Math.max(1, Math.round(box.width * RENDER_SCALE));
			crop.height = Math.max(1, Math.round(box.height * RENDER_SCALE));
			crop.getContext("2d")!.drawImage(canvas, box.x * RENDER_SCALE, box.y * RENDER_SCALE, crop.width, crop.height, 0, 0, crop.width, crop.height);
			result.set(box.id, await canvasToPng(crop));
		}
	} finally {
		canvas.width = 0;
		canvas.height = 0;
	}
	return result;
};

const readKunciFile = async (pdfjs: PdfJs, bytes: Uint8Array): Promise<KunciMaps> => {
	const doc = await openPdfDocument(pdfjs, bytes);
	try {
		const { pages } = await readPdfPages(pdfjs, doc, false);
		if (!hasText(pages)) throw new Error(`File kunci jawaban: ${SCAN_MESSAGE}`);
		const texts = buildReadingLines(pages).lines.flatMap((l) => (l.kind === "text" ? [l.text] : []));
		return parseKunciLines(texts);
	} finally {
		await doc.loadingTask.destroy();
	}
};

export const importSoalFromPdf = async (naskahBytes: Uint8Array, kunciBytes?: Uint8Array): Promise<NaskahImportResult> => {
	const pdfjs = await loadPdfJs();
	const kunciFile = kunciBytes ? await readKunciFile(pdfjs, kunciBytes) : undefined;

	const doc = await openPdfDocument(pdfjs, naskahBytes);
	try {
		const { pages, proxies } = await readPdfPages(pdfjs, doc, true);
		if (!hasText(pages)) throw new Error(SCAN_MESSAGE);
		const reading = buildReadingLines(pages);
		const boxes = new Map(pages.flatMap((p, pageIndex) => p.images.map((box) => [box.id, { box, pageIndex }] as const)));

		const loadImages = async (ids: string[]): Promise<LoadedImages> => {
			const byPage = new Map<number, PdfImageBox[]>();
			for (const id of ids) {
				const found = boxes.get(id);
				if (found) byPage.set(found.pageIndex, [...(byPage.get(found.pageIndex) ?? []), found.box]);
			}
			const result: LoadedImages = { loaded: new Map(), failed: 0, unsupported: 0 };
			for (const [pageIndex, list] of byPage) {
				try {
					for (const [id, bytes] of await cropImages(proxies[pageIndex]!, list)) result.loaded.set(id, { bytes, fileName: `soal-pdf-${id}.png` });
				} catch {
					result.failed += list.length;
				}
			}
			return result;
		};

		return await assembleNaskahImport({
			format: "pdf",
			lines: reading.lines,
			kunciFile,
			pages: pages.length,
			twoColumnPages: reading.twoColumnPages,
			loadImages
		});
	} finally {
		await doc.loadingTask.destroy();
	}
};
