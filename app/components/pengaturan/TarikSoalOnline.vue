<script lang="ts" setup>
	import type { TableColumn } from "@nuxt/ui";
	import type { Katalog, PaketEntry } from "~/composables/useBankSoalGugus";
	import type { BankSoalOnlineRow, OnlineSubject } from "~/composables/useBankSoalOnline";
	import { invoke } from "@tauri-apps/api/core";
	import { fetch as tauriFetch } from "@tauri-apps/plugin-http";
	import { fetchKatalog, fetchPaketRows, paketImageUrl, paketUntukGugus, SEMUA_GUGUS } from "~/composables/useBankSoalGugus";
	import { checkBankSoalConnection } from "~/composables/useBankSoalOnline";

	// Spasi tak-terputus: label "kosong" yang tetap punya tinggi baris, untuk menyejajarkan tombol.
	const LABEL_KOSONG = String.fromCharCode(0xA0);

	const emit = defineEmits<{
		selected: [rows: BankSoalOnlineRow[]]
	}>();

	interface PilihanSekolah {
		kecamatan: string | null
		gugus: string | null
	}

	const toast = useToast();
	const katalog = ref<Katalog | null>(null);
	const loadingKatalog = ref(false);
	const localSubjects = ref<OnlineSubject[]>([]);
	const localClasses = ref<string[]>([]);

	const selectedKecamatan = ref("");
	const selectedGugus = ref("");
	const selectedKelas = ref("");
	const selectedMapel = ref("");
	const selectedFolder = ref("");
	const targetSubjectId = ref<number | undefined>();
	const jenisSoal = ref("");

	const rows = ref<BankSoalOnlineRow[]>([]);
	const loadedPaket = ref<PaketEntry | null>(null);
	const loadingRows = ref(false);
	const checkingConnection = ref(false);
	const errorMessage = ref("");
	const selectedRows = ref(new Set<string>());
	const connectionStatus = ref<"checking" | "online" | "offline">("checking");
	const connectionMessage = ref("Memeriksa koneksi internet...");
	const connectionCheckedAt = ref<Date>();
	const scopeOptions = [
		{ label: "Kelas & mata pelajaran ini saja", value: "narrow" as const },
		{ label: "Semua pelajaran di kelas ini saja", value: "class" as const },
		{ label: "Semua kelas, mata pelajaran ini saja", value: "subject" as const },
		{ label: "Semua kelas & semua mata pelajaran", value: "global" as const }
	];
	const scope = ref<"narrow" | "class" | "subject" | "global">("narrow");
	const importing = ref(false);
	const importError = ref("");
	const importedIds = ref(new Set<string>());

	const kecamatan = computed(() => katalog.value?.kecamatan.find((k) => k.id === selectedKecamatan.value));
	const kecamatanItems = computed(() => (katalog.value?.kecamatan ?? []).map((k) => ({ label: k.nama, value: k.id })));
	const gugusItems = computed(() => (kecamatan.value?.gugus ?? []).map((g) => ({ label: g.nama, value: g.id })));
	const paketGugus = computed(() => (katalog.value && selectedGugus.value ? paketUntukGugus(katalog.value, selectedKecamatan.value, selectedGugus.value) : []));

	const kelasItems = computed(() =>
		[...new Set(paketGugus.value.map((p) => p.kelas))].sort().map((k) => ({ label: `Kelas ${k}`, value: k }))
	);
	const paketKelas = computed(() => paketGugus.value.filter((p) => p.kelas === selectedKelas.value));
	const mapelItems = computed(() => {
		const seen = new Map<string, string>();
		for (const p of paketKelas.value) seen.set(p.mapelKode, `${p.mapelKode} - ${p.mapelNama}`);
		return [...seen.entries()].map(([value, label]) => ({ label, value }));
	});
	const paketMapel = computed(() => paketKelas.value.filter((p) => p.mapelKode === selectedMapel.value));
	const jenisItems = computed(() =>
		paketMapel.value.map((p) => ({
			label: `${p.jenis}${p.gugus === SEMUA_GUGUS ? " (semua gugus)" : ""} · ${p.pg} PG, ${p.esai} esai`,
			value: p.folder
		}))
	);
	const selectedPaket = computed(() => paketMapel.value.find((p) => p.folder === selectedFolder.value));

	const subjectItems = computed(() => localSubjects.value.map((s) => ({ label: s.code ? `${s.code} - ${s.name}` : s.name, value: s.id })));
	const kelasTidakTerdaftar = computed(() => Boolean(loadedPaket.value && !localClasses.value.includes(loadedPaket.value.kelas)));

	const canFetch = computed(() => Boolean(selectedPaket.value && connectionStatus.value === "online"));
	const validCount = computed(() => rows.value.filter((row) => row.valid).length);
	const selectedCount = computed(() => selectedRows.value.size);

	const columns: TableColumn<BankSoalOnlineRow>[] = [
		{ id: "select", header: "Pilih" },
		{ accessorKey: "rowNumber", header: "No" },
		{ id: "tipe", header: "Tipe" },
		{ id: "soal", header: "Soal" },
		{ id: "status", header: "Status" },
		{ id: "gambar", header: "Gambar" }
	];

	const resetPreview = () => {
		rows.value = [];
		loadedPaket.value = null;
		errorMessage.value = "";
		importError.value = "";
	};

	const refreshConnection = async () => {
		checkingConnection.value = true;
		connectionStatus.value = "checking";
		connectionMessage.value = "Memeriksa koneksi internet...";
		try {
			const status = await checkBankSoalConnection();
			connectionStatus.value = status.connected ? "online" : "offline";
			connectionMessage.value = status.message;
			connectionCheckedAt.value = status.checkedAt;
			return status.connected;
		} finally {
			checkingConnection.value = false;
		}
	};

	const loadKatalog = async () => {
		loadingKatalog.value = true;
		errorMessage.value = "";
		try {
			katalog.value = await fetchKatalog();
			const saved = await invoke<PilihanSekolah>("bank_soal_pilihan_sekolah");
			const list = katalog.value.kecamatan;
			const kec = list.find((k) => k.id === saved.kecamatan) ?? (list.length === 1 ? list[0] : undefined);
			selectedKecamatan.value = kec?.id ?? "";
			selectedGugus.value = kec?.gugus.some((g) => g.id === saved.gugus) ? saved.gugus! : "";
		} catch (error) {
			errorMessage.value = error instanceof Error ? error.message : String(error);
		} finally {
			loadingKatalog.value = false;
		}
	};

	const reload = async () => {
		if (await refreshConnection()) await loadKatalog();
	};

	const selectKecamatan = (value: string) => {
		selectedKecamatan.value = value;
		selectedGugus.value = "";
		selectedKelas.value = "";
		resetPreview();
	};

	// Gugus disimpan sebagai gugus sekolah ini, supaya otomatis terpilih lain kali.
	const selectGugus = async (value: string) => {
		selectedGugus.value = value;
		selectedKelas.value = "";
		selectedMapel.value = "";
		selectedFolder.value = "";
		resetPreview();
		try {
			await invoke("bank_soal_simpan_pilihan", { pilihan: { kecamatan: selectedKecamatan.value, gugus: value } });
		} catch {
			// Gagal menyimpan pilihan tidak menghalangi menarik soal.
		}
	};

	const selectKelas = (value: string) => {
		selectedKelas.value = value;
		selectedMapel.value = "";
		selectedFolder.value = "";
		resetPreview();
	};

	const selectMapel = (value: string) => {
		selectedMapel.value = value;
		selectedFolder.value = paketMapel.value.length === 1 ? paketMapel.value[0]!.folder : "";
		resetPreview();
	};

	// Mapel sekolah dicocokkan dari kode, lalu nama. Bila tidak ada yang cocok, sekolah memilih sendiri.
	const matchLocalSubject = (paket: PaketEntry) => {
		const norm = (s: string | null | undefined) => (s ?? "").trim().toLowerCase();
		return (
			localSubjects.value.find((s) => norm(s.code) && norm(s.code) === norm(paket.mapelKode))
			?? localSubjects.value.find((s) => norm(s.name) === norm(paket.mapelNama))
		)?.id;
	};

	const fetchRows = async () => {
		const paket = selectedPaket.value;
		if (!canFetch.value || !paket) return;
		loadingRows.value = true;
		resetPreview();
		selectedRows.value = new Set();
		importedIds.value = new Set();
		try {
			const isConnected = await refreshConnection();
			if (!isConnected) return;
			rows.value = await fetchPaketRows(paket);
			loadedPaket.value = paket;
			jenisSoal.value = paket.jenis;
			targetSubjectId.value = matchLocalSubject(paket);
			selectedRows.value = new Set(rows.value.filter((row) => row.valid).map((row) => row.id));
		} catch (error) {
			errorMessage.value = error instanceof Error ? error.message : String(error);
		} finally {
			loadingRows.value = false;
		}
	};

	const isSelected = (row: BankSoalOnlineRow) => selectedRows.value.has(row.id);

	const toggleRow = (row: BankSoalOnlineRow, value: boolean | "indeterminate") => {
		if (!row.valid || value === "indeterminate") return;
		const next = new Set(selectedRows.value);
		if (value) next.add(row.id);
		else next.delete(row.id);
		selectedRows.value = next;
	};

	const importSelected = async () => {
		importError.value = "";
		const paket = loadedPaket.value;
		if (!paket) return;
		if (!jenisSoal.value.trim()) {
			importError.value = "Isi nama jenis ujian dulu sebelum impor (misalnya \"STS Ganjil\").";
			return;
		}
		if (!targetSubjectId.value) {
			importError.value = "Pilih mata pelajaran sekolah tempat soal ini disimpan.";
			return;
		}

		const selected = rows.value.filter((row) => selectedRows.value.has(row.id) && !importedIds.value.has(row.id));
		if (!selected.length) return;

		importing.value = true;
		try {
			await invoke("create_question_type", {
				name: jenisSoal.value.trim(),
				description: null,
				class: (scope.value === "narrow" || scope.value === "class") ? paket.kelas : null,
				subjectId: (scope.value === "narrow" || scope.value === "subject") ? targetSubjectId.value : null
			});
		} catch {
			// Jenis ujian sudah ada.
		}

		const imageCache = new Map<string, string>();
		const newlyImported: string[] = [];
		const failed: string[] = [];

		const downloadImage = async (fileName: string): Promise<string> => {
			const cached = imageCache.get(fileName);
			if (cached) return cached;
			const response = await tauriFetch(paketImageUrl(paket.folder, fileName));
			if (!response.ok) throw new Error(`Gagal mengunduh gambar ${fileName}`);
			const saved = await invoke<{ path: string }>("save_question_image_bytes", {
				fileName,
				bytes: Array.from(new Uint8Array(await response.arrayBuffer()))
			});
			imageCache.set(fileName, saved.path);
			return saved.path;
		};
		const optionalImage = (fileName?: string) => (fileName ? downloadImage(fileName) : Promise.resolve(undefined));

		try {
			for (const row of selected) {
				try {
					const [image, optionAImage, optionBImage, optionCImage, optionDImage, optionEImage] = await Promise.all([
						optionalImage(row.nama_file_gambar || undefined),
						optionalImage(row.pilihan_a_gambar),
						optionalImage(row.pilihan_b_gambar),
						optionalImage(row.pilihan_c_gambar),
						optionalImage(row.pilihan_d_gambar),
						optionalImage(row.pilihan_e_gambar)
					]);
					await invoke("create_question", {
						input: {
							subjectId: targetSubjectId.value,
							class: paket.kelas,
							jenis: jenisSoal.value.trim(),
							questionText: row.soal,
							questionType: row.tipe === "pg" ? "multiple_choice" : "essay",
							image,
							score: Number(row.skor) || 1,
							optionA: row.tipe === "pg" ? row.pilihan_a : "",
							optionB: row.tipe === "pg" ? row.pilihan_b : "",
							optionC: row.tipe === "pg" ? (row.pilihan_c || undefined) : undefined,
							optionD: row.tipe === "pg" ? (row.pilihan_d || undefined) : undefined,
							optionE: row.tipe === "pg" ? (row.pilihan_e || undefined) : undefined,
							optionAImage,
							optionBImage,
							optionCImage,
							optionDImage,
							optionEImage,
							correctOption: row.tipe === "pg" ? row.kunci_jawaban.trim().toUpperCase() : ""
						}
					});
					newlyImported.push(row.id);
				} catch (error) {
					failed.push(`Soal ${row.rowNumber}: ${error instanceof Error ? error.message : String(error)}`);
				}
			}
		} finally {
			importing.value = false;
		}

		if (newlyImported.length) {
			importedIds.value = new Set([...importedIds.value, ...newlyImported]);
			emit("selected", rows.value.filter((row) => newlyImported.includes(row.id)));
			toast.add({
				title: "Impor selesai",
				description: `${newlyImported.length} soal berhasil ditambahkan ke Bank Soal${failed.length ? `, ${failed.length} gagal` : ""}.`,
				icon: "lucide:check",
				color: failed.length ? "warning" : "success"
			});
		}
		if (failed.length) {
			importError.value = failed.slice(0, 5).join("\n") + (failed.length > 5 ? `\n...dan ${failed.length - 5} soal lainnya gagal.` : "");
		}
	};

	onMounted(async () => {
		[localSubjects.value, localClasses.value] = await Promise.all([
			invoke<OnlineSubject[]>("list_subjects"),
			invoke<string[]>("list_classes")
		]);
		await reload();
	});
