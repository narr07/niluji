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
		correctOption: string
	}

	const loading = ref(false);
	const detail = ref<QuestionDetail | null>(null);
	const imagePreview = ref("");

	const options = computed(() => {
		if (!detail.value) return [];
		return (
			[
				{ key: "A", text: detail.value.optionA },
				{ key: "B", text: detail.value.optionB },
				{ key: "C", text: detail.value.optionC },
				{ key: "D", text: detail.value.optionD },
				{ key: "E", text: detail.value.optionE }
			] as const
		).filter((o) => o.text?.trim());
	});

	watch(
		() => [props.open, props.questionId] as const,
		async ([isOpen, id]) => {
			if (!isOpen || !id) return;
			loading.value = true;
			detail.value = null;
			imagePreview.value = "";
			try {
				detail.value = await invoke<QuestionDetail>("get_question", { id });
				if (detail.value.image) {
					imagePreview.value = await invoke<string>("load_image_data_url", { path: detail.value.image });
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
						<span class="font-medium" v-html="renderSoalMarkdown(detail.questionText)" />
					</template>

					<img
						v-if="imagePreview"
						:src="imagePreview"
						class="max-w-full rounded-md mb-3"
						alt="">

					<div v-if="detail.questionType === 'essay'">
						<UTextarea :model-value="''" :rows="5" placeholder="Siswa akan menjawab dengan mengetik di sini..." disabled />
					</div>
					<div v-else class="space-y-2">
						<div
							v-for="opt in options"
							:key="opt.key"
							class="flex items-center justify-between gap-3 p-3 rounded-md border"
							:class="opt.key === detail.correctOption ? 'border-success bg-success/10' : 'border-default'">
							<span>{{ opt.key }}. {{ opt.text }}</span>
							<UBadge v-if="opt.key === detail.correctOption" color="success" variant="subtle" size="xs">
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
