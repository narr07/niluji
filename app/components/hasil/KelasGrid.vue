<script lang="ts" setup>
	const props = defineProps<{
		classes: string[]
	}>();

	// Tailwind needs full class names to appear literally in source to generate them, so this is
	// a lookup table instead of a dynamically built "grid-cols-${n}" string — biar kartu selalu
	// melebar penuh sesuai jumlah kelas yang ada, bukan kepotong sempit gara-gara jumlah kolom
	// tetap yang lebih banyak dari jumlah kelasnya.
	const colsClass: Record<number, string> = {
		1: "sm:grid-cols-1 lg:grid-cols-1",
		2: "sm:grid-cols-2 lg:grid-cols-2",
		3: "sm:grid-cols-3 lg:grid-cols-3",
		4: "sm:grid-cols-2 lg:grid-cols-4",
		5: "sm:grid-cols-3 lg:grid-cols-5",
		6: "sm:grid-cols-3 lg:grid-cols-6"
	};

	const gridColsClass = computed(() => colsClass[Math.min(props.classes.length, 6) || 1] ?? colsClass[4]);
</script>

<template>
	<UPageGrid :class="[gridColsClass, 'gap-4 sm:gap-6 lg:gap-px']">
		<UPageCard
			v-for="c in classes"
			:key="c"
			icon="lucide:bar-chart-3"
			:title="`Kelas ${c}`"
			:to="`/hasil/${c}`"
			variant="subtle"
			:ui="{
				container: 'gap-y-1.5',
				wrapper: 'items-start',
				leading: 'p-2.5 rounded-full bg-primary/10 ring ring-inset ring-primary/25 flex-col',
				title: 'font-medium text-lg'
			}"
			class="lg:rounded-none first:rounded-l-lg last:rounded-r-lg hover:z-1"
		/>
	</UPageGrid>
</template>
