<script lang="ts" setup>
	import { invoke } from "@tauri-apps/api/core";
	import { open as openDialog } from "@tauri-apps/plugin-dialog";

	const props = defineProps<{
		open: boolean
		kelas: string
		jenis: string
		subjectId: number | undefined
	}>();

	const emit = defineEmits<{
		"update:open": [value: boolean]
		saved: []
	}>();

	const openModel = computed({
		get: () => props.open,
		set: (value) => emit("update:open", value)
	});

	const toast = useToast();
	const notifyTemplateDownload = (format: string) => {
		toast.add({ title: "Template diunduh", description: `Contoh soal format ${format} sedang diunduh.`, icon: "lucide:download", color: "success" });
	};

	const questionTypeItems = [
		{ label: "Pilihan Ganda", value: "multiple_choice" },
		{ label: "Esai", value: "essay" }
	];

	const importing = ref(false);
	const importError = ref("");
	const importResult = ref("");
	const form = reactive({
		questionType: questionTypeItems[0]!,
		filePath: "",
		fileName: "",
		imagePaths: [] as string[]
	});

	const templateLinks = computed(() => {
		const suffix = form.questionType.value === "essay" ? "-esai" : "";
		return {
			csv: `/templates/soal-template${suffix}.csv`,
			xlsx: `/templates/soal-template${suffix}.xlsx`
		};
	});

	watch(
		() => props.open,
		(isOpen) => {
			if (!isOpen) return;
			importError.value = "";
			importResult.value = "";
			Object.assign(form, { questionType: questionTypeItems[0]!, filePath: "", fileName: "", imagePaths: [] });
		}
	);

	const pickImportFile = async () => {
		const path = await openDialog({ multiple: false, filters: [{ name: "Soal", extensions: ["csv", "xlsx", "xls"] }] });
		if (!path) return;
		form.filePath = path;
		form.fileName = path.split(/[/\\]/).pop() ?? path;
	};

	const pickImportImages = async () => {
		const paths = await openDialog({ multiple: true, filters: [{ name: "Gambar", extensions: ["jpg", "jpeg", "png", "gif", "webp"] }] });
		if (!paths) return;
		form.imagePaths = Array.isArray(paths) ? paths : [paths];
	};

	const doImport = async () => {
		importError.value = "";
		importResult.value = "";

		if (!props.subjectId || !form.filePath) {
			importError.value = "File soal wajib dipilih.";
			return;
		}

		importing.value = true;
		try {
			const imageMap: Record<string, string> = {};
			for (const imgPath of form.imagePaths) {
				const filename = imgPath.split(/[/\\]/).pop() ?? imgPath;
				const saved = await invoke<{ path: string, dataUrl: string }>("save_question_image", { path: imgPath });
				imageMap[filename] = saved.path;
			}

			const summary = await invoke<{ questionsImported: number, subjectsCreated: number }>("import_questions", {
				path: form.filePath,
				opts: {
					defaultSubjectId: props.subjectId,
					defaultClass: props.kelas,
					defaultJenis: props.jenis,
					questionType: form.questionType.value,
					imageMap
				}
			});
			importResult.value = `Berhasil import ${summary.questionsImported} soal.`;
			emit("saved");
		} catch (error) {
			importError.value = error instanceof Error ? error.message : String(error);
		} finally {
			importing.value = false;
		}
	};
</script>

<template>
	<UModal v-model:open="openModel" title="Import Soal dari File" :ui="{ content: 'max-w-lg' }">
		<template #body>
			<form class="space-y-4" @submit.prevent="doImport">
				<UFormField label="Tipe Soal" description="Satu file hanya untuk satu tipe. Buat file terpisah untuk PG dan Esai.">
					<USelectMenu v-model="form.questionType" :items="questionTypeItems" />
				</UFormField>

				<p class="text-sm text-muted -mt-2">
					Contoh format:
					<a
						:href="templateLinks.csv"
						download
						class="text-primary underline"
						@click="notifyTemplateDownload('CSV')">CSV</a>
					·
					<a
						:href="templateLinks.xlsx"
						download
						class="text-primary underline"
						@click="notifyTemplateDownload('Excel')">Excel</a>
				</p>

				<UFormField label="File Soal (.csv atau .xlsx)">
					<UButton variant="soft" icon="lucide:file-spreadsheet" @click="pickImportFile">
						{{ form.fileName || "Pilih File" }}
					</UButton>
				</UFormField>

				<UFormField label="Gambar Soal (opsional)" description="Upload semua gambar yang direferensikan di kolom 'image' pada file.">
					<UButton variant="soft" icon="lucide:images" @click="pickImportImages">
						{{ form.imagePaths.length ? `${form.imagePaths.length} gambar dipilih` : "Pilih Gambar" }}
					</UButton>
				</UFormField>

				<UAlert
					v-if="importError"
					color="error"
					variant="subtle"
					:title="importError" />
				<UAlert
					v-if="importResult"
					color="success"
					variant="subtle"
					:title="importResult" />

				<UButton type="submit" block :loading="importing">
					Import
				</UButton>
			</form>
		</template>
	</UModal>
</template>
