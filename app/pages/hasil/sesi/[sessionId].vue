<script lang="ts" setup>
	import { invoke } from "@tauri-apps/api/core";

	definePageMeta({
		hideFromNav: true
	});

	interface AnswerDetail {
		questionId: number
		questionText: string
		questionType: string
		studentAnswer: string | null
		correctAnswer: string | null
		isCorrect: boolean
		essayScore: number | null
		maxScore: number
	}

	interface SessionDetail {
		studentName: string
		examTitle: string
		class: string | null
		subject: string
		score: number | null
		pgSubmittedAt: number | null
		submittedAt: number | null
		items: AnswerDetail[]
	}

	const route = useRoute();
	const sessionId = computed(() => Number(route.params.sessionId));

	const detail = ref<SessionDetail | null>(null);
	const loading = ref(false);
	const activeTab = ref("0");

	const breadcrumbItems = computed(() => [
		{ label: "Hasil Ujian", icon: "lucide:bar-chart-3", to: "/hasil" },
		...(detail.value?.class
			? [
				{ label: `Kelas ${detail.value.class}`, to: `/hasil/${detail.value.class}` },
				{ label: detail.value.subject, to: `/hasil/${detail.value.class}/${encodeURIComponent(detail.value.subject)}` }
			]
			: []),
		{ label: detail.value?.studentName ?? "Detail" }
	]);

	const pgItems = computed(() => detail.value?.items.filter((i) => i.questionType !== "essay") ?? []);
	const essayItems = computed(() => detail.value?.items.filter((i) => i.questionType === "essay") ?? []);

	const tabs = computed(() => [
		{ label: `Pilihan Ganda (${pgItems.value.length})`, slot: "pg" as const },
		{ label: `Esai (${essayItems.value.length})`, slot: "essay" as const }
	]);

	const load = async () => {
		loading.value = true;
		try {
			detail.value = await invoke<SessionDetail>("get_exam_session_detail", { sessionId: sessionId.value });
		} finally {
			loading.value = false;
		}
	};

	// Local editable draft per essay answer, seeded from the saved essayScore (or empty if
	// not graded yet) so typing doesn't fight with the fetched value.
	const drafts = reactive<Record<number, number | undefined>>({});
	watch(essayItems, (items) => {
		for (const item of items) {
			if (!(item.questionId in drafts)) drafts[item.questionId] = item.essayScore ?? undefined;
		}
	}, { immediate: true });

	const savingId = ref<number | null>(null);
	const gradeError = ref("");

	const saveGrade = async (item: AnswerDetail) => {
		gradeError.value = "";
		const value = drafts[item.questionId];
		if (value === undefined || Number.isNaN(value)) {
			gradeError.value = "Nilai wajib diisi.";
			return;
		}
		savingId.value = item.questionId;
		try {
			await invoke("grade_essay_answer", { sessionId: sessionId.value, questionId: item.questionId, score: value });
			await load();
		} catch (e) {
			gradeError.value = e instanceof Error ? e.message : String(e);
		} finally {
			savingId.value = null;
		}
	};

	const downloadCsv = (filename: string, rows: (string | number)[][]) => {
		const csv = rows.map((r) => r.map((v) => `"${String(v).replace(/"/g, "\"\"")}"`).join(",")).join("\n");
		const blob = new Blob([csv], { type: "text/csv;charset=utf-8;" });
		const url = URL.createObjectURL(blob);
		const a = document.createElement("a");
		a.href = url;
		a.download = filename;
		a.click();
		URL.revokeObjectURL(url);
	};

	const exportDetail = () => {
		if (!detail.value) return;
		downloadCsv(`hasil-${detail.value.studentName}.csv`, [
			["No", "Tipe", "Soal", "Jawaban Siswa", "Kunci/Nilai", "Status"],
			...detail.value.items.map((it, i) => [
				i + 1,
				it.questionType === "essay" ? "Esai" : "PG",
				it.questionText,
				it.studentAnswer ?? "-",
				it.questionType === "essay" ? `${it.essayScore ?? "-"} / ${it.maxScore}` : (it.correctAnswer ?? "-"),
				it.questionType === "essay" ? (it.essayScore === null ? "Belum dinilai" : "Sudah dinilai") : (it.isCorrect ? "Benar" : "Salah")
			])
		]);
	};

	onMounted(load);
</script>

