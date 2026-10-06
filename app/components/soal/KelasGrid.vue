<script lang="ts" setup>
	const props = defineProps<{
		classes: string[]
		stats?: Record<string, { mapel: number, jenis: number, soal: number }>
		totalSubjects?: number
	}>();

	const statItems = (kelas: string) => {
		const s = props.stats?.[kelas];
		if (!s) return [];
		return [
			{ icon: "i-lucide-book-open", value: s.mapel, label: "Mapel" },
			{ icon: "i-lucide-folder-open", value: s.jenis, label: "Ujian" },
			{ icon: "i-lucide-file-question", value: s.soal, label: "Soal" }
		];
	};

	/** Kelas yang belum punya jenis ujian maupun soal — ditampilkan pesan kosong, bukan deretan angka 0. */
	const isEmpty = (kelas: string) => {
		const s = props.stats?.[kelas];
		return !s || (!s.jenis && !s.soal);
	};

	/**
	 * Persentase mapel yang sudah punya jenis ujian (aturan yang sama dengan badge di kartu mapel,
	 * SoalPelajaranGrid); null bila total mapel tidak diketahui.
	 */
	const cakupan = (kelas: string) => {
		const s = props.stats?.[kelas];
		if (!s || !props.totalSubjects) return null;
		return Math.min(100, Math.round((s.mapel / props.totalSubjects) * 100));
	};
</script>

<template>
	<div class="space-y-8">
		<UPageHeader
			title="Pilih Kelas"
			description="Pilih kelas untuk mulai mengerjakan atau mengelola soal."
			headline="Bank Soal"
			:ui="{ root: 'border-none pb-0' }"
		/>

		<UPageGrid
			v-if="classes.length"
			class="grid-cols-[repeat(auto-fill,minmax(17rem,1fr))] gap-4 lg:grid-cols-[repeat(auto-fill,minmax(17rem,1fr))]"
		>
			<UPageCard
				v-for="c in classes"
				:key="c"
				:to="`/soal/${c}`"
				variant="outline"
				spotlight
				spotlight-color="primary"
				:ui="{ container: 'p-5 sm:p-5 gap-4', wrapper: 'gap-1' }"
			>
				<template #leading>
					<span class="flex size-10 items-center justify-center rounded-lg bg-primary/10 text-primary ring ring-inset ring-primary/25">
						<UIcon name="i-lucide-layers" class="size-5" />
					</span>
				</template>

				<template #title>
					<span class="text-lg font-semibold text-highlighted">Kelas {{ c }}</span>
				</template>

				<template #description>
					<span v-if="cakupan(c) !== null" class="text-sm text-muted">
						{{ stats![c]!.mapel }} dari {{ totalSubjects }} mapel terisi
					</span>
				</template>

				<div v-if="stats" class="space-y-4">
					<UProgress
						v-if="cakupan(c) !== null"
						:model-value="cakupan(c)"
						size="xs"
						:aria-label="`Cakupan mapel kelas ${c}`"
					/>

					<dl
						v-if="!isEmpty(c)"
						class="grid grid-cols-3 divide-x divide-default border-t border-default pt-4"
					>
						<!-- dt (label) harus sebelum dd (nilai) di HTML; flex-col-reverse tetap menampilkan angkanya di atas. -->
						<div
							v-for="item in statItems(c)"
							:key="item.label"
							class="flex flex-col-reverse px-3 first:ps-0 last:pe-0"
						>
							<dt class="mt-0.5 flex items-center gap-1 text-xs text-muted">
								<UIcon :name="item.icon" class="size-3.5 shrink-0" />
								{{ item.label }}
							</dt>
							<dd class="text-xl font-semibold tabular-nums text-highlighted">
								{{ item.value }}
							</dd>
						</div>
					</dl>

					<p v-else class="border-t border-default pt-4 text-sm text-muted">
						Belum ada jenis ujian atau soal di kelas ini.
					</p>
				</div>
			</UPageCard>
		</UPageGrid>

		<UEmpty
			v-else
			icon="i-lucide-inbox"
			title="Belum ada kelas"
			description="Tambahkan kelas terlebih dahulu di halaman Pengaturan."
			:actions="[{ label: 'Buka Pengaturan', icon: 'i-lucide-settings', to: '/pengaturan', color: 'neutral', variant: 'outline' }]"
		/>
	</div>
</template>