import type { SqlStatement } from "./useDb";
import { invoke } from "@tauri-apps/api/core";

export interface WebService {
	id: number
	nama_aplikasi: string
	ip_erapor: string
	ip_dapodik: string
	port: number
	token: string
	npsn: string
}

interface RawResponse {
	status: number
	body: string
}

export class DapodikError extends Error {}

// Satu request ke Web Service Dapodik → rows (selalu array). Status asli diambil dari body
// (Dapodik menyelipkan "HTTP/1.0 403 ..." di awal body), pesan errornya dari field "message".
export async function dapodikGet(ws: WebService, endpoint: string, semesterId?: string | null): Promise<any[]> {
	let raw: RawResponse;
	try {
		raw = await invoke<RawResponse>("dapodik_request", {
			req: {
				baseUrl: `http://${ws.ip_dapodik.trim()}:${ws.port}`,
				path: `WebService/${endpoint}`,
				token: ws.token,
				npsn: ws.npsn,
				semesterId: semesterId ?? null,
				timeoutSecs: 120
			}
		});
	}
	catch (e) {
		throw new DapodikError(pesanError(e));
	}
	const { status, body } = unwrapDapodikBody(raw.status, raw.body);
	let json: any;
	try {
		json = JSON.parse(body);
	}
	catch {
		throw new DapodikError(status === 404 ? `Endpoint ${endpoint} tidak tersedia di Dapodik ini` : `Response ${endpoint} bukan JSON (HTTP ${status})`);
	}
	if (json?.success === false || status >= 300)
		throw new DapodikError(json?.message || explainStatus(status));
	const rows = json?.rows;
	if (Array.isArray(rows))
		return rows;
	return rows && typeof rows === "object" ? [rows] : [];
}

// Tulis ke Web Service Dapodik, persis cara e-Rapor resmi: badan = JSON mentah dengan header form.
// Dapodik menjawab {success, message}; "tidak ada perubahan data" juga success (data sudah sama).
export async function dapodikPost(
	ws: WebService,
	endpoint: string,
	semesterId: string,
	data: Record<string, unknown>
): Promise<{ success: boolean, message: string }> {
	let raw: RawResponse;
	try {
		raw = await invoke<RawResponse>("dapodik_request", {
			req: {
				baseUrl: `http://${ws.ip_dapodik.trim()}:${ws.port}`,
				path: `WebService/${endpoint}`,
				token: ws.token,
				npsn: ws.npsn,
				semesterId,
				method: "POST",
				rawBody: JSON.stringify(data),
				timeoutSecs: 60
			}
		});
	}
	catch (e) {
		throw new DapodikError(pesanError(e));
	}
	const { status, body } = unwrapDapodikBody(raw.status, raw.body);
	let json: any;
	try {
		json = JSON.parse(body);
	}
	catch {
		throw new DapodikError(status === 404 ? `Endpoint ${endpoint} tidak tersedia di Dapodik ini` : `Response ${endpoint} bukan JSON (HTTP ${status})`);
	}
	if (status >= 300)
		throw new DapodikError(json?.message || explainStatus(status));
	return { success: json?.success !== false, message: String(json?.message ?? "") };
}

const now = () => new Date().toLocaleString("sv-SE");
const s = (v: unknown) => (v === undefined || v === null || v === "" ? null : String(v));

