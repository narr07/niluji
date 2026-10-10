<script lang="ts" setup>
	// Sama dengan nuxtor-main/app/components/dashboard/StatsGrid.vue.
	defineProps<{
		stats: { title: string, icon: string, value: number | string, to?: string }[]
	}>();
</script>

<template>
	<UPageGrid class="gap-4 sm:gap-6 lg:grid-cols-4 lg:gap-px">
		<UPageCard
			v-for="(stat, index) in stats"
			:key="index"
			:icon="stat.icon"
			:title="stat.title"
			:to="stat.to"
			variant="subtle"
			:ui="{
				container: 'gap-y-1.5',
				wrapper: 'items-start',
				leading: 'p-2.5 rounded-full bg-primary/10 ring ring-inset ring-primary/25 flex-col',
				title: 'font-normal text-muted text-xs uppercase'
			}"
			class="first:rounded-l-lg last:rounded-r-lg hover:z-1 lg:rounded-none"
		>
			<span class="text-2xl font-semibold text-highlighted">
				{{ stat.value }}
			</span>
		</UPageCard>
	</UPageGrid>
</template>
