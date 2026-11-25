import type { PdfImageBox, PdfPageData, PdfTextItem, ReadingLine } from "../app/composables/naskahSoalParser";
import { describe, expect, it } from "bun:test";
import { buildSoalRows, buildReadingLines, extractOptions, parseKunciLines, parseNaskahLines } from "../app/composables/naskahSoalParser";

const CHAR_WIDTH = 5;
const LINE_HEIGHT = 14;

// Satu baris teks di posisi tertentu, dipecah per kata seperti potongan teks dari pdf.js.
const textAt = (text: string, x: number, y: number): PdfTextItem[] => {
	const items: PdfTextItem[] = [];
	let cursor = x;
	for (const word of text.split(" ")) {
		items.push({ str: word, x: cursor, y, width: word.length * CHAR_WIDTH, height: 10 });
		cursor += (word.length + 1) * CHAR_WIDTH;
	}
	return items;
};

const page = (lines: string[], options: { x?: number, startY?: number, images?: PdfImageBox[] } = {}): PdfPageData => ({
	width: 600,
	height: 840,
	items: lines.flatMap((line, i) => textAt(line, options.x ?? 50, (options.startY ?? 60) + i * LINE_HEIGHT)),
	images: options.images ?? []
});

const textLines = (lines: string[]): ReadingLine[] => lines.map((text) => ({ kind: "text", text, page: 0 }));

const rowsFrom = (lines: string[], kunciFileLines?: string[]) => {
	const naskah = parseNaskahLines(textLines(lines));
	return buildSoalRows({
		blocks: naskah.blocks,
		kunciNaskah: naskah.kunciLines.length ? parseKunciLines(naskah.kunciLines) : undefined,
		kunciFile: kunciFileLines ? parseKunciLines(kunciFileLines) : undefined
	});
};

describe("extractOptions", () => {
	it("membaca opsi satu per baris dan memisahkan teks soal", () => {
		const result = extractOptions(["Ibu kota Indonesia adalah ....", "a. Bandung", "b. Jakarta", "c. Surabaya", "d. Medan"]);
		expect(result.soal).toBe("Ibu kota Indonesia adalah ....");
		expect(result.options).toEqual(["Bandung", "Jakarta", "Surabaya", "Medan", ""]);
	});

	it("membaca opsi berjajar dalam satu atau dua baris", () => {
		expect(extractOptions(["5 + 3 = ....", "A. 6 B. 7 C. 8 D. 9"]).options).toEqual(["6", "7", "8", "9", ""]);
		expect(extractOptions(["Pilih", "a. satu c. tiga", "b. dua d. empat"]).options).toEqual(["satu", "dua", "tiga", "empat", ""]);
	});

	it("menyambung opsi yang terpotong ke baris berikutnya", () => {
		const result = extractOptions(["Soal", "a. bagian awal", "lanjutan opsi a", "b. dua", "c. tiga"]);
		expect(result.options?.[0]).toBe("bagian awal lanjutan opsi a");
	});

	it("mengenali kunci yang ditandai bintang", () => {
		const result = extractOptions(["Soal", "a. satu", "*b. dua", "c. tiga", "d. empat"]);
		expect(result.markedKunci).toBe("B");
		expect(result.options).toEqual(["satu", "dua", "tiga", "empat", ""]);
		const trailing = extractOptions(["Soal", "a. satu b. dua c. tiga* d. empat"]);
		expect(trailing.markedKunci).toBe("C");
		expect(trailing.options?.[2]).toBe("tiga");
	});

	it("tidak salah membaca singkatan atau huruf di tengah kalimat sebagai opsi", () => {
		const result = extractOptions(["Kekurangan vitamin C. menyebabkan sariawan. Jelaskan!"]);
		expect(result.options).toBeNull();
	});

	it("memakai rantai opsi terakhir bila soal memuat daftar huruf", () => {
		const result = extractOptions(["Perhatikan pernyataan:", "a. hujan", "b. angin", "c. panas", "Yang benar adalah ....", "A. a dan b", "B. b dan c", "C. a dan c"]);
		expect(result.soal).toContain("Perhatikan pernyataan");
		expect(result.options).toEqual(["a dan b", "b dan c", "a dan c", "", ""]);
	});
});

