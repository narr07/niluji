<script lang="ts" setup>
	const props = defineProps<{
		kelas: string
		pelajaran: string
	}>();

	const { loading, ujianIn, sessions } = useHasilTerlaksana();

	// Hanya jenis ujian yang TERLAKSANA (ada di Kelola Ujian, soalnya ada, dan sudah dimulai —
	// lihat useHasilTerlaksana). Jenis yang ujiannya masih dijadwalkan untuk nanti belum muncul.
	// Satu jenis bisa punya lebih dari satu ujian; statusnya "berlangsung" kalau salah satunya
	// masih terbuka.
	const jenisGroups = computed(() => {
		const relevantSessions = sessions.value.filter((s) => s.class === props.kelas && s.subject === props.pelajaran);
		const byJenis = new Map<string, "berlangsung" | "selesai">();
		for (const u of ujianIn(props.kelas, props.pelajaran)) {
			if (byJenis.get(u.jenis) !== "berlangsung") byJenis.set(u.jenis, u.status);
		}
		return [...byJenis.entries()].map(([jenis, status]) => ({
			id: jenis,
			jenis,
			status,
			count: relevantSessions.filter((s) => (s.jenis ?? "") === jenis).length
		}));
	});
</script>

<template>
	<div class="space-y-6">
		<div v-if="loading" class="grid gap-3 sm:grid-cols-3 lg:grid-cols-4">
			<USkeleton v-for="i in 3" :key="i" class="h-24 rounded-lg" />
		</div>

		<div v-else-if="jenisGroups.length" class="grid gap-3 grid-cols-2 sm:grid-cols-3 lg:grid-cols-4">
			<UCard
				v-for="g in jenisGroups"
				:key="g.id"
				variant="subtle"
				class="relative cursor-pointer hover:ring-primary transition-colors"
				:ui="{ body: 'p-3' }"
				@click="navigateTo(`/hasil/${kelas}/${encodeURIComponent(pelajaran)}/${encodeURIComponent(g.jenis)}`)"
			>
				<UBadge
					color="neutral"
					variant="subtle"
					size="lg"
					class="absolute top-2 right-2">
					{{ g.count }} sesi
				</UBadge>

				<div class="pr-14">
					<p class="text-xl font-bold truncate">
						{{ g.jenis }}
					</p>
					<UBadge
						:color="g.status === 'berlangsung' ? 'success' : 'neutral'"
						:icon="g.status === 'berlangsung' ? 'i-lucide-radio' : 'i-lucide-check'"
						variant="soft"
						size="sm"
						class="mt-2">
						{{ g.status === 'berlangsung' ? 'Sedang berlangsung' : 'Selesai' }}
					</UBadge>
				</div>
			</UCard>
		</div>

		<UAlert
			v-else
			icon="i-lucide-inbox"
			title="Belum ada ujian yang terlaksana"
			description="Jenis ujian muncul di sini setelah ujiannya dibuat di Kelola Ujian, soalnya tersedia di Bank Soal, dan waktu ujiannya sudah dimulai."
			variant="subtle"
			color="neutral" />
	</div>
</template>
