<script lang="ts" setup>
	import { invoke } from "@tauri-apps/api/core";
	import { open as openDialog } from "@tauri-apps/plugin-dialog";

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
	type OptionKey = (typeof optionKeys)[number];
	const requiredOptionKeys: OptionKey[] = ["A", "B"];

	interface OptionField {
		text: string
		image: string | null
		imagePreview: string
	}
	const emptyOption = (): OptionField => ({ text: "", image: null, imagePreview: "" });

	const emptyQuestionFields = () => ({
		questionType: questionTypeItems[0]!,
		questionText: "",
		image: null as string | null,
		options: { A: emptyOption(), B: emptyOption(), C: emptyOption(), D: emptyOption(), E: emptyOption() } as Record<OptionKey, OptionField>,
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
				optionAImage: string | null
				optionBImage: string | null
				optionCImage: string | null
				optionDImage: string | null
				optionEImage: string | null
				correctOption: string
			}>("get_question", { id: editingId });

			const loadOption = async (text: string, image: string | null): Promise<OptionField> => ({
				text,
				image,
				imagePreview: image ? await invoke<string>("load_image_data_url", { path: image }) : ""
			});

			Object.assign(form, {
				questionType: questionTypeItems.find((t) => t.value === detail.questionType) ?? questionTypeItems[0]!,
				questionText: detail.questionText,
				image: detail.image,
				options: {
					A: await loadOption(detail.optionA, detail.optionAImage),
					B: await loadOption(detail.optionB, detail.optionBImage),
					C: await loadOption(detail.optionC, detail.optionCImage),
					D: await loadOption(detail.optionD, detail.optionDImage),
					E: await loadOption(detail.optionE, detail.optionEImage)
				},
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

	const pickOptionImage = async (key: OptionKey) => {
		const path = await openDialog({
			multiple: false,
			filters: [{ name: "Gambar", extensions: ["jpg", "jpeg", "png", "gif", "webp"] }]
		});
		if (!path) return;
		try {
			const saved = await invoke<{ path: string, dataUrl: string }>("save_question_image", { path });
			form.options[key].image = saved.path;
			form.options[key].imagePreview = saved.dataUrl;
		} catch (e) {
			formError.value = e instanceof Error ? e.message : String(e);
		}
	};

	const removeOptionImage = (key: OptionKey) => {
		form.options[key].image = null;
		form.options[key].imagePreview = "";
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
		const optionFilled = (key: OptionKey) => Boolean(form.options[key].text.trim() || form.options[key].image);
		if (!isEssay && (!optionFilled("A") || !optionFilled("B") || !form.correctOption)) {
			formError.value = "Pilihan A & B (teks atau gambar), dan kunci jawaban wajib diisi untuk soal pilihan ganda.";
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
			optionA: isEssay ? "" : form.options.A.text,
			optionB: isEssay ? "" : form.options.B.text,
			optionC: isEssay ? null : form.options.C.text || null,
			optionD: isEssay ? null : form.options.D.text || null,
			optionE: isEssay ? null : form.options.E.text || null,
			optionAImage: isEssay ? null : form.options.A.image,
			optionBImage: isEssay ? null : form.options.B.image,
			optionCImage: isEssay ? null : form.options.C.image,
			optionDImage: isEssay ? null : form.options.D.image,
			optionEImage: isEssay ? null : form.options.E.image,
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
					<div v-for="key in optionKeys" :key="key" class="space-y-1">
						<UInput
							v-model="form.options[key].text"
							size="md"
							variant="outline"
							color="primary"
							:placeholder="requiredOptionKeys.includes(key) ? 'Wajib (teks atau gambar)' : 'Opsional'"
							class="w-full">
							<template #leading>
								<span class="font-bold text-primary text-sm">{{ key }}</span>
							</template>
							<template #trailing>
								<UButton
									size="2xs"
									variant="ghost"
									color="neutral"
									icon="lucide:image-plus"
									title="Sisipkan gambar buat pilihan ini"
									@click="pickOptionImage(key)" />
							</template>
						</UInput>
						<div v-if="form.options[key].imagePreview" class="flex items-center gap-2 pl-7">
							<img :src="form.options[key].imagePreview" class="h-10 rounded border border-default" alt="">
							<UButton
								size="2xs"
								variant="ghost"
								color="error"
								icon="lucide:x"
								@click="removeOptionImage(key)" />
						</div>
					</div>
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
