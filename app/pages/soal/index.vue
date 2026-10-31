<script lang="ts" setup>
	import { invoke } from "@tauri-apps/api/core";

	definePageMeta({
		name: "Bank Soal",
		icon: "lucide:list-checks",
		top: true,
		order: 2,
		description: "Soal per kelas dan mata pelajaran"
	});

	interface QuestionSummary {
		id: number
		subject: string
		class: string | null
		jenis: string | null
		questionType: string
	}

	const classes = ref<string[]>([]);
	const questions = ref<QuestionSummary[]>([]);
	const searchQuery = ref("");

	const packages = computed(() => {
		const map = new Map<string, { class: string, subject: string, jenis: string, pgCount: number, essayCount: number }>();
		for (const q of questions.value) {
			if (!q.class || !q.jenis) continue;
			const key = `${q.class}::${q.subject}::${q.jenis}`;
			let entry = map.get(key);
			if (!entry) {
				entry = { class: q.class, subject: q.subject, jenis: q.jenis, pgCount: 0, essayCount: 0 };
				map.set(key, entry);
			}
			if (q.questionType === "essay") entry.essayCount++;
			else entry.pgCount++;
		}
		const all = [...map.values()].sort((a, b) => a.class.localeCompare(b.class) || a.subject.localeCompare(b.subject));
		const q = searchQuery.value.trim().toLowerCase();
		if (!q) return all;
		return all.filter(
			(p) =>
				p.jenis.toLowerCase().includes(q)
				|| p.subject.toLowerCase().includes(q)
				|| `kelas ${p.class}`.toLowerCase().includes(q)
		);
	});

	onMounted(async () => {
		const [cls, qs] = await Promise.all([
			invoke<string[]>("list_classes"),
			invoke<QuestionSummary[]>("list_questions")
		]);
		classes.value = cls;
		questions.value = qs;
	});
</script>

<template>
	<UDashboardPanel id="soal">
		<template #header>
			<UDashboardNavbar title="Bank Soal">
				<template #leading>
					<UDashboardSidebarCollapse />
				</template>
			</UDashboardNavbar>
		</template>

		<template #body>
			<div class="space-y-6">
				<SoalKelasGrid :classes="classes" />

				<!-- Quick Jump / Search across all question packages -->
				<UCard v-if="questions.length">
					<template #header>
						<div class="flex items-center justify-between flex-wrap gap-3">
							<div class="flex items-center gap-2">
								<UIcon name="lucide:folder-search" class="size-4 text-primary" />
								<span class="font-semibold text-highlighted">Akses Cepat Paket Soal</span>
							</div>
							<UInput
								v-model="searchQuery"
								icon="lucide:search"
								size="sm"
								placeholder="Cari kelas, mapel, atau jenis..."
								class="w-full sm:w-64" />
						</div>
					</template>

					<div v-if="packages.length" class="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
						<NuxtLink
							v-for="pkg in packages"
							:key="`${pkg.class}-${pkg.subject}-${pkg.jenis}`"
							:to="`/soal/${pkg.class}/${encodeURIComponent(pkg.subject)}/${encodeURIComponent(pkg.jenis)}`"
							class="p-3 rounded-lg border border-default hover:border-primary hover:bg-elevated/50 transition-colors flex items-start justify-between gap-2">
							<div class="min-w-0">
								<div class="flex items-center gap-1.5">
									<UBadge color="neutral" variant="subtle" size="xs">
										Kelas {{ pkg.class }}
									</UBadge>
									<span class="text-xs text-muted truncate">{{ pkg.subject }}</span>
								</div>
								<p class="font-semibold text-sm text-highlighted mt-1 truncate">
									{{ pkg.jenis }}
								</p>
								<p class="text-xs text-muted mt-0.5">
									{{ pkg.pgCount }} PG · {{ pkg.essayCount }} Esai
								</p>
							</div>
							<UIcon name="lucide:arrow-up-right" class="size-4 text-muted shrink-0 mt-1" />
						</NuxtLink>
					</div>
					<div v-else class="text-center py-6 text-sm text-muted">
						Tidak ada paket soal yang cocok dengan pencarian "{{ searchQuery }}".
					</div>
				</UCard>
			</div>
		</template>
	</UDashboardPanel>
</template>
