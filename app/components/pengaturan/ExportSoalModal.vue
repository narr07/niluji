<script lang="ts" setup>
	import { invoke } from "@tauri-apps/api/core";
	import { open as openDialog } from "@tauri-apps/plugin-dialog";

	const props = defineProps<{
		open: boolean
	}>();

	const emit = defineEmits<{
		"update:open": [value: boolean]
	}>();

	const openModel = computed({
		get: () => props.open,
		set: (value) => emit("update:open", value)
	});

	interface Subject {
		id: number
		name: string
		code: string | null
	}

	interface QuestionTypeRecord {
		id: number
		name: string
	}

	const toast = useToast();

	const classes = ref<string[]>([]);
	const subjects = ref<Subject[]>([]);
	const jenisOptions = ref<QuestionTypeRecord[]>([]);
	const loadingJenis = ref(false);

	const kelas = ref("");
	const subjectId = ref<number | undefined>();
	const jenis = ref("");
	const pin = ref("");
	const exporting = ref(false);
	const errorMessage = ref("");

	// Supaya guru tidak perlu navigasi ulang ke folder db-soal setiap kali export — dialog
	// folder-picker langsung default ke folder terakhir yang dipilih (per perangkat).
	const LAST_FOLDER_KEY = "niluji-export-soal-folder";
	const lastFolder = ref<string | undefined>();
	onMounted(() => {
		try {
			lastFolder.value = localStorage.getItem(LAST_FOLDER_KEY) ?? undefined;
		} catch {
			// Abaikan kalau localStorage tidak bisa diakses — dialog cuma tidak punya default.
		}
	});

	// "Tarik Soal Online" narik data dari folder `db-soal/` di repo GitHub narr07/niluji (lihat
	// CDN_BASE di useBankSoalOnline.ts) — jadi kalau folder tujuan export bukan folder `db-soal`
	// persis di dalam clone repo itu, hasil export tidak akan pernah ketemu sekolah lain
	// walaupun sudah di-push, karena strukturnya tidak nyambung sama yang dicari Tarik Soal Online.
	const folderNameWarning = computed(() => {
		if (!lastFolder.value) return null;
		const name = lastFolder.value.split(/[/\\]/).filter(Boolean).pop();
		if (name && name.toLowerCase() !== "db-soal") {
			return `Folder tujuan saat ini bernama "${name}", bukan "db-soal". Supaya hasil export ini bisa ditarik sekolah lain lewat Tarik Soal Online setelah di-push, folder tujuannya harus persis folder "db-soal" di dalam clone repo GitHub Niluji kamu.`;
		}
		return null;
	});

	const subjectItems = computed(() => subjects.value.map((s) => ({ label: s.code ? `${s.code} — ${s.name}` : s.name, value: s.id })));
	const jenisItems = computed(() => jenisOptions.value.map((j) => j.name));
	const canExport = computed(() => Boolean(kelas.value && subjectId.value && jenis.value && pin.value.trim()));

	const loadJenis = async () => {
		if (!kelas.value || !subjectId.value) {
			jenisOptions.value = [];
			jenis.value = "";
			return;
		}
		loadingJenis.value = true;
		try {
			jenisOptions.value = await invoke<QuestionTypeRecord[]>("list_question_types", { class: kelas.value, subjectId: subjectId.value });
			jenis.value = "";
		} finally {
			loadingJenis.value = false;
		}
	};

	watch([kelas, subjectId], loadJenis);

	watch(
		() => props.open,
		async (isOpen) => {
			if (!isOpen) return;
			errorMessage.value = "";
			pin.value = "";
			[classes.value, subjects.value] = await Promise.all([
				invoke<string[]>("list_classes"),
				invoke<Subject[]>("list_subjects")
			]);
		},
		{ immediate: true }
	);

	const doExport = async () => {
		if (!canExport.value || !subjectId.value) return;
		errorMessage.value = "";

		const outputDir = await openDialog({
			directory: true,
			multiple: false,
			title: "Pilih folder tujuan export",
			defaultPath: lastFolder.value
		});
		if (!outputDir) return;

		lastFolder.value = outputDir;
		try {
			localStorage.setItem(LAST_FOLDER_KEY, outputDir);
		} catch {
			// Sama seperti di atas — pilihan folder cuma tidak tersimpan untuk export berikutnya.
		}

		exporting.value = true;
		try {
			const summary = await invoke<{ filePath: string, questionsExported: number, imagesExported: number }>("export_jenis_soal", {
				outputDir,
				kelas: kelas.value,
				subjectId: subjectId.value,
				jenis: jenis.value,
				pin: pin.value.trim()
			});
			toast.add({
				title: "Export selesai",
				description: `${summary.questionsExported} soal (${summary.imagesExported} gambar) disimpan ke ${summary.filePath}`,
				icon: "lucide:check",
				color: "success"
			});
			openModel.value = false;
		} catch (error) {
			errorMessage.value = error instanceof Error ? error.message : String(error);
		} finally {
			exporting.value = false;
		}
	};
</script>

<template>
	<UModal v-model:open="openModel" title="Export Bank Soal" :ui="{ content: 'max-w-lg' }">
		<template #body>
			<div class="space-y-4">
				<p class="text-sm text-muted">
					Pilih kelas, mata pelajaran, dan jenis ujian yang mau diekspor ke file Markdown + folder gambar.
					Pilih folder <strong>db-soal</strong> di dalam clone repo GitHub Niluji kamu sebagai tujuan —
					supaya setelah di-push, hasilnya bisa ditarik sekolah lain lewat Tarik Soal Online.
				</p>

				<UAlert
					v-if="folderNameWarning"
					color="warning"
					variant="subtle"
					title="Cek lagi folder tujuannya"
					:description="folderNameWarning" />

				<UFormField label="Kelas">
					<USelectMenu
						v-model="kelas"
						:items="classes"
						placeholder="Pilih kelas"
						class="w-full" />
				</UFormField>

				<UFormField label="Mata Pelajaran">
					<USelectMenu
						:model-value="subjectId"
						:items="subjectItems"
						value-key="value"
						:disabled="!kelas"
						placeholder="Pilih mata pelajaran"
						class="w-full"
						@update:model-value="(v) => (subjectId = v as unknown as number)" />
				</UFormField>

				<UFormField label="Jenis Ujian">
					<USelectMenu
						v-model="jenis"
						:items="jenisItems"
						:loading="loadingJenis"
						:disabled="!subjectId"
						placeholder="Pilih jenis ujian"
						class="w-full" />
				</UFormField>

				<UFormField label="PIN Export">
					<UInput
						v-model="pin"
						type="password"
						placeholder="Masukkan PIN"
						class="w-full" />
				</UFormField>

				<UAlert
					v-if="errorMessage"
					color="error"
					variant="subtle"
					title="Gagal export"
					:description="errorMessage" />

				<UButton
					icon="lucide:download"
					block
					:loading="exporting"
					:disabled="!canExport"
					@click="doExport">
					Export
				</UButton>
			</div>
		</template>
	</UModal>
</template>
