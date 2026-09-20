<script lang="ts" setup>
	definePageMeta({
		name: "Tentang",
		icon: "lucide:info",
		category: "settings",
		order: 2,
		description: "Info aplikasi, versi, dan pembuat"
	});
</script>

<template>
	<UDashboardPanel id="tentang">
		<template #header>
			<UDashboardNavbar title="Tentang">
				<template #leading>
					<UDashboardSidebarCollapse />
				</template>
			</UDashboardNavbar>
		</template>

		<template #body>
			<PengaturanTentang />
		</template>
	</UDashboardPanel>
</template>