export function useDapodikSync() {
	const db = useDb();

	async function getWebService(): Promise<WebService | undefined> {
		return db.first<WebService>("SELECT * FROM webservice ORDER BY id LIMIT 1");
	}

	async function requireWebService(): Promise<WebService> {
		const ws = await getWebService();
		if (!ws)
			throw new DapodikError("Data Web Service belum diisi. Buka menu Web Service Dapodik dulu.");
		return ws;
	}

	// "Tes Koneksi Dapodik" — sama seperti e-Rapor: sukses = "Terhubung dengan Dapodik <nama sekolah>".
	async function testKoneksi(ws?: WebService): Promise<string> {
		const conn = ws ?? await requireWebService();
		const [sekolah] = await dapodikGet(conn, "getSekolah");
		if (!sekolah)
			throw new DapodikError("Dapodik tidak mengirim data sekolah");
		return `Terhubung dengan Dapodik ${sekolah.nama}`;
	}

	function sekolahStatements(rows: any[]): SqlStatement[] {
		return rows.map(r => ({
			sql: `INSERT INTO sekolah (sekolah_id, npsn, nama, alamat, kecamatan, kabupaten_kota, provinsi, email, raw, synced_at)
				VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
				ON CONFLICT(sekolah_id) DO UPDATE SET npsn = excluded.npsn, nama = excluded.nama, alamat = excluded.alamat,
					kecamatan = excluded.kecamatan, kabupaten_kota = excluded.kabupaten_kota, provinsi = excluded.provinsi,
					email = excluded.email, raw = excluded.raw, synced_at = excluded.synced_at`,
			params: [r.sekolah_id, s(r.npsn), r.nama, [r.alamat_jalan, r.desa_kelurahan].filter(Boolean).join(", "),
				s(r.kecamatan), s(r.kabupaten_kota), s(r.provinsi), s(r.email), JSON.stringify(r), now()]
		}));
	}

	function semesterStatement(semesterId: string): SqlStatement {
		return {
			sql: `INSERT INTO semester (semester_id, tahun_ajaran, nama, synced_at) VALUES (?, ?, ?, ?)
				ON CONFLICT(semester_id) DO UPDATE SET synced_at = excluded.synced_at`,
			params: [semesterId, tahunAjaran(semesterId), semesterLabel(semesterId), now()]
		};
	}

	// "Ambil Semester Saja": daftarkan semester + data sekolah, supaya bisa dipilih di halaman login.
	async function ambilSemester(semesterId: string): Promise<string> {
		const ws = await requireWebService();
		const sekolah = await dapodikGet(ws, "getSekolah", semesterId);
		await db.batch([...sekolahStatements(sekolah), semesterStatement(semesterId)]);
		return "Data Semester berhasil diupdate";
	}

	// "Ambil Seluruh Data": berurutan (bukan paralel) supaya PC Dapodik tidak terbebani.
	async function ambilSemua(semesterId: string, onStep: (msg: string) => void): Promise<Record<string, number>> {
		const ws = await requireWebService();
		const ts = now();

		onStep("Mengambil data sekolah…");
		const sekolah = await dapodikGet(ws, "getSekolah", semesterId);

		onStep("Mengambil data guru (GTK)…");
		const gtk = await dapodikGet(ws, "getGtk", semesterId);

		onStep("Mengambil data rombongan belajar…");
		const rombel = await dapodikGet(ws, "getRombonganBelajar", semesterId);

		onStep("Mengambil data peserta didik…");
		const pd = await dapodikGet(ws, "getPesertaDidik", semesterId);

		// Referensi mapel nasional (±5.500) untuk "Tambah Mapel" di halaman Mata Pelajaran.
		// Tidak wajib: kalau gagal, sinkron tetap jalan dan referensi lama tetap dipakai.
		onStep("Mengambil referensi mata pelajaran…");
		const refMapel = await dapodikGet(ws, "getMataPelajaran").catch(() => [] as any[]);

		onStep("Menyimpan ke database…");
		const st: SqlStatement[] = [...sekolahStatements(sekolah), semesterStatement(semesterId)];

		// GTK & siswa: upsert, kolom lokal (gelar) tidak ditimpa.
		for (const g of gtk) {
			st.push({
				sql: `INSERT INTO ptk (ptk_id, nama, nip, nuptk, nik, jenis_kelamin, tempat_lahir, tanggal_lahir, jenis_ptk, jabatan_ptk, status_kepegawaian, raw, synced_at)
					VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
					ON CONFLICT(ptk_id) DO UPDATE SET nama = excluded.nama, nip = excluded.nip, nuptk = excluded.nuptk, nik = excluded.nik,
						jenis_kelamin = excluded.jenis_kelamin, tempat_lahir = excluded.tempat_lahir, tanggal_lahir = excluded.tanggal_lahir,
						jenis_ptk = excluded.jenis_ptk, jabatan_ptk = excluded.jabatan_ptk, status_kepegawaian = excluded.status_kepegawaian,
						raw = excluded.raw, synced_at = excluded.synced_at`,
				params: [g.ptk_id, g.nama, s(g.nip), s(g.nuptk), s(g.nik), s(g.jenis_kelamin), s(g.tempat_lahir), s(g.tanggal_lahir),
					s(g.jenis_ptk_id_str), s(g.jabatan_ptk_id_str), s(g.status_kepegawaian_id_str), JSON.stringify(g), ts]
			});
		}
		for (const p of pd) {
			st.push({
				sql: `INSERT INTO peserta_didik (peserta_didik_id, nama, nisn, nipd, nik, jenis_kelamin, tempat_lahir, tanggal_lahir, agama, nama_ayah, nama_ibu, raw, synced_at)
					VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
					ON CONFLICT(peserta_didik_id) DO UPDATE SET nama = excluded.nama, nisn = excluded.nisn, nipd = excluded.nipd, nik = excluded.nik,
						jenis_kelamin = excluded.jenis_kelamin, tempat_lahir = excluded.tempat_lahir, tanggal_lahir = excluded.tanggal_lahir,
						agama = excluded.agama, nama_ayah = excluded.nama_ayah, nama_ibu = excluded.nama_ibu, raw = excluded.raw, synced_at = excluded.synced_at`,
				params: [p.peserta_didik_id, p.nama, s(p.nisn), s(p.nipd), s(p.nik), s(p.jenis_kelamin), s(p.tempat_lahir), s(p.tanggal_lahir),
					s(p.agama_id_str), s(p.nama_ayah), s(p.nama_ibu), JSON.stringify(p), ts]
			});
		}

		// Rombel, anggota, pembelajaran: ganti total per semester, supaya rombel/anggota yang
		// dihapus di Dapodik ikut hilang di sini.
		st.push({ sql: "DELETE FROM anggota_rombel WHERE semester_id = ?", params: [semesterId] });
		st.push({ sql: "DELETE FROM pembelajaran WHERE semester_id = ?", params: [semesterId] });
		st.push({ sql: "DELETE FROM rombel WHERE semester_id = ?", params: [semesterId] });
		const mapel = new Map<string, string>();
		let jmlAnggota = 0;
		let jmlPemb = 0;
		for (const r of rombel) {
			st.push({
				sql: `INSERT OR REPLACE INTO rombel (rombongan_belajar_id, semester_id, nama, tingkat, jenis_rombel, jenis_rombel_str, kurikulum, ptk_id, raw, synced_at)
					VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`,
				params: [r.rombongan_belajar_id, s(r.semester_id) ?? semesterId, r.nama, s(r.tingkat_pendidikan_id), s(r.jenis_rombel),
					s(r.jenis_rombel_str), s(r.kurikulum_id_str), s(r.ptk_id),
					JSON.stringify({ ...r, anggota_rombel: undefined, pembelajaran: undefined }), ts]
			});
			for (const a of r.anggota_rombel ?? []) {
				jmlAnggota++;
				st.push({
					sql: `INSERT OR REPLACE INTO anggota_rombel (anggota_rombel_id, rombongan_belajar_id, peserta_didik_id, semester_id, jenis_pendaftaran)
						VALUES (?, ?, ?, ?, ?)`,
					params: [a.anggota_rombel_id, r.rombongan_belajar_id, a.peserta_didik_id, semesterId, s(a.jenis_pendaftaran_id_str)]
				});
			}
			for (const p of r.pembelajaran ?? []) {
				jmlPemb++;
				if (p.mata_pelajaran_id)
					mapel.set(String(p.mata_pelajaran_id), p.mata_pelajaran_id_str || p.nama_mata_pelajaran);
				st.push({
					sql: `INSERT OR REPLACE INTO pembelajaran (pembelajaran_id, rombongan_belajar_id, semester_id, mata_pelajaran_id, mata_pelajaran_str, nama_mata_pelajaran, ptk_id, jam_per_minggu, raw)
						VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)`,
					params: [p.pembelajaran_id, r.rombongan_belajar_id, semesterId, s(p.mata_pelajaran_id), s(p.mata_pelajaran_id_str),
						s(p.nama_mata_pelajaran), s(p.ptk_id), p.jam_mengajar_per_minggu ?? null, JSON.stringify(p)]
				});
			}
		}
		for (const m of refMapel) {
			if (m.expired_date || !m.mata_pelajaran_id || !m.nama)
				continue;
			st.push({
				sql: "INSERT INTO mata_pelajaran (mata_pelajaran_id, nama) VALUES (?, ?) ON CONFLICT(mata_pelajaran_id) DO UPDATE SET nama = excluded.nama",
				params: [String(m.mata_pelajaran_id), String(m.nama).trim()]
			});
		}
		// Mapel yang dipakai pembelajaran (selalu ada, walau referensi nasional gagal diambil).
		for (const [id, nama] of mapel) {
			st.push({
				sql: "INSERT INTO mata_pelajaran (mata_pelajaran_id, nama) VALUES (?, ?) ON CONFLICT(mata_pelajaran_id) DO UPDATE SET nama = excluded.nama",
				params: [id, nama]
			});
		}

		// Kepala sekolah belum dipilih → tebak dari GTK berjabatan "Kepala Sekolah". Pilihan admin tidak ditimpa.
		st.push({
			sql: `UPDATE sekolah SET kepsek_ptk_id = (SELECT ptk_id FROM ptk WHERE jabatan_ptk LIKE '%kepala sekolah%' OR jenis_ptk LIKE '%kepala sekolah%' LIMIT 1)
				WHERE kepsek_ptk_id IS NULL OR kepsek_ptk_id NOT IN (SELECT ptk_id FROM ptk)`,
			params: []
		});

		await db.batch(st);
		await db.setSetting("last_sync", JSON.stringify({ semesterId, at: ts }));

		return {
			sekolah: sekolah.length,
			guru: gtk.length,
			rombel: rombel.length,
			siswa: pd.length,
			anggota: jmlAnggota,
			pembelajaran: jmlPemb,
			mapel: mapel.size
		};
	}

	return { getWebService, requireWebService, testKoneksi, ambilSemester, ambilSemua };
}
