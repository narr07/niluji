<script lang="ts" setup>
	import { invoke } from "@tauri-apps/api/core";

	definePageMeta({
		hideFromNav: true
	});

	interface Subject {
		id: number
		name: string
		code: string | null
	}

	interface QuestionTypeRecord {
		id: number
		class: string | null
		subjectId: number | null
	}

	const route = useRoute();
	const kelas = computed(() => route.params.kelas as string);

	const breadcrumbItems = computed(() => [
		{ label: "Bank Soal", icon: "lucide:list-checks", to: "/soal" },
		{ label: `Kelas ${kelas.value}` }
	]);

	const subjects = ref<Subject[]>([]);
	const questionTypes = ref<QuestionTypeRecord[]>([]);

	onMounted(async () => {
		[subjects.value, questionTypes.value] = await Promise.all([
			invoke<Subject[]>("list_subjects"),
			// Tanpa class/subjectId, backend mengembalikan SEMUA jenis ujian terdaftar (tidak
			// difilter) — pencocokan per pelajaran+kelas dilakukan di SoalPelajaranGrid.
			invoke<QuestionTypeRecord[]>("list_question_types", { class: null, subjectId: null })
		]);
	});
</script>

<template>
	<UDashboardPanel id="soal-kelas">
		<template #header>
			<UDashboardNavbar :title="`Kelas ${kelas}`">
				<template #leading>
					<UDashboardSidebarCollapse />
					<UButton icon="lucide:arrow-left" variant="ghost" to="/soal" />
				</template>
			</UDashboardNavbar>
		</template>

		<template #body>
			<UBreadcrumb :items="breadcrumbItems" class="mb-4" />
			<SoalPelajaranGrid :kelas="kelas" :subjects="subjects" :question-types="questionTypes" />
		</template>
	</UDashboardPanel>
</template>
