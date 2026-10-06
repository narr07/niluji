<script lang="ts" setup>
	definePageMeta({
		hideFromNav: true
	});

	const route = useRoute();
	const kelas = computed(() => route.params.kelas as string);
	const pelajaran = computed(() => decodeURIComponent(route.params.pelajaran as string));

	const { subject, label: subjectLabel } = useSubjectByName(pelajaran);

	provide("hasilSubject", subject);

	const title = computed(() => (subjectLabel.value ? `Kelas ${kelas.value} — ${subjectLabel.value}` : `Kelas ${kelas.value}`));
	const breadcrumbItems = computed(() => [
		{ label: "Hasil Ujian", icon: "lucide:bar-chart-3", to: "/hasil" },
		{ label: `Kelas ${kelas.value}`, to: `/hasil/${kelas.value}` },
		...(subjectLabel.value ? [{ label: subjectLabel.value }] : [])
	]);
</script>

<template>
	<UDashboardPanel id="hasil-detail">
		<template #header>
			<UDashboardNavbar :title="title">
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