describe("parseNaskahLines", () => {
	it("memisahkan soal berdasarkan nomor berurutan dan mengabaikan angka di tengah teks", () => {
		const { blocks } = parseNaskahLines(textLines([
			"PENILAIAN AKHIR SEMESTER",
			"1. Proklamasi dibacakan pada tahun",
			"1945. Siapa yang membacakannya?",
			"a. Soekarno b. Hatta c. Sjahrir",
			"2. Soal kedua",
			"a. x b. y c. z"
		]));
		expect(blocks.map((b) => b.number)).toEqual([1, 2]);
		expect(blocks[0]!.lines).toContain("1945. Siapa yang membacakannya?");
	});

	it("mengenali bagian PG dan esai serta penomoran esai yang mulai lagi dari 1", () => {
		const rows = rowsFrom([
			"I. Pilihan Ganda",
			"Berilah tanda silang (X) pada jawaban yang benar!",
			"1. Satu tambah satu",
			"a. 1 b. 2 c. 3",
			"2. Dua tambah dua",
			"a. 2 b. 3 c. 4",
			"II. Uraian",
			"Jawablah pertanyaan berikut!",
			"1. Sebutkan nama hari!",
			"2. Jelaskan arti gotong royong!"
		]);
		expect(rows.map((r) => [r.number, r.tipe])).toEqual([[1, "pg"], [2, "pg"], [1, "esai"], [2, "esai"]]);
		expect(rows[3]!.soal).toBe("Jelaskan arti gotong royong!");
	});

	it("tidak memotong soal karena kalimat perintah di dalamnya", () => {
		const { blocks } = parseNaskahLines(textLines([
			"1. Bacalah teks berikut.",
			"Pilihlah jawaban yang paling tepat!",
			"a. satu b. dua c. tiga",
			"2. Soal dua",
			"a. x b. y c. z"
		]));
		expect(blocks).toHaveLength(2);
		expect(extractOptions(blocks[0]!.lines).options).toEqual(["satu", "dua", "tiga", "", ""]);
	});

	it("melewati petunjuk umum bernomor sebelum soal nomor 1", () => {
		const { blocks } = parseNaskahLines(textLines([
			"Petunjuk Umum:",
			"1. Berdoalah sebelum mengerjakan.",
			"2. Tulis nama dan kelas.",
			"1. Soal pertama",
			"a. x b. y c. z",
			"2. Soal kedua",
			"a. x b. y c. z"
		]));
		expect(blocks.map((b) => b.lines[0])).toEqual(["Soal pertama", "Soal kedua"]);
	});

	it("tidak membuang soal nomor 1 sesudah petunjuk tanpa nomor", () => {
		const { blocks } = parseNaskahLines(textLines([
			"Petunjuk: Berilah tanda silang pada jawaban yang benar!",
			"1. Soal pertama",
			"a. x b. y c. z",
			"2. Soal kedua",
			"a. x b. y c. z"
		]));
		expect(blocks.map((b) => b.number)).toEqual([1, 2]);
	});

	it("mencatat nomor yang terlewat", () => {
		const { blocks, missingNumbers } = parseNaskahLines(textLines(["1. satu", "2. dua", "4. empat", "5. lima"]));
		expect(blocks.map((b) => b.number)).toEqual([1, 2, 4, 5]);
		expect(missingNumbers).toEqual([3]);
	});

	it("mengabaikan kalimat tentang kunci jawaban di bagian petunjuk", () => {
		const { blocks, kunciLines } = parseNaskahLines(textLines([
			"Kunci jawaban ditulis di lembar jawaban yang disediakan",
			"1. Soal satu",
			"a. x b. y c. z",
			"2. Soal dua",
			"a. x b. y c. z"
		]));
		expect(blocks).toHaveLength(2);
		expect(kunciLines).toEqual([]);
	});
});

