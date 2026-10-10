<script setup lang="ts">
	// Padanan e-Rapor: Status Penilaian → Statistik Nilai Rapor. Per kelas: tiap mapel berapa
	// yang dinilai, rata-rata, tertinggi, terendah, dan sebaran nilai (5 rentang, satu hue bertingkat).
	const props = defineProps<{ rombelIds?: string[] }>();

	interface Baris { kode: string, mapel: string, nilai: number[], siswa: number }

	const db = useDb();
	const toast = useToast();
	const { session } = useAuth();

	const kelasList = ref<{ value: string, label: string }[]>([]);
	const rombelId = ref<string>();
	const baris = ref<Baris[]>([]);
	const loading = ref(true);
	let permintaan = 0;

	async function loadKelas() {
		const sem = session.value?.semesterId;
		const ids = props.rombelIds;
		kelasList.value = !sem || (ids && !ids.length)
			? []
			: await db.query(
				`SELECT rombongan_belajar_id AS value, nama AS label FROM rombel WHERE semester_id = ? AND jenis_rombel = '1'
				${ids ? `AND rombongan_belajar_id IN (${ids.map(() => "?").join(",")})` : ""} ORDER BY CAST(tingkat AS INTEGER), nama`,
				[sem, ...(ids ?? [])]
			);
		if (!kelasList.value.some(k => k.value === rombelId.value))
			rombelId.value = kelasList.value[0]?.value;
	}

	async function load() {
		const nomor = ++permintaan;
		if (!rombelId.value) {
			baris.value = [];
			loading.value = false;
			return;
		}
		loading.value = true;
		try {
			const rows = await db.query<{ kode: string, mapel: string, nilai: string | null, siswa: number }>(
				`SELECT m.kode, m.nama AS mapel,
					(SELECT json_group_array(n.nilai) FROM nilai_rapor n WHERE n.pembelajaran_rapor_id = pr.id AND n.nilai IS NOT NULL) AS nilai,
					(SELECT COUNT(*) FROM anggota_rombel a WHERE a.rombongan_belajar_id = pr.rombongan_belajar_id) AS siswa
				FROM pembelajaran_rapor pr JOIN mapel_rapor m ON m.kode = pr.kode_mapel
				WHERE pr.rombongan_belajar_id = ? AND pr.semester_id = ? ORDER BY m.urutan`,
				[rombelId.value, session.value?.semesterId]
			);
			if (nomor === permintaan)
				baris.value = rows.map(r => ({ ...r, nilai: JSON.parse(r.nilai || "[]") as number[] }));
		}
		catch (e) {
			toast.add({ title: "Gagal memuat statistik", description: pesanError(e), color: "error" });
		}
		finally {
			if (nomor === permintaan)
				loading.value = false;
		}
	}

	async function init() {
		try {
			await loadKelas();
			await load();
		}
		catch (e) {
			toast.add({ title: "Gagal memuat kelas", description: pesanError(e), color: "error" });
			loading.value = false;
		}
	}
	watch([() => session.value?.semesterId, () => props.rombelIds?.join()], init, { immediate: true });
	watch(rombelId, load);

	// Sebaran: satu hue (primary) dari muda ke tua = nilai rendah ke tinggi.
	const RENTANG = [
		{ label: "< 60", min: 0, max: 59, kelas: "bg-primary/20" },
		{ label: "60–69", min: 60, max: 69, kelas: "bg-primary/40" },
		{ label: "70–79", min: 70, max: 79, kelas: "bg-primary/60" },
		{ label: "80–89", min: 80, max: 89, kelas: "bg-primary/80" },
		{ label: "90–100", min: 90, max: 100, kelas: "bg-primary" }
	];
	const sebaran = (b: Baris) => RENTANG.map(r => ({ ...r, n: b.nilai.filter(v => v >= r.min && v <= r.max).length }));
	const rata = (xs: number[]) => (xs.length ? (xs.reduce((a, c) => a + c, 0) / xs.length).toFixed(1) : "–");
	const semua = computed(() => baris.value.flatMap(b => b.nilai));
</script>

<template>
	<div v-if="!loading && !kelasList.length" class="flex flex-col items-center gap-3 py-24 text-center text-muted">
		<UIcon name="lucide:chart-column" class="size-12" />
		<p>Belum ada data kelas di semester ini.</p>
	</div>

	<div v-else class="space-y-4">
		<div class="flex flex-wrap items-center gap-2">
			<USelect
				v-if="kelasList.length > 1"
				v-model="rombelId"
				:items="kelasList"
				icon="lucide:layers"
				aria-label="Kelas"
				class="w-36" />
			<p class="ms-auto text-sm text-muted">
				Rata-rata kelas <b class="text-highlighted tabular-nums">{{ rata(semua) }}</b> dari {{ semua.length }} nilai
			</p>
		</div>

		<USkeleton v-if="loading" class="h-80 w-full" />
		<div v-else class="overflow-x-auto rounded-md border border-default">
			<table class="w-full text-sm">
				<thead class="bg-elevated/50 text-xs text-muted">
					<tr>
						<th class="p-2 text-left">
							Mata Pelajaran
						</th>
						<th class="w-24 p-2 text-right">
							Dinilai
						</th>
						<th class="w-20 p-2 text-right">
							Rata-rata
						</th>
						<th class="w-20 p-2 text-right">
							Tertinggi
						</th>
						<th class="w-20 p-2 text-right">
							Terendah
						</th>
						<th class="min-w-64 p-2 text-left">
							Sebaran nilai
						</th>
					</tr>
				</thead>
				<tbody class="divide-y divide-default">
					<tr v-for="b in baris" :key="b.kode">
						<td class="p-2 font-medium">
							{{ b.mapel }}
						</td>
						<td class="p-2 text-right tabular-nums" :class="b.nilai.length < b.siswa ? 'text-warning' : ''">
							{{ b.nilai.length }}/{{ b.siswa }}
						</td>
						<td class="p-2 text-right font-semibold tabular-nums">
							{{ rata(b.nilai) }}
						</td>
						<td class="p-2 text-right tabular-nums">
							{{ b.nilai.length ? Math.max(...b.nilai) : "–" }}
						</td>
						<td class="p-2 text-right tabular-nums">
							{{ b.nilai.length ? Math.min(...b.nilai) : "–" }}
						</td>
						<td class="p-2">
							<div
								v-if="b.nilai.length"
								class="flex h-3 w-full gap-0.5 overflow-hidden rounded-sm"
								role="img"
								:aria-label="`Sebaran ${b.mapel}: ${sebaran(b).map(r => `${r.label} ${r.n} siswa`).join(', ')}`">
								<UTooltip v-for="r in sebaran(b).filter(x => x.n)" :key="r.label" :text="`${r.label}: ${r.n} siswa`">
									<div class="h-full" :class="r.kelas" :style="{ flexGrow: r.n }" />
								</UTooltip>
							</div>
							<span v-else class="text-xs text-muted">Belum ada nilai</span>
						</td>
					</tr>
				</tbody>
			</table>
			<p v-if="!baris.length" class="py-8 text-center text-sm text-muted">
				Kelas ini belum punya pembelajaran.
			</p>
		</div>

		<div class="flex flex-wrap items-center gap-3 text-xs text-muted" aria-hidden="true">
			<span>Sebaran:</span>
			<span v-for="r in RENTANG" :key="r.label" class="flex items-center gap-1">
				<span class="inline-block size-3 rounded-sm" :class="r.kelas" />{{ r.label }}
			</span>
		</div>
	</div>
</template>
