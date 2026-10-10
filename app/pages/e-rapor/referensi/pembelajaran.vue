<script setup lang="ts">
	import type { TableColumn } from "@nuxt/ui";

	interface Row {
		id: number
		rombongan_belajar_id: string
		rombel: string
		tingkat: string
		kode_mapel: string
		singkat: string
		nama: string
		ptk_id: string | null
		sumber: string
		nilai: number
	}

	interface DapodikRow {
		rombel: string
		nama_mata_pelajaran: string | null
		guru: string | null
		jam_per_minggu: number | null
	}

	const db = useDb();
	const swal = useSwal();
	const toast = useToast();
	const { session } = useAuth();
	const { susun } = usePembelajaranRapor();
	const sem = computed(() => session.value?.semesterId ?? "");

	const tab = ref("rapor");
	const loading = ref(true);
	const rows = ref<Row[]>([]);
	const dapodik = ref<DapodikRow[]>([]);
	const guru = ref<{ label: string, value: string }[]>([]);
	const tanpaGuruKelas = ref<string[]>([]);
	const running = ref(false);

	async function load() {
		if (!sem.value) {
			rows.value = [];
			dapodik.value = [];
			tanpaGuruKelas.value = [];
			loading.value = false;
			return;
		}
		loading.value = true;
		try {
			const [r, d, g, gk] = await Promise.all([
				db.query<Row>(`SELECT pr.id, pr.rombongan_belajar_id, r.nama AS rombel, r.tingkat, pr.kode_mapel, m.singkat, m.nama, pr.ptk_id, pr.sumber,
						(SELECT COUNT(*) FROM nilai_rapor n WHERE n.pembelajaran_rapor_id = pr.id AND n.nilai IS NOT NULL) AS nilai
					FROM pembelajaran_rapor pr JOIN rombel r USING (rombongan_belajar_id) JOIN mapel_rapor m ON m.kode = pr.kode_mapel
					WHERE pr.semester_id = ? ORDER BY CAST(r.tingkat AS INTEGER), r.nama, m.urutan`, [sem.value]),
				db.query<DapodikRow>(`SELECT r.nama AS rombel, p.nama_mata_pelajaran, g.nama AS guru, p.jam_per_minggu
					FROM pembelajaran p JOIN rombel r USING (rombongan_belajar_id) LEFT JOIN ptk g ON g.ptk_id = p.ptk_id
					WHERE p.semester_id = ? ORDER BY CAST(r.tingkat AS INTEGER), r.nama, p.nama_mata_pelajaran`, [sem.value]),
				db.query<{ ptk_id: string, nama: string }>("SELECT ptk_id, nama FROM ptk ORDER BY nama COLLATE NOCASE"),
				db.query<{ nama: string }>(`SELECT r.nama FROM rombel r WHERE r.semester_id = ? AND r.jenis_rombel = '1'
					AND NOT EXISTS (SELECT 1 FROM pembelajaran p WHERE p.rombongan_belajar_id = r.rombongan_belajar_id AND p.mata_pelajaran_id = ?)
					ORDER BY CAST(r.tingkat AS INTEGER)`, [sem.value, KODE_GURU_KELAS])
			]);
			rows.value = r;
			dapodik.value = d;
			guru.value = g.map(x => ({ label: x.nama, value: x.ptk_id }));
			tanpaGuruKelas.value = gk.map(x => x.nama);
		}
		catch (e) {
			toast.add({ title: "Gagal memuat pembelajaran", description: pesanError(e), color: "error" });
		}
		finally {
			loading.value = false;
		}
	}
	watch(sem, load, { immediate: true });

	const perKelas = computed(() => {
		const map = new Map<string, { nama: string, items: Row[] }>();
		for (const r of rows.value) {
			if (!map.has(r.rombongan_belajar_id))
				map.set(r.rombongan_belajar_id, { nama: r.rombel, items: [] });
			map.get(r.rombongan_belajar_id)!.items.push(r);
		}
		return [...map.values()];
	});

	async function jalankanSusun() {
		running.value = true;
		try {
			const r = await susun(sem.value);
			await load();
			toast.add({ title: "Pembelajaran tersusun", description: `${r.pembelajaran} pembelajaran untuk ${r.kelas} kelas`, color: "success" });
		}
		catch (e) {
			swal.error("Gagal menyusun", pesanError(e));
		}
		finally {
			running.value = false;
		}
	}

	// Baris baru diubah setelah tersimpan, jadi kalau gagal pilihan di layar tetap pengampu lama.
	async function gantiPengampu(r: Row, ptk: string) {
		if (ptk === r.ptk_id)
			return;
		try {
			await db.execute("UPDATE pembelajaran_rapor SET ptk_id = ?, sumber = 'manual' WHERE id = ?", [ptk, r.id]);
			r.ptk_id = ptk;
			r.sumber = "manual";
			toast.add({ title: `Pengampu ${r.singkat} ${r.rombel} diganti`, description: guru.value.find(g => g.value === ptk)?.label, color: "success" });
		}
		catch (e) {
			toast.add({ title: "Gagal mengganti pengampu", description: pesanError(e), color: "error" });
		}
	}

	async function hapus(r: Row) {
		const pesan = r.nilai > 0
			? `${r.singkat} ${r.rombel} sudah punya ${r.nilai} nilai. Nilainya ikut terhapus.`
			: `${r.singkat} tidak akan tampil di rapor ${r.rombel}. Bisa dimunculkan lagi lewat Susun Otomatis.`;
		if (!await swal.confirm("Hapus pembelajaran?", pesan, "Hapus"))
			return;
		try {
			await db.batch([
				{ sql: "DELETE FROM nilai_rapor WHERE pembelajaran_rapor_id = ?", params: [r.id] },
				{ sql: "DELETE FROM pembelajaran_rapor WHERE id = ?", params: [r.id] }
			]);
			rows.value = rows.value.filter(x => x.id !== r.id);
			toast.add({ title: `${r.singkat} ${r.rombel} dihapus`, color: "success" });
		}
		catch (e) {
			swal.error("Gagal menghapus", pesanError(e));
		}
	}

	const SUMBER: Record<string, { label: string, color: "neutral" | "primary" | "warning" }> = {
		dapodik: { label: "Dapodik", color: "neutral" },
		sub: { label: "Guru Kelas", color: "primary" },
		manual: { label: "Manual", color: "warning" }
	};

	const dapodikColumns: TableColumn<DapodikRow>[] = [
		{ accessorKey: "rombel", header: "Kelas" },
		{ accessorKey: "nama_mata_pelajaran", header: "Mata Pelajaran", cell: ({ row }) => atauStrip(row.original.nama_mata_pelajaran) },
		{ accessorKey: "guru", header: "Guru", cell: ({ row }) => atauStrip(row.original.guru) },
		{ accessorKey: "jam_per_minggu", header: "Jam/Minggu", cell: ({ row }) => atauStrip(row.original.jam_per_minggu), meta: { class: { th: "text-right", td: "text-right tabular-nums" } } }
	];
