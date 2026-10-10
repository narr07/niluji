<script setup lang="ts">
	// Guru & wali kelas: kelas yang diajar ditambah kelas yang diwalikan.
	const db = useDb();
	const { user, session, waliRombel } = useAuth();
	const ids = ref<string[]>();
	onMounted(async () => {
		const ajar = await db.query<{ id: string }>(
			"SELECT DISTINCT rombongan_belajar_id AS id FROM pembelajaran_rapor WHERE ptk_id = ? AND semester_id = ?",
			[user.value?.ptk_id, session.value?.semesterId]
		);
		ids.value = [...new Set([...ajar.map(a => a.id), ...waliRombel.value.map(r => r.rombongan_belajar_id)])];
	});
</script>

<template>
	<ErPage id="guru-perkembangan" title="Perkembangan Nilai">
		<PerkembanganNilai v-if="ids" :rombel-ids="ids" />
	</ErPage>
</template>
