<script lang="ts" setup>
	import type { TableColumn } from "@nuxt/ui";
	import { invoke } from "@tauri-apps/api/core";
	import { fetch as tauriFetch } from "@tauri-apps/plugin-http";
	import { open as openUrl } from "@tauri-apps/plugin-shell";

	const CSV_URL =
		"https://docs.google.com/spreadsheets/d/e/2PACX-1vS2dCk5Yo9KFKBqo8fT0vq8941YB6AIZvjseHzEwyOOcN6CwfCiPqc944JJwKlnCYs1e9VC5CpK4lBf/pub?output=csv";
	const CONNECTION_CHECK_URL = "https://docs.google.com/";
	const EDIT_SHEET_URL = "https://docs.google.com/spreadsheets/d/1p4_wCArJP0p2Ap70UUx3hcIXZLgP7cZumF2AJNQN-LY/edit?gid=0#gid=0";

	type TarikStatus = "baru" | "berubah" | "sama" | "tidak_valid";
	type StatusFilter = TarikStatus | "semua";

	interface TarikPreviewRow {
		nisn: string
		name: string
		class: string
		rombel: string | null
		status: TarikStatus
		oldName: string | null
		oldClass: string | null
		note: string | null
	}

	const STATUS_INFO: Record<TarikStatus, { label: string, color: "success" | "warning" | "neutral" | "error" }> = {
		baru: { label: "Baru", color: "success" },
		berubah: { label: "Berubah", color: "warning" },
		sama: { label: "Sama", color: "neutral" },
		tidak_valid: { label: "Tidak valid", color: "error" }
	};

	const school = ref("");
	const sourceSchools = ref<string[]>([]);
	const loadingSchools = ref(false);
	const schoolsError = ref("");
	const loading = ref(false);
	const saving = ref(false);
	const confirmOpen = ref(false);
	const errorMessage = ref("");
	const rows = ref<TarikPreviewRow[]>([]);
	const previewSchool = ref("");
	const selected = ref(new Set<string>());
	const statusFilter = ref<StatusFilter>("semua");

	const columns: TableColumn<TarikPreviewRow>[] = [
		{ id: "select", header: "Pilih" },
		{ id: "status", header: "Status" },
		{ accessorKey: "nisn", header: "NISN" },
		{ id: "name", header: "Nama" },
		{ id: "class", header: "Kelas" }
	];

	const checkingConnection = ref(false);
	const connectionStatus = ref<"checking" | "online" | "offline">("checking");
	const connectionMessage = ref("Memeriksa koneksi internet...");
	const connectionCheckedAt = ref<Date>();

	const canFetch = computed(() => connectionStatus.value === "online");
	const toast = useToast();

	const counts = computed(() => {
		const result: Record<TarikStatus, number> = { baru: 0, berubah: 0, sama: 0, tidak_valid: 0 };
		for (const row of rows.value) result[row.status]++;
		return result;
	});
	const filterItems = computed(() => [
		{ value: "semua" as const, label: `Semua (${rows.value.length})` },
		...(Object.keys(STATUS_INFO) as TarikStatus[]).map((status) => ({ value: status, label: `${STATUS_INFO[status].label} (${counts.value[status]})` }))
	]);
	const visibleRows = computed(() => (statusFilter.value === "semua" ? rows.value : rows.value.filter((row) => row.status === statusFilter.value)));
	const selectableVisible = computed(() => visibleRows.value.filter((row) => row.status !== "tidak_valid"));
	const allVisibleSelected = computed(() => selectableVisible.value.length > 0 && selectableVisible.value.every((row) => selected.value.has(row.nisn)));
	const selectedRows = computed(() => rows.value.filter((row) => selected.value.has(row.nisn)));
	const selectedSummary = computed(() => ({
		baru: selectedRows.value.filter((row) => row.status === "baru").length,
		berubah: selectedRows.value.filter((row) => row.status === "berubah").length,
		sama: selectedRows.value.filter((row) => row.status === "sama").length
	}));

	const toggleRow = (row: TarikPreviewRow, value: boolean | "indeterminate") => {
		const next = new Set(selected.value);
		if (value === true) next.add(row.nisn);
		else next.delete(row.nisn);
		selected.value = next;
	};
	const toggleVisible = () => {
		const next = new Set(selected.value);
		const select = !allVisibleSelected.value;
		for (const row of selectableVisible.value) {
			if (select) next.add(row.nisn);
			else next.delete(row.nisn);
		}
		selected.value = next;
	};

	// Pratinjau berlaku untuk satu sekolah saja. Ganti sekolah berarti pratinjau lama dibuang.
	watch(school, (value) => {
		if (previewSchool.value && value.trim() !== previewSchool.value) {
			rows.value = [];
			selected.value = new Set();
			previewSchool.value = "";
		}
	});

	const refreshAndReload = async () => {
		await refreshConnection();
		if (canFetch.value) await loadSourceSchools();
	};

	const refreshConnection = async () => {
		checkingConnection.value = true;
		connectionStatus.value = "checking";
		connectionMessage.value = "Memeriksa koneksi internet...";
		try {
			const response = await tauriFetch(CONNECTION_CHECK_URL, { cache: "no-store" });
			if (!response.ok) throw new Error(`Server merespons status ${response.status}`);
			connectionStatus.value = "online";
			connectionMessage.value = "Internet terhubung, sumber data siswa dapat dijangkau.";
		} catch {
			connectionStatus.value = "offline";
			connectionMessage.value = "Tidak ada koneksi internet atau sumber data siswa tidak dapat dijangkau.";
		} finally {
			connectionCheckedAt.value = new Date();
			checkingConnection.value = false;
		}
		return connectionStatus.value === "online";
	};

	const loadSourceSchools = async () => {
		loadingSchools.value = true;
		schoolsError.value = "";
		try {
			sourceSchools.value = await invoke<string[]>("list_schools_from_source", { csvUrl: CSV_URL });
		} catch (error) {
			schoolsError.value = error instanceof Error ? error.message : String(error);
		} finally {
			loadingSchools.value = false;
		}
	};

	// Langkah 1: ambil data dan bandingkan dengan database. Belum ada yang disimpan.
	const ambilPratinjau = async () => {
		const target = school.value.trim();
		if (!target) {
			errorMessage.value = "Isi nama sekolah dulu.";
			return;
		}
		if (!canFetch.value) {
			errorMessage.value = "Tidak ada koneksi internet.";
			return;
		}

		loading.value = true;
		errorMessage.value = "";
		try {
			rows.value = await invoke<TarikPreviewRow[]>("preview_tarik_siswa", { csvUrl: CSV_URL, school: target });
			previewSchool.value = target;
			statusFilter.value = "semua";
			// Hanya siswa baru dan yang berubah yang terpilih otomatis.
			selected.value = new Set(rows.value.filter((row) => row.status === "baru" || row.status === "berubah").map((row) => row.nisn));
		} catch (error) {
			rows.value = [];
			previewSchool.value = "";
			errorMessage.value = error instanceof Error ? error.message : String(error);
		} finally {
			loading.value = false;
		}
	};

	// Langkah 2: simpan siswa yang dipilih setelah guru memeriksa pratinjau.
	const simpan = async () => {
		saving.value = true;
		try {
			const count = await invoke<number>("simpan_tarik_siswa", { csvUrl: CSV_URL, school: previewSchool.value, nisns: [...selected.value] });
			confirmOpen.value = false;
			toast.add({ title: "Data siswa disimpan", description: `${count} siswa dari ${previewSchool.value} tersimpan ke database lokal.`, icon: "lucide:check", color: "success" });
			await ambilPratinjau();
		} catch (error) {
			toast.add({ title: "Gagal menyimpan", description: error instanceof Error ? error.message : String(error), icon: "lucide:x", color: "error" });
		} finally {
			saving.value = false;
		}
	};

	onMounted(async () => {
		const isConnected = await refreshConnection();
		if (isConnected) void loadSourceSchools();
	});
