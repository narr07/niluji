<script lang="ts" setup>
	import { invoke } from "@tauri-apps/api/core";

	definePageMeta({
		name: "Bank Soal",
		icon: "lucide:list-checks",
		category: "cbt",
		order: 2,
		description: "Soal per kelas dan mata pelajaran"
	});

	interface QuestionTypeRecord {
		id: number
		class: string | null
		subjectId: number | null
	}

	const classes = ref<string[]>([]);
	const createOpen = ref(false);
	const subjects = ref<{ id: number }[]>([]);
	const questionTypes = ref<QuestionTypeRecord[]>([]);
	const questions = ref<{ class: string | null }[]>([]);

	// Ringkasan per kelas untuk kartu: jenis ujian yang berlaku di kelas itu (class NULL = berlaku
	// untuk semua kelas), berapa mapel yang sudah punya jenis ujian (aturan pencocokan sama dengan
	// badge di SoalPelajaranGrid), dan jumlah soal.
	const stats = computed(() => {
		const result: Record<string, { mapel: number, jenis: number, soal: number }> = {};
		for (const kelas of classes.value) {
			const jenisKelas = questionTypes.value.filter((jt) => jt.class === null || jt.class === kelas);
			result[kelas] = {
				jenis: jenisKelas.length,
				mapel: subjects.value.filter((s) => jenisKelas.some((jt) => jt.subjectId === null || jt.subjectId === s.id)).length,
				soal: questions.value.filter((q) => q.class === kelas).length
			};
		}
		return result;
	});

	onMounted(async () => {
		[classes.value, subjects.value, questionTypes.value, questions.value] = await Promise.all([
			invoke<string[]>("list_classes"),
			invoke<{ id: number }[]>("list_subjects"),
			invoke<QuestionTypeRecord[]>("list_question_types", { class: null, subjectId: null }),
			invoke<{ class: string | null }[]>("list_questions")
		]);
	});
</script>

<template>
	<UDashboardPanel id="soal">
		<template #header>
			<UDashboardNavbar title="Bank Soal">
				<template #leading>
					<UDashboardSidebarCollapse />
				</template>
				<template #right>
					<UButton icon="i-lucide-plus" @click="createOpen = true">
						Buat Jenis Ujian
					</UButton>
				</template>
			</UDashboardNavbar>
		</template>

		<template #body>
			<SoalKelasGrid :classes="classes" :stats="stats" :total-subjects="subjects.length" />
			<SoalBuatJenisSlideover v-model:open="createOpen" />
		</template>
	</UDashboardPanel>
</template>
