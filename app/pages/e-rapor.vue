<script lang="ts" setup>
	definePageMeta({
		name: "E-Rapor",
		icon: "lucide:file-badge",
		top: true,
		description: "Rapor elektronik siswa"
	});
</script>

<template>
	<UDashboardPanel id="e-rapor">
		<template #header>
			<UDashboardNavbar title="E-Rapor">
				<template #leading>
					<UDashboardSidebarCollapse />
				</template>
			</UDashboardNavbar>
		</template>

		<template #body>
			<div class="flex flex-col items-center justify-center py-24 text-center gap-3">
				<UIcon name="lucide:construction" class="size-12 text-muted" />
				<h2 class="text-xl font-semibold">
					Coming Soon
				</h2>
				<p class="text-muted max-w-sm">
					Fitur E-Rapor sedang dalam pengembangan.
				</p>
			</div>
		</template>
	</UDashboardPanel>
</template>
