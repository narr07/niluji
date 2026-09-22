<script lang="ts" setup>
	import { invoke } from "@tauri-apps/api/core";

	const props = defineProps<{
		kelas: string;
		subjectId: number | undefined;
		jenis: string;
	}>();

	interface QuestionStat {
		id: number;
		questionText: string;
		correctCount: number;
		incorrectCount: number;
	}

	interface StudentStat {
		sessionId: number;
		name: string;
		correctCount: number;
		incorrectCount: number;
	}

	const questionStats = ref<QuestionStat[]>([]);
	const studentStats = ref<StudentStat[]>([]);
	const loading = ref(false);

	const load = async () => {
		if (!props.subjectId) return;
		loading.value = true;
		try {
			const result = await invoke<{ questionStats: QuestionStat[]; studentStats: StudentStat[] }>("get_exam_analytics", {
				class: props.kelas,
				subjectId: props.subjectId,
				jenis: props.jenis,
			});
			questionStats.value = result.questionStats;
			studentStats.value = result.studentStats;
		} finally {
			loading.value = false;
		}
	};

	watch(() => [props.subjectId, props.jenis], load, { immediate: true });

	const categories = {
		benar: { name: "Benar", color: "#22c55e" },
		salah: { name: "Salah", color: "#ef4444" },
	};

	const numberedQuestions = computed(() =>
		[...questionStats.value].sort((a, b) => a.id - b.id).map((q, i) => ({ ...q, nomor: i + 1 }))
	);

	const questionDifficultyData = computed(() =>
		[...numberedQuestions.value]
			.sort((a, b) => b.incorrectCount - a.incorrectCount)
			.map((q) => ({ label: `Soal ${q.nomor}`, benar: q.correctCount, salah: q.incorrectCount }))
	);

	const studentRankingData = computed(() =>
		[...studentStats.value]
			.sort((a, b) => b.correctCount - a.correctCount)
			.map((s) => ({ label: s.name, benar: s.correctCount, salah: s.incorrectCount }))
	);

	const questionChartHeight = computed(() => Math.max(120, questionDifficultyData.value.length * 24));
	const studentChartHeight = computed(() => Math.max(120, studentRankingData.value.length * 24));

	const pct = (benar: number, salah: number) => {
		const total = benar + salah;
		return total > 0 ? Math.round((benar / total) * 100) : 0;
	};

	const easiestQuestions = computed(() => questionDifficultyData.value.slice(-5).reverse());
	const hardestQuestions = computed(() => questionDifficultyData.value.slice(0, 5));
	const topStudents = computed(() => studentRankingData.value.slice(0, 5));
	const bottomStudents = computed(() => studentRankingData.value.slice(-5).reverse());
</script>

<template>
	<div class="space-y-6">
		<UCard>
			<template #header>
				<div class="flex items-center gap-2">
					<UIcon name="i-lucide-bar-chart-3" class="size-4 text-muted" />
					<span class="font-semibold">Tingkat Kesulitan Soal</span>
				</div>
			</template>
			<BarChart
				v-if="questionDifficultyData.length"
				:data="questionDifficultyData"
				:categories="categories"
				:height="questionChartHeight"
				:stacked="true"
				orientation="horizontal"
				x-axis="label"
				:y-axis="['benar', 'salah']"
			/>
			<div v-else class="flex flex-col items-center gap-2 py-10 text-muted">
				<UIcon name="i-lucide-bar-chart-3" class="size-6" />
				<span>Belum ada siswa yang menyelesaikan sesi Pilihan Ganda.</span>
			</div>
		</UCard>

		<UCard>
			<template #header>
				<div class="flex items-center gap-2">
					<UIcon name="i-lucide-trophy" class="size-4 text-muted" />
					<span class="font-semibold">Peringkat Siswa</span>
				</div>
			</template>
			<BarChart
				v-if="studentRankingData.length"
				:data="studentRankingData"
				:categories="categories"
				:height="studentChartHeight"
				:stacked="true"
				orientation="horizontal"
				x-axis="label"
				:y-axis="['benar', 'salah']"
			/>
			<div v-else class="flex flex-col items-center gap-2 py-10 text-muted">
				<UIcon name="i-lucide-trophy" class="size-6" />
				<span>Belum ada siswa yang menyelesaikan sesi Pilihan Ganda.</span>
			</div>
		</UCard>

		<div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
			<UCard>
				<template #header>
					<div class="flex items-center gap-2">
						<UIcon name="i-lucide-smile" class="size-4 text-success" />
						<span class="font-semibold text-sm">5 Soal Termudah</span>
					</div>
				</template>
				<ul class="space-y-3 text-sm">
					<li v-for="(q, i) in easiestQuestions" :key="q.label">
						<div class="flex items-center justify-between mb-1">
							<span class="font-medium">{{ q.label }}</span>
							<span class="text-xs text-muted">{{ pct(q.benar, q.salah) }}% benar</span>
						</div>
						<UProgress :model-value="pct(q.benar, q.salah)" size="sm" color="success" />
					</li>
				</ul>
			</UCard>

			<UCard>
				<template #header>
					<div class="flex items-center gap-2">
						<UIcon name="i-lucide-frown" class="size-4 text-error" />
						<span class="font-semibold text-sm">5 Soal Tersulit</span>
					</div>
				</template>
				<ul class="space-y-3 text-sm">
					<li v-for="q in hardestQuestions" :key="q.label">
						<div class="flex items-center justify-between mb-1">
							<span class="font-medium">{{ q.label }}</span>
							<span class="text-xs text-muted">{{ pct(q.benar, q.salah) }}% benar</span>
						</div>
						<UProgress :model-value="pct(q.benar, q.salah)" size="sm" color="error" />
					</li>
				</ul>
			</UCard>

			<UCard>
				<template #header>
					<div class="flex items-center gap-2">
						<UIcon name="i-lucide-trending-up" class="size-4 text-success" />
						<span class="font-semibold text-sm">5 Siswa Nilai Tertinggi</span>
					</div>
				</template>
				<ul class="space-y-2 text-sm">
					<li v-for="(s, i) in topStudents" :key="s.label" class="flex items-center gap-2">
						<UBadge
							color="success"
							variant="subtle"
							size="sm"
							class="shrink-0 w-6 justify-center">
							{{ i + 1 }}
						</UBadge>
						<span class="truncate flex-1">{{ s.label }}</span>
						<span class="text-xs text-muted shrink-0">{{ pct(s.benar, s.salah) }}%</span>
					</li>
				</ul>
			</UCard>

			<UCard>
				<template #header>
					<div class="flex items-center gap-2">
						<UIcon name="i-lucide-trending-down" class="size-4 text-error" />
						<span class="font-semibold text-sm">5 Siswa Nilai Terendah</span>
					</div>
				</template>
				<ul class="space-y-2 text-sm">
					<li v-for="s in bottomStudents" :key="s.label" class="flex items-center gap-2">
						<UBadge
							color="error"
							variant="subtle"
							size="sm"
							class="shrink-0 w-6 justify-center">
							!
						</UBadge>
						<span class="truncate flex-1">{{ s.label }}</span>
						<span class="text-xs text-muted shrink-0">{{ pct(s.benar, s.salah) }}%</span>
					</li>
				</ul>
			</UCard>
		</div>
	</div>
</template>