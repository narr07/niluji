import type { BankSoalOnlineRow } from "./useBankSoalOnline";
import { fetch as tauriFetch } from "@tauri-apps/plugin-http";
import { parseSoalMarkdown } from "./useBankSoalOnline";

// Bank soal per gugus yang dipublikasikan pembuat soal (lihat src-tauri/src/bank_soal.rs).
// Sekolah hanya membaca: katalog.json lalu file soal.md dan gambarnya, langsung dari repo publik.
export const BANK_SOAL_REPO = "narr07/niluji-bank-soal";
export const BANK_SOAL_RAW = `https://raw.githubusercontent.com/${BANK_SOAL_REPO}/main`;
export const SEMUA_GUGUS = "semua-gugus";

export interface GugusInfo {
	id: string
	nama: string
}

export interface KecamatanInfo {
	id: string
	nama: string
	gugus: GugusInfo[]
}

export interface PaketEntry {
	kecamatan: string
	gugus: string
	kelas: string
	mapelKode: string
	mapelNama: string
	jenis: string
	folder: string
	pg: number
	esai: number
	diperbarui: number
}

export interface Katalog {
	versi: number
	diperbarui: number
	kecamatan: KecamatanInfo[]
	paket: PaketEntry[]
}

const encodePath = (path: string) => path.split("/").map(encodeURIComponent).join("/");

// Penanda waktu di URL supaya tidak mendapat salinan lama dari cache GitHub.
const fresh = (url: string) => `${url}?t=${Date.now()}`;

export const fetchKatalog = async (): Promise<Katalog> => {
	const response = await tauriFetch(fresh(`${BANK_SOAL_RAW}/katalog.json`));
	if (response.status === 404) throw new Error("Bank soal gugus belum tersedia. Pembuat soal belum mempublikasikan apa pun.");
	if (!response.ok) throw new Error(`Gagal mengambil daftar bank soal (status ${response.status}).`);
	return (await response.json()) as Katalog;
};

export const fetchPaketRows = async (entry: PaketEntry): Promise<BankSoalOnlineRow[]> => {
	const response = await tauriFetch(fresh(`${BANK_SOAL_RAW}/${encodePath(entry.folder)}/soal.md`));
	if (!response.ok) throw new Error(`File soal "${entry.jenis}" tidak ditemukan (status ${response.status}).`);
	return parseSoalMarkdown(await response.text()).rows;
};

export const paketImageUrl = (folder: string, fileName: string) =>
	`${BANK_SOAL_RAW}/${encodePath(folder)}/gambar/${encodeURIComponent(fileName)}`;

// Paket yang tampil untuk satu gugus: milik gugus itu sendiri dan yang berlaku untuk semua gugus.
export const paketUntukGugus = (katalog: Katalog, kecamatan: string, gugus: string) =>
	katalog.paket.filter((p) => p.kecamatan === kecamatan && (p.gugus === gugus || p.gugus === SEMUA_GUGUS));
