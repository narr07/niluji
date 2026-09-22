<script lang="ts" setup>
	definePageMeta({
		hideFromNav: true,
	});

	const route = useRoute();
	const kelas = computed(() => route.params.kelas as string);
	const pelajaran = computed(() => decodeURIComponent(route.params.pelajaran as string));
	const pelajaranParam = computed(() => route.params.pelajaran as string);
	const jenis = computed(() => decodeURIComponent(route.params.jenis as string));
	const jenisParam = computed(() => route.params.jenis as string);

	const links = computed(() => [[
		{
			label: "Daftar Siswa",
			icon: "i-lucide-users",
			to: `/hasil/${kelas.value}/${pelajaranParam.value}/${jenisParam.value}`,
			exact: true,
		},
		{
			label: "Analisis Soal",
			icon: "i-lucide-bar-chart-3",
			to: `/hasil/${kelas.value}/${pelajaranParam.value}/${jenisParam.value}/analisis`,
		},
	]]);
</script>

<template>
	<div class="space-y-4">
		<div class="flex items-center gap-3">
			<div class="flex items-center justify-center size-10 rounded-lg bg-primary/10 text-primary shrink-0">
				<UIcon name="i-lucide-file-check-2" class="size-5" />
			</div>
			<div>
				<h2 class="text-lg font-semibold text-highlighted">
					{{ jenis }}
				</h2>
				<p class="text-sm text-muted">
					Kelas {{ kelas }} · {{ pelajaran }}
				</p>
			</div>
		</div>

		<UNavigationMenu
			:items="links"
			variant="pill"
			color="neutral"
			class="w-fit" />

		<NuxtPage />
	</div>
</template>