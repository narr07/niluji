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

	const route = useRoute();
	const kelas = computed(() => route.params.kelas as string);
	const pelajaran = computed(() => decodeURIComponent(route.params.pelajaran as string));

	const subjects = ref<Subject[]>([]);
	const subject = computed(() => subjects.value.find((s) => s.name === pelajaran.value));
	const subjectLabel = computed(() => subject.value?.code || pelajaran.value);

	onMounted(async () => {
		subjects.value = await invoke<Subject[]>("list_subjects");
	});

	provide("hasilSubject", subject);

	const breadcrumbItems = computed(() => [
		{ label: "Hasil Ujian", icon: "lucide:bar-chart-3", to: "/hasil" },
		{ label: `Kelas ${kelas.value}`, to: `/hasil/${kelas.value}` },
		{ label: subjectLabel.value }
	]);
</script>

<template>
	<UDashboardPanel id="hasil-detail">
		<template #header>
			<UDashboardNavbar :title="`Kelas ${kelas} — ${subjectLabel}`">
				<template #leading>
					<UDashboardSidebarCollapse />
					<UButton icon="lucide:arrow-left" variant="ghost" :to="`/hasil/${kelas}`" />
				</template>
			</UDashboardNavbar>
		</template>

		<template #body>
			<UBreadcrumb :items="breadcrumbItems" class="mb-4" />
			<NuxtPage />
		</template>
	</UDashboardPanel>
</template>
