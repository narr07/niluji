import { describe, expect, it } from "bun:test";
import { splitOptions } from "../app/composables/useDocxSoalImport";

describe("splitOptions", () => {
	it("membaca lima opsi dalam satu baris", () => {
		const result = splitOptions(["Pilih satu", "a. satu b. dua c. tiga d. empat e. lima"]);
		expect(result.soalLines).toEqual(["Pilih satu"]);
		expect(result.options).toEqual(["satu", "dua", "tiga", "empat", "lima"]);
	});

	it("membaca opsi E pada baris sendiri", () => {
		const result = splitOptions(["Pilih", "a. satu c. tiga", "b. dua d. empat", "e. lima"]);
		expect(result.options).toEqual(["satu", "dua", "tiga", "empat", "lima"]);
	});

	it("membaca susunan dua kolom dengan opsi E di kolom kanan", () => {
		const result = splitOptions(["Pilih", "a. satu   c. tiga   e. lima", "b. dua   d. empat"]);
		expect(result.options).toEqual(["satu", "dua", "tiga", "empat", "lima"]);
	});

	it("membaca lima baris berlabel", () => {
		const result = splitOptions(["Pilih", "a. satu", "b. dua", "c. tiga", "d. empat", "e. lima"]);
		expect(result.soalLines).toEqual(["Pilih"]);
		expect(result.options).toEqual(["satu", "dua", "tiga", "empat", "lima"]);
	});

	it("tetap membaca soal empat opsi dan tidak salah mengenali akhir kata", () => {
		const result = splitOptions(["Pilih", "a. satu b. dua c. tiga d. kode."]);
		expect(result.options).toEqual(["satu", "dua", "tiga", "kode.", ""]);
	});

	it("mempertahankan empat baris tanpa label", () => {
		const result = splitOptions(["Pilih", "satu", "dua", "tiga", "empat"]);
		expect(result.options).toEqual(["satu", "dua", "tiga", "empat", ""]);
	});
});
