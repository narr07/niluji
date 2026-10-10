<script setup lang="ts">
	import type { PelengkapKelas, RaporKelas } from "~/composables/useRapor";
	import { invoke } from "@tauri-apps/api/core";

	// Cetak rapor satu kelas (padanan Cetak Nilai → Nilai Rapor). Hasilnya lewat dialog cetak
	// Windows: langsung ke printer, atau "Microsoft Print to PDF" / "Simpan sebagai PDF".
	// semuaKelas = admin (pilih kelas mana saja); selain itu hanya kelas yang diwalikan.
	const props = defineProps<{ semuaKelas?: boolean }>();

	const db = useDb();
	const toast = useToast();
	const { session, waliRombel } = useAuth();
	const { muat, muatPelengkap } = useRapor();

	// Jenis cetak: nilai rapor (Laporan Hasil Belajar) atau pelengkap (sampul + identitas).
	const jenis = ref<"nilai" | "pelengkap">("nilai");
	const jenisItems = [
		{ value: "nilai", label: "Nilai Rapor" },
		{ value: "pelengkap", label: "Pelengkap Rapor" }
	];

	// ===== Pengaturan cetak (diingat) =====
	const KERTAS = {
		A4: { label: "A4 (21 × 29,7 cm)", size: "210mm 297mm", w: 210, h: 297 },
		F4: { label: "F4 / Folio (21,5 × 33 cm)", size: "215mm 330mm", w: 215, h: 330 }
	} as const;
	type Kertas = keyof typeof KERTAS;
	// Margin diatur di dalam halaman rapor sendiri (@page margin 0), jadi browser tidak mencetak tanggal/URL.
	const ukuranKertas = computed(() => ({ w: KERTAS[atur.kertas].w, h: KERTAS[atur.kertas].h }));

	// ===== Zoom pratinjau: pas halaman / pas lebar (transform scale, tidak memengaruhi cetak) =====
	type Zoom = "halaman" | "lebar";
	const zoom = ref<Zoom>("lebar");
	const zoomItems = [
		{ value: "halaman", icon: "lucide:file", label: "Pas halaman" },
		{ value: "lebar", icon: "lucide:move-horizontal", label: "Pas lebar" }
	] as const;
	const wadah = ref<HTMLElement>();
	const isi = ref<HTMLElement>();
	const ukWadah = reactive({ w: 0, h: 0 });
	const ukIsi = reactive({ w: 0, h: 0 });
	let amati: ResizeObserver | undefined;
	watch([wadah, isi], ([w, i]) => {
		amati?.disconnect();
		if (!w || !i)
			return;
		amati = new ResizeObserver(() => {
			Object.assign(ukWadah, { w: w.clientWidth - 32, h: w.clientHeight - 32 }); // dikurangi padding p-4
			Object.assign(ukIsi, { w: i.offsetWidth, h: i.offsetHeight });
		});
		amati.observe(w);
		amati.observe(i);
	});
	onBeforeUnmount(() => amati?.disconnect());
	const skala = computed(() => {
		if (!ukWadah.w)
			return 1;
		const pxMm = 96 / 25.4;
		const lebar = ukWadah.w / (ukuranKertas.value.w * pxMm);
		return Math.min(zoom.value === "lebar" ? lebar : Math.min(lebar, ukWadah.h / (ukuranKertas.value.h * pxMm)), 2);
	});
	const kertasItems = Object.entries(KERTAS).map(([value, k]) => ({ value: value as Kertas, label: k.label }));

	const atur = reactive({ kertas: "A4" as Kertas, tempat: "", tanggal: "", tandaTangan: true, pakaiKop: false });
	const kop = ref<Kop>();
	const kunciTanggal = computed(() => `rapor_tanggal:${session.value?.semesterId}`);

	async function muatPengaturan() {
		const sekolah = await db.first<{ kecamatan: string | null }>("SELECT kecamatan FROM sekolah LIMIT 1");
		atur.kertas = ((await db.getSetting("rapor_kertas")) as Kertas) || "A4";
		atur.tempat = (await db.getSetting("rapor_tempat")) ?? (sekolah?.kecamatan?.replace(/^kec(amatan)?\.?\s*/i, "") ?? "");
		atur.tanggal = (await db.getSetting(kunciTanggal.value)) ?? new Date().toLocaleDateString("sv-SE");
		atur.tandaTangan = (await db.getSetting("rapor_ttd")) !== "0";
		atur.pakaiKop = (await db.getSetting("pelengkap_kop")) === "1";
		kop.value = await muatKop(db);
	}
	async function simpanPengaturan() {
		await Promise.all([
			db.setSetting("rapor_kertas", atur.kertas),
			db.setSetting("rapor_tempat", atur.tempat.trim()),
			db.setSetting(kunciTanggal.value, atur.tanggal),
			db.setSetting("rapor_ttd", atur.tandaTangan ? "1" : "0"),
			db.setSetting("pelengkap_kop", atur.pakaiKop ? "1" : "0")
		]);
	}

	// Ukuran & margin kertas untuk cetak (margin kiri lebih lebar untuk jilid).
	useHead(() => ({
		style: [{ key: "rapor-page", textContent: `@page { size: ${KERTAS[atur.kertas].size}; margin: 0; }` }]
	}));

	// ===== Kelas & data =====
	const kelasList = ref<{ value: string, label: string }[]>([]);
	const rombelId = ref<string>();
	const data = ref<RaporKelas>();
	const pelengkap = ref<PelengkapKelas>();
	const loading = ref(true);
	let permintaan = 0;

	async function muatKelas() {
		const sem = session.value?.semesterId;
		const ids = props.semuaKelas ? null : waliRombel.value.map(r => r.rombongan_belajar_id);
		kelasList.value = !sem || (ids && !ids.length)
			? []
			: await db.query(
				`SELECT rombongan_belajar_id AS value, nama AS label FROM rombel WHERE semester_id = ? AND jenis_rombel = '1'
				${ids ? `AND rombongan_belajar_id IN (${ids.map(() => "?").join(",")})` : ""} ORDER BY CAST(tingkat AS INTEGER), nama`,
				[sem, ...(ids ?? [])]
			);
		if (!kelasList.value.some(k => k.value === rombelId.value))
			rombelId.value = kelasList.value[0]?.value;
	}

	async function muatData() {
		const nomor = ++permintaan;
		const sem = session.value?.semesterId;
		if (!rombelId.value || !sem) {
			data.value = undefined;
			loading.value = false;
			return;
		}
		loading.value = true;
		try {
			const [d, p] = await Promise.all([muat(rombelId.value, sem), muatPelengkap(rombelId.value)]);
			if (nomor !== permintaan)
				return;
			data.value = d;
			pelengkap.value = p;
			pilihSiswa.value = d.siswa[0]?.id ?? "";
		}
		catch (e) {
			toast.add({ title: "Gagal memuat data rapor", description: pesanError(e), color: "error" });
		}
		finally {
			if (nomor === permintaan)
				loading.value = false;
		}
	}

	async function init() {
		loading.value = true;
		try {
			await muatPengaturan();
			await muatKelas();
			await muatData();
		}
		catch (e) {
			toast.add({ title: "Gagal memuat halaman cetak", description: pesanError(e), color: "error" });
			loading.value = false;
		}
	}
	watch(() => session.value?.semesterId, init, { immediate: true });
	watch(rombelId, muatData);

	// ===== Yang dicetak: satu siswa, satu kelas, atau semua kelas (admin) =====
	type Cakupan = "siswa" | "kelas" | "semua";
	const cakupan = ref<Cakupan>("kelas");
	const cakupanItems = computed(() => [
		{ value: "siswa", label: "Satu siswa" },
		{ value: "kelas", label: "Satu kelas" },
		...(kelasList.value.length > 1 ? [{ value: "semua", label: "Semua kelas" }] : [])
	]);
	const pilihSiswa = ref("");
	const siswaItems = computed(() => data.value?.siswa.map(s => ({ value: s.id, label: s.nama })) ?? []);
	const dicetak = computed(() => (data.value?.siswa ?? []).filter(s => cakupan.value !== "siswa" || s.id === pilihSiswa.value));

	// Semua kelas dimuat hanya saat mau dicetak (bisa ratusan siswa).
	const semuaKelasData = ref<{ data: RaporKelas, pelengkap: PelengkapKelas }[]>([]);
	const jumlahSemua = ref(0);
	watch(cakupan, async (c) => {
		if (c === "semua" && !jumlahSemua.value)
			jumlahSemua.value = (await db.scalar<number>(
				"SELECT COUNT(*) FROM anggota_rombel a JOIN rombel r USING (rombongan_belajar_id) WHERE r.semester_id = ? AND r.jenis_rombel = '1'",
				[session.value?.semesterId]
			)) ?? 0;
	});
	const lembar = computed(() => (cakupan.value === "semua"
		? semuaKelasData.value.flatMap(k => k.data.siswa.map(s => ({ key: s.id, data: k.data, pelengkap: k.pelengkap, siswa: s })))
		: data.value && pelengkap.value ? dicetak.value.map(s => ({ key: s.id, data: data.value!, pelengkap: pelengkap.value!, siswa: s })) : []));
	const pelengkapDari = (p: PelengkapKelas, id: string) => p.siswa.find(s => s.id === id);
	const pratinjauId = ref<string>();
	const pratinjau = computed(() => dicetak.value.find(s => s.id === pratinjauId.value) ?? dicetak.value[0]);
	const pratinjauItems = computed(() => dicetak.value.map(s => ({ value: s.id, label: s.nama })));

	const belumLengkap = computed(() => (jenis.value === "nilai" ? dicetak.value.filter(s => s.kurang.length) : []));
	const pelengkapOf = (id: string) => pelengkap.value?.siswa.find(s => s.id === id);
	const labelCetak = computed(() => {
		const n = cakupan.value === "semua" ? jumlahSemua.value : dicetak.value.length;
		const apa = jenis.value === "pelengkap" ? "Pelengkap" : "Rapor";
		return cakupan.value === "siswa" ? `Cetak ${apa}` : `Cetak ${n} ${apa}${cakupan.value === "semua" ? " (semua kelas)" : ""}`;
	});
	const peringatanUmum = computed(() => [
		!atur.tanggal && "Tanggal rapor belum diisi",
		!data.value?.kepsek && "Kepala sekolah belum dipilih (Data Referensi → Sekolah)",
		!data.value?.wali && "Wali kelas belum ada di Dapodik"
	].filter(Boolean) as string[]);

	// ===== Dibagikan ke siswa (padanan "Tampilkan pada siswa" e-Rapor) =====
	const dibagikan = ref(false);
	const kunciBagi = computed(() => `rapor_dibagikan:${session.value?.semesterId}:${rombelId.value}`);
	watch(kunciBagi, async (k) => {
		dibagikan.value = (await db.getSetting(k)) === "1";
	}, { immediate: true });
	async function setDibagikan(v: boolean) {
		dibagikan.value = v;
		try {
			await db.setSetting(kunciBagi.value, v ? "1" : "0");
			toast.add({ title: v ? "Rapor kelas ini bisa dilihat siswa" : "Rapor disembunyikan dari siswa", color: "success" });
		}
		catch (e) {
			dibagikan.value = !v;
			toast.add({ title: "Gagal menyimpan", description: pesanError(e), color: "error" });
		}
	}

	// ===== Leger (Excel) =====
	const membuatLeger = ref(false);
	async function downloadLeger(semua: boolean) {
		const sem = session.value?.semesterId;
		if (!sem || !data.value)
			return;
		membuatLeger.value = true;
		try {
			const daftar = semua ? await Promise.all(kelasList.value.map(k => muat(k.value, sem))) : [data.value];
			const nama = semua ? `Leger Semua Kelas ${semesterLabel(sem).replace("/", "-")}.xlsx` : `Leger ${data.value.kelas.nama} ${semesterLabel(sem).replace("/", "-")}.xlsx`;
			const path = await invoke<string>("save_to_downloads", { fileName: nama, bytes: Array.from(buildLeger(daftar)) });
			toast.add({
				title: "Leger tersimpan di Downloads",
				description: path,
				color: "success",
				actions: [{ label: "Buka Folder", onClick: () => { invoke("reveal_file", { path }); } }]
			});
		}
		catch (e) {
			toast.add({ title: "Gagal membuat leger", description: pesanError(e), color: "error" });
		}
		finally {
			membuatLeger.value = false;
		}
	}
	const legerItems = computed(() => [
		{ label: `Leger ${data.value?.kelas.nama ?? "kelas ini"}`, icon: "lucide:file-spreadsheet", onSelect: () => downloadLeger(false) },
		{ label: `Leger semua kelas (${kelasList.value.length} sheet)`, icon: "lucide:files", onSelect: () => downloadLeger(true) }
	]);

	// ===== Cetak =====
	const mencetak = ref(false);
	async function cetak() {
		if (cakupan.value !== "semua" && !dicetak.value.length)
			return;
		mencetak.value = true;
		try {
			await simpanPengaturan();
			const sem = session.value?.semesterId;
			if (cakupan.value === "semua" && sem) {
				semuaKelasData.value = await Promise.all(kelasList.value.map(async k => ({ data: await muat(k.value, sem), pelengkap: await muatPelengkap(k.value) })));
				jumlahSemua.value = semuaKelasData.value.reduce((n, k) => n + k.data.siswa.length, 0);
			}
			await nextTick();
			window.print();
		}
		catch (e) {
			toast.add({ title: "Gagal membuka dialog cetak", description: pesanError(e), color: "error" });
		}
		finally {
			mencetak.value = false;
		}
	}
