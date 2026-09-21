<script lang="ts" setup>
	import { invoke } from "@tauri-apps/api/core";

	const props = defineProps<{
		open: boolean
		kelas: string
		jenis: string
		subjectId: number | undefined
		editingId: number | null
	}>();

	const emit = defineEmits<{
		"update:open": [value: boolean]
		saved: []
	}>();

	const openModel = computed({
		get: () => props.open,
		set: (value) => emit("update:open", value)
	});

	const imagePreview = ref("");
	const formError = ref("");
	const savedMessage = ref("");
	const saving = ref(false);

	const questionTypeItems = [
		{ label: "Pilihan Ganda", value: "multiple_choice" },
		{ label: "Esai", value: "essay" }
	];

	const optionKeys = ["A", "B", "C", "D", "E"] as const;
	const requiredOptionKeys: (typeof optionKeys)[number][] = ["A", "B"];

	const emptyQuestionFields = () => ({
		questionType: questionTypeItems[0]!,
		questionText: "",
		image: null as string | null,
		options: { A: "", B: "", C: "", D: "", E: "" } as Record<(typeof optionKeys)[number], string>,
		correctOption: undefined as string | undefined,
		score: 1
	});

	const form = reactive(emptyQuestionFields());

	watch(
		() => [props.open, props.editingId] as const,
		async ([isOpen, editingId]) => {
			if (!isOpen) return;
			formError.value = "";
			savedMessage.value = "";
			imagePreview.value = "";

			if (!editingId) {
				Object.assign(form, emptyQuestionFields());
				return;
			}

			const detail = await invoke<{
				id: number
				questionText: string
				questionType: string
				jenis: string | null
				image: string | null
				score: number
				optionA: string
				optionB: string
				optionC: string
				optionD: string
				optionE: string
				correctOption: string
			}>("get_question", { id: editingId });

			Object.assign(form, {
				questionType: questionTypeItems.find((t) => t.value === detail.questionType) ?? questionTypeItems[0]!,
				questionText: detail.questionText,
				image: detail.image,
				options: { A: detail.optionA, B: detail.optionB, C: detail.optionC, D: detail.optionD, E: detail.optionE },
				correctOption: detail.correctOption || undefined,
				score: detail.score
			});
			imagePreview.value = detail.image ? await invoke<string>("load_image_data_url", { path: detail.image }) : "";
		},
		{ immediate: true }
	);

	const removeImage = () => {
		form.image = null;
		imagePreview.value = "";
	};

	const onImagePicked = ({ path, dataUrl }: { path: string, dataUrl: string }) => {
		form.image = path;
		imagePreview.value = dataUrl;
	};

	// keepOpen cuma relevan buat soal baru (bukan edit) — "Simpan" nutup modal otomatis begitu
	// tersimpan, "Simpan & Lanjutkan" tetap buka modal & bersihkan form buat input soal berikutnya.
	const submitForm = async (keepOpen: boolean) => {
		formError.value = "";
		savedMessage.value = "";
		const isEssay = form.questionType.value === "essay";

		if (!props.subjectId) {
			formError.value = "Mata pelajaran tidak ditemukan.";
			return;
		}
		if (!form.questionText.trim()) {
			formError.value = "Pertanyaan wajib diisi.";
			return;
		}
		if (!isEssay && (!form.options.A.trim() || !form.options.B.trim() || !form.correctOption)) {
			formError.value = "Pilihan A & B, dan kunci jawaban wajib diisi untuk soal pilihan ganda.";
			return;
		}

		const input = {
			subjectId: props.subjectId,
			class: props.kelas,
			jenis: props.jenis,
			questionText: form.questionText,
			questionType: form.questionType.value,
			image: form.image,
			score: form.score,
			optionA: isEssay ? "" : form.options.A,
			optionB: isEssay ? "" : form.options.B,
			optionC: isEssay ? null : form.options.C || null,
			optionD: isEssay ? null : form.options.D || null,
			optionE: isEssay ? null : form.options.E || null,
			correctOption: isEssay ? "" : (form.correctOption ?? "")
		};

		saving.value = true;
		try {
			if (props.editingId) {
				await invoke("update_question", { id: props.editingId, input });
				openModel.value = false;
			} else {
				await invoke("create_question", { input });
				if (keepOpen) {
					savedMessage.value = "Soal tersimpan. Lanjut input soal berikutnya.";
					Object.assign(form, emptyQuestionFields());
					imagePreview.value = "";
				} else {
					openModel.value = false;
				}
			}
			emit("saved");
		} catch (e) {
			formError.value = e instanceof Error ? e.message : String(e);
		} finally {
			saving.value = false;
		}
	};
