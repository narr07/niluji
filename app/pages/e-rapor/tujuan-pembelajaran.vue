<script setup lang="ts">
	import { invoke } from "@tauri-apps/api/core";

	// Bank TP: per mapel + tingkat + semester, tidak terikat tahun ajaran — TP tahun lalu otomatis
	// muncul lagi. Guru: mapel yang diampu saja. Admin: semua mapel & kelas, plus rekap dan import
	// Excel satu sekolah (di e-Rapor hanya guru yang bisa mengisi TP).

	interface Tp { id: number, deskripsi: string, urutan: number, sumber: string }
	interface Pilihan { kode_mapel: string, singkat: string, nama: string, tingkat: number }

	const db = useDb();
	const toast = useToast();
	const swal = useSwal();
	const { user, session } = useAuth();
	const isAdmin = computed(() => user.value?.level === "admin");
	// Di laptop guru: TP buatan guru ditandai "guru" (ikut terkirim ke admin), TP dari admin hanya-baca.
	const { isGuruMode, load: loadMode } = useAppMode();
	const sumberBaru = computed(() => (isGuruMode.value ? "guru" : "admin"));
	const bisaUbah = (tp: Tp) => !isGuruMode.value || tp.sumber === "guru";
	// Penjaga di SQL juga (bukan cuma tombol yang dimatikan): di laptop guru hanya TP buatan guru yang bisa diubah.
	const jagaSumber = computed(() => (isGuruMode.value ? " AND sumber = 'guru'" : ""));

	const pilihan = ref<Pilihan[]>([]);
	const loading = ref(true);
	const smt = ref(semesterKe(session.value?.semesterId));
	const semesterItems = [{ label: "Ganjil", value: 1 }, { label: "Genap", value: 2 }];

	async function loadPilihan() {
		const sem = session.value?.semesterId;
		pilihan.value = sem
			? await db.query<Pilihan>(
				`SELECT DISTINCT pr.kode_mapel, m.singkat, m.nama, CAST(r.tingkat AS INTEGER) AS tingkat
				FROM pembelajaran_rapor pr JOIN rombel r USING (rombongan_belajar_id) JOIN mapel_rapor m ON m.kode = pr.kode_mapel
				WHERE pr.semester_id = ? ${isAdmin.value ? "" : "AND pr.ptk_id = ?"}
				ORDER BY m.urutan, tingkat`,
				isAdmin.value ? [sem] : [sem, user.value?.ptk_id]
			)
			: [];
	}

	// Mode (admin/guru) dimuat lebih dulu supaya tombol ubah/hapus tidak sempat tampil aktif untuk TP admin.
	async function init() {
		loading.value = true;
		try {
			await loadMode();
			smt.value = semesterKe(session.value?.semesterId);
			await loadPilihan();
			await loadRekap();
		}
		catch (e) {
			toast.add({ title: "Gagal memuat Tujuan Pembelajaran", description: pesanError(e), color: "error" });
		}
		finally {
			loading.value = false;
		}
	}
	watch(() => session.value?.semesterId, init, { immediate: true });

	const mapelItems = computed(() => {
		const seen = new Map<string, string>();
		pilihan.value.forEach(m => seen.set(m.kode_mapel, m.singkat));
		return [...seen].map(([value, label]) => ({ value, label }));
	});
	const kode = ref<string>();
	const tingkatItems = computed(() => pilihan.value.filter(m => m.kode_mapel === kode.value).map(m => ({ value: m.tingkat, label: `Kelas ${m.tingkat}` })));
	const tingkat = ref<number>();

	watch(mapelItems, (items) => {
		if (!items.some(i => i.value === kode.value))
			kode.value = items[0]?.value;
	});
	watch(tingkatItems, (items) => {
		if (!items.some(i => i.value === tingkat.value))
			tingkat.value = items[0]?.value;
	});

	// ===== Daftar TP terpilih =====
	const tps = ref<Tp[]>([]);
	const asli = new Map<number, string>(); // teks tersimpan, untuk deteksi perubahan & mengembalikan
	const dipakai = ref<Record<number, number>>({});
	const tpLoading = ref(false);
	const draft = ref("");
	const saving = ref(false);
	let permintaan = 0;

	async function load() {
		const nomor = ++permintaan;
		if (!kode.value || !tingkat.value) {
			tps.value = [];
			return;
		}
		tpLoading.value = true;
		try {
			const rows = await db.query<Tp>(
				"SELECT id, deskripsi, urutan, sumber FROM tujuan_pembelajaran WHERE kode_mapel = ? AND tingkat = ? AND semester = ? ORDER BY urutan, id",
				[kode.value, tingkat.value, smt.value]
			);
			// Hitung pemakaian sekali untuk semua TP di daftar (bukan subquery per baris).
			const ids = rows.map(r => r.id);
			const pakai = ids.length
				? await db.query<{ id: number, n: number }>(
					`SELECT value AS id, COUNT(*) AS n FROM (
						SELECT j.value FROM nilai_rapor n, json_each(n.tp_optimal) j
						UNION ALL SELECT j.value FROM nilai_rapor n, json_each(n.tp_perlu) j
					) WHERE value IN (${ids.map(() => "?").join(",")}) GROUP BY value`,
					ids
				)
				: [];
			// Pilihan sudah diganti lagi selama menunggu: buang hasil lama.
			if (nomor !== permintaan)
				return;
			tps.value = rows;
			asli.clear();
			rows.forEach(r => asli.set(r.id, r.deskripsi));
			dipakai.value = Object.fromEntries(pakai.map(p => [p.id, p.n]));
		}
		catch (e) {
			toast.add({ title: "Gagal memuat TP", description: pesanError(e), color: "error" });
		}
		finally {
			if (nomor === permintaan)
				tpLoading.value = false;
		}
	}
	watch([kode, tingkat, smt], load);

	// Sesuai panduan e-Rapor: tanpa awalan "Peserta didik dapat", diawali huruf kecil.
	function rapikan(line: string) {
		let t = line.trim().replace(/^[-•*\d.)\s]+/, "").replace(/^peserta didik (dapat|mampu)\s+/i, "").replace(/\.$/, "");
		if (t)
			t = t.charAt(0).toLowerCase() + t.slice(1);
		return t;
	}

	// Batas 100 karakter sama dengan e-Rapor (supaya deskripsi rapor tidak kepanjangan).
	const draftLines = computed(() => draft.value.split(/\r?\n/).map(rapikan).filter(Boolean));
	const draftTerlalu = computed(() => draftLines.value
		.map((d, i) => ({ no: i + 1, panjang: d.length }))
		.filter(x => x.panjang > TP_MAX));

	async function tambah() {
		const lines = draftLines.value;
		if (!lines.length || draftTerlalu.value.length || saving.value)
			return;
		saving.value = true;
		try {
			const start = Math.max(0, ...tps.value.map(t => t.urutan)) + 1;
			// TP yang teksnya sudah ada di mapel/kelas/semester ini dilewati.
			const n = await db.batch(lines.map((d, i) => ({
				sql: `INSERT INTO tujuan_pembelajaran (kode_mapel, tingkat, semester, deskripsi, urutan, sumber)
					SELECT ?, ?, ?, ?, ?, ? WHERE NOT EXISTS (
						SELECT 1 FROM tujuan_pembelajaran WHERE kode_mapel = ? AND tingkat = ? AND semester = ? AND deskripsi = ?)`,
				params: [kode.value, tingkat.value, smt.value, d, start + i, sumberBaru.value, kode.value, tingkat.value, smt.value, d]
			})));
			draft.value = "";
			await Promise.all([load(), loadRekap()]);
			toast.add({ title: `${n} TP ditambahkan`, description: n < lines.length ? `${lines.length - n} sudah ada (dilewati).` : undefined, color: "success" });
		}
		catch (e) {
			toast.add({ title: "Gagal menambah TP", description: pesanError(e), color: "error" });
		}
		finally {
			saving.value = false;
		}
	}

	function kembalikan(tp: Tp) {
		tp.deskripsi = asli.get(tp.id) ?? tp.deskripsi;
	}

	async function simpan(tp: Tp) {
		if (!bisaUbah(tp))
			return;
		const d = rapikan(tp.deskripsi);
		if (d === asli.get(tp.id)) {
			tp.deskripsi = d;
			return; // tidak berubah, tidak perlu menulis
		}
		if (!d) {
			toast.add({ title: "TP tidak boleh kosong", description: "Teks dikembalikan. Untuk menghapus, pakai tombol hapus.", color: "warning" });
			return kembalikan(tp);
		}
		if (d.length > TP_MAX) {
			toast.add({ title: `TP${tps.value.indexOf(tp) + 1} terlalu panjang (${d.length}/${TP_MAX} karakter)`, description: "Teks dikembalikan. Ringkas dulu kalimatnya.", color: "error" });
			return kembalikan(tp);
		}
		if (tps.value.some(x => x.id !== tp.id && x.deskripsi === d)) {
			toast.add({ title: "TP kembar", description: "Teks yang sama sudah ada di daftar ini. Teks dikembalikan.", color: "warning" });
			return kembalikan(tp);
		}
		try {
			const n = await db.execute(`UPDATE tujuan_pembelajaran SET deskripsi = ? WHERE id = ?${jagaSumber.value}`, [d, tp.id]);
			if (!n)
				throw new Error("TP ini tidak bisa diubah dari laptop guru.");
			tp.deskripsi = d;
			asli.set(tp.id, d);
		}
		catch (e) {
			kembalikan(tp);
			toast.add({ title: "Gagal menyimpan TP", description: pesanError(e), color: "error" });
		}
	}

	async function hapus(tp: Tp) {
		const n = dipakai.value[tp.id] ?? 0;
		if (!await swal.confirm("Hapus TP?", n ? `TP ini sudah dicentang di ${n} nilai siswa. Centangnya ikut hilang dari deskripsi.` : `"${tp.deskripsi}"`, "Hapus"))
			return;
		try {
			const hasil = await db.execute(`DELETE FROM tujuan_pembelajaran WHERE id = ?${jagaSumber.value}`, [tp.id]);
			if (!hasil)
				throw new Error("TP ini tidak bisa dihapus dari laptop guru.");
			await Promise.all([load(), loadRekap()]);
		}
		catch (e) {
			toast.add({ title: "Gagal menghapus TP", description: pesanError(e), color: "error" });
		}
	}

	// Urutan TP sengaja tidak bisa diubah (keputusan sekolah): nomor TP1, TP2, … tetap sama di admin,
	// laptop guru, file Excel, dan centang nilai. Urutan = urutan saat diketik / kolom NO di Excel.

	async function salinSemesterLain() {
		const lain = smt.value === 1 ? 2 : 1;
		const namaLain = lain === 1 ? "Ganjil" : "Genap";
		const namaIni = smt.value === 1 ? "Ganjil" : "Genap";
		if (!await swal.confirm(`Salin TP dari semester ${namaLain}?`, `TP ${mapelItems.value.find(m => m.value === kode.value)?.label} kelas ${tingkat.value} semester ${namaLain} disalin ke semester ${namaIni}. TP yang sudah ada dilewati.`, "Salin"))
			return;
		try {
			const n = await db.execute(
				`INSERT INTO tujuan_pembelajaran (kode_mapel, tingkat, semester, deskripsi, urutan, sumber)
				SELECT kode_mapel, tingkat, ?, deskripsi, urutan, ? FROM tujuan_pembelajaran t WHERE kode_mapel = ? AND tingkat = ? AND semester = ?
				AND NOT EXISTS (SELECT 1 FROM tujuan_pembelajaran x WHERE x.kode_mapel = t.kode_mapel AND x.tingkat = t.tingkat AND x.semester = ? AND x.deskripsi = t.deskripsi)`,
				[smt.value, sumberBaru.value, kode.value, tingkat.value, lain, smt.value]
			);
			await Promise.all([load(), loadRekap()]);
			toast.add({ title: n ? `${n} TP disalin` : `Tidak ada TP baru di semester ${namaLain}`, color: n ? "success" : "warning" });
		}
		catch (e) {
			toast.add({ title: "Gagal menyalin TP", description: pesanError(e), color: "error" });
		}
	}

	// ===== Rekap (admin): jumlah TP per mapel × kelas =====
	const rekap = ref<Record<string, number>>({});
	const rekapMapel = computed(() => mapelItems.value);
	// Kolom kelas mengikuti kelas yang benar-benar ada (tidak dikunci 6 kelas).
	const rekapKelas = computed(() => [...new Set(pilihan.value.map(p => p.tingkat))].sort((a, b) => a - b));
	const kelasAda = (k: string, t: number) => pilihan.value.some(p => p.kode_mapel === k && p.tingkat === t);

	async function loadRekap() {
		if (!isAdmin.value)
			return;
		try {
			const rows = await db.query<{ kode_mapel: string, tingkat: number, n: number }>(
				"SELECT kode_mapel, tingkat, COUNT(*) AS n FROM tujuan_pembelajaran WHERE semester = ? GROUP BY kode_mapel, tingkat",
				[smt.value]
			);
			rekap.value = Object.fromEntries(rows.map(r => [`${r.kode_mapel}:${r.tingkat}`, r.n]));
		}
		catch (e) {
			toast.add({ title: "Gagal memuat rekap TP", description: pesanError(e), color: "error" });
		}
	}
	watch(smt, loadRekap);

	function pilih(k: string, t: number) {
		kode.value = k;
		tingkat.value = t;
	}

	const lengkap = computed(() => {
		const sel = pilihan.value.length;
		const terisi = pilihan.value.filter(p => rekap.value[`${p.kode_mapel}:${p.tingkat}`]).length;
		return { sel, terisi };
	});

	// ===== Excel: template, export, import (format sama dengan e-Rapor) =====
	const fileInput = ref<HTMLInputElement>();
	const busy = ref(false);
	const BARIS_KOSONG = 10; // per kelas, seperti batas 10 TP sekali entri di e-Rapor

	async function simpanFile(fileName: string, bytes: Uint8Array) {
		const path = await invoke<string>("save_to_downloads", { fileName, bytes: Array.from(bytes) });
		toast.add({
			title: "File tersimpan di Downloads",
			description: path,
			color: "success",
			actions: [{ label: "Buka Folder", onClick: () => { invoke("reveal_file", { path }); } }]
		});
	}

	// Sheet per mapel: TP yang sudah ada (isi = true) atau baris kosong siap diisi per kelas.
	async function buatSheets(kodeList: string[], isi: boolean): Promise<TpSheet[]> {
		const out: TpSheet[] = [];
		for (const k of kodeList) {
			const p = pilihan.value.filter(x => x.kode_mapel === k);
			if (!p[0])
				continue;
			const kelas = p.map(x => x.tingkat);
			let rows: TpSheet["rows"];
			if (isi) {
				rows = await db.query(
					`SELECT tingkat, semester, deskripsi FROM tujuan_pembelajaran WHERE kode_mapel = ?
					AND tingkat IN (${kelas.map(() => "?").join(",")}) ORDER BY tingkat, semester, urutan, id`,
					[k, ...kelas]
				);
			}
			else {
				rows = kelas.flatMap(t => Array.from({ length: BARIS_KOSONG }, () => ({ tingkat: t, semester: smt.value, deskripsi: "" })));
			}
			out.push({ kode: k, nama: p[0].nama, singkat: p[0].singkat, rows });
		}
		return out;
	}

	async function download(jenis: "template-mapel" | "template-semua" | "export") {
		busy.value = true;
		try {
			const semua = mapelItems.value.map(m => m.value);
			const mapelIni = mapelItems.value.find(m => m.value === kode.value)?.label ?? "Mapel";
			if (jenis === "template-mapel")
				await simpanFile(`f_tp_${mapelIni}.xlsx`, buildTpWorkbook(await buatSheets([kode.value!], false)));
			else if (jenis === "template-semua")
				await simpanFile("f_tp_Semua Mapel.xlsx", buildTpWorkbook(await buatSheets(semua, false)));
			else
				await simpanFile("Tujuan Pembelajaran.xlsx", buildTpWorkbook(await buatSheets(semua, true)));
		}
		catch (e) {
			swal.error("Gagal membuat file", pesanError(e));
		}
		finally {
			busy.value = false;
		}
	}

	const downloadItems = computed(() => [
		{ label: `Template ${mapelItems.value.find(m => m.value === kode.value)?.label ?? ""}`, icon: "lucide:file-spreadsheet", onSelect: () => download("template-mapel") },
		{ label: "Template semua mapel", icon: "lucide:files", onSelect: () => download("template-semua") },
		{ label: "TP yang sudah ada", icon: "lucide:file-down", onSelect: () => download("export") }
	]);

	async function importTp(e: Event) {
		const file = (e.target as HTMLInputElement).files?.[0];
		(e.target as HTMLInputElement).value = "";
		if (!file)
			return;
		busy.value = true;
		try {
			const mapel = pilihan.value.map(p => ({ kode: p.kode_mapel, nama: p.nama, singkat: p.singkat }));
			const { rows, issues } = readTpWorkbook(await file.arrayBuffer(), mapel, kode.value);
			const valid: TpRow[] = [];
			for (const r of rows) {
				const d = rapikan(r.deskripsi);
				const label = `${mapel.find(m => m.kode === r.kode_mapel)?.singkat} kelas ${r.tingkat}`;
				if (d.length > TP_MAX)
					issues.push({ sheet: label, baris: 0, alasan: `"${d.slice(0, 40)}…" ${d.length}/${TP_MAX} karakter` });
				else if (!pilihan.value.some(p => p.kode_mapel === r.kode_mapel && p.tingkat === r.tingkat))
					issues.push({ sheet: label, baris: 0, alasan: isAdmin.value ? "Kelas ini tidak punya mapel tersebut" : "Bukan mapel/kelas yang Anda ampu" });
				else
					valid.push({ ...r, deskripsi: d });
			}
			if (!valid.length && !issues.length)
				throw new Error("File tidak berisi TP. Pakai template dari tombol Download.");

			// Urutan mengikuti kolom NO di file (baris tanpa NO tetap urutan baris di file).
			valid.sort((a, b) => (a.no ?? Number.MAX_SAFE_INTEGER) - (b.no ?? Number.MAX_SAFE_INTEGER));
			// TP yang teksnya sudah ada dilewati; urutan lanjut dari TP terakhir di grup yang sama.
			// Jumlah baru = jumlah baris yang benar-benar tertulis (hasil db.batch).
			const baru = await db.batch(valid.map(r => ({
				sql: `INSERT INTO tujuan_pembelajaran (kode_mapel, tingkat, semester, deskripsi, sumber, urutan)
					SELECT ?, ?, ?, ?, ?, COALESCE((SELECT MAX(urutan) FROM tujuan_pembelajaran WHERE kode_mapel = ? AND tingkat = ? AND semester = ?), 0) + 1
					WHERE NOT EXISTS (SELECT 1 FROM tujuan_pembelajaran WHERE kode_mapel = ? AND tingkat = ? AND semester = ? AND deskripsi = ?)`,
				params: [r.kode_mapel, r.tingkat, r.semester, r.deskripsi, sumberBaru.value, r.kode_mapel, r.tingkat, r.semester, r.kode_mapel, r.tingkat, r.semester, r.deskripsi]
			})));
			await Promise.all([load(), loadRekap()]);

			const ringkas = issues.slice(0, 8).map(i => `• ${i.sheet}${i.baris ? ` baris ${i.baris}` : ""}: ${i.alasan}`).join("\n");
			const lebih = issues.length > 8 ? `\n…dan ${issues.length - 8} lainnya` : "";
			await swal.fire({
				type: issues.length ? "warning" : "success",
				title: "Import selesai",
				text: `${baru} TP baru ditambahkan, ${valid.length - baru} sudah ada (dilewati).${issues.length ? `\n\n${issues.length} baris ditolak:\n${ringkas}${lebih}` : ""}`
			});
		}
		catch (err) {
			swal.error("Gagal import", pesanError(err));
		}
		finally {
			busy.value = false;
		}
	}
