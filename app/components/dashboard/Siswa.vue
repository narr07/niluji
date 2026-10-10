<script setup lang="ts">
	interface Rombel {
		nama: string
		jenis_rombel: string
		jenis_rombel_str: string | null
		wali: string | null
	}

	const { session, user } = useAuth();
	const db = useDb();
	const toast = useToast();

	const sem = computed(() => session.value?.semesterId ?? "");
	const pd = computed(() => user.value?.peserta_didik_id ?? "");

	const loading = ref(true);
	const rombel = ref<Rombel[]>([]);

	async function load() {
		if (!pd.value || !sem.value) {
			loading.value = false;
			return;
		}
		loading.value = true;
		try {
			rombel.value = await db.query<Rombel>(
				`SELECT r.nama, r.jenis_rombel, r.jenis_rombel_str, g.nama AS wali FROM anggota_rombel a
				JOIN rombel r USING (rombongan_belajar_id) LEFT JOIN ptk g ON g.ptk_id = r.ptk_id
				WHERE a.peserta_didik_id = ? AND a.semester_id = ? ORDER BY r.jenis_rombel, r.nama`,
				[pd.value, sem.value]
			);
		}
		catch (e) {
			toast.add({ title: "Gagal memuat dashboard", description: pesanError(e), color: "error" });
		}
		finally {
			loading.value = false;
		}
	}

	watch([sem, pd], load, { immediate: true });
</script>

<template>
	<ErPage id="home" title="Dashboard">
		<template #title>
			<div class="flex items-center gap-2">
				<UIcon name="lucide:file-badge" class="size-6 shrink-0 text-primary" />
				<span class="font-bold">{{ session?.sekolahNama }}</span>
			</div>
		</template>

		<div v-if="loading" class="space-y-6">
			<div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
				<USkeleton class="h-24" />
				<USkeleton class="h-24" />
			</div>
			<USkeleton class="h-28 w-full" />
		</div>

		<div v-else class="space-y-6">
			<UAlert
				v-if="!user?.peserta_didik_id"
				color="warning"
				variant="subtle"
				icon="lucide:triangle-alert"
				title="Akun ini belum terhubung dengan data siswa Dapodik"
				description="Minta administrator membuat ulang akun Anda lewat Generate Semua Pengguna."
			/>

			<UPageGrid v-if="rombel.length" class="gap-4 sm:grid-cols-2 lg:grid-cols-3">
				<UPageCard
					v-for="r in rombel"
					:key="r.nama"
					:icon="r.jenis_rombel === '1' ? 'lucide:layers' : 'lucide:trophy'"
					:title="r.nama"
					:description="`${r.jenis_rombel === '1' ? 'Kelas' : r.jenis_rombel_str}${r.wali ? ` · ${r.wali}` : ''}`"
					variant="subtle"
					:ui="{ leading: 'p-2.5 rounded-full bg-primary/10 ring ring-inset ring-primary/25' }"
				/>
			</UPageGrid>
			<p v-else-if="user?.peserta_didik_id" class="text-sm text-muted">
				Belum terdaftar di rombel mana pun pada semester ini.
			</p>

			<PanelCard title="File Rapor" icon="lucide:file-down">
				<p class="text-sm text-muted">
					File rapor akan muncul di sini setelah wali kelas mencetak dan administrator mengizinkan unduhan.
				</p>
			</PanelCard>
		</div>
	</ErPage>
</template>
