<script setup lang="ts">
	// Menu yang sudah ada di sidebar tapi halamannya belum dibuat.
	const route = useRoute();
	const menu = computed(() => findMenuItem(route.path));
</script>

<template>
	<ErPage id="segera" :title="menu ? String(menu.label) : 'Halaman tidak ditemukan'">
		<div class="flex flex-col items-center justify-center gap-3 py-24 text-center">
			<UIcon :name="menu ? 'lucide:construction' : 'lucide:circle-help'" class="size-12 text-muted" />
			<template v-if="menu">
				<h2 class="text-xl font-semibold">
					Coming Soon
				</h2>
				<p v-if="menu.description" class="max-w-md text-muted">
					{{ menu.description }}
				</p>
				<p v-if="menu.lama" class="text-sm text-dimmed">
					Di e-Rapor SD: <b>{{ menu.lama }}</b>
				</p>
			</template>
			<p v-else class="text-muted">
				Alamat {{ route.path }} tidak ada.
			</p>
			<UButton
				to="/"
				variant="subtle"
				icon="lucide:layout-dashboard"
				class="mt-2">
				Kembali ke Dashboard
			</UButton>
		</div>
	</ErPage>
</template>
