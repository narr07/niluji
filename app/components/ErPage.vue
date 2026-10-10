<script setup lang="ts">
	// Kerangka halaman ala niluji: UDashboardPanel + UDashboardNavbar (judul, tombol lipat
	// sidebar, opsional tombol kembali) dan slot #right untuk tombol aksi di navbar.
	defineProps<{ id: string, title: string, back?: string }>();
</script>

<template>
	<UDashboardPanel :id="id">
		<template #header>
			<UDashboardNavbar :title="title" :ui="{ right: 'gap-2' }">
				<template #leading>
					<UDashboardSidebarCollapse />
					<UButton
						v-if="back"
						icon="lucide:arrow-left"
						variant="ghost"
						:to="back" />
				</template>
				<template v-if="$slots.title" #title>
					<slot name="title" />
				</template>
				<template #right>
					<slot name="right" />
				</template>
			</UDashboardNavbar>
		</template>

		<template #body>
			<slot />
		</template>
	</UDashboardPanel>
</template>