</script>

<template>
	<ErPage id="tp" title="Tujuan Pembelajaran">
		<template #right>
			<UDropdownMenu :items="downloadItems">
				<UButton
					icon="lucide:file-down"
					variant="subtle"
					trailing-icon="lucide:chevron-down"
					:loading="busy"
					:disabled="!kode">
					Download
				</UButton>
			</UDropdownMenu>
			<UButton
				icon="lucide:upload"
				variant="soft"
				:disabled="busy || !kode"
				@click="fileInput?.click()">
				Import
			</UButton>
			<input
				ref="fileInput"
				type="file"
				accept=".xlsx,.xls"
				class="hidden"
				aria-label="Pilih file Excel TP"
				@change="importTp">
			<UButton
				icon="lucide:copy"
				variant="soft"
				:disabled="!kode"
				@click="salinSemesterLain">
				Salin dari {{ smt === 1 ? "Genap" : "Ganjil" }}
			</UButton>
		</template>

		<div v-if="loading" class="space-y-4">
			<USkeleton class="h-12 w-full" />
			<USkeleton class="h-64 w-full" />
		</div>

		<div v-else-if="!pilihan.length" class="flex flex-col items-center gap-3 py-24 text-center text-muted">
			<UIcon name="lucide:book-x" class="size-12" />
			<p v-if="isAdmin">
				Belum ada pembelajaran rapor. Susun dulu di Data Referensi → Pembelajaran.
			</p>
			<p v-else>
				Anda belum mengampu pembelajaran di semester ini. Minta admin menyusun pembelajaran.
			</p>
		</div>

		<div v-else class="space-y-4">
			<!-- Filter ringkas satu baris: ikon menggantikan label supaya tidak makan tempat. -->
			<div class="flex flex-wrap items-center gap-2">
				<USelect
					v-model="kode"
					:items="mapelItems"
					icon="lucide:book-open"
					placeholder="Mata pelajaran"
					aria-label="Mata pelajaran"
					class="w-48" />
				<USelect
					v-model="tingkat"
					:items="tingkatItems"
					icon="lucide:layers"
					placeholder="Kelas"
					aria-label="Kelas"
					class="w-32" />
				<USelect
					v-model="smt"
					:items="semesterItems"
					icon="lucide:calendar"
					aria-label="Semester"
					class="w-32" />
				<UBadge variant="subtle" color="neutral" class="ms-auto">
					{{ tps.length }} TP
				</UBadge>
				<UTooltip text="TP tidak terikat tahun ajaran: tahun depan otomatis muncul lagi.">
					<UIcon name="lucide:info" class="size-4 text-muted" />
				</UTooltip>
			</div>

			<PanelCard
				v-if="isAdmin"
				title="Rekap TP"
				icon="lucide:grid-3x3"
				:description="`${lengkap.terisi}/${lengkap.sel} mapel-kelas sudah punya TP (semester ${smt === 1 ? 'Ganjil' : 'Genap'}). Klik angka untuk mengisi.`"
			>
				<div class="overflow-x-auto">
					<table class="w-full text-sm">
						<thead class="text-xs text-muted">
							<tr>
								<th class="p-2 text-left">
									Mapel
								</th>
								<th v-for="t in rekapKelas" :key="t" class="w-16 p-2 text-center">
									Kls {{ t }}
								</th>
							</tr>
						</thead>
						<tbody class="divide-y divide-default">
							<tr v-for="m in rekapMapel" :key="m.value">
								<td class="p-2 font-medium">
									{{ m.label }}
								</td>
								<td v-for="t in rekapKelas" :key="t" class="p-1 text-center">
									<button
										v-if="kelasAda(m.value, t)"
										type="button"
										class="w-12 cursor-pointer rounded py-1 text-xs font-semibold"
										:class="[
											rekap[`${m.value}:${t}`] ? 'bg-success/15 text-success' : 'bg-error/10 text-error',
											kode === m.value && tingkat === t ? 'ring-2 ring-primary' : ''
										]"
										:aria-label="`Isi TP ${m.label} kelas ${t} (${rekap[`${m.value}:${t}`] ?? 0} TP)`"
										@click="pilih(m.value, t)"
									>
										{{ rekap[`${m.value}:${t}`] ?? 0 }}
									</button>
									<span v-else class="text-dimmed">–</span>
								</td>
							</tr>
						</tbody>
					</table>
				</div>
			</PanelCard>

			<div class="grid gap-4 xl:grid-cols-[1fr_380px]">
				<PanelCard
					:title="`TP ${mapelItems.find(m => m.value === kode)?.label ?? ''} Kelas ${tingkat ?? ''}`"
					icon="lucide:list-ordered"
					description="Nomor TP mengikuti urutan input / kolom NO di Excel"
				>
					<div v-if="tpLoading && !tps.length" class="space-y-2">
						<USkeleton v-for="i in 4" :key="i" class="h-9 w-full" />
					</div>
					<ol v-else-if="tps.length" class="divide-y divide-default">
						<li v-for="(tp, i) in tps" :key="tp.id" class="flex items-start gap-2 bg-default py-2">
							<span class="mt-1.5 w-10 shrink-0 text-xs font-semibold text-muted">TP{{ i + 1 }}</span>
							<div class="flex-1">
								<UTextarea
									v-model="tp.deskripsi"
									:rows="1"
									autoresize
									variant="ghost"
									:disabled="!bisaUbah(tp)"
									:maxlength="TP_MAX + 20"
									:color="tp.deskripsi.length > TP_MAX ? 'error' : undefined"
									:aria-label="`TP${i + 1}`"
									class="w-full"
									@blur="simpan(tp)"
								/>
								<p v-if="tp.deskripsi.length > TP_MAX - 10" class="px-2 text-xs" :class="tp.deskripsi.length > TP_MAX ? 'text-error' : 'text-muted'">
									{{ tp.deskripsi.length }}/{{ TP_MAX }} karakter
								</p>
							</div>
							<UBadge
								v-if="isGuruMode && tp.sumber === 'admin'"
								color="neutral"
								variant="subtle"
								size="sm"
								class="mt-1.5"
								icon="lucide:lock">
								dari admin
							</UBadge>
							<UBadge
								v-if="dipakai[tp.id]"
								variant="subtle"
								size="sm"
								class="mt-1.5">
								{{ dipakai[tp.id] }} nilai
							</UBadge>
							<UButton
								icon="lucide:trash-2"
								variant="ghost"
								color="error"
								size="xs"
								class="shrink-0"
								:disabled="!bisaUbah(tp)"
								:aria-label="`Hapus TP${i + 1}`"
								@click="hapus(tp)"
							/>
						</li>
					</ol>
					<p v-else class="py-8 text-center text-sm text-muted">
						Belum ada TP untuk mapel, kelas, dan semester ini.
					</p>
				</PanelCard>

				<PanelCard title="Tambah TP" icon="lucide:list-plus" description="Satu baris satu TP. Bisa tempel langsung dari dokumen ATP.">
					<div class="flex flex-col gap-3">
						<UTextarea
							v-model="draft"
							:rows="8"
							autoresize
							aria-label="TP baru, satu baris satu TP"
							placeholder="memahami makna kedaulatan rakyat dalam sistem pemerintahan Indonesia&#10;menganalisis pelaksanaan sistem pemerintahan yang baik"
						/>
						<UAlert
							v-if="draftTerlalu.length"
							color="error"
							variant="subtle"
							icon="lucide:triangle-alert"
							:title="`Maksimal ${TP_MAX} karakter per TP`"
							:description="draftTerlalu.map(x => `Baris ${x.no}: ${x.panjang} karakter`).join(' · ')"
						/>
						<p class="text-xs text-muted">
							Maksimal {{ TP_MAX }} karakter per TP. Awalan "Peserta didik dapat", nomor, dan titik di akhir dibuang otomatis. TP yang sudah ada dilewati.
							Bisa juga lewat <b>Download → Template</b> lalu <b>Import</b> (format sama dengan e-Rapor, file f_tp lama juga bisa).
						</p>
						<UButton
							icon="lucide:plus"
							variant="solid"
							:loading="saving"
							:disabled="!draftLines.length || !!draftTerlalu.length || !kode"
							class="self-end"
							@click="tambah">
							Tambahkan
						</UButton>
					</div>
				</PanelCard>
			</div>
		</div>
	</ErPage>
</template>