describe("kunci jawaban", () => {
	it("membaca kunci di akhir naskah yang sama", () => {
		const rows = rowsFrom([
			"1. Soal satu",
			"a. x b. y c. z",
			"2. Soal dua",
			"a. x b. y c. z",
			"KUNCI JAWABAN",
			"1. C 2. A"
		]);
		expect(rows).toHaveLength(2);
		expect(rows.map((r) => [r.kunci, r.kunciSource])).toEqual([["C", "naskah"], ["A", "naskah"]]);
		expect(rows.every((r) => r.reasons.length === 0)).toBe(true);
	});

	it("membaca tabel kunci berjajar dan jawaban esai dari file terpisah", () => {
		const maps = parseKunciLines([
			"Kunci Jawaban PAS Kelas 6",
			"No Jawaban No Jawaban",
			"1 A 11 C",
			"2 B 12 D",
			"Uraian",
			"1. Senin, Selasa, Rabu",
			"dan seterusnya",
			"2. Bekerja bersama-sama"
		]);
		expect([...maps.pg.entries()]).toEqual([[1, "A"], [11, "C"], [2, "B"], [12, "D"]]);
		expect(maps.esai.get(1)).toBe("Senin, Selasa, Rabu dan seterusnya");
		expect(maps.esai.get(2)).toBe("Bekerja bersama-sama");
	});

	it("tidak membaca baris identitas seperti \"Kelas 6 A\" sebagai kunci", () => {
		expect(parseKunciLines(["Kelas : 6 A", "Mata Pelajaran: Matematika"]).pg.size).toBe(0);
	});

	it("mengutamakan tanda di naskah, lalu file kunci, lalu kunci di akhir naskah", () => {
		const rows = rowsFrom(
			[
				"1. Soal satu",
				"a. x *b. y c. z",
				"2. Soal dua",
				"a. x b. y c. z",
				"3. Soal tiga",
				"a. x b. y c. z",
				"Kunci: c",
				"Kunci Jawaban",
				"1. A 2. A 3. A"
			],
			["2. C"]
		);
		expect(rows.map((r) => [r.kunci, r.kunciSource])).toEqual([["B", "tanda"], ["C", "file"], ["C", "tanda"]]);
	});

	it("menandai soal PG tanpa kunci agar dipilih manual", () => {
		const [row] = rowsFrom(["1. Soal satu", "a. x b. y c. z"]);
		expect(row!.kunci).toBe("");
		expect(row!.reasons).toContain("kunci jawaban belum ada, pilih di kolom Kunci");
	});

	it("mencocokkan jawaban esai berdasarkan urutan bila penomorannya berbeda", () => {
		const rows = rowsFrom(["Uraian", "1. Soal esai satu", "2. Soal esai dua"], ["Uraian", "31. jawaban satu", "32. jawaban dua"]);
		expect(rows.map((r) => r.catatanJawabanEsai)).toEqual(["jawaban satu", "jawaban dua"]);
	});
});

describe("buildReadingLines", () => {
	it("membaca halaman dua kolom per kolom, bukan per baris", () => {
		const left = ["1. Soal kiri satu", "a. x b. y c. z", "2. Soal kiri dua", "a. x b. y c. z"];
		const right = ["3. Soal kanan tiga", "a. x b. y c. z", "4. Soal kanan empat", "a. x b. y c. z"];
		const data: PdfPageData = {
			width: 600,
			height: 840,
			items: [
				...textAt("UJIAN SEKOLAH TAHUN PELAJARAN 2026 MATA PELAJARAN MATEMATIKA KELAS ENAM", 40, 40),
				...left.flatMap((line, i) => textAt(line, 40, 80 + i * LINE_HEIGHT)),
				...right.flatMap((line, i) => textAt(line, 330, 80 + i * LINE_HEIGHT))
			],
			images: []
		};
		const { lines, twoColumnPages } = buildReadingLines([data]);
		expect(twoColumnPages).toBe(1);
		expect(lines.map((l) => (l.kind === "text" ? l.text : ""))).toEqual([
			"UJIAN SEKOLAH TAHUN PELAJARAN 2026 MATA PELAJARAN MATEMATIKA KELAS ENAM",
			...left,
			...right
		]);
	});

	it("tidak menganggap opsi berjajar sebagai dua kolom", () => {
		const data = page(["1. Soal", "a. satu                                   c. tiga", "b. dua                                    d. empat", "2. Soal", "a. satu                                   c. tiga", "b. dua                                    d. empat"]);
		expect(buildReadingLines([data]).twoColumnPages).toBe(0);
	});

	it("membuang kepala dan nomor halaman yang berulang", () => {
		const header = "SD NEGERI 1 CONTOH";
		const pages = [
			page([header, "1. Soal satu", "a. x b. y c. z", "Halaman 1 dari 2"]),
			page([header, "2. Soal dua", "a. x b. y c. z", "Halaman 2 dari 2"])
		];
		const texts = buildReadingLines(pages).lines.map((l) => (l.kind === "text" ? l.text : ""));
		expect(texts).toEqual(["1. Soal satu", "a. x b. y c. z", "2. Soal dua", "a. x b. y c. z"]);
	});

	it("menaruh gambar di soal tempat gambar itu berada", () => {
		const data = page(["1. Perhatikan gambar berikut!", "", "", "", "Gambar di atas adalah ....", "a. x b. y c. z", "2. Soal dua", "a. x b. y c. z"], {
			images: [{ id: "p0-0", x: 50, y: 80, width: 120, height: 36 }]
		});
		const { blocks } = parseNaskahLines(buildReadingLines([data]).lines);
		expect(blocks[0]!.imageIds).toEqual(["p0-0"]);
		expect(blocks[1]!.imageIds).toEqual([]);
	});
});

