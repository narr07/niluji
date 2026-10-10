<script setup lang="ts">
	import { invoke } from "@tauri-apps/api/core";

	// Padanan e-Rapor: menu Transkrip Ijazah (Import Nomor Ijazah, Setting, Mapping Mapel, Input/Import Nilai,
	// Cetak) digabung jadi satu halaman. Mapel transkrip = mapel rapor yang "Transkrip"-nya aktif
	// (Data Referensi → Mata Pelajaran), urutan sama dengan rapor. Nilai bisa diisi otomatis dari
	// rata-rata nilai rapor semua semester yang tersimpan.
	const props = defineProps<{ semuaKelas?: boolean }>();

	const db = useDb();
	const toast = useToast();
	const swal = useSwal();
	const { session, waliRombel } = useAuth();
	const genap = computed(() => semesterKe(session.value?.semesterId) === 2);

	// ===== Kelas 6 =====
	const kelasList = ref<{ value: string, label: string }[]>([]);
	const rombelId = ref<string>();
	const kelasNama = computed(() => kelasList.value.find(k => k.value === rombelId.value)?.label ?? "");

	async function loadKelas() {
		const sem = session.value?.semesterId;
		const ids = props.semuaKelas ? null : waliRombel.value.map(r => r.rombongan_belajar_id);
		kelasList.value = !sem || (ids && !ids.length)
			? []
			: await db.query(
				`SELECT rombongan_belajar_id AS value, nama AS label FROM rombel WHERE semester_id = ? AND jenis_rombel = '1'
				AND CAST(tingkat AS INTEGER) = 6 ${ids ? `AND rombongan_belajar_id IN (${ids.map(() => "?").join(",")})` : ""} ORDER BY nama`,
				[sem, ...(ids ?? [])]
			);
		if (!kelasList.value.some(k => k.value === rombelId.value))
			rombelId.value = kelasList.value[0]?.value;
	}

	// ===== Data =====
	interface Siswa extends TranskripBaris { tempatLahir: string, tanggalLahir: string }
	const mapel = ref<{ kode: string, nama: string, singkat: string }[]>([]);
	const siswa = ref<Siswa[]>([]);
	const loading = ref(true);
	let permintaan = 0;

	async function load() {
		const nomor = ++permintaan;
		if (!rombelId.value) {
			siswa.value = [];
			loading.value = false;
			return;
		}
		loading.value = true;
		try {
			const [m, s, n] = await Promise.all([
				db.query<{ kode: string, nama: string, singkat: string }>(
					`SELECT m.kode, m.nama, m.singkat FROM mapel_rapor m WHERE m.transkrip = 1
					AND m.kode IN (SELECT kode_mapel FROM pembelajaran_rapor WHERE rombongan_belajar_id = ?) ORDER BY m.urutan`,
					[rombelId.value]
				),
				db.query<{ id: string, nisn: string | null, nama: string, tempat_lahir: string | null, tanggal_lahir: string | null, no_ijazah: string | null, no_transkrip: string | null, tanggal_lulus: string | null }>(
					`SELECT s.peserta_didik_id AS id, s.nisn, s.nama, s.tempat_lahir, s.tanggal_lahir, t.no_ijazah, t.no_transkrip, t.tanggal_lulus
					FROM anggota_rombel a JOIN siswa_rapor s USING (peserta_didik_id) LEFT JOIN transkrip_siswa t USING (peserta_didik_id)
					WHERE a.rombongan_belajar_id = ? ORDER BY s.nama COLLATE NOCASE`,
					[rombelId.value]
				),
				db.query<{ siswa: string, kode: string, nilai: number | null }>(
					`SELECT peserta_didik_id AS siswa, kode_mapel AS kode, nilai FROM nilai_transkrip
					WHERE peserta_didik_id IN (SELECT peserta_didik_id FROM anggota_rombel WHERE rombongan_belajar_id = ?)`,
					[rombelId.value]
				)
			]);
			if (nomor !== permintaan)
				return;
			mapel.value = m;
			siswa.value = s.map(x => ({
				id: x.id,
				nisn: x.nisn,
				nama: x.nama,
				tempatLahir: x.tempat_lahir ?? "",
				tanggalLahir: x.tanggal_lahir ?? "",
				noIjazah: x.no_ijazah ?? "",
				noTranskrip: x.no_transkrip ?? "",
				tanggalLulus: x.tanggal_lulus ?? "",
				nilai: Object.fromEntries(n.filter(v => v.siswa === x.id).map(v => [v.kode, v.nilai]))
			}));
		}
		catch (e) {
			toast.add({ title: "Gagal memuat transkrip", description: pesanError(e), color: "error" });
		}
		finally {
			if (nomor === permintaan)
				loading.value = false;
		}
	}

	async function init() {
		try {
			await muatPengaturan();
			await loadKelas();
			await load();
		}
		catch (e) {
			toast.add({ title: "Gagal memuat halaman", description: pesanError(e), color: "error" });
			loading.value = false;
		}
	}
	watch(() => session.value?.semesterId, init, { immediate: true });
	watch(rombelId, async () => {
		await flush();
		await load();
	});

	// ===== Simpan otomatis =====
	const pending = new Map<string, { timer: ReturnType<typeof setTimeout>, run: () => Promise<void> }>();
	const saveState = ref<"idle" | "saving" | "saved" | "error">("idle");
	function antre(key: string, sql: string, params: unknown[]) {
		clearTimeout(pending.get(key)?.timer);
		saveState.value = "saving";
		const run = async () => {
			pending.delete(key);
			try {
				await db.execute(sql, params);
				if (!pending.size)
					saveState.value = "saved";
			}
			catch (e) {
				saveState.value = "error";
				toast.add({ title: "Gagal menyimpan", description: pesanError(e), color: "error" });
			}
		};
		pending.set(key, { timer: setTimeout(run, 500), run });
	}
	async function flush() {
		const semua = [...pending.values()];
		semua.forEach(p => clearTimeout(p.timer));
		await Promise.all(semua.map(p => p.run()));
	}
	onBeforeRouteLeave(flush);

	const SQL_IDENTITAS = `INSERT INTO transkrip_siswa (peserta_didik_id, no_ijazah, no_transkrip, tanggal_lulus) VALUES (?, ?, ?, ?)
		ON CONFLICT (peserta_didik_id) DO UPDATE SET no_ijazah = excluded.no_ijazah, no_transkrip = excluded.no_transkrip, tanggal_lulus = excluded.tanggal_lulus`;
	const SQL_NILAI = `INSERT INTO nilai_transkrip (peserta_didik_id, kode_mapel, nilai) VALUES (?, ?, ?)
		ON CONFLICT (peserta_didik_id, kode_mapel) DO UPDATE SET nilai = excluded.nilai`;
	const paramIdentitas = (s: Siswa) => [s.id, s.noIjazah.trim() || null, s.noTranskrip.trim() || null, s.tanggalLulus || null];

	function setIdentitas(s: Siswa, kolom: "noIjazah" | "noTranskrip" | "tanggalLulus", v: string) {
		s[kolom] = v;
		antre(`i:${s.id}`, SQL_IDENTITAS, paramIdentitas(s));
	}

	function setNilai(s: Siswa, kode: string, el: HTMLInputElement) {
		const raw = el.value.trim().replace(",", ".");
		const n = raw === "" ? null : Math.round(Number(raw) * 100) / 100;
		if (n !== null && (!Number.isFinite(n) || n < 0 || n > 100)) {
			toast.add({ title: "Nilai harus 0–100", color: "warning" });
			el.value = s.nilai[kode]?.toString() ?? "";
			return;
		}
		s.nilai[kode] = n;
		antre(`n:${s.id}:${kode}`, SQL_NILAI, [s.id, kode, n]);
	}

	function turun(e: KeyboardEvent, kode: string, i: number) {
		e.preventDefault();
		(document.querySelector(`[data-tr="${kode}:${i + 1}"]`) as HTMLInputElement | null)?.focus();
	}

	// Rata-rata nilai rapor semua semester yang tersimpan (mapel sama), sebagai usulan nilai transkrip.
	async function isiOtomatis() {
		if (!await swal.confirm("Isi nilai dari rata-rata rapor?", "Nilai transkrip yang masih kosong diisi rata-rata nilai rapor mapel itu dari semua semester yang tersimpan di aplikasi ini. Nilai yang sudah ada tidak diubah.", "Isi"))
			return;
		try {
			const rows = await db.query<{ siswa: string, kode: string, rata: number }>(
				`SELECT n.peserta_didik_id AS siswa, pr.kode_mapel AS kode, ROUND(AVG(n.nilai), 2) AS rata
				FROM nilai_rapor n JOIN pembelajaran_rapor pr ON pr.id = n.pembelajaran_rapor_id
				WHERE n.nilai IS NOT NULL AND n.peserta_didik_id IN (SELECT peserta_didik_id FROM anggota_rombel WHERE rombongan_belajar_id = ?)
				GROUP BY n.peserta_didik_id, pr.kode_mapel`,
				[rombelId.value]
			);
			const st: { sql: string, params: unknown[] }[] = [];
			for (const r of rows) {
				const s = siswa.value.find(x => x.id === r.siswa);
				if (!s || !mapel.value.some(m => m.kode === r.kode) || (s.nilai[r.kode] ?? null) !== null)
					continue;
				s.nilai[r.kode] = r.rata;
				st.push({ sql: SQL_NILAI, params: [s.id, r.kode, r.rata] });
			}
			if (st.length)
				await db.batch(st);
			toast.add({ title: st.length ? `${st.length} nilai diisi dari rata-rata rapor` : "Tidak ada nilai kosong yang bisa diisi", color: st.length ? "success" : "neutral" });
		}
		catch (e) {
			toast.add({ title: "Gagal mengisi otomatis", description: pesanError(e), color: "error" });
		}
	}

	// ===== Excel =====
	const fileInput = ref<HTMLInputElement>();
	const busy = ref(false);
	async function exportExcel() {
		busy.value = true;
		try {
			const path = await invoke<string>("save_to_downloads", { fileName: `Transkrip ${kelasNama.value}.xlsx`, bytes: Array.from(buildTranskripWorkbook(kelasNama.value, mapel.value, siswa.value)) });
			toast.add({ title: "File tersimpan di Downloads", description: path, color: "success", actions: [{ label: "Buka Folder", onClick: () => { invoke("reveal_file", { path }); } }] });
		}
		catch (e) {
			toast.add({ title: "Gagal membuat file", description: pesanError(e), color: "error" });
		}
		finally {
			busy.value = false;
		}
	}
	async function importExcel(e: Event) {
		const el = e.target as HTMLInputElement;
		const file = el.files?.[0];
		el.value = "";
		if (!file)
			return;
		busy.value = true;
		try {
			const { hasil, ditolak } = readTranskripWorkbook(await file.arrayBuffer(), mapel.value, siswa.value);
			const st: { sql: string, params: unknown[] }[] = [];
			for (const h of hasil) {
				st.push({ sql: SQL_IDENTITAS, params: [h.id, h.noIjazah || null, h.noTranskrip || null, h.tanggalLulus || null] });
				for (const [kode, v] of Object.entries(h.nilai))
					st.push({ sql: SQL_NILAI, params: [h.id, kode, v] });
			}
			if (st.length)
				await db.batch(st);
			await load();
			await swal.fire({
				type: ditolak.length ? "warning" : "success",
				title: "Import selesai",
				text: `${hasil.length} siswa diperbarui.${ditolak.length ? `\n\n${ditolak.length} baris ditolak:\n${ditolak.slice(0, 8).join("\n")}` : ""}`
			});
		}
		catch (err) {
			swal.error("Gagal import", pesanError(err));
		}
		finally {
			busy.value = false;
		}
	}

	// ===== Cetak =====
	const tab = ref<"isi" | "cetak">("isi");
	const tabs = [{ value: "isi", label: "Isi Nilai", icon: "lucide:table" }, { value: "cetak", label: "Cetak", icon: "lucide:printer" }];
	const atur = reactive({ tempat: "", tanggal: "", desimal: 2, rataRata: true, tandaTangan: true, pakaiKop: true });
	const kop = ref<Kop>();
	const gambar = ref<GambarRapor>({});
	const kepsek = ref<{ nama: string, nip: string | null } | null>(null);
	const sekolah = ref<{ nama: string, npsn: string }>({ nama: "", npsn: "" });

	async function muatPengaturan() {
		const s = await db.first<{ nama: string, npsn: string, kecamatan: string | null, kepsek_ptk_id: string | null }>("SELECT nama, npsn, kecamatan, kepsek_ptk_id FROM sekolah LIMIT 1");
		sekolah.value = { nama: s?.nama ?? "", npsn: s?.npsn ?? "" };
		const k = s?.kepsek_ptk_id
			? await db.first<{ nama: string, nip: string | null, gelar_depan: string | null, gelar_belakang: string | null }>("SELECT nama, nip, gelar_depan, gelar_belakang FROM ptk WHERE ptk_id = ?", [s.kepsek_ptk_id])
			: undefined;
		kepsek.value = k ? { nama: namaLengkap(k), nip: k.nip } : null;
		gambar.value = await muatGambarRapor(db);
		atur.tempat = (await db.getSetting("transkrip_tempat")) ?? (await db.getSetting("rapor_tempat")) ?? (s?.kecamatan?.replace(/^kec(amatan)?\.?\s*/i, "") ?? "");
		atur.tanggal = (await db.getSetting("transkrip_tanggal")) ?? new Date().toLocaleDateString("sv-SE");
		atur.desimal = Number((await db.getSetting("transkrip_desimal")) ?? 2);
		atur.rataRata = (await db.getSetting("transkrip_rata")) !== "0";
		atur.tandaTangan = (await db.getSetting("rapor_ttd")) !== "0";
		atur.pakaiKop = (await db.getSetting("transkrip_kop")) !== "0";
		kop.value = await muatKop(db);
	}
	useHead(() => ({ style: [{ key: "transkrip-page", textContent: "@page { size: 215mm 330mm; margin: 0; }" }] }));

	const angka = (v: number | null | undefined) => (v === null || v === undefined ? "-" : v.toFixed(atur.desimal).replace(".", ","));
	const rataSiswa = (s: Siswa) => {
		const xs = mapel.value.map(m => s.nilai[m.kode]).filter((v): v is number => typeof v === "number");
		return xs.length ? xs.reduce((a, b) => a + b, 0) / xs.length : null;
	};
	const belumLengkap = computed(() => siswa.value.filter(s => !s.noIjazah || !s.tanggalLulus || mapel.value.some(m => s.nilai[m.kode] === null || s.nilai[m.kode] === undefined)));

	async function cetak() {
		await flush();
		await Promise.all([
			db.setSetting("transkrip_tempat", atur.tempat.trim()),
			db.setSetting("transkrip_tanggal", atur.tanggal),
			db.setSetting("transkrip_desimal", String(atur.desimal)),
			db.setSetting("transkrip_rata", atur.rataRata ? "1" : "0"),
			db.setSetting("transkrip_kop", atur.pakaiKop ? "1" : "0")
		]);
		await nextTick();
		window.print();
	}