</script>

<template>
	<div v-if="!loading && !kelasList.length" class="flex flex-col items-center gap-3 py-24 text-center text-muted">
		<UIcon name="lucide:printer" class="size-12" />
		<p>{{ semuaKelas ? "Belum ada data kelas di semester ini." : "Anda bukan wali kelas di semester ini." }}</p>
	</div>

	<!-- Layar lebar: kolom kiri & pratinjau setinggi layar, masing-masing scroll sendiri; tombol aksi selalu terlihat. -->
	<div v-else class="grid gap-4 xl:h-[calc(100dvh-5.5rem)] xl:grid-cols-[340px_1fr]">
		<div class="flex min-h-0 flex-col gap-3">
			<div class="min-h-0 flex-1 space-y-4 overflow-y-auto pe-1">
				<PanelCard title="Pengaturan Cetak" icon="lucide:settings-2">
					<div class="space-y-3">
						<UFormField v-if="kelasList.length > 1" label="Kelas">
							<USelect
								v-model="rombelId"
								:items="kelasList"
								icon="lucide:layers"
								class="w-full" />
						</UFormField>
						<UFormField label="Jenis cetak">
							<UTabs
								v-model="jenis"
								:items="jenisItems"
								:content="false"
								size="sm"
								class="w-full" />
						</UFormField>
						<UFormField label="Yang dicetak">
							<UTabs
								v-model="cakupan"
								:items="cakupanItems"
								:content="false"
								size="sm"
								class="w-full" />
						</UFormField>
						<UFormField v-if="cakupan === 'siswa'" label="Siswa">
							<USelectMenu
								v-model="pilihSiswa"
								:items="siswaItems"
								value-key="value"
								icon="lucide:user-round"
								class="w-full" />
						</UFormField>
						<UFormField label="Ukuran kertas">
							<USelect
								v-model="atur.kertas"
								:items="kertasItems"
								icon="lucide:file"
								class="w-full" />
						</UFormField>
						<div class="grid grid-cols-2 gap-3">
							<UFormField label="Tempat">
								<UInput v-model="atur.tempat" placeholder="mis. Cibiru" class="w-full" />
							</UFormField>
							<UFormField label="Tanggal rapor" :error="!atur.tanggal ? 'Wajib diisi' : undefined">
								<UInput v-model="atur.tanggal" type="date" class="w-full" />
							</UFormField>
						</div>
						<USwitch v-model="atur.tandaTangan" label="Tampilkan nama & NIP kepala sekolah dan wali kelas" />
						<USwitch v-if="jenis === 'pelengkap'" v-model="atur.pakaiKop" label="Pakai kop surat di halaman identitas sekolah" />
						<USwitch
							:model-value="dibagikan"
							label="Siswa kelas ini boleh melihat & mengunduh rapornya"
							@update:model-value="setDibagikan(!!$event)"
						/>
					</div>
				</PanelCard>

				<UAlert
					v-if="peringatanUmum.length"
					color="warning"
					variant="subtle"
					icon="lucide:triangle-alert"
					title="Perlu dilengkapi"
					:description="peringatanUmum.join(' · ')"
				/>

				<PanelCard
					v-if="data && jenis === 'nilai'"
					title="Kelengkapan Data"
					icon="lucide:list-checks"
					:description="belumLengkap.length ? `${belumLengkap.length} dari ${dicetak.length} siswa belum lengkap. Rapor tetap bisa dicetak.` : 'Semua data siswa lengkap.'"
				>
					<ul v-if="belumLengkap.length" class="space-y-2 text-sm">
						<li v-for="s in belumLengkap" :key="s.id">
							<button type="button" class="cursor-pointer text-left font-medium hover:text-primary" @click="pratinjauId = s.id">
								{{ s.nama }}
							</button>
							<p class="text-xs text-muted">
								Belum: {{ s.kurang.slice(0, 4).join(", ") }}{{ s.kurang.length > 4 ? `, +${s.kurang.length - 4} lagi` : "" }}
							</p>
						</li>
					</ul>
					<div v-else class="flex items-center gap-2 text-sm text-success">
						<UIcon name="lucide:circle-check" class="size-4" /> Siap dicetak
					</div>
				</PanelCard>
			</div>

			<!-- Tombol aksi: selalu terlihat -->
			<div class="shrink-0 space-y-2 rounded-lg border border-default bg-default p-3">
				<UButton
					block
					size="lg"
					icon="lucide:printer"
					:loading="mencetak || loading"
					:disabled="cakupan !== 'semua' && !dicetak.length"
					@click="cetak"
				>
					{{ labelCetak }}
				</UButton>
				<UDropdownMenu v-if="kelasList.length > 1" :items="legerItems">
					<UButton
						block
						variant="soft"
						icon="lucide:file-spreadsheet"
						trailing-icon="lucide:chevron-down"
						:loading="membuatLeger"
						:disabled="!data">
						Download Leger (Excel)
					</UButton>
				</UDropdownMenu>
				<UButton
					v-else
					block
					variant="soft"
					icon="lucide:file-spreadsheet"
					:loading="membuatLeger"
					:disabled="!data"
					@click="downloadLeger(false)">
					Download Leger (Excel)
				</UButton>
				<UPopover>
					<UButton
						block
						color="neutral"
						variant="link"
						size="xs"
						icon="lucide:info">
						Tips dialog cetak
					</UButton>
					<template #content>
						<p class="max-w-xs p-3 text-xs text-muted">
							Pilih printer untuk mencetak, atau <b>Microsoft Print to PDF</b> untuk satu file PDF.
							Pastikan kertas {{ atur.kertas }}, skala 100%, margin "Default" atau "Tidak ada", dan "Header dan footer" tidak dicentang.
						</p>
					</template>
				</UPopover>
			</div>
		</div>

		<!-- Kanan: judul pratinjau tetap, hanya lembar yang scroll -->
		<PanelCard
			title="Pratinjau"
			icon="lucide:eye"
			class="min-h-0 xl:h-full"
			:ui="{ root: 'flex h-full flex-col', body: 'min-h-0 flex-1' }">
			<template #actions>
				<UFieldGroup size="sm">
					<UTooltip v-for="z in zoomItems" :key="z.value" :text="z.label">
						<UButton
							:icon="z.icon"
							:color="zoom === z.value ? 'primary' : 'neutral'"
							:variant="zoom === z.value ? 'soft' : 'outline'"
							:aria-label="z.label"
							:aria-pressed="zoom === z.value"
							@click="zoom = z.value"
						/>
					</UTooltip>
				</UFieldGroup>
				<USelectMenu
					v-if="pratinjauItems.length > 1"
					:model-value="pratinjau?.id"
					:items="pratinjauItems"
					value-key="value"
					aria-label="Siswa yang dipratinjau"
					class="w-56"
					@update:model-value="pratinjauId = $event as string"
				/>
			</template>
			<USkeleton v-if="loading" class="h-[60vh] w-full" />
			<div v-else-if="data && pratinjau" ref="wadah" class="h-full overflow-auto rounded-md bg-elevated p-4">
				<!-- Ukuran wadah = ukuran isi × skala, supaya area scroll pas dengan halaman yang diperkecil. -->
				<div class="mx-auto" :style="{ width: `${ukIsi.w * skala}px`, height: `${ukIsi.h * skala}px` }">
					<div ref="isi" class="w-max origin-top-left" :style="{ transform: `scale(${skala})` }">
						<RaporLembar
							v-if="jenis === 'nilai'"
							:data="data"
							:siswa="pratinjau"
							:tempat="atur.tempat"
							:tanggal="atur.tanggal"
							:tanda-tangan="atur.tandaTangan"
							:kertas="ukuranKertas"
						/>
						<RaporPelengkap
							v-else-if="pelengkap && pelengkapOf(pratinjau.id)"
							:kop="atur.pakaiKop ? kop : undefined"
							:data="pelengkap"
							:siswa="pelengkapOf(pratinjau.id)!"
							:tempat="atur.tempat"
							:tanggal="atur.tanggal"
							:tanda-tangan="atur.tandaTangan"
							:kertas="ukuranKertas"
							:kaki="data.kelas.nama"
						/>
					</div>
				</div>
			</div>
			<p v-else class="py-12 text-center text-sm text-muted">
				Kelas ini belum punya siswa.
			</p>
		</PanelCard>
	</div>

	<!-- Isi yang benar-benar dicetak: hanya tampil saat mencetak, satu rapor per halaman baru. -->
	<Teleport to="body">
		<div class="cetak-root">
			<div v-for="l in lembar" :key="l.key" class="cetak-siswa">
				<RaporLembar
					v-if="jenis === 'nilai'"
					:data="l.data"
					:siswa="l.siswa"
					:tempat="atur.tempat"
					:tanggal="atur.tanggal"
					:tanda-tangan="atur.tandaTangan"
					:kertas="ukuranKertas"
				/>
				<RaporPelengkap
					v-else-if="pelengkapDari(l.pelengkap, l.siswa.id)"
					:kop="atur.pakaiKop ? kop : undefined"
					:data="l.pelengkap"
					:siswa="pelengkapDari(l.pelengkap, l.siswa.id)!"
					:tempat="atur.tempat"
					:tanggal="atur.tanggal"
					:tanda-tangan="atur.tandaTangan"
					:kertas="ukuranKertas"
					:kaki="l.data.kelas.nama"
				/>
			</div>
		</div>
	</Teleport>
</template>
