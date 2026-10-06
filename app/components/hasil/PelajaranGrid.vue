<script lang="ts" setup>
	interface Subject {
		id: number
		name: string
		code: string | null
	}

	const props = defineProps<{
		kelas: string
		subjects: Subject[]
		/** Jumlah jenis ujian TERLAKSANA per nama mapel (dihitung di halaman, lihat useHasilTerlaksana). */
		jenisCounts: Record<string, number>
	}>();

	const jenisCountFor = (_subjectId: number, subjectName: string) => props.jenisCounts[subjectName] ?? 0;
</script>

<template>
	<div class="grid grid-cols-[repeat(auto-fill,minmax(13rem,1fr))] gap-3">
		<UPageCard
			v-for="s in subjects"
			:key="s.id"
			:to="`/hasil/${kelas}/${encodeURIComponent(s.name)}`"
			variant="outline"
			:ui="{ container: 'p-4 sm:p-4 gap-3', wrapper: 'gap-0.5 min-w-0' }"
		>
			<template #title>
				<span class="block truncate text-lg font-semibold text-highlighted">
					{{ s.code || s.name }}
				</span>
			</template>

			<template #description>
				<span v-if="s.code" class="block truncate text-[10px] text-muted">
					{{ s.name }}
				</span>
			</template>

			<div class="flex items-center gap-1.5 border-t border-default pt-3 text-xs">
				<UIcon name="i-lucide-folder-open" class="size-3.5 shrink-0 text-muted" />
				<template v-if="jenisCountFor(s.id, s.name)">
					<span class="font-semibold tabular-nums text-highlighted">
						{{ jenisCountFor(s.id, s.name) }}
					</span>
					<span class="text-muted">jenis ujian</span>
				</template>
				<span v-else class="text-muted">Belum ada jenis ujian</span>
			</div>
		</UPageCard>
	</div>
</template>