</script>

<template>
	<div v-if="!loading && !kelasList.length" class="flex flex-col items-center gap-3 py-24 text-center text-muted">
		<UIcon name="lucide:scroll-text" class="size-12" />
		<p>{{ semuaKelas ? "Tidak ada kelas 6 di semester ini." : "Transkrip ijazah hanya untuk wali kelas 6." }}</p>
	</div>

	<div v-else class="space-y-4">
		<div class="flex flex-wrap items-center gap-2">
			<UTabs
				v-model="tab"
				:items="tabs"
				:content="false"
				class="w-fit" />
			<USelect
				v-if="kelasList.length > 1"
				v-model="rombelId"
				:items="kelasList"
				icon="lucide:layers"
				aria-label="Kelas"
				class="w-36" />
			<span class="ms-auto text-xs text-muted" aria-live="polite">
				<template v-if="saveState === 'saving'">Menyimpan…</template>
				<template v-else-if="saveState === 'saved'">Tersimpan otomatis</template>
			</span>
		</div>

		<UAlert
			v-if="!genap"
			color="info"
			variant="subtle"
			icon="lucide:info"
			description="Transkrip nilai ijazah biasanya diisi di semester genap untuk siswa kelas 6 yang sudah dinyatakan lulus. Data tetap bisa disiapkan dari sekarang."
		/>
		<UAlert
			v-if="!loading && !mapel.length"
			color="warning"
			variant="subtle"
			icon="lucide:triangle-alert"
			title="Belum ada mapel transkrip"
			description="Aktifkan kolom Transkrip di Data Referensi → Mata Pelajaran."
		/>

		<!-- ===== Isi nilai ===== -->
		<template v-if="tab === 'isi'">
			<div class="flex flex-wrap items-center gap-2">
				<UButton
					size="sm"
					variant="soft"
					icon="lucide:sigma"
					:disabled="!siswa.length"
					@click="isiOtomatis">
					Isi kosong dari rata-rata rapor
				</UButton>
				<UButton
					size="sm"
					variant="subtle"
					icon="lucide:file-down"
					:loading="busy"
					:disabled="!siswa.length"
					@click="exportExcel">
					Export Excel
				</UButton>
				<UButton
					size="sm"
					variant="soft"
					icon="lucide:upload"
					:disabled="busy || !siswa.length"
					@click="fileInput?.click()">
					Import Excel
				</UButton>
				<input
					ref="fileInput"
					type="file"
					accept=".xlsx,.xls"
					class="hidden"
					aria-label="Pilih file Excel transkrip"
					@change="importExcel">
			</div>

			<USkeleton v-if="loading" class="h-80 w-full" />
			<div v-else class="overflow-x-auto rounded-md border border-default">
				<table class="w-full text-sm">
					<thead class="bg-elevated/50 text-xs text-muted">
						<tr>
							<th class="sticky left-0 min-w-48 bg-elevated p-2 text-left">
								Nama
							</th>
							<th class="min-w-44 p-2 text-left">
								Nomor Ijazah
							</th>
							<th class="min-w-36 p-2 text-left">
								Nomor Transkrip
							</th>
							<th class="w-40 p-2 text-left">
								Tanggal Lulus
							</th>
							<th v-for="m in mapel" :key="m.kode" class="w-20 p-1 text-center">
								<UTooltip :text="m.nama">
									<span>{{ m.singkat }}</span>
								</UTooltip>
							</th>
							<th class="w-20 p-2 text-right">
								Rata-rata
							</th>
						</tr>
					</thead>
					<tbody class="divide-y divide-default">
						<tr v-for="(s, i) in siswa" :key="s.id" class="hover:bg-elevated/30">
							<td class="sticky left-0 bg-default p-2">
								<p class="font-medium">
									{{ s.nama }}
								</p>
								<p class="text-xs text-muted">
									{{ s.nisn ?? "–" }}
								</p>
							</td>
							<td class="p-1">
								<UInput
									:model-value="s.noIjazah"
									size="sm"
									:aria-label="`Nomor ijazah ${s.nama}`"
									@update:model-value="setIdentitas(s, 'noIjazah', String($event))" />
							</td>
							<td class="p-1">
								<UInput
									:model-value="s.noTranskrip"
									size="sm"
									:aria-label="`Nomor transkrip ${s.nama}`"
									@update:model-value="setIdentitas(s, 'noTranskrip', String($event))" />
							</td>
							<td class="p-1">
								<UInput
									:model-value="s.tanggalLulus"
									type="date"
									size="sm"
									:aria-label="`Tanggal lulus ${s.nama}`"
									@update:model-value="setIdentitas(s, 'tanggalLulus', String($event))" />
							</td>
							<td v-for="m in mapel" :key="m.kode" class="p-1 text-center">
								<input
									:data-tr="`${m.kode}:${i}`"
									type="text"
									inputmode="decimal"
									:value="s.nilai[m.kode] ?? ''"
									placeholder="–"
									:aria-label="`Nilai ${m.nama} ${s.nama}`"
									class="w-16 rounded border border-default bg-default px-1 py-1 text-center tabular-nums placeholder:text-dimmed focus:border-primary focus:outline-none"
									@change="setNilai(s, m.kode, $event.target as HTMLInputElement)"
									@keydown.enter="turun($event, m.kode, i)"
								>
							</td>
							<td class="p-2 text-right font-semibold tabular-nums">
								{{ angka(rataSiswa(s)) }}
							</td>
						</tr>
					</tbody>
				</table>
				<p v-if="!siswa.length" class="py-8 text-center text-sm text-muted">
					Kelas ini belum punya anggota.
				</p>
			</div>
		</template>

		<!-- ===== Cetak ===== -->
		<div v-else class="grid gap-4 xl:grid-cols-[320px_1fr]">
			<PanelCard title="Pengaturan Cetak" icon="lucide:settings-2">
				<div class="space-y-3">
					<div class="grid grid-cols-2 gap-3">
						<UFormField label="Tempat">
							<UInput v-model="atur.tempat" class="w-full" />
						</UFormField>
						<UFormField label="Tanggal">
							<UInput v-model="atur.tanggal" type="date" class="w-full" />
						</UFormField>
					</div>
					<UFormField label="Angka desimal">
						<USelect v-model="atur.desimal" :items="[{ label: 'Bulat', value: 0 }, { label: '1 desimal', value: 1 }, { label: '2 desimal', value: 2 }]" class="w-full" />
					</UFormField>
					<USwitch v-model="atur.rataRata" label="Tampilkan baris rata-rata" />
					<USwitch v-model="atur.pakaiKop" label="Pakai kop surat" />
					<USwitch v-model="atur.tandaTangan" label="Tampilkan nama, NIP & tanda tangan kepala sekolah" />
					<UButton
						block
						size="lg"
						icon="lucide:printer"
						:disabled="!siswa.length"
						@click="cetak">
						Cetak {{ siswa.length }} Transkrip
					</UButton>
					<p class="text-xs text-muted">
						Kertas F4. Di dialog cetak pilih printer atau Microsoft Print to PDF.
					</p>
					<UAlert
						v-if="belumLengkap.length"
						color="warning"
						variant="subtle"
						icon="lucide:triangle-alert"
						:title="`${belumLengkap.length} siswa belum lengkap`"
						description="Nomor ijazah, tanggal lulus, atau nilai masih kosong."
					/>
				</div>
			</PanelCard>
			<PanelCard title="Pratinjau" icon="lucide:eye">
				<div v-if="siswa[0]" class="overflow-x-auto rounded-md bg-elevated p-4">
					<div class="mx-auto w-fit">
						<TranskripLembar
							:kop="atur.pakaiKop ? kop : undefined"
							:sekolah="sekolah"
							:kepsek="kepsek"
							:gambar="gambar"
							:mapel="mapel"
							:siswa="siswa[0]"
							:atur="atur"
							:angka="angka"
							:rata="rataSiswa(siswa[0])"
						/>
					</div>
				</div>
			</PanelCard>
		</div>
	</div>

	<Teleport to="body">
		<div class="cetak-root">
			<div v-for="s in siswa" :key="s.id" class="cetak-siswa">
				<TranskripLembar
					:kop="atur.pakaiKop ? kop : undefined"
					:sekolah="sekolah"
					:kepsek="kepsek"
					:gambar="gambar"
					:mapel="mapel"
					:siswa="s"
					:atur="atur"
					:angka="angka"
					:rata="rataSiswa(s)" />
			</div>
		</div>
	</Teleport>
</template>
