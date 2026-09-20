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

	const uploadingImage = ref(false);
	const imagePreview = ref("");
	const formError = ref("");
	const savedMessage = ref("");
	const saving = ref(false);

	const questionTypeItems = [
		{ label: "Pilihan Ganda", value: "multiple_choice" },
		{ label: "Esai", value: "essay" }
	];

	const emptyQuestionFields = () => ({
		questionType: questionTypeItems[0]!,
		questionText: "",
		image: null as string | null,
		optionA: "",
		optionB: "",
		optionC: "",
		optionD: "",
		optionE: "",
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
				optionA: detail.optionA,
				optionB: detail.optionB,
				optionC: detail.optionC,
				optionD: detail.optionD,
				optionE: detail.optionE,
				correctOption: detail.correctOption || undefined,
				score: detail.score
			});
			imagePreview.value = detail.image ? await invoke<string>("load_image_data_url", { path: detail.image }) : "";
		},
		{ immediate: true }
	);

	const pickImage = async () => {
		const path = await openDialog({
			multiple: false,
			filters: [{ name: "Gambar", extensions: ["jpg", "jpeg", "png", "gif", "webp"] }]
		});
		if (!path) return;

		uploadingImage.value = true;
		try {
			const saved = await invoke<{ path: string, dataUrl: string }>("save_question_image", { path });
			form.image = saved.path;
			imagePreview.value = saved.dataUrl;
		} catch (e) {
			formError.value = e instanceof Error ? e.message : String(e);
		} finally {
			uploadingImage.value = false;
		}
	};

	const submitForm = async () => {
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
		if (!isEssay && (!form.optionA.trim() || !form.optionB.trim() || !form.correctOption)) {
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
			optionA: isEssay ? "" : form.optionA,
			optionB: isEssay ? "" : form.optionB,
			optionC: isEssay ? null : form.optionC || null,
			optionD: isEssay ? null : form.optionD || null,
			optionE: isEssay ? null : form.optionE || null,
			correctOption: isEssay ? "" : (form.correctOption ?? "")
		};

		saving.value = true;
		try {
			if (props.editingId) {
				await invoke("update_question", { id: props.editingId, input });
				openModel.value = false;
			} else {
				await invoke("create_question", { input });
				savedMessage.value = "Soal tersimpan. Lanjut input soal berikutnya.";
				Object.assign(form, emptyQuestionFields());
				imagePreview.value = "";
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
	<UModal v-model:open="openModel" :title="editingId ? 'Edit Soal' : 'Tambah Soal'" :ui="{ content: 'max-w-2xl' }">
		<template #body>
			<form class="grid gap-4 sm:grid-cols-2" @submit.prevent="submitForm">
				<UFormField label="Tipe Soal" class="sm:col-span-2">
					<USelectMenu v-model="form.questionType" :items="questionTypeItems" />
				</UFormField>

				<UFormField label="Pertanyaan" class="sm:col-span-2">
					<UTextarea v-model="form.questionText" :rows="3" />
				</UFormField>

				<div class="sm:col-span-2 flex items-center gap-3">
					<UButton
						size="sm"
						variant="soft"
						icon="lucide:image"
						:loading="uploadingImage"
						@click="pickImage">
						{{ form.image ? "Ganti Gambar" : "Tambah Gambar" }}
					</UButton>
					<img
						v-if="imagePreview"
						:src="imagePreview"
						class="h-16 rounded border border-default"
						alt="">
				</div>

				<template v-if="form.questionType.value !== 'essay'">
					<UFormField label="Pilihan A">
						<UInput v-model="form.optionA" />
					</UFormField>
					<UFormField label="Pilihan B">
						<UInput v-model="form.optionB" />
					</UFormField>
					<UFormField label="Pilihan C (opsional)">
						<UInput v-model="form.optionC" />
					</UFormField>
					<UFormField label="Pilihan D (opsional)">
						<UInput v-model="form.optionD" />
					</UFormField>
					<UFormField label="Pilihan E (opsional)">
						<UInput v-model="form.optionE" />
					</UFormField>

					<UFormField label="Kunci Jawaban">
						<USelectMenu v-model="form.correctOption" :items="['A', 'B', 'C', 'D', 'E']" placeholder="Pilih kunci" />
					</UFormField>
				</template>

				<UFormField label="Skor">
					<UInputNumber v-model="form.score" :min="0" />
				</UFormField>

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
					<UButton type="submit" :loading="saving">
						{{ editingId ? "Simpan Perubahan" : "Simpan & Lanjut Soal Berikutnya" }}
					</UButton>
					<UButton variant="soft" color="neutral" @click="openModel = false">
						Selesai
					</UButton>
				</div>
			</form>
		</template>
	</UModal>
</template>
