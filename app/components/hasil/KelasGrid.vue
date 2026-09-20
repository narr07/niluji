<script lang="ts" setup>
	defineProps<{
		classes: string[]
	}>();
</script>

<template>
	<div class="grid gap-4 sm:grid-cols-3 lg:grid-cols-4">
		<UCard
			v-for="c in classes"
			:key="c"
			class="cursor-pointer hover:ring-2 hover:ring-primary transition-shadow"
			@click="navigateTo(`/hasil/${c}`)"
		>
			<div class="text-center py-6">
				<p class="text-2xl font-bold">
					Kelas {{ c }}
				</p>
			</div>
		</UCard>
	</div>
</template>