// Pola-pola berikut diambil dari naskah STS kelas 4 yang dipakai guru.
describe("pola naskah STS", () => {
	it("membagikan bacaan bersama ke semua soal pada rentang nomornya", () => {
		const rows = rowsFrom([
			"I. Berilah tanda silang (X) pada huruf a, b, c atau d untuk jawaban yang paling benar!",
			"Bacalah text berikut untuk menjawab soal no. 1-2",
			"Tak muat lagi",
			"Dina memiliki seorang kakak bernama Rani. Mereka sering bermain bersama.",
			"1. Tokoh dalam cerita adalah ....",
			"a. Dina c. Sinta",
			"b. Sari d. Rina",
			"2. Perasaan Dina adalah ....",
			"a. marah b. sedih c. takut d. senang",
			"3. Soal tanpa bacaan",
			"a. x b. y c. z d. w",
			"Bacalah text berikut untuk menjawab soal no. 4 - 4",
			"Rani adalah siswa kelas 4.",
			"4. Siapa tokohnya?",
			"a. Rani b. Ibu c. Guru d. Teman"
		]);
		expect(rows.map((r) => r.number)).toEqual([1, 2, 3, 4]);
		expect(rows[0]!.soal).toBe("Tak muat lagi\nDina memiliki seorang kakak bernama Rani. Mereka sering bermain bersama.\n\nTokoh dalam cerita adalah ....");
		expect(rows[1]!.soal.startsWith("Tak muat lagi")).toBe(true);
		expect(rows[2]!.soal).toBe("Soal tanpa bacaan");
		expect(rows[2]!.options[3]).toBe("w");
		expect(rows[3]!.soal).toBe("Rani adalah siswa kelas 4.\n\nSiapa tokohnya?");
	});

	it("tidak memecah soal yang memuat daftar bernomor", () => {
		const rows = rowsFrom([
			"20. Soal dua puluh",
			"a. x b. y c. z d. w",
			"21. Manakah kata yang mendapatkan imbuhan me-!",
			"1. membaca",
			"2. rumah",
			"a. 1 dan 2 c. 1 dan 3",
			"b. 3 dan 4 d. 2 dan 4",
			"22. Soal berikutnya",
			"a. x b. y c. z d. w"
		]);
		expect(rows.map((r) => r.number)).toEqual([20, 21, 22]);
		expect(rows[1]!.soal).toBe("Manakah kata yang mendapatkan imbuhan me-!\n1. membaca\n2. rumah");
		expect(rows[1]!.options.slice(0, 4)).toEqual(["1 dan 2", "3 dan 4", "1 dan 3", "2 dan 4"]);
	});

	it("membaca opsi yang menempel ke titiknya", () => {
		expect(extractOptions(["Jumlah boneka?", "a.42 b. 12 c. 21 d. 24"]).options).toEqual(["42", "12", "21", "24", ""]);
		expect(extractOptions(["Kunci?", "a.20 kg b. 30 kg c. 60 kg d . 40 kg"]).options).toEqual(["20 kg", "30 kg", "60 kg", "40 kg", ""]);
		expect(extractOptions(["Read it, e.g. a book."]).options).toBeNull();
	});

	it("memperlakukan butir a-d di bagian uraian sebagai sub-soal dan membuang garis jawaban", () => {
		const rows = rowsFrom([
			"30. Soal PG terakhir",
			"a. x b. y c. z d. w",
			"II. Jawablah pertanyaan di bawah ini dengan jawaban yang benar!",
			"31. Ubahlah kata dasar berikut menjadi kata berawalan me- !",
			"a. baca : ………………………………",
			"b. tulis : ………………………………",
			"32. Apa yang dimaksud dengan ide pokok ?",
			"Jawab",
			"……………………………………………………………",
			"……………………………………………………………"
		]);
		expect(rows.map((r) => [r.number, r.tipe])).toEqual([[30, "pg"], [31, "esai"], [32, "esai"]]);
		expect(rows[1]!.soal).toBe("Ubahlah kata dasar berikut menjadi kata berawalan me- !\na. baca : ....\nb. tulis : ....");
		expect(rows[2]!.soal).toBe("Apa yang dimaksud dengan ide pokok ?");
	});

	it("membaca tabel kunci jawaban berisi ringkasan jumlah soal dan uraian tanpa titik", () => {
		const maps = parseKunciLines([
			"KUNCI JAWABAN",
			"Jumlah Soal: 30 Soal Pilihan Ganda, 10 Soal Uraian",
			"I. Pilihan Ganda",
			"No Jawaban No Jawaban",
			"1 D 21 D",
			"2 A 22 B",
			"II. Uraian",
			"No Jawaban",
			"31 2/8",
			"32 Semangka"
		]);
		expect(maps.pg.get(1)).toBe("D");
		expect(maps.pg.get(22)).toBe("B");
		expect(maps.esai.get(31)).toBe("2/8");
		expect(maps.esai.get(32)).toBe("Semangka");
	});
});

