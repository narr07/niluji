<script lang="ts" setup>
	import { invoke } from "@tauri-apps/api/core";

	const props = defineProps<{
		open: boolean
		questionId: number | null
	}>();

	const emit = defineEmits<{
		"update:open": [value: boolean]
	}>();

	const openModel = computed({
		get: () => props.open,
		set: (value) => emit("update:open", value)
	});

	interface QuestionDetail {
		id: number
		questionText: string
		questionType: string
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
	}

	const loading = ref(false);
	const detail = ref<QuestionDetail | null>(null);
	const imagePreview = ref("");
	const optionImagePreviews = reactive<Record<string, string>>({});

	const options = computed(() => {
		if (!detail.value) return [];
		return (
			[
				{ key: "A", text: detail.value.optionA, image: detail.value.optionAImage },
				{ key: "B", text: detail.value.optionB, image: detail.value.optionBImage },
				{ key: "C", text: detail.value.optionC, image: detail.value.optionCImage },
				{ key: "D", text: detail.value.optionD, image: detail.value.optionDImage },
				{ key: "E", text: detail.value.optionE, image: detail.value.optionEImage }
			] as const
		).filter((o) => o.text?.trim() || o.image);
	});

	watch(
		() => [props.open, props.questionId] as const,
		async ([isOpen, id]) => {
			if (!isOpen || !id) return;
			loading.value = true;
			detail.value = null;
			imagePreview.value = "";
			for (const k of Object.keys(optionImagePreviews)) optionImagePreviews[k] = "";
			try {
				detail.value = await invoke<QuestionDetail>("get_question", { id });
				if (detail.value.image) {
					imagePreview.value = await invoke<string>("load_image_data_url", { path: detail.value.image });
				}
				for (const opt of options.value) {
					if (opt.image) {
						optionImagePreviews[opt.key] = await invoke<string>("load_image_data_url", { path: opt.image });
					}
				}
			} finally {
				loading.value = false;
			}
		},
		{ immediate: true }
	);
</script>

<template>
	<UModal v-model:open="openModel" title="Preview Soal" :ui="{ content: 'max-w-xl' }">
		<template #body>
			<div v-if="loading" class="py-10 text-center text-muted">
				Memuat soal...
			</div>

			<div v-else-if="detail" class="space-y-4">
				<p class="text-xs text-muted">
					Simulasi tampilan soal ini di halaman ujian siswa.
				</p>

				<UCard>
					<template #header>
						<span class="font-medium" v-html="renderSoalMarkdown(detail.questionText, imagePreview || undefined)" />
					</template>

					<img
						v-if="imagePreview && !hasSoalImagePlaceholder(detail.questionText)"
						:src="imagePreview"
						class="max-w-full rounded-md mb-3"
						alt="">

					<div v-if="detail.questionType === 'essay'">
						<UTextarea
							:model-value="''"
							:rows="5"
							placeholder="Siswa akan menjawab dengan mengetik di sini..."
							disabled />
					</div>
					<div v-else class="space-y-2">
						<div
							v-for="opt in options"
							:key="opt.key"
							class="flex items-center justify-between gap-3 p-3 rounded-md border"
							:class="opt.key === detail.correctOption ? 'border-success bg-success/10' : 'border-default'">
							<div class="flex items-center gap-3">
								<span>{{ opt.key }}. {{ opt.text }}</span>
								<img
									v-if="optionImagePreviews[opt.key]"
									:src="optionImagePreviews[opt.key]"
									class="h-12 rounded border border-default"
									alt="">
							</div>
							<UBadge
								v-if="opt.key === detail.correctOption"
								color="success"
								variant="subtle"
								size="xs">
								Kunci Jawaban
							</UBadge>
						</div>
					</div>
				</UCard>

				<p class="text-xs text-muted">
					Skor: {{ detail.score }}
				</p>
			</div>
		</template>
	</UModal>
</template>
