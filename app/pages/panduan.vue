<script lang="ts" setup>
	definePageMeta({
		hideFromNav: true
	});
</script>

<template>
	<UDashboardPanel id="panduan">
		<template #header>
			<UDashboardNavbar title="Panduan Guru">
				<template #leading>
					<UDashboardSidebarCollapse />
				</template>
			</UDashboardNavbar>
		</template>

		<template #body>
			<PengaturanPanduan />
		</template>
	</UDashboardPanel>
</template>