</script>

<template>
	<ErPage id="ref-pembelajaran" title="Pembelajaran">
		<template #right>
			<UButton icon="lucide:list-checks" variant="soft" to="/e-rapor/referensi/mapel">
				Atur Mapel
			</UButton>
			<UButton
				icon="lucide:wand-sparkles"
				:loading="running"
				:disabled="loading || !sem"
				@click="jalankanSusun">
				Susun Otomatis
			</UButton>
		</template>

		<div class="space-y-4">
			<UTabs
				v-model="tab"
				:content="false"
				:items="[
					{ label: 'Pembelajaran Rapor', value: 'rapor', icon: 'lucide:book-open' },
					{ label: 'Data Dapodik', value: 'dapodik', icon: 'lucide:database' }
				]"
			/>

			<template v-if="tab === 'rapor'">
				<div v-if="loading" class="grid gap-4 xl:grid-cols-2">
					<USkeleton v-for="i in 4" :key="i" class="h-64" />
				</div>

				<template v-else>
					<UAlert
						v-if="tanpaGuruKelas.length"
						color="warning"
						variant="subtle"
						icon="lucide:triangle-alert"
						:title="`${tanpaGuruKelas.join(', ')} belum punya pembelajaran Guru Kelas di Dapodik`"
						description="Mapelnya tetap disusun dan diampu wali kelas. Sebaiknya lengkapi juga di Dapodik (Rombongan Belajar → Pembelajaran)."
					/>

					<div v-if="!rows.length" class="flex flex-col items-center gap-3 py-16 text-center">
						<UIcon name="lucide:wand-sparkles" class="size-12 text-muted" />
						<p class="max-w-md text-muted">
							<template v-if="sem">
								Di Dapodik, guru kelas SD tercatat sebagai satu pembelajaran "Guru Kelas". Klik <b>Susun Otomatis</b> untuk memecahnya jadi mapel rapor sesuai fase, sekaligus semua kelas.
							</template>
							<template v-else>
								Belum ada semester. Tarik dulu data lewat Sinkron Dapodik.
							</template>
						</p>
						<UButton
							v-if="sem"
							icon="lucide:wand-sparkles"
							variant="solid"
							:loading="running"
							@click="jalankanSusun">
							Susun Otomatis
						</UButton>
					</div>

					<div v-else class="grid gap-4 xl:grid-cols-2">
						<PanelCard
							v-for="k in perKelas"
							:key="k.nama"
							:title="k.nama"
							icon="lucide:layers"
							:description="`${k.items.length} mapel`">
							<ul class="divide-y divide-default">
								<li v-for="r in k.items" :key="r.id" class="flex items-center gap-2 py-2">
									<span class="w-28 shrink-0 truncate text-sm font-medium" :title="r.nama">{{ r.singkat }}</span>
									<USelectMenu
										:model-value="r.ptk_id ?? undefined"
										:items="guru"
										value-key="value"
										size="sm"
										class="min-w-0 flex-1"
										placeholder="Belum ada pengampu"
										:aria-label="`Pengampu ${r.singkat} ${r.rombel}`"
										@update:model-value="v => v && gantiPengampu(r, v)"
									/>
									<UBadge
										:color="SUMBER[r.sumber]?.color ?? 'neutral'"
										variant="subtle"
										size="sm"
										class="shrink-0">
										{{ SUMBER[r.sumber]?.label ?? r.sumber }}
									</UBadge>
									<UButton
										icon="lucide:trash-2"
										color="error"
										variant="ghost"
										size="xs"
										:aria-label="`Hapus ${r.singkat} ${r.rombel}`"
										@click="hapus(r)"
									/>
								</li>
							</ul>
						</PanelCard>
					</div>
				</template>
			</template>

			<template v-else>
				<div class="flex justify-end">
					<DapodikNote />
				</div>
				<RefTable :data="dapodik" :columns="dapodikColumns" :loading="loading" />
			</template>
		</div>
	</ErPage>
</template>
