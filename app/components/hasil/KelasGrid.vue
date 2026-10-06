<script lang="ts" setup>
	const props = defineProps<{
		classes: string[]
		loading?: boolean
		stats?: Record<string, {
			ujian: number
			sesi: number
			selesai: number
			target: number
			rataRata: number | null
			totalSiswa: number
		}>
	}>();

	const statItems = (kelas: string) => {
		const s = props.stats?.[kelas];
		if (!s) return [];
		return [
			{ icon: "i-lucide-calendar-check", value: s.ujian, label: "Ujian" },
			{ icon: "i-lucide-circle-check", value: s.selesai, label: "Selesai" },
			{
				icon: "i-lucide-trending-up",
				value: s.rataRata === null ? "—" : s.rataRata.toLocaleString("id-ID", { maximumFractionDigits: 1 }),
				label: "Rata-rata PG",
				hint: "Nilai otomatis saat submit (pilihan ganda). Nilai esai dari guru tidak termasuk."
			}
		];
	};

	/** Kelas yang belum punya ujian maupun sesi — ditampilkan pesan kosong, bukan deretan angka 0. */
	const isEmpty = (kelas: string) => {
		const s = props.stats?.[kelas];
		return !s || (!s.ujian && !s.sesi);
	};

	/**
	 * Persentase pengerjaan selesai dari target (jumlah ujian × jumlah siswa kelas); null bila
	 * targetnya nol (belum ada siswa atau belum ada ujian).
	 */
	const progres = (kelas: string) => {
		const s = props.stats?.[kelas];
		if (!s || !s.target) return null;
		return Math.min(100, Math.round((s.selesai / s.target) * 100));
	};

	const progresText = (kelas: string) => {
		const s = props.stats?.[kelas];
		if (!s) return "";
		if (!s.totalSiswa) return "Belum ada siswa di kelas ini";
		if (!s.ujian) return "Belum ada ujian dijadwalkan";
		return `${s.selesai} dari ${s.target} pengerjaan selesai`;
	};
</script>

<template>
	<div class="space-y-8">
		<UPageHeader
			title="Pilih Kelas"
			description="Pilih kelas untuk melihat rekap nilai, progres pengerjaan, dan analisis soal."
			headline="Hasil Ujian"
			:ui="{ root: 'border-none pb-0' }"
		/>

		<div v-if="loading" class="grid grid-cols-[repeat(auto-fill,minmax(17rem,1fr))] gap-4">
			<USkeleton v-for="i in 3" :key="i" class="h-44 rounded-lg" />
		</div>

		<UPageGrid
			v-else-if="classes.length"
			class="grid-cols-[repeat(auto-fill,minmax(17rem,1fr))] gap-4 lg:grid-cols-[repeat(auto-fill,minmax(17rem,1fr))]"
		>
			<UPageCard
				v-for="c in classes"
				:key="c"
				:to="`/hasil/${c}`"
				variant="outline"
				spotlight
				spotlight-color="primary"
				:ui="{ container: 'p-5 sm:p-5 gap-4', wrapper: 'gap-1' }"
			>
				<template #leading>
					<span class="flex size-10 items-center justify-center rounded-lg bg-primary/10 text-primary ring ring-inset ring-primary/25">
						<UIcon name="i-lucide-bar-chart-3" class="size-5" />
					</span>
				</template>

				<template #title>
					<span class="text-lg font-semibold text-highlighted">Kelas {{ c }}</span>
				</template>

				<template #description>
					<span v-if="stats" class="text-sm text-muted">{{ progresText(c) }}</span>
				</template>

				<div v-if="stats" class="space-y-4">
					<UProgress
						v-if="progres(c) !== null"
						:model-value="progres(c)"
						size="xs"
						:aria-label="`Progres pengerjaan ujian kelas ${c}`"
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
								<UTooltip v-if="item.hint" :text="item.hint">
									<UIcon name="i-lucide-info" class="size-3.5 shrink-0" />
								</UTooltip>
							</dt>
							<dd class="text-xl font-semibold tabular-nums text-highlighted">
								{{ item.value }}
							</dd>
						</div>
					</dl>

					<p v-else class="border-t border-default pt-4 text-sm text-muted">
						Belum ada ujian atau hasil di kelas ini.
					</p>
				</div>
			</UPageCard>
		</UPageGrid>

		<UEmpty
			v-else
			icon="i-lucide-inbox"
			title="Belum ada ujian yang terlaksana"
			description="Kelas muncul di sini setelah ujiannya dibuat di Kelola Ujian, soalnya tersedia di Bank Soal, dan waktu ujiannya sudah dimulai."
			:actions="[{ label: 'Buka Kelola Ujian', icon: 'i-lucide-calendar-clock', to: '/ujian', color: 'neutral', variant: 'outline' }]"
		/>
	</div>
</template>
