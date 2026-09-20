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

	interface QuestionSummary {
		subject: string
		class: string | null
		jenis: string | null
	}

	const route = useRoute();
	const kelas = computed(() => route.params.kelas as string);

	const breadcrumbItems = computed(() => [
		{ label: "Bank Soal", icon: "lucide:list-checks", to: "/soal" },
		{ label: `Kelas ${kelas.value}` }
	]);

	const subjects = ref<Subject[]>([]);
	const questions = ref<QuestionSummary[]>([]);

	onMounted(async () => {
		[subjects.value, questions.value] = await Promise.all([
			invoke<Subject[]>("list_subjects"),
			invoke<QuestionSummary[]>("list_questions")
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
			<SoalPelajaranGrid :kelas="kelas" :subjects="subjects" :questions="questions" />
		</template>
	</UDashboardPanel>
</template>