</script>

<template>
	<UModal
		v-model:open="openModel"
		:title="editingId ? 'Edit Soal' : 'Tambah Soal'"
		:ui="{ content: 'max-w-2xl', header: 'py-2', body: 'py-3' }">
		<template #title>
			<div class="flex items-center justify-between gap-3 w-full">
				<span>{{ editingId ? "Edit Soal" : "Tambah Soal" }}</span>
				<USelectMenu
					v-model="form.questionType"
					:items="questionTypeItems"
					size="sm"
					class="w-40" />
			</div>
		</template>

		<template #body>
			<form class="grid gap-2.5 sm:grid-cols-2" @submit.prevent="submitForm(false)">

				<UFormField class="sm:col-span-2">
					<template #label>
						<div class="flex items-center justify-between gap-3 w-full flex-nowrap">
							<span class="font-medium text-sm shrink-0">Pertanyaan</span>

							<div class="flex items-center gap-2 flex-nowrap shrink-0">
								<template v-if="form.questionType.value !== 'essay'">
									<span class="text-xs text-muted shrink-0">Kunci</span>
									<USelectMenu
										v-model="form.correctOption"
										:items="['A', 'B', 'C', 'D', 'E']"
										placeholder="—"
										size="xs"
										class="w-14" />
								</template>

								<span class="text-xs text-muted shrink-0">Skor</span>
								<UInputNumber
									v-model="form.score"
									:min="0"
									size="xs"
									class="w-16" />
							</div>
						</div>
					</template>

					<SoalEditor
						v-model="form.questionText"
						placeholder="Tulis pertanyaan di sini..."
						@image-picked="onImagePicked" />
				</UFormField>

				<div v-if="imagePreview" class="sm:col-span-2 flex items-center gap-3">
					<img :src="imagePreview" class="h-12 rounded border border-default" alt="">
					<UButton
						size="xs"
						variant="soft"
						color="error"
						icon="lucide:trash-2"
						@click="removeImage">
						Hapus Gambar
					</UButton>
				</div>

				<div v-if="form.questionType.value !== 'essay'" class="sm:col-span-2 grid grid-cols-1 sm:grid-cols-2 gap-2">
					<UInput
						v-for="key in optionKeys"
						:key="key"
						v-model="form.options[key]"
						size="md"
						variant="outline"
						color="primary"
						:placeholder="requiredOptionKeys.includes(key) ? 'Wajib' : 'Opsional'"
						class="w-full">
						<template #leading>
							<span class="font-bold text-primary text-sm">{{ key }}</span>
						</template>
					</UInput>
				</div>

				<UAlert
					v-if="formError"
					color="error"
					variant="subtle"
					class="sm:col-span-2"
					:title="formError" />
				<UAlert
					v-if="savedMessage"
					color="success"
					variant="subtle"
					class="sm:col-span-2"
					:title="savedMessage" />

				<div class="sm:col-span-2 flex gap-2">
					<UButton type="submit" size="sm" :loading="saving">
						{{ editingId ? "Simpan Perubahan" : "Simpan" }}
					</UButton>
					<UButton
						v-if="!editingId"
						type="button"
						size="sm"
						variant="soft"
						:loading="saving"
						@click="submitForm(true)">
						Simpan & Lanjutkan
					</UButton>
				</div>
			</form>
		</template>
	</UModal>
</template>
