<script lang="ts" setup>
	import { invoke } from "@tauri-apps/api/core";

	definePageMeta({
		name: "Bank Soal",
		icon: "lucide:list-checks",
		category: "cbt",
		order: 2,
		description: "Soal per kelas dan mata pelajaran"
	});

	const classes = ref<string[]>([]);

	onMounted(async () => {
		classes.value = await invoke<string[]>("list_classes");
	});
</script>

<template>
	<UDashboardPanel id="soal">
		<template #header>
			<UDashboardNavbar title="Bank Soal">
				<template #leading>
					<UDashboardSidebarCollapse />
				</template>
			</UDashboardNavbar>
		</template>

		<template #body>
			<SoalKelasGrid :classes="classes" />
		</template>
	</UDashboardPanel>
</template>
