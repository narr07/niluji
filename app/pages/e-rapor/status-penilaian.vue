<script setup lang="ts">
	// Status (siapa yang belum menilai) dan Statistik (sebaran nilai) dalam satu halaman, seperti e-Rapor.
	const tab = ref("status");
	const tabs = [
		{ value: "status", label: "Status", icon: "lucide:list-checks" },
		{ value: "statistik", label: "Statistik", icon: "lucide:chart-column" }
	];
</script>

<template>
	<ErPage id="status-penilaian" title="Status Penilaian">
		<div class="space-y-4">
			<UTabs
				v-model="tab"
				:items="tabs"
				:content="false"
				class="w-fit" />
			<StatusPenilaian v-if="tab === 'status'" />
			<StatistikNilai v-else />
		</div>
	</ErPage>
</template>
