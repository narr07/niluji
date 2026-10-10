<script setup lang="ts">
	import * as XLSX from "xlsx";
	import { invoke } from "@tauri-apps/api/core";
	import type { DapodikMatev } from "~/composables/useKirimDapodik";

	// Fitur Kirim ke Dapodik (Tes Koneksi → Kirim Matev → Kirim Nilai).
	// Menggunakan REST API Web Service Dapodik (postMatevRapor & postNilai).

	interface Matev {
		rombel: string
		rombelId: string
		kode: string
		mapel: string
		siswa: number
		dinilai: number
		kodeValid: boolean
		terkunci: number
	}

	const db = useDb();
	const swal = useSwal();
	const toast = useToast();
	const { session } = useAuth();
	const { testKoneksi } = useDapodikSync();
	const { ambilMatevDapodik, kirimMatevKelas, kirimMatevSemua, kirimNilaiKelas, kirimNilaiSemua } = useKirimDapodik();

	const koneksi = ref<{ ok: boolean, pesan: string } | null>(null);
	const menguji = ref(false);
	const dapodikMatev = ref<DapodikMatev[]>([]);
	const loadingMatev = ref(false);

	async function muatMatevDapodik() {
		loadingMatev.value = true;
		try {
			dapodikMatev.value = await ambilMatevDapodik();
		}
		catch {
			dapodikMatev.value = [];
		}
		finally {
			loadingMatev.value = false;
		}
	}

	async function tes() {
		menguji.value = true;
		try {
			const pesan = await testKoneksi();
			koneksi.value = { ok: true, pesan };
			await muatMatevDapodik();
			toast.add({ title: "Terhubung ke Dapodik", description: pesan, color: "success" });
		}
		catch (e) {
			koneksi.value = { ok: false, pesan: pesanError(e) };
			toast.add({ title: "Gagal terhubung ke Dapodik", description: pesanError(e), color: "error" });
		}
		finally {
			menguji.value = false;
		}
	}

	const matev = ref<Matev[]>([]);
	const refMapel = ref(0);
	const loading = ref(true);

	async function load() {
		loading.value = true;
		try {
			const sem = session.value?.semesterId;
			refMapel.value = await db.scalar<number>("SELECT COUNT(*) FROM mata_pelajaran") ?? 0;
			matev.value = await db.query<Matev>(
				`SELECT r.nama AS rombel, r.rombongan_belajar_id AS rombelId, m.kode, m.nama AS mapel, pr.terkunci,
					(SELECT COUNT(*) FROM anggota_rombel a WHERE a.rombongan_belajar_id = pr.rombongan_belajar_id) AS siswa,
					(SELECT COUNT(*) FROM nilai_rapor n WHERE n.pembelajaran_rapor_id = pr.id AND n.nilai IS NOT NULL) AS dinilai,
					EXISTS (SELECT 1 FROM mata_pelajaran x WHERE x.mata_pelajaran_id = m.kode) AND m.kode <> ? AS kodeValid
				FROM pembelajaran_rapor pr JOIN rombel r USING (rombongan_belajar_id) JOIN mapel_rapor m ON m.kode = pr.kode_mapel
				WHERE pr.semester_id = ? ORDER BY CAST(r.tingkat AS INTEGER), r.nama, m.urutan`,
				[KODE_GURU_KELAS, sem]
			);

			// Muat data matev Dapodik secara otomatis
			await muatMatevDapodik();
		}
		catch (e) {
			toast.add({ title: "Gagal memuat data kirim", description: pesanError(e), color: "error" });
		}
		finally {
			loading.value = false;
		}
	}
	watch(() => session.value?.semesterId, load, { immediate: true });

	const perKelas = computed(() => {
		const map = new Map<string, { id: string, nama: string, items: Matev[], matevAdaCount: number }>();
		for (const m of matev.value) {
			if (!map.has(m.rombelId))
				map.set(m.rombelId, { id: m.rombelId, nama: m.rombel, items: [], matevAdaCount: 0 });
			const k = map.get(m.rombelId)!;
			k.items.push(m);
		}

		// Hitung jumlah matev yang sudah ada di Dapodik untuk kelas ini
		for (const k of map.values()) {
			k.matevAdaCount = k.items.filter(m =>
				dapodikMatev.value.some(dm =>
					dm.rombongan_belajar_id === k.id && (
						String(dm.mata_pelajaran_id).trim() === String(m.kode).trim()
						|| dm.nm_mata_evaluasi.trim().toLowerCase() === m.mapel.trim().toLowerCase()
					)
				)
			).length;
		}

		return [...map.values()];
	});

	function cekMatevAda(rombelId: string, kodeMapel: string, mapelNama?: string) {
		return dapodikMatev.value.some(dm =>
			dm.rombongan_belajar_id === rombelId && (
				String(dm.mata_pelajaran_id).trim() === String(kodeMapel).trim()
				|| (mapelNama && dm.nm_mata_evaluasi.trim().toLowerCase() === mapelNama.trim().toLowerCase())
			)
		);
	}

	const refKurang = computed(() => refMapel.value < 100);
	const ringkas = computed(() => ({
		matev: matev.value.length,
		valid: matev.value.filter(m => m.kodeValid).length,
		lengkap: matev.value.filter(m => m.siswa && m.dinilai >= m.siswa).length,
		matevDapodik: dapodikMatev.value.length,
		semuaMatevSelesai: dapodikMatev.value.length > 0 && perKelas.value.every(k => k.matevAdaCount >= k.items.length)
	}));

	// State Pelacakan Progress & Loading Spesifik
	const modalOpen = ref(false);
	const modalJudul = ref("");
	const modalStatus = ref("");
	const modalCurrent = ref(0);
	const modalTotal = ref(0);
	const sedangBerjalan = ref(false);
	const logs = ref<string[]>([]);

	// Loading spesifik per aksi agar tombol lain tidak ikut berputar
	const sedangBerjalanSemuaMatev = ref(false);
	const sedangBerjalanSemuaNilai = ref(false);
	const rombelSedangProsesMatev = ref<string | null>(null);
	const rombelSedangProsesNilai = ref<string | null>(null);

	function tambahLog(msg: string) {
		logs.value.push(msg);
		modalStatus.value = msg;
	}

	// ===== Aksi Kirim Matev =====
	async function kirimMatevSemuaClick() {
		const confirm = await swal.confirm(
			"Kirim Semua Mata Evaluasi?",
			"Mata evaluasi yang belum ada di Dapodik untuk seluruh kelas akan didaftarkan ke Web Service Dapodik."
		);
		if (!confirm) return;

		modalJudul.value = "Mendaftarkan Mata Evaluasi ke Dapodik";
		modalStatus.value = "Mempersiapkan pengiriman...";
		modalCurrent.value = 0;
		modalTotal.value = perKelas.value.length;
		logs.value = [];
		modalOpen.value = true;
		sedangBerjalan.value = true;
		sedangBerjalanSemuaMatev.value = true;

		try {
			const res = await kirimMatevSemua((msg, cur, tot) => {
				modalCurrent.value = cur;
				modalTotal.value = tot;
				tambahLog(msg);
			});

			await muatMatevDapodik();
			tambahLog("Selesai!");

			// Segera hentikan status loading & tutup modal progress agar tidak gantung/berputar terus
			sedangBerjalan.value = false;
			sedangBerjalanSemuaMatev.value = false;
			modalOpen.value = false;

			let summary = `${res.berhasilDibuat} mata evaluasi baru berhasil didaftarkan.\n${res.sudahAda} mata evaluasi sudah ada sebelumnya.`;
			if (res.gagal > 0) summary += `\n${res.gagal} mata evaluasi gagal.`;
			if (res.warnings.length) summary += `\n\nCatatan:\n${res.warnings.join("\n")}`;

			await swal.success("Pengiriman Matev Selesai", summary);
		}
		catch (e) {
			sedangBerjalan.value = false;
			sedangBerjalanSemuaMatev.value = false;
			modalOpen.value = false;
			await swal.error("Pengiriman Matev Gagal", pesanError(e));
		}
		finally {
			sedangBerjalan.value = false;
			sedangBerjalanSemuaMatev.value = false;
		}
	}

	async function kirimMatevKelasClick(rombelId: string, rombelNama: string, sudahLengkap: boolean) {
		if (sudahLengkap) {
			toast.add({
				title: "Mata Evaluasi Sudah Lengkap",
				description: `Seluruh mata evaluasi ${rombelNama} sudah terdaftar di Dapodik.`,
				color: "info"
			});
			return;
		}

		const confirm = await swal.confirm(
			`Kirim Matev ${rombelNama}?`,
			`Mata evaluasi ${rombelNama} yang belum ada di Dapodik akan didaftarkan.`
		);
		if (!confirm) return;

		modalJudul.value = `Mendaftarkan Matev ${rombelNama}`;
		modalStatus.value = "Mempersiapkan pengiriman...";
		modalCurrent.value = 0;
		modalTotal.value = 0;
		logs.value = [];
		modalOpen.value = true;
		sedangBerjalan.value = true;
		rombelSedangProsesMatev.value = rombelId;

		try {
			const res = await kirimMatevKelas(rombelId, msg => {
				tambahLog(msg);
			});

			await muatMatevDapodik();
			tambahLog("Selesai!");

			// Hentikan status loading & tutup modal
			sedangBerjalan.value = false;
			rombelSedangProsesMatev.value = null;
			modalOpen.value = false;

			let summary = `${res.berhasilDibuat} mata evaluasi baru berhasil didaftarkan.\n${res.sudahAda} mata evaluasi sudah ada sebelumnya.`;
			if (res.gagal > 0) summary += `\n${res.gagal} gagal didaftarkan.`;
			if (res.warnings.length) summary += `\n\nCatatan:\n${res.warnings.join("\n")}`;

			await swal.success(`Matev ${rombelNama} Selesai`, summary);
		}
		catch (e) {
			sedangBerjalan.value = false;
			rombelSedangProsesMatev.value = null;
			modalOpen.value = false;
			await swal.error("Pengiriman Matev Gagal", pesanError(e));
		}
		finally {
			sedangBerjalan.value = false;
			rombelSedangProsesMatev.value = null;
		}
	}

	// ===== Aksi Kirim Nilai =====
	async function kirimNilaiSemuaClick() {
		const confirm = await swal.confirm(
			"Kirim Seluruh Nilai ke Dapodik?",
			"Nilai rapor akhir dan deskripsi capaian kompetensi untuk seluruh kelas akan dikirim ke Dapodik."
		);
		if (!confirm) return;

		modalJudul.value = "Mengirim Nilai Rapor ke Dapodik";
		modalStatus.value = "Mempersiapkan pengiriman nilai...";
		modalCurrent.value = 0;
		modalTotal.value = 0;
		logs.value = [];
		modalOpen.value = true;
		sedangBerjalan.value = true;
		sedangBerjalanSemuaNilai.value = true;

		try {
			const res = await kirimNilaiSemua((msg, cur, tot) => {
				modalCurrent.value = cur;
				modalTotal.value = tot;
				tambahLog(msg);
			});

			tambahLog("Selesai!");

			// Hentikan status loading & tutup modal
			sedangBerjalan.value = false;
			sedangBerjalanSemuaNilai.value = false;
			modalOpen.value = false;

			let summary = `${res.terkirimBaru} nilai baru berhasil disimpan ke Dapodik.\n${res.diperbarui} nilai berhasil diperbarui.\n${res.sama} nilai sudah sama (dilewati).`;
			if (res.gagal > 0) summary += `\n${res.gagal} nilai gagal dikirim.`;
			if (res.warnings.length) summary += `\n\nCatatan:\n${res.warnings.join("\n")}`;

			await swal.success("Pengiriman Nilai Selesai", summary);
		}
		catch (e) {
			sedangBerjalan.value = false;
			sedangBerjalanSemuaNilai.value = false;
			modalOpen.value = false;
			await swal.error("Pengiriman Nilai Gagal", pesanError(e));
		}
		finally {
			sedangBerjalan.value = false;
			sedangBerjalanSemuaNilai.value = false;
		}
	}

	async function kirimNilaiKelasClick(rombelId: string, rombelNama: string) {
		const confirm = await swal.confirm(
			`Kirim Nilai ${rombelNama}?`,
			`Nilai rapor dan deskripsi capaian siswa ${rombelNama} akan dikirim ke Dapodik.`
		);
		if (!confirm) return;

		modalJudul.value = `Mengirim Nilai ${rombelNama}`;
		modalStatus.value = "Mempersiapkan pengiriman...";
		modalCurrent.value = 0;
		modalTotal.value = 0;
		logs.value = [];
		modalOpen.value = true;
		sedangBerjalan.value = true;
		rombelSedangProsesNilai.value = rombelId;

		try {
			const res = await kirimNilaiKelas(rombelId, (msg, cur, tot) => {
				modalCurrent.value = cur;
				modalTotal.value = tot;
				tambahLog(msg);
			});

			tambahLog("Selesai!");

			// Hentikan status loading & tutup modal
			sedangBerjalan.value = false;
			rombelSedangProsesNilai.value = null;
			modalOpen.value = false;

			let summary = `${res.terkirimBaru} nilai baru tersimpan.\n${res.diperbarui} nilai diperbarui.\n${res.sama} nilai sudah sama (dilewati).`;
			if (res.gagal > 0) summary += `\n${res.gagal} nilai gagal dikirim.`;
			if (res.warnings.length) summary += `\n\nCatatan:\n${res.warnings.join("\n")}`;

			await swal.success(`Nilai ${rombelNama} Selesai`, summary);
		}
		catch (e) {
			sedangBerjalan.value = false;
			rombelSedangProsesNilai.value = null;
			modalOpen.value = false;
			await swal.error("Pengiriman Nilai Gagal", pesanError(e));
		}
		finally {
			sedangBerjalan.value = false;
			rombelSedangProsesNilai.value = null;
		}
	}

	// Rekap nilai per kelas (NISN, nama, nilai tiap mapel + kode Dapodik) untuk input/cek di Dapodik.
	const membuat = ref(false);
	async function rekapExcel() {
		membuat.value = true;
		try {
			const sem = session.value?.semesterId;
			const rows = await db.query<{ rombelId: string, kode: string, nisn: string | null, nama: string, nilai: number | null }>(
				`SELECT pr.rombongan_belajar_id AS rombelId, pr.kode_mapel AS kode, s.nisn, s.nama, n.nilai
				FROM pembelajaran_rapor pr JOIN anggota_rombel a ON a.rombongan_belajar_id = pr.rombongan_belajar_id
				JOIN siswa_rapor s ON s.peserta_didik_id = a.peserta_didik_id
				LEFT JOIN nilai_rapor n ON n.pembelajaran_rapor_id = pr.id AND n.peserta_didik_id = a.peserta_didik_id
				WHERE pr.semester_id = ? ORDER BY s.nama COLLATE NOCASE`,
				[sem]
			);
			const wb = XLSX.utils.book_new();
			for (const k of perKelas.value) {
				const id = k.id;
				const siswa = [...new Map(rows.filter(r => r.rombelId === id).map(r => [r.nisn ?? r.nama, r])).values()];
				const header = ["No", "NISN", "Nama", ...k.items.map(m => `${m.mapel} (${m.kode})`)];
				const data = siswa.map((s, i) => [i + 1, s.nisn ?? "", s.nama, ...k.items.map(m => rows.find(r => r.rombelId === id && r.kode === m.kode && (r.nisn ?? r.nama) === (s.nisn ?? s.nama))?.nilai ?? "")]);
				const ws = XLSX.utils.aoa_to_sheet([header, ...data]);
				ws["!cols"] = [{ wch: 4 }, { wch: 12 }, { wch: 32 }, ...k.items.map(() => ({ wch: 14 }))];
				XLSX.utils.book_append_sheet(wb, ws, sheetName(k.nama));
			}
			const bytes = new Uint8Array(XLSX.write(wb, { type: "array", bookType: "xlsx" }));
			const path = await invoke<string>("save_to_downloads", { fileName: `Rekap Nilai Dapodik ${semesterLabel(sem).replace("/", "-")}.xlsx`, bytes: Array.from(bytes) });
			toast.add({ title: "Rekap tersimpan di Downloads", description: path, color: "success", actions: [{ label: "Buka Folder", onClick: () => { invoke("reveal_file", { path }); } }] });
		}
		catch (e) {
			toast.add({ title: "Gagal membuat rekap", description: pesanError(e), color: "error" });
		}
		finally {
			membuat.value = false;
		}
	}