// Kasus yang dulu diuji untuk pembaca opsi import Word lama, kini dibaca oleh pengurai bersama.
describe("variasi susunan opsi", () => {
	it("membaca lima opsi dalam satu baris", () => {
		expect(extractOptions(["Pilih satu", "a. satu b. dua c. tiga d. empat e. lima"]).options).toEqual(["satu", "dua", "tiga", "empat", "lima"]);
	});

	it("membaca opsi E pada baris sendiri dan susunan dua kolom", () => {
		expect(extractOptions(["Pilih", "a. satu c. tiga", "b. dua d. empat", "e. lima"]).options).toEqual(["satu", "dua", "tiga", "empat", "lima"]);
		expect(extractOptions(["Pilih", "a. satu   c. tiga   e. lima", "b. dua   d. empat"]).options).toEqual(["satu", "dua", "tiga", "empat", "lima"]);
	});

	it("membaca lima baris berlabel dan tidak salah mengenali akhir kata", () => {
		expect(extractOptions(["Pilih", "a. satu", "b. dua", "c. tiga", "d. empat", "e. lima"]).options).toEqual(["satu", "dua", "tiga", "empat", "lima"]);
		expect(extractOptions(["Pilih", "a. satu b. dua c. tiga d. kode."]).options).toEqual(["satu", "dua", "tiga", "kode.", ""]);
	});

	it("membaca opsi huruf kecil yang menempel di awal baris dan titik dobel", () => {
		expect(extractOptions(["Bunyi crik crik adalah ....", "a.kendang", "b.marakas", "c.symbal", "d.drum"]).options).toEqual(["kendang", "marakas", "symbal", "drum", ""]);
		expect(extractOptions(["Skala peta untuk ....", "a. jumlah penduduk", "b.. jarak sebenarnya", "c. keindahan", "d. warna"]).options?.[1]).toBe("jarak sebenarnya");
	});

	it("membetulkan label pertama yang dobel lalu melaporkannya", () => {
		const result = extractOptions(["Pilih nomor", "a. 1-2-3", "a. 3-5-6", "b. 2-3-5", "c. 3-6-4"]);
		expect(result.options).toEqual(["1-2-3", "3-5-6", "2-3-5", "3-6-4", ""]);
		expect(result.relabeled).toBe("a, a, b, c");
	});

	it("menandai label opsi yang tidak berurutan alih-alih menganggapnya esai", () => {
		const [row] = rowsFrom(["8. Gerakan menekuk lutut termasuk gerak ....", "a. menendang c. menekuk", "c. menggiring d. melempar", "9. Soal berikutnya", "a. x b. y c. z"]);
		expect(row!.tipe).toBe("pg");
		expect(row!.reasons[0]).toContain("label opsi tidak berurutan (a, c, c, d)");
	});

	it("menomori ulang soal yang nomornya salah ketik sesudah soal berisi opsi lengkap", () => {
		const rows = rowsFrom(["1. Soal satu", "a. x b. y c. z", "2. Soal dua", "a. x b. y c. z", "1. Soal yang seharusnya nomor tiga", "a. x b. y c. z", "4 Soal empat tanpa titik", "a. x b. y c. z"]);
		expect(rows.map((r) => r.number)).toEqual([1, 2, 3, 4]);
		expect(rows[2]!.fixes).toEqual(["nomor tertulis 1, dibaca sebagai nomor 3"]);
	});

	it("mengenali judul bagian berbahasa Inggris dan kata yang hanya mirip judul", () => {
		const rows = rowsFrom([
			"I. Choose the correct answer by crossing (X) a, b, c, or d!",
			"1. Lembaga di tingkat paling dasar adalah ....",
			"a. RT b. Kecamatan c. Kepolisian d. Puskesmas",
			"2. Soal dua",
			"a. x b. y c. z",
			"III. Answer the following question with the correct answer!",
			"3. Translate into English!",
			"a. 65",
			"b. 70",
			"Answer : ……………………………"
		]);
		expect(rows.map((r) => [r.number, r.tipe])).toEqual([[1, "pg"], [2, "pg"], [3, "esai"]]);
		expect(rows[0]!.options[2]).toBe("Kepolisian");
		expect(rows[2]!.soal).toBe("Translate into English!\na. 65\nb. 70");
	});
});
