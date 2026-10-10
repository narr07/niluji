<script setup lang="ts">
	const { waliRombel } = useAuth();
	const ids = computed(() => waliRombel.value.map(r => r.rombongan_belajar_id));
	const tab = ref("status");
	const tabs = [
		{ value: "status", label: "Status", icon: "lucide:list-checks" },
		{ value: "statistik", label: "Statistik", icon: "lucide:chart-column" }
	];
</script>

<template>
	<ErPage id="walas-status" title="Status Penilaian">
		<div v-if="ids.length" class="space-y-4">
			<UTabs
				v-model="tab"
				:items="tabs"
				:content="false"
				class="w-fit" />
			<StatusPenilaian v-if="tab === 'status'" :rombel-ids="ids" />
			<StatistikNilai v-else :rombel-ids="ids" />
		</div>
	</ErPage>
</template>
