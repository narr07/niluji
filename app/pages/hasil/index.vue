<script lang="ts" setup>
	import { invoke } from "@tauri-apps/api/core";

	definePageMeta({
		name: "Hasil Ujian",
		icon: "lucide:bar-chart-3",
		category: "cbt",
		order: 4,
		description: "Hasil ujian per kelas dan mata pelajaran"
	});

	const classes = ref<string[]>([]);

	onMounted(async () => {
		classes.value = await invoke<string[]>("list_classes");
	});
</script>

<template>
	<UDashboardPanel id="hasil">
		<template #header>
			<UDashboardNavbar title="Hasil Ujian">
				<template #leading>
					<UDashboardSidebarCollapse />
				</template>
			</UDashboardNavbar>
		</template>

		<template #body>
			<UPageHeader
				title="Hasil Ujian Siswa"
				description="Pilih kelas untuk melihat rekap nilai, progres pengerjaan, dan analisis soal."
			/>
			<HasilKelasGrid :classes="classes" />
		</template>
	</UDashboardPanel>
</template>
