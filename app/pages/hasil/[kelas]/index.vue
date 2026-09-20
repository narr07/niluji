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

	interface SessionProgress {
		class: string | null
		subject: string
	}

	const route = useRoute();
	const kelas = computed(() => route.params.kelas as string);

	const breadcrumbItems = computed(() => [
		{ label: "Hasil Ujian", icon: "lucide:bar-chart-3", to: "/hasil" },
		{ label: `Kelas ${kelas.value}` }
	]);

	const subjects = ref<Subject[]>([]);
	const sessions = ref<SessionProgress[]>([]);

	onMounted(async () => {
		[subjects.value, sessions.value] = await Promise.all([
			invoke<Subject[]>("list_subjects"),
			invoke<SessionProgress[]>("list_exam_sessions")
		]);
	});
</script>

<template>
	<UDashboardPanel id="hasil-kelas">
		<template #header>
			<UDashboardNavbar :title="`Kelas ${kelas}`">
				<template #leading>
					<UDashboardSidebarCollapse />
					<UButton icon="lucide:arrow-left" variant="ghost" to="/hasil" />
				</template>
			</UDashboardNavbar>
		</template>

		<template #body>
			<UBreadcrumb :items="breadcrumbItems" class="mb-4" />
			<HasilPelajaranGrid :kelas="kelas" :subjects="subjects" :sessions="sessions" />
		</template>
	</UDashboardPanel>
</template>
