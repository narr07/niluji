<script setup lang="ts">
	// Padanan e-Rapor: Perkembangan Nilai Rapor (dari awal hingga akhir) + Grafik + Perkembangan Deskripsi.
	// Pilih kelas → siswa; baris = mapel, kolom = semester, plus garis tren kecil per mapel.
	// rombelIds kosong/undefined = semua kelas (admin). pesertaDidikId = satu siswa tetap (halaman siswa).
	const props = defineProps<{ rombelIds?: string[], pesertaDidikId?: string }>();

	interface Sel { nilai: number | null, deskripsi: string, kelas: string }
	interface Baris { kode: string, nama: string, sel: Record<string, Sel> }

	const db = useDb();
	const toast = useToast();
	const { session } = useAuth();

	const kelasList = ref<{ value: string, label: string }[]>([]);
	const rombelId = ref<string>();
	const siswaList = ref<{ value: string, label: string }[]>([]);
	const siswaId = ref<string>();
	const semester = ref<string[]>([]);
	const baris = ref<Baris[]>([]);
	const loading = ref(true);
	const tampilDeskripsi = ref(!!props.pesertaDidikId);
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

	async function loadSiswa() {
		siswaList.value = rombelId.value
			? await db.query(
				`SELECT s.peserta_didik_id AS value, s.nama AS label FROM anggota_rombel a JOIN siswa_rapor s USING (peserta_didik_id)
				WHERE a.rombongan_belajar_id = ? ORDER BY s.nama COLLATE NOCASE`,
				[rombelId.value]
			)
			: [];
		if (!siswaList.value.some(s => s.value === siswaId.value))
			siswaId.value = siswaList.value[0]?.value;
	}

	async function load() {
		const nomor = ++permintaan;
		if (!siswaId.value) {
			baris.value = [];
			loading.value = false;
			return;
		}
		loading.value = true;
		try {
			const rows = await db.query<{ semester_id: string, kelas: string, kode: string, nama: string, nilai: number | null, tp_optimal: string, tp_perlu: string }>(
				`SELECT pr.semester_id, r.nama AS kelas, m.kode, m.nama, n.nilai, n.tp_optimal, n.tp_perlu
				FROM nilai_rapor n JOIN pembelajaran_rapor pr ON pr.id = n.pembelajaran_rapor_id
				JOIN rombel r USING (rombongan_belajar_id) JOIN mapel_rapor m ON m.kode = pr.kode_mapel
				WHERE n.peserta_didik_id = ? ORDER BY pr.semester_id, m.urutan`,
				[siswaId.value]
			);
			const tpIds = [...new Set(rows.flatMap(r => [...JSON.parse(r.tp_optimal), ...JSON.parse(r.tp_perlu)] as number[]))];
			const tp = tpIds.length
				? await db.query<{ id: number, deskripsi: string }>(`SELECT id, deskripsi FROM tujuan_pembelajaran WHERE id IN (${tpIds.map(() => "?").join(",")})`, tpIds)
				: [];
			if (nomor !== permintaan)
				return;
			const teks = new Map(tp.map(t => [t.id, t.deskripsi]));
			const ambil = (json: string) => (JSON.parse(json) as number[]).map(id => teks.get(id)).filter((t): t is string => !!t);
			const map = new Map<string, Baris>();
			for (const r of rows) {
				if (!map.has(r.kode))
					map.set(r.kode, { kode: r.kode, nama: r.nama, sel: {} });
				const d = deskripsiCapaian(ambil(r.tp_optimal), ambil(r.tp_perlu));
				map.get(r.kode)!.sel[r.semester_id] = { nilai: r.nilai, deskripsi: [d.capai, d.perlu].filter(Boolean).join(" "), kelas: r.kelas };
			}
			semester.value = [...new Set(rows.map(r => r.semester_id))].sort();
			baris.value = [...map.values()];
		}
		catch (e) {
			toast.add({ title: "Gagal memuat perkembangan nilai", description: pesanError(e), color: "error" });
		}
		finally {
			if (nomor === permintaan)
				loading.value = false;
		}
	}

	async function init() {
		if (props.pesertaDidikId) {
			siswaId.value = props.pesertaDidikId;
			await load();
			return;
		}
		try {
			await loadKelas();
			await loadSiswa();
			await load();
		}
		catch (e) {
			toast.add({ title: "Gagal memuat data", description: pesanError(e), color: "error" });
			loading.value = false;
		}
	}
	watch([() => session.value?.semesterId, () => props.rombelIds?.join()], init, { immediate: true });
	watch(rombelId, async () => {
		await loadSiswa();
		await load();
	});
	watch(siswaId, load);

	// Garis tren kecil per mapel (satu seri, skala tetap 0–100 supaya antar mapel jujur dibandingkan).
	const W = 132;
	const H = 32;
	function titik(b: Baris) {
		const ada = semester.value.map((s, i) => ({ s, i, v: b.sel[s]?.nilai })).filter((p): p is { s: string, i: number, v: number } => typeof p.v === "number");
		const x = (i: number) => (semester.value.length <= 1 ? W / 2 : 6 + i * (W - 12) / (semester.value.length - 1));
		const y = (v: number) => H - 5 - (v / 100) * (H - 10);
		return ada.map(p => ({ ...p, x: x(p.i), y: y(p.v) }));
	}
	const garis = (b: Baris) => titik(b).map((p, i) => `${i ? "L" : "M"}${p.x.toFixed(1)},${p.y.toFixed(1)}`).join(" ");
	function tren(b: Baris) {
		const t = titik(b);
		if (t.length < 2)
			return null;
		const d = t.at(-1)!.v - t.at(-2)!.v;
		return d > 0 ? { icon: "lucide:trending-up", kelas: "text-success", teks: `+${d}` } : d < 0 ? { icon: "lucide:trending-down", kelas: "text-error", teks: `${d}` } : { icon: "lucide:minus", kelas: "text-muted", teks: "0" };
	}