</script>

<template>
	<div class="space-y-6">
		<div>
			<h2 class="text-lg font-semibold">
				Tarik Data Siswa
			</h2>
			<p class="text-sm text-muted mt-1">
				Ambil data siswa dari sumber online, periksa pratinjaunya, lalu pilih siswa yang disimpan ke database lokal.
			</p>
		</div>

		<UAlert
			color="info"
			variant="subtle"
			title="Ada nama siswa atau NISN yang salah?"
			icon="lucide:info">
			<template #description>
				<p>Data ini bersumber dari Google Sheets. Jika ada kesalahan, kamu bisa membuka dan mengedit langsung di sana, lalu tarik ulang datanya di sini.</p>
				<UButton
					variant="link"
					color="info"
					icon="lucide:external-link"
					class="px-0 mt-1"
					@click="openUrl(EDIT_SHEET_URL)">
					Buka &amp; Edit Data Sumber
				</UButton>
			</template>
		</UAlert>

		<div class="flex flex-wrap items-center gap-3 rounded-lg border border-default bg-elevated/40 px-4 py-3">
			<UIcon
				:name="connectionStatus === 'online' ? 'lucide:wifi' : connectionStatus === 'offline' ? 'lucide:wifi-off' : 'lucide:loader-circle'"
				:class="['size-5', connectionStatus === 'online' ? 'text-success' : connectionStatus === 'offline' ? 'text-error' : 'animate-spin text-muted']" />
			<div class="flex-1 text-sm">
				<p :class="connectionStatus === 'online' ? 'text-success' : connectionStatus === 'offline' ? 'text-error' : 'text-muted'">
					{{ connectionMessage }}
				</p>
				<p v-if="connectionCheckedAt" class="mt-0.5 text-xs text-muted">
					Diperiksa pukul {{ connectionCheckedAt.toLocaleTimeString('id-ID') }}
				</p>
			</div>
			<UButton
				variant="soft"
				icon="lucide:refresh-cw"
				:loading="checkingConnection"
				@click="refreshAndReload">
				Refresh koneksi
			</UButton>
		</div>

		<div class="grid gap-4 md:grid-cols-3 items-end">
			<UFormField label="Nama Sekolah" description="Ketik untuk mencari, lalu pilih nama sekolah yang persis dari daftar." class="md:col-span-2">
				<UInputMenu
					v-model="school"
					:items="sourceSchools"
					:loading="loadingSchools"
					:disabled="!canFetch"
					open-on-click
					placeholder="Contoh: SD NEGERI CIPINANG I"
					class="w-full" />
			</UFormField>

			<UButton
				icon="lucide:eye"
				:loading="loading"
				:disabled="!canFetch || loadingSchools || saving"
				@click="ambilPratinjau">
				Lihat Pratinjau
			</UButton>
		</div>

		<UAlert
			v-if="schoolsError"
			color="warning"
			variant="subtle"
			title="Gagal mengambil daftar sekolah dari sumber"
			:description="schoolsError" />

		<UAlert
			v-if="errorMessage"
			color="error"
			variant="subtle"
			title="Gagal menarik data"
			:description="errorMessage" />

		<template v-if="rows.length">
			<UAlert
				color="neutral"
				variant="subtle"
				icon="lucide:eye"
				:title="`Pratinjau ${rows.length} siswa dari ${previewSchool}. Belum ada yang disimpan.`"
				description="Siswa baru dan yang datanya berubah sudah dipilih. Siswa yang sama dengan database tidak perlu disimpan ulang, dan baris tidak valid tidak bisa dipilih." />

			<div class="flex flex-wrap items-center justify-between gap-3">
				<div class="flex flex-wrap gap-1.5">
					<UButton
						v-for="item in filterItems"
						:key="item.value"
						size="sm"
						:color="item.value === 'semua' ? 'neutral' : STATUS_INFO[item.value].color"
						:variant="statusFilter === item.value ? 'solid' : 'soft'"
						@click="statusFilter = item.value">
						{{ item.label }}
					</UButton>
				</div>
				<div class="flex flex-wrap items-center gap-2">
					<UButton
						size="sm"
						variant="ghost"
						color="neutral"
						:disabled="!selectableVisible.length"
						@click="toggleVisible">
						{{ allVisibleSelected ? "Batal pilih yang tampil" : "Pilih semua yang tampil" }}
					</UButton>
					<UButton
						icon="lucide:database"
						:disabled="!selected.size"
						:loading="saving"
						@click="confirmOpen = true">
						Simpan {{ selected.size }} Siswa
					</UButton>
				</div>
			</div>

			<div class="overflow-x-auto rounded-lg border border-default max-h-[28rem] overflow-y-auto">
				<UTable :data="visibleRows" :columns="columns">
					<template #select-cell="{ row }">
						<UCheckbox
							:model-value="selected.has(row.original.nisn)"
							:disabled="row.original.status === 'tidak_valid'"
							:aria-label="`Pilih ${row.original.name || row.original.nisn}`"
							@update:model-value="(value) => toggleRow(row.original, value)" />
					</template>
					<template #status-cell="{ row }">
						<UBadge :color="STATUS_INFO[row.original.status].color" variant="subtle">
							{{ STATUS_INFO[row.original.status].label }}
						</UBadge>
						<p v-if="row.original.note" class="text-xs text-error mt-1">
							{{ row.original.note }}
						</p>
					</template>
					<template #nisn-cell="{ row }">
						<span class="font-mono">{{ row.original.nisn || "(kosong)" }}</span>
					</template>
					<template #name-cell="{ row }">
						<p>{{ row.original.name || "(kosong)" }}</p>
						<p v-if="row.original.oldName" class="text-xs text-muted line-through">
							{{ row.original.oldName }}
						</p>
					</template>
					<template #class-cell="{ row }">
						<p>{{ labelKelas(row.original.class, row.original.rombel) }}</p>
						<p v-if="row.original.oldClass !== null" class="text-xs text-muted line-through">
							{{ row.original.oldClass || "(kosong)" }}
						</p>
					</template>
				</UTable>
			</div>
		</template>

		<UModal v-model:open="confirmOpen" title="Simpan data siswa?" :description="`Dari ${previewSchool}`">
			<template #body>
				<ul class="space-y-2 text-sm">
					<li v-if="selectedSummary.baru" class="flex gap-2">
						<UIcon name="lucide:user-plus" class="size-4 mt-0.5 text-success shrink-0" />
						<span>{{ selectedSummary.baru }} siswa baru ditambahkan dan dibuatkan PIN kartu peserta.</span>
					</li>
					<li v-if="selectedSummary.berubah" class="flex gap-2">
						<UIcon name="lucide:pencil" class="size-4 mt-0.5 text-warning shrink-0" />
						<span>{{ selectedSummary.berubah }} siswa diperbarui nama, kelas, atau sekolahnya. PIN kartu yang sudah dicetak tidak berubah.</span>
					</li>
					<li v-if="selectedSummary.sama" class="flex gap-2">
						<UIcon name="lucide:equal" class="size-4 mt-0.5 text-muted shrink-0" />
						<span>{{ selectedSummary.sama }} siswa sama dengan database, disimpan ulang tanpa perubahan.</span>
					</li>
					<li class="flex gap-2 text-muted">
						<UIcon name="lucide:database-backup" class="size-4 mt-0.5 shrink-0" />
						<span>Backup database dibuat otomatis sebelum menyimpan, jadi perubahan ini bisa dipulihkan dari Tentang &gt; Fitur.</span>
					</li>
				</ul>
			</template>
			<template #footer>
				<div class="flex justify-end gap-2 w-full">
					<UButton
						color="neutral"
						variant="ghost"
						:disabled="saving"
						@click="confirmOpen = false">
						Batal
					</UButton>
					<UButton icon="lucide:database" :loading="saving" @click="simpan">
						Simpan {{ selected.size }} Siswa
					</UButton>
				</div>
			</template>
		</UModal>
	</div>
</template>
