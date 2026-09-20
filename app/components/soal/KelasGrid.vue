<script lang="ts" setup>
	defineProps<{
		classes: string[]
	}>();
</script>

<template>
	<div class="space-y-6">
		<UPageHeader
			title="Pilih Kelas"
			description="Pilih kelas untuk mulai mengerjakan atau mengelola soal."
			headline="Ujian"
		/>

		<div v-if="classes.length" class="grid gap-4 grid-cols-2 sm:grid-cols-3 lg:grid-cols-6">
			<UCard
				v-for="c in classes"
				:key="c"
				variant="subtle"
				class="cursor-pointer hover:ring-primary transition-colors"
				@click="navigateTo(`/soal/${c}`)"
			>
				<div class="flex flex-col items-center justify-center gap-2 py-6 text-center">
					<UIcon name="lucide:layers" class="size-6 text-primary" />
					<p class="text-lg font-semibold">
						Kelas {{ c }}
					</p>
				</div>
			</UCard>
		</div>

		<UAlert
			v-else
			icon="lucide:inbox"
			title="Belum ada kelas"
			description="Tambahkan kelas terlebih dahulu di halaman Pengaturan."
			variant="subtle"
			color="neutral"
		/>
	</div>
</template>