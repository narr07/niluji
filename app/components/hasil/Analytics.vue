<script lang="ts" setup>
	import { invoke } from "@tauri-apps/api/core";

	const props = defineProps<{
		kelas: string
		subjectId: number | undefined
		jenis: string
	}>();

	interface QuestionStat {
		id: number
		questionText: string
		correctCount: number
		incorrectCount: number
	}

	interface StudentStat {
		sessionId: number
		name: string
		correctCount: number
		incorrectCount: number
	}

	const questionStats = ref<QuestionStat[]>([]);
	const studentStats = ref<StudentStat[]>([]);
	const loading = ref(false);

	const load = async () => {
		if (!props.subjectId) return;
		loading.value = true;
		try {
			const result = await invoke<{ questionStats: QuestionStat[], studentStats: StudentStat[] }>("get_exam_analytics", {
				class: props.kelas,
				subjectId: props.subjectId,
				jenis: props.jenis
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
		salah: { name: "Salah", color: "#ef4444" }
	};

	// "Soal N" is numbered once, stably, before any difficulty sorting — otherwise which
	// question is "Soal 1" would shift around every time the data refreshes.
	const numberedQuestions = computed(() =>
		[...questionStats.value].sort((a, b) => a.id - b.id).map((q, i) => ({ ...q, nomor: i + 1 }))
	);

	// Komponen 1: soal paling banyak dijawab salah tampil paling atas.
	const questionDifficultyData = computed(() =>
		[...numberedQuestions.value]
			.sort((a, b) => b.incorrectCount - a.incorrectCount)
			.map((q) => ({ label: `Soal ${q.nomor}`, benar: q.correctCount, salah: q.incorrectCount }))
	);

	// Komponen 2: siswa dengan nilai tertinggi tampil paling atas.
	const studentRankingData = computed(() =>
		[...studentStats.value]
			.sort((a, b) => b.correctCount - a.correctCount)
			.map((s) => ({ label: s.name, benar: s.correctCount, salah: s.incorrectCount }))
	);

	const questionChartHeight = computed(() => Math.max(120, questionDifficultyData.value.length * 24));
	const studentChartHeight = computed(() => Math.max(120, studentRankingData.value.length * 24));

	// Komponen 3: 4 kartu ringkasan, semuanya turunan dari data komponen 1 & 2 di atas.
	const easiestQuestions = computed(() => questionDifficultyData.value.slice(-5));
	const hardestQuestions = computed(() => questionDifficultyData.value.slice(0, 5));
	const topStudents = computed(() => studentRankingData.value.slice(0, 5));
	const bottomStudents = computed(() => studentRankingData.value.slice(-5));
</script>

<template>
	<div class="space-y-6">
		<UCard>
			<template #header>
				<span class="font-semibold">Tingkat Kesulitan Soal</span>
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
			<p v-else class="text-center text-muted py-10">
				Belum ada siswa yang menyelesaikan sesi Pilihan Ganda.
			</p>
		</UCard>

		<UCard>
			<template #header>
				<span class="font-semibold">Peringkat Siswa</span>
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
			<p v-else class="text-center text-muted py-10">
				Belum ada siswa yang menyelesaikan sesi Pilihan Ganda.
			</p>
		</UCard>

		<div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
			<UCard>
				<template #header>
					<span class="font-semibold text-sm">5 Soal Termudah</span>
				</template>
				<ul class="space-y-1 text-sm">
					<li v-for="q in easiestQuestions" :key="q.label" class="flex justify-between">
						<span>{{ q.label }}</span>
						<span class="text-muted">{{ q.benar }} benar / {{ q.salah }} salah</span>
					</li>
				</ul>
			</UCard>

			<UCard>
				<template #header>
					<span class="font-semibold text-sm">5 Soal Tersulit</span>
				</template>
				<ul class="space-y-1 text-sm">
					<li v-for="q in hardestQuestions" :key="q.label" class="flex justify-between">
						<span>{{ q.label }}</span>
						<span class="text-muted">{{ q.benar }} benar / {{ q.salah }} salah</span>
					</li>
				</ul>
			</UCard>

			<UCard>
				<template #header>
					<span class="font-semibold text-sm">5 Siswa Nilai Tertinggi</span>
				</template>
				<ul class="space-y-1 text-sm">
					<li v-for="s in topStudents" :key="s.label" class="flex justify-between">
						<span class="truncate">{{ s.label }}</span>
						<span class="text-muted shrink-0 ml-2">{{ s.benar }} benar / {{ s.salah }} salah</span>
					</li>
				</ul>
			</UCard>

			<UCard>
				<template #header>
					<span class="font-semibold text-sm">5 Siswa Nilai Terendah</span>
				</template>
				<ul class="space-y-1 text-sm">
					<li v-for="s in bottomStudents" :key="s.label" class="flex justify-between">
						<span class="truncate">{{ s.label }}</span>
						<span class="text-muted shrink-0 ml-2">{{ s.benar }} benar / {{ s.salah }} salah</span>
					</li>
				</ul>
			</UCard>
		</div>
	</div>
</template>
