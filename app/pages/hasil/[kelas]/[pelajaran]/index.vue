<script lang="ts" setup>
	definePageMeta({
		hideFromNav: true
	});

	const route = useRoute();
	const kelas = computed(() => route.params.kelas as string);
	const pelajaran = computed(() => decodeURIComponent(route.params.pelajaran as string));
</script>

<template>
	<HasilJenisGrid :kelas="kelas" :pelajaran="pelajaran" />
</template>
