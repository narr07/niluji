<script lang="ts" setup>
	definePageMeta({
		hideFromNav: true
	});

	const route = useRoute();
	const kelas = computed(() => route.params.kelas as string);
	const pelajaran = computed(() => decodeURIComponent(route.params.pelajaran as string));

	const breadcrumbItems = computed(() => [
		{ label: "Bank Soal", icon: "lucide:list-checks", to: "/soal" },
		{ label: `Kelas ${kelas.value}`, to: `/soal/${kelas.value}` },
		{ label: pelajaran.value }
	]);
</script>

<template>
	<UDashboardPanel id="soal-pelajaran">
		<template #header>
			<UDashboardNavbar :title="`Kelas ${kelas} — ${pelajaran}`">
				<template #leading>
					<UDashboardSidebarCollapse />
					<UButton icon="lucide:arrow-left" variant="ghost" :to="`/soal/${kelas}`" />
				</template>
			</UDashboardNavbar>
		</template>

		<template #body>
			<UBreadcrumb :items="breadcrumbItems" class="mb-4" />
			<SoalJenisGrid :kelas="kelas" :pelajaran="pelajaran" />
		</template>
	</UDashboardPanel>
</template>
