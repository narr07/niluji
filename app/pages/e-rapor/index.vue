<script setup lang="ts">
	definePageMeta({
		name: "E-Rapor",
		icon: "lucide:file-badge",
		top: true,
		description: "Rapor elektronik siswa & sinkron Dapodik"
	});

	const { user } = useAuth();
</script>

<template>
	<DashboardAdmin v-if="user?.level === 'admin'" />
	<DashboardGuru v-else-if="user?.level === 'guru'" />
	<DashboardSiswa v-else-if="user?.level === 'siswa'" />
</template>
