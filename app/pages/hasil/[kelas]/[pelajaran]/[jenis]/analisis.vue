<script lang="ts" setup>
	definePageMeta({
		hideFromNav: true
	});

	interface Subject {
		id: number
		name: string
		code: string | null
	}

	const route = useRoute();
	const kelas = computed(() => route.params.kelas as string);
	const jenis = computed(() => decodeURIComponent(route.params.jenis as string));
	const subject = inject<ComputedRef<Subject | undefined>>("hasilSubject");
</script>

<template>
	<HasilAnalytics :kelas="kelas" :subject-id="subject?.id" :jenis="jenis" />
</template>