<template>
	<UDashboardPanel id="hasil-sesi">
		<template #header>
			<UDashboardNavbar :title="detail ? `Hasil: ${detail.studentName}` : 'Hasil'">
				<template #leading>
					<UDashboardSidebarCollapse />
					<UButton icon="lucide:arrow-left" variant="ghost" @click="$router.back()" />
				</template>

				<template #right>
					<UButton
						icon="lucide:download"
						variant="soft"
						:disabled="!detail"
						@click="exportDetail">
						Export (CSV)
					</UButton>
				</template>
			</UDashboardNavbar>
		</template>

		<template #body>
			<UBreadcrumb :items="breadcrumbItems" class="mb-4" />

			<div v-if="detail" class="space-y-6">
				<UCard>
					<div class="flex items-center justify-between flex-wrap gap-4">
						<div>
							<p class="text-muted text-sm">
								{{ detail.examTitle }}
							</p>
							<p class="text-2xl font-bold">
								Nilai PG: {{ detail.score?.toFixed(0) ?? "-" }}
							</p>
						</div>
						<div class="flex gap-2">
							<UBadge v-if="detail.pgSubmittedAt" color="success" variant="subtle">
								Sesi PG selesai
							</UBadge>
							<UBadge v-if="detail.submittedAt" color="success" variant="subtle">
								Ujian selesai
							</UBadge>
							<UBadge v-else color="warning" variant="subtle">
								Belum dikumpulkan
							</UBadge>
						</div>
					</div>
					<p class="text-sm text-muted mt-2">
						Nilai esai dinilai manual di bawah dan tidak ikut masuk ke "Nilai PG" di atas.
					</p>
				</UCard>

				<UTabs v-model="activeTab" :items="tabs">
					<template #pg>
						<div class="space-y-2 mt-4">
							<div
								v-for="(item, i) in pgItems"
								:key="item.questionId"
								class="p-3 rounded-md border"
								:class="item.isCorrect ? 'border-success/50 bg-success/5' : 'border-error/50 bg-error/5'"
							>
								<p class="font-medium">
									{{ i + 1 }}. <span v-html="renderSoalMarkdown(item.questionText)" />
								</p>
								<p class="text-sm mt-1">
									Jawaban siswa: <strong>{{ item.studentAnswer ?? "(tidak dijawab)" }}</strong>
									&middot; Kunci: <strong>{{ item.correctAnswer ?? "-" }}</strong>
								</p>
								<UBadge :color="item.isCorrect ? 'success' : 'error'" variant="subtle" class="mt-1">
									{{ item.isCorrect ? "Benar" : "Salah" }}
								</UBadge>
							</div>
							<p v-if="!pgItems.length" class="text-center text-muted py-10">
								Tidak ada soal Pilihan Ganda.
							</p>
						</div>
					</template>

					<template #essay>
						<div class="space-y-3 mt-4">
							<div v-for="(item, i) in essayItems" :key="item.questionId" class="p-3 rounded-md border border-default">
								<p class="font-medium">
									{{ i + 1 }}. <span v-html="renderSoalMarkdown(item.questionText)" />
								</p>
								<p class="text-sm mt-1 whitespace-pre-wrap">
									Jawaban siswa: <strong>{{ item.studentAnswer || "(tidak dijawab)" }}</strong>
								</p>

								<div class="flex items-center gap-3 mt-3">
									<UFormField label="Nilai" :description="`Maks. ${item.maxScore}`">
										<UInputNumber
											v-model="drafts[item.questionId]"
											:min="0"
											:max="item.maxScore"
											class="max-w-32" />
									</UFormField>
									<UButton
										size="sm"
										variant="soft"
										:loading="savingId === item.questionId"
										class="mt-6"
										@click="saveGrade(item)">
										Simpan Nilai
									</UButton>
									<UBadge
										v-if="item.essayScore !== null"
										color="success"
										variant="subtle"
										class="mt-6">
										Sudah dinilai
									</UBadge>
									<UBadge
										v-else
										color="warning"
										variant="subtle"
										class="mt-6">
										Perlu dinilai
									</UBadge>
								</div>
							</div>
							<p v-if="!essayItems.length" class="text-center text-muted py-10">
								Tidak ada soal Esai.
							</p>
						</div>
					</template>
				</UTabs>

				<UAlert
					v-if="gradeError"
					color="error"
					variant="subtle"
					:title="gradeError" />
			</div>
			<div v-else-if="loading" class="text-center text-muted py-20">
				Memuat...
			</div>
		</template>
	</UDashboardPanel>
</template>
