<script lang="ts" setup>
	definePageMeta({
		hideFromNav: true
	});

	const selectedRows = ref(0);
</script>

<template>
	<div class="space-y-4">
		<PengaturanTarikSoalOnline @selected="selectedRows = $event.length" />
		<p v-if="selectedRows" class="sr-only">{{ selectedRows }} soal siap digunakan.</p>
	</div>
</template>