</script>

<template>
	<div class="space-y-6">
		<div>
			<h2 class="text-lg font-semibold">
				Tarik Soal Online
			</h2>
			<p class="text-sm text-muted mt-1">
				Ambil bank soal yang dipublikasikan pembuat soal untuk gugus sekolah Anda, periksa, lalu simpan ke Bank Soal untuk ujian offline.
			</p>
		</div>

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
				:loading="checkingConnection || loadingKatalog"
				@click="reload">
				Muat ulang
			</UButton>
		</div>

		<div v-if="katalog" class="space-y-4">
			<div class="grid gap-4 items-start md:grid-cols-2">
				<UFormField
					v-if="kecamatanItems.length > 1"
					label="Kecamatan"
					class="min-w-0">
					<USelectMenu
						:model-value="selectedKecamatan"
						:items="kecamatanItems"
						value-key="value"
						placeholder="Pilih kecamatan"
						class="w-full"
						@update:model-value="selectKecamatan" />
				</UFormField>
				<UFormField
					label="Gugus sekolah"
					:help="kecamatan ? `${kecamatan.nama}. Pilihan ini diingat untuk sekolah Anda.` : undefined"
					class="min-w-0">
					<USelectMenu
						:model-value="selectedGugus"
						:items="gugusItems"
						value-key="value"
						:disabled="!kecamatan"
						placeholder="Pilih gugus"
						class="w-full"
						@update:model-value="selectGugus" />
				</UFormField>
			</div>

			<div v-if="selectedGugus" class="grid gap-4 items-start md:grid-cols-[minmax(0,1fr)_minmax(0,2fr)_minmax(0,2fr)_auto]">
				<UFormField label="Kelas" class="min-w-0">
					<USelectMenu
						:model-value="selectedKelas"
						:items="kelasItems"
						value-key="value"
						placeholder="Pilih kelas"
						class="w-full"
						@update:model-value="selectKelas" />
				</UFormField>

				<UFormField label="Mata Pelajaran" class="min-w-0">
					<USelectMenu
						:model-value="selectedMapel"
						:items="mapelItems"
						value-key="value"
						:disabled="!selectedKelas"
						placeholder="Pilih mata pelajaran"
						class="w-full"
						@update:model-value="selectMapel" />
				</UFormField>

				<UFormField label="Jenis Ujian" class="min-w-0">
					<USelectMenu
						v-model="selectedFolder"
						:items="jenisItems"
						value-key="value"
						:disabled="!selectedMapel"
						placeholder="Pilih jenis ujian"
						class="w-full"
						@update:model-value="resetPreview" />
				</UFormField>

				<UFormField :label="LABEL_KOSONG" class="min-w-0">
					<UButton
						icon="lucide:cloud-download"
						:loading="loadingRows"
						:disabled="!canFetch"
						class="w-full justify-center"
						@click="fetchRows">
						Ambil &amp; Preview
					</UButton>
				</UFormField>
			</div>

			<p v-if="selectedGugus && !paketGugus.length" class="text-sm text-muted">
				Belum ada soal yang dipublikasikan untuk gugus ini.
			</p>
		</div>

		<UAlert
			v-if="errorMessage"
			color="error"
			variant="subtle"
			title="Gagal mengambil bank soal"
			:description="errorMessage" />

		<div v-if="rows.length && loadedPaket" class="space-y-4">
			<div class="grid gap-4 items-start md:grid-cols-2">
				<UFormField label="Simpan ke mata pelajaran" :help="`Di bank soal: ${loadedPaket.mapelKode} - ${loadedPaket.mapelNama}`" required>
					<USelectMenu
						v-model="targetSubjectId"
						:items="subjectItems"
						value-key="value"
						placeholder="Pilih mata pelajaran sekolah"
						class="w-full" />
				</UFormField>
				<UFormField label="Simpan sebagai jenis ujian" required>
					<UInput v-model="jenisSoal" class="w-full" />
				</UFormField>
			</div>

			<UAlert
				v-if="kelasTidakTerdaftar"
				color="warning"
				variant="subtle"
				icon="lucide:triangle-alert"
				:title="`Kelas ${loadedPaket.kelas} belum ada di Data Kelas sekolah ini`"
				description="Soal tetap bisa diimpor, tetapi tambahkan kelasnya di Pengaturan › Data Kelas supaya muncul di Bank Soal." />

			<div class="flex flex-wrap items-end justify-between gap-3">
				<div class="text-sm text-muted">
					{{ rows.length }} soal, {{ validCount }} valid, {{ rows.length - validCount }} perlu dicek
				</div>
				<UButton
					icon="lucide:database"
					:loading="importing"
					:disabled="!selectedCount"
					@click="importSelected">
					Impor ke Bank Soal ({{ selectedCount }})
				</UButton>
			</div>

			<UFormField label="Jenis ujian berlaku untuk">
				<URadioGroup v-model="scope" orientation="horizontal" :items="scopeOptions" />
			</UFormField>

			<UAlert
				v-if="importError"
				color="error"
				variant="subtle"
				title="Sebagian/semua impor gagal"
				:description="importError" />

			<div class="overflow-x-auto rounded-lg border border-default">
				<UTable :data="rows" :columns="columns">
					<template #select-cell="{ row }">
						<UCheckbox
							:model-value="isSelected(row.original)"
							:disabled="!row.original.valid || importedIds.has(row.original.id)"
							@update:model-value="(value) => toggleRow(row.original, value)" />
					</template>
					<template #tipe-cell="{ row }">
						<UBadge :color="row.original.tipe === 'pg' ? 'info' : 'neutral'" variant="subtle">
							{{ row.original.tipe === 'pg' ? 'PG' : 'Esai' }}
						</UBadge>
					</template>
					<template #soal-cell="{ row }">
						<div class="line-clamp-3 max-w-xl whitespace-normal">{{ row.original.soal || "(kosong)" }}</div>
					</template>
					<template #status-cell="{ row }">
						<UBadge :color="row.original.valid ? 'success' : 'error'" variant="subtle">
							{{ row.original.valid ? "Valid" : "Cek" }}
						</UBadge>
						<UBadge
							v-if="importedIds.has(row.original.id)"
							color="info"
							variant="subtle"
							class="ml-1">
							Diimpor
						</UBadge>
						<p v-if="row.original.alasan_invalid" class="mt-1 text-xs text-error">{{ row.original.alasan_invalid }}</p>
					</template>
					<template #gambar-cell="{ row }">
						<UIcon v-if="row.original.nama_file_gambar" name="lucide:image" class="size-4" />
						<span v-else class="text-muted">-</span>
					</template>
				</UTable>
			</div>
		</div>

		<div v-else-if="!loadingRows && katalog" class="rounded-lg border border-dashed border-default p-10 text-center text-muted">
			<UIcon name="lucide:cloud-download" class="size-8 mb-2" />
			<p>Pilih gugus, kelas, mata pelajaran, dan jenis ujian, lalu klik Ambil &amp; Preview.</p>
		</div>
	</div>
</template>