</script>

<template>
	<ErPage id="kirim-nilai" title="Kirim ke Dapodik">
		<div class="space-y-4">
			<!-- Header Action Cards -->
			<div class="grid gap-4 lg:grid-cols-3">
				<!-- 1. Tes Koneksi -->
				<PanelCard title="1. Tes Koneksi" icon="lucide:plug" description="Hubungkan dengan Web Service Dapodik lokal">
					<div class="flex flex-col gap-2">
						<div class="flex items-center gap-2">
							<UButton
								icon="lucide:plug-zap"
								variant="soft"
								:loading="menguji"
								@click="tes">
								Tes Koneksi
							</UButton>
							<UButton
								v-if="koneksi?.ok"
								icon="lucide:refresh-cw"
								variant="ghost"
								size="sm"
								:loading="loadingMatev"
								@click="muatMatevDapodik">
								Refresh Matev
							</UButton>
						</div>
						<p v-if="koneksi" class="flex items-center gap-1.5 text-sm" :class="koneksi.ok ? 'text-success' : 'text-error'">
							<UIcon :name="koneksi.ok ? 'lucide:circle-check' : 'lucide:circle-x'" class="size-4 shrink-0" />
							<span>{{ koneksi.pesan }}</span>
						</p>
					</div>
				</PanelCard>

				<!-- 2. Kirim Matev -->
				<PanelCard
					title="2. Kirim Matev"
					icon="lucide:list-tree"
					:description="`${ringkas.matevDapodik} matev aktif di Dapodik · ${ringkas.valid}/${ringkas.matev} mapel valid`"
				>
					<div class="flex flex-col gap-2">
						<div v-if="ringkas.semuaMatevSelesai" class="flex items-center gap-2">
							<UBadge
								color="success"
								variant="subtle"
								size="md"
								class="gap-1.5">
								<UIcon name="lucide:circle-check" class="size-4" />
								Mata Evaluasi Semua Kelas Lengkap
							</UBadge>
							<UButton
								size="xs"
								variant="ghost"
								icon="lucide:refresh-cw"
								:loading="sedangBerjalanSemuaMatev"
								title="Kirim Ulang Matev"
								@click="kirimMatevSemuaClick"
							>
								Kirim Ulang
							</UButton>
						</div>
						<UButton
							v-else
							icon="lucide:upload"
							variant="soft"
							:loading="sedangBerjalanSemuaMatev"
							@click="kirimMatevSemuaClick"
						>
							Kirim Semua Matev
						</UButton>
						<p class="text-xs text-muted">
							Mendaftarkan mapel sebagai mata evaluasi rapor di rombel Dapodik.
						</p>
					</div>
				</PanelCard>

				<!-- 3. Kirim Nilai -->
				<PanelCard
					title="3. Kirim Nilai"
					icon="lucide:send"
					:description="`${ringkas.lengkap}/${ringkas.matev} mapel nilainya lengkap`"
				>
					<div class="flex flex-col gap-2">
						<div class="flex flex-wrap gap-2">
							<UButton
								icon="lucide:send"
								color="primary"
								:loading="sedangBerjalanSemuaNilai"
								@click="kirimNilaiSemuaClick"
							>
								Kirim Semua Nilai
							</UButton>
							<UButton
								icon="lucide:file-spreadsheet"
								variant="soft"
								:loading="membuat"
								:disabled="!matev.length"
								@click="rekapExcel"
							>
								Rekap Excel
							</UButton>
						</div>
						<p class="text-xs text-muted">
							Mengirim nilai akhir dan capaian kompetensi siswa ke Dapodik.
						</p>
					</div>
				</PanelCard>
			</div>

			<!-- Status Info Banner -->
			<UAlert
				v-if="koneksi?.ok"
				color="success"
				variant="subtle"
				icon="lucide:circle-check"
				title="Web Service Dapodik Siap"
				description="Tombol Kirim Matev dan Kirim Nilai siap digunakan. Anda dapat mengirim per kelas atau langsung semua kelas sekaligus."
			/>
			<UAlert
				v-else
				color="info"
				variant="subtle"
				icon="lucide:info"
				title="Petunjuk Kirim ke Dapodik"
				description="1. Klik 'Tes Koneksi' untuk memastikan terhubung dengan Web Service Dapodik. 2. Klik 'Kirim Matev' untuk mendaftarkan mata pelajaran ke Dapodik. 3. Klik 'Kirim Nilai' untuk mengirim nilai rapor dan capaian siswa."
			/>

			<UAlert
				v-if="refKurang"
				color="warning"
				variant="subtle"
				icon="lucide:refresh-cw"
				title="Referensi mapel Dapodik belum lengkap"
				description="Jalankan Sinkron Dapodik sekali lagi supaya referensi mata pelajaran nasional ikut tersimpan. Setelah itu validasi kode mapel jadi akurat."
				:actions="[{ label: 'Sinkron Dapodik', to: '/dapodik', icon: 'lucide:arrow-right', variant: 'soft' }]"
			/>

			<!-- Classes List -->
			<USkeleton v-if="loading" class="h-64 w-full" />
			<div v-else class="grid gap-4 xl:grid-cols-2">
				<PanelCard
					v-for="k in perKelas"
					:key="k.id"
					:title="k.nama"
					icon="lucide:layers"
				>
					<template #actions>
						<div class="flex items-center gap-1.5">
							<!-- Status / Tombol Kirim Matev per Kelas -->
							<UBadge
								v-if="k.matevAdaCount >= k.items.length"
								color="success"
								variant="subtle"
								size="sm"
								class="gap-1 font-normal cursor-pointer"
								title="Semua matev sudah terdaftar di Dapodik (klik jika ingin kirim ulang)"
								@click="kirimMatevKelasClick(k.id, k.nama, false)"
							>
								<UIcon name="lucide:check-circle" class="size-3.5" />
								Matev Terdaftar
							</UBadge>
							<UButton
								v-else
								size="xs"
								variant="ghost"
								icon="lucide:upload"
								:loading="rombelSedangProsesMatev === k.id || sedangBerjalanSemuaMatev"
								title="Kirim Matev Kelas Ini"
								@click="kirimMatevKelasClick(k.id, k.nama, false)"
							>
								Kirim Matev
							</UButton>

							<!-- Tombol Kirim Nilai per Kelas -->
							<UButton
								size="xs"
								variant="soft"
								color="primary"
								icon="lucide:send"
								:loading="rombelSedangProsesNilai === k.id || sedangBerjalanSemuaNilai"
								title="Kirim Nilai Kelas Ini"
								@click="kirimNilaiKelasClick(k.id, k.nama)"
							>
								Kirim Nilai
							</UButton>
						</div>
					</template>

					<div class="mb-2 flex items-center justify-between text-xs text-muted">
						<span>{{ k.matevAdaCount }}/{{ k.items.length }} matev terdaftar di Dapodik</span>
						<span>{{ k.items.filter(m => m.siswa && m.dinilai >= m.siswa).length }}/{{ k.items.length }} mapel lengkap</span>
					</div>

					<ul class="divide-y divide-default text-sm">
						<li v-for="m in k.items" :key="m.kode" class="flex items-center gap-2 py-2">
							<!-- Status Matev di Dapodik -->
							<UTooltip :text="cekMatevAda(k.id, m.kode, m.mapel) ? 'Mata evaluasi terdaftar di Dapodik' : 'Belum terdaftar di Dapodik (klik Kirim Matev)'">
								<UIcon
									:name="cekMatevAda(k.id, m.kode, m.mapel) ? 'lucide:circle-check' : 'lucide:circle-dashed'"
									class="size-4 shrink-0"
									:class="cekMatevAda(k.id, m.kode, m.mapel) ? 'text-success' : 'text-neutral-400'"
								/>
							</UTooltip>

							<span class="min-w-0 flex-1 truncate">{{ m.mapel }}</span>
							<span class="font-mono text-xs text-muted">{{ m.kode }}</span>

							<UBadge
								:color="m.siswa && m.dinilai >= m.siswa ? 'success' : 'warning'"
								variant="subtle"
								size="sm"
								class="tabular-nums"
							>
								{{ m.dinilai }}/{{ m.siswa }} dinilai
							</UBadge>
						</li>
					</ul>
				</PanelCard>
			</div>
		</div>

		<!-- Modal Progress Kirim -->
		<UModal v-model:open="modalOpen" :prevent-close="sedangBerjalan">
			<template #content>
				<div class="p-6 space-y-4">
					<div class="flex items-center gap-3">
						<UIcon
							:name="sedangBerjalan ? 'lucide:loader-2' : 'lucide:check-circle-2'"
							class="size-6 text-primary"
							:class="{ 'animate-spin': sedangBerjalan }"
						/>
						<div>
							<h3 class="font-semibold text-base">{{ modalJudul }}</h3>
							<p class="text-sm text-muted">{{ modalStatus }}</p>
						</div>
					</div>

					<div v-if="modalTotal > 0" class="space-y-1.5">
						<div class="flex justify-between text-xs text-muted">
							<span>Kemajuan</span>
							<span>{{ modalCurrent }} / {{ modalTotal }}</span>
						</div>
						<UProgress :model-value="modalCurrent" :max="modalTotal" size="sm" />
					</div>

					<div v-if="logs.length" class="max-h-40 overflow-y-auto rounded bg-neutral-100 p-2 text-xs font-mono dark:bg-neutral-800 space-y-1">
						<div v-for="(log, i) in logs" :key="i" class="text-muted">
							{{ log }}
						</div>
					</div>

					<div class="flex justify-end pt-2">
						<UButton
							:disabled="sedangBerjalan"
							variant="solid"
							@click="modalOpen = false"
						>
							Tutup
						</UButton>
					</div>
				</div>
			</template>
		</UModal>
	</ErPage>
</template>