</script>

<template>
	<div v-if="!pesertaDidikId && !loading && !kelasList.length" class="flex flex-col items-center gap-3 py-24 text-center text-muted">
		<UIcon name="lucide:trending-up" class="size-12" />
		<p>Belum ada data kelas di semester ini.</p>
	</div>

	<div v-else class="space-y-4">
		<div class="flex flex-wrap items-center gap-2">
			<template v-if="!pesertaDidikId">
				<USelect
					v-if="kelasList.length > 1"
					v-model="rombelId"
					:items="kelasList"
					icon="lucide:layers"
					aria-label="Kelas"
					class="w-36" />
				<USelectMenu
					v-model="siswaId"
					:items="siswaList"
					value-key="value"
					icon="lucide:user-round"
					aria-label="Siswa"
					class="w-64" />
			</template>
			<USwitch v-model="tampilDeskripsi" label="Tampilkan deskripsi" class="ms-auto" />
		</div>

		<USkeleton v-if="loading" class="h-80 w-full" />
		<div v-else-if="!baris.length" class="py-16 text-center text-sm text-muted">
			Siswa ini belum punya nilai rapor.
		</div>
		<div v-else class="overflow-x-auto rounded-md border border-default">
			<table class="w-full text-sm">
				<thead class="bg-elevated/50 text-xs text-muted">
					<tr>
						<th class="sticky left-0 min-w-48 bg-elevated p-2 text-left">
							Mata Pelajaran
						</th>
						<th v-for="s in semester" :key="s" class="min-w-24 p-2 text-center">
							{{ semesterLabel(s) }}
						</th>
						<th class="w-44 p-2 text-left">
							Tren
						</th>
					</tr>
				</thead>
				<tbody class="divide-y divide-default">
					<tr v-for="b in baris" :key="b.kode" class="align-top">
						<td class="sticky left-0 bg-default p-2 font-medium">
							{{ b.nama }}
						</td>
						<td v-for="s in semester" :key="s" class="p-2 text-center">
							<template v-if="b.sel[s]">
								<p class="font-semibold tabular-nums">
									{{ b.sel[s].nilai ?? "–" }}
								</p>
								<p class="text-xs text-muted">
									{{ b.sel[s].kelas }}
								</p>
								<p v-if="tampilDeskripsi" class="mt-1 max-w-64 text-left text-xs">
									{{ b.sel[s].deskripsi || "–" }}
								</p>
							</template>
							<span v-else class="text-dimmed">–</span>
						</td>
						<td class="p-2">
							<div class="flex items-center gap-2">
								<svg
									:width="W"
									:height="H"
									:viewBox="`0 0 ${W} ${H}`"
									role="img"
									:aria-label="`Tren ${b.nama}: ${titik(b).map(p => `${semesterLabel(p.s)} ${p.v}`).join(', ')}`">
									<line
										:x1="0"
										:x2="W"
										:y1="H - 5"
										:y2="H - 5"
										class="stroke-(--ui-border)"
										stroke-width="1" />
									<path
										:d="garis(b)"
										fill="none"
										class="stroke-(--ui-primary)"
										stroke-width="2"
										stroke-linecap="round"
										stroke-linejoin="round" />
									<circle
										v-for="p in titik(b)"
										:key="p.s"
										:cx="p.x"
										:cy="p.y"
										r="4"
										class="fill-(--ui-primary) stroke-(--ui-bg)"
										stroke-width="2"
									>
										<title>{{ semesterLabel(p.s) }}: {{ p.v }}</title>
									</circle>
								</svg>
								<span v-if="tren(b)" class="flex items-center gap-0.5 text-xs tabular-nums" :class="tren(b)!.kelas">
									<UIcon :name="tren(b)!.icon" class="size-3.5" />{{ tren(b)!.teks }}
								</span>
							</div>
						</td>
					</tr>
				</tbody>
			</table>
		</div>
		<p class="text-xs text-muted">
			Nilai diambil dari semua semester yang tersimpan di aplikasi ini. Semester sebelum aplikasi dipakai bisa dimasukkan lewat Restore backup lama atau import nilai.
		</p>
	</div>
</template>
