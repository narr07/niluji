<script lang="ts" setup>
	definePageMeta({
		hideFromNav: true
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
			icon: "lucide:users",
			to: `/hasil/${kelas.value}/${pelajaranParam.value}/${jenisParam.value}`,
			exact: true
		},
		{
			label: "Analisis Soal",
			icon: "lucide:bar-chart-3",
			to: `/hasil/${kelas.value}/${pelajaranParam.value}/${jenisParam.value}/analisis`
		}
	]]);
</script>

<template>
	<div class="space-y-4">
		<div>
			<h2 class="text-lg font-semibold">
				{{ jenis }}
			</h2>
			<p class="text-sm text-muted">
				Kelas {{ kelas }} · {{ pelajaran }}
			</p>
		</div>

		<UNavigationMenu :items="links" highlight class="-mx-1" />

		<NuxtPage />
	</div>
</template>
