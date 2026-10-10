<script setup lang="ts">
	// Papan status: tiap kelas × mapel, berapa siswa yang sudah dinilai dan siapa pengampunya.
	// Banyak kelas (admin) → matriks mapel × kelas supaya satu sekolah muat satu layar.
	// Satu kelas (wali kelas) → kartu dengan detail per mapel.
	const props = defineProps<{ rombelIds?: string[] }>();

	interface Row { id: number, rombel: string, rombongan_belajar_id: string, kode_mapel: string, singkat: string, guru: string | null, terisi: number, deskripsi: number, siswa: number, terkunci: number, dikirim_at: string | null }
	type Status = "lengkap" | "sebagian" | "belum";

	const db = useDb();
	const toast = useToast();
	const swal = useSwal();
	const { user, session } = useAuth();
	const isAdmin = computed(() => user.value?.level === "admin");
	const rows = ref<Row[]>([]);
	const loading = ref(true);

	async function load() {
		loading.value = true;
		try {
			const filter = props.rombelIds?.length ? `AND pr.rombongan_belajar_id IN (${props.rombelIds.map(() => "?").join(",")})` : "";
			rows.value = await db.query<Row>(
				`SELECT pr.id, r.nama AS rombel, pr.rombongan_belajar_id, pr.kode_mapel, m.singkat, g.nama AS guru, pr.terkunci, pr.dikirim_at,
					(SELECT COUNT(*) FROM nilai_rapor n WHERE n.pembelajaran_rapor_id = pr.id AND n.nilai IS NOT NULL) AS terisi,
					(SELECT COUNT(*) FROM nilai_rapor n WHERE n.pembelajaran_rapor_id = pr.id AND n.tp_optimal <> '[]' AND n.tp_perlu <> '[]') AS deskripsi,
					(SELECT COUNT(*) FROM anggota_rombel a WHERE a.rombongan_belajar_id = pr.rombongan_belajar_id) AS siswa
				FROM pembelajaran_rapor pr JOIN rombel r USING (rombongan_belajar_id) JOIN mapel_rapor m ON m.kode = pr.kode_mapel
				LEFT JOIN ptk g ON g.ptk_id = pr.ptk_id
				WHERE pr.semester_id = ? ${filter} ORDER BY CAST(r.tingkat AS INTEGER), r.nama, m.urutan`,
				[session.value?.semesterId, ...(props.rombelIds ?? [])]
			);
		}
		catch (e) {
			toast.add({ title: "Gagal memuat status penilaian", description: pesanError(e), color: "error" });
		}
		finally {
			loading.value = false;
		}
	}
	watch([() => session.value?.semesterId, () => props.rombelIds?.join()], load, { immediate: true });

	async function bukaKunci(r: Row) {
		if (!await swal.confirm(`Buka kunci ${r.singkat} ${r.rombel}?`, `${r.guru ?? "Guru"} bisa mengubah dan mengirim ulang nilainya setelah tarik data ulang.`, "Buka Kunci"))
			return;
		try {
			await db.execute("UPDATE pembelajaran_rapor SET terkunci = 0 WHERE id = ?", [r.id]);
			r.terkunci = 0;
			toast.add({ title: `${r.singkat} ${r.rombel} dibuka kuncinya`, color: "success" });
		}
		catch (e) {
			toast.add({ title: "Gagal membuka kunci", description: pesanError(e), color: "error" });
		}
	}

	// Lengkap = nilai terisi DAN deskripsi sesuai aturan (minimal 1 ✓ dan 1 !), untuk semua siswa.
	const siap = (r: Row) => Math.min(r.terisi, r.deskripsi);
	const pct = (r: Row) => (r.siswa ? Math.round(siap(r) / r.siswa * 100) : 0);
	const status = (r: Row): Status => (r.siswa && siap(r) >= r.siswa ? "lengkap" : r.terisi > 0 || r.deskripsi > 0 ? "sebagian" : "belum");
	// Satu sumber warna/ikon untuk ikon, progress, sel matriks, dan ringkasan — supaya selalu selaras.
	const GAYA = {
		lengkap: { label: "Lengkap", icon: "lucide:circle-check", warna: "success", teks: "text-success", sel: "bg-success/15 text-success" },
		sebagian: { label: "Sebagian", icon: "lucide:circle-dot", warna: "warning", teks: "text-warning", sel: "bg-warning/15 text-warning" },
		belum: { label: "Belum mulai", icon: "lucide:circle", warna: "neutral", teks: "text-dimmed", sel: "bg-elevated text-muted" }
	} as const;

	// Filter dari badge ringkasan: klik "Sebagian" → hanya yang sebagian. Klik lagi → semua.
	const saring = ref<Status | null>(null);
	const lolos = (r: Row) => !saring.value || status(r) === saring.value;

	// Kelas yang belum lengkap di atas, supaya mata langsung tertuju ke yang perlu ditindak.
	const perKelas = computed(() => {
		const map = new Map<string, { id: string, nama: string, items: Row[], selesai: number, urut: number }>();
		for (const r of rows.value) {
			if (!map.has(r.rombongan_belajar_id))
				map.set(r.rombongan_belajar_id, { id: r.rombongan_belajar_id, nama: r.rombel, items: [], selesai: 0, urut: map.size });
			const k = map.get(r.rombongan_belajar_id)!;
			k.items.push(r);
			if (status(r) === "lengkap")
				k.selesai++;
		}
		const lengkap = (k: { items: Row[], selesai: number }) => k.selesai === k.items.length;
		return [...map.values()].sort((a, b) => Number(lengkap(a)) - Number(lengkap(b)) || a.urut - b.urut);
	});
	const persenKelas = (k: { items: Row[], selesai: number }) => (k.items.length ? Math.round(k.selesai / k.items.length * 100) : 0);
	const warnaKelas = (k: { items: Row[], selesai: number }) => (k.selesai === k.items.length ? "success" : k.selesai ? "warning" : "neutral");

	const ringkasan = computed(() => {
		const n: Record<Status, number> = { lengkap: 0, sebagian: 0, belum: 0 };
		rows.value.forEach(r => n[status(r)]++);
		return n;
	});

	// ===== Matriks (banyak kelas) =====
	const modeMatriks = computed(() => perKelas.value.length > 1);
	// Kolom kelas tetap urut tingkat (tidak ikut diurutkan "belum lengkap dulu") supaya mudah dicari.
	const kolomKelas = computed(() => [...perKelas.value].sort((a, b) => a.urut - b.urut));
	const sel = computed(() => new Map(rows.value.map(r => [`${r.kode_mapel}:${r.rombongan_belajar_id}`, r])));
	const ambil = (kode: string, rombel: string) => sel.value.get(`${kode}:${rombel}`);
	// Baris mapel mengikuti urutan mapel rapor; saat difilter, mapel tanpa sel yang cocok disembunyikan.
	const barisMapel = computed(() => {
		const seen = new Map<string, string>();
		rows.value.forEach(r => seen.set(r.kode_mapel, r.singkat));
		return [...seen]
			.map(([kode, singkat]) => ({ kode, singkat }))
			.filter(m => !saring.value || rows.value.some(r => r.kode_mapel === m.kode && lolos(r)));
	});

	// ===== Kartu (satu kelas) =====
	const kartuKelas = computed(() => perKelas.value
		.map(k => ({ ...k, tampil: k.items.filter(lolos) }))
		.filter(k => k.tampil.length));
</script>

<template>
	<div v-if="loading && !rows.length" class="space-y-4">
		<USkeleton class="h-8 w-80" />
		<USkeleton class="h-96 w-full" />
	</div>

	<div v-else-if="!rows.length" class="flex flex-col items-center gap-3 py-24 text-center text-muted">
		<UIcon name="lucide:list-checks" class="size-12" />
		<p>Belum ada pembelajaran rapor untuk semester ini.</p>
		<UButton
			v-if="isAdmin"
			to="/e-rapor/referensi/pembelajaran"
			icon="lucide:arrow-right"
			trailing
			variant="soft">
			Susun Pembelajaran
		</UButton>
	</div>

	<div v-else class="space-y-4">
		<!-- Ringkasan sekaligus filter. -->
		<div class="flex flex-wrap items-center gap-2 text-sm">
			<span class="text-muted">{{ rows.length }} mapel-kelas:</span>
			<UButton
				v-for="(g, s) in GAYA"
				:key="s"
				:color="g.warna"
				size="sm"
				:icon="g.icon"
				:variant="saring === s ? 'solid' : 'subtle'"
				:aria-pressed="saring === s"
				@click="saring = saring === s ? null : s"
			>
				{{ g.label }} <span class="tabular-nums">{{ ringkasan[s] }}</span>
			</UButton>
			<UButton
				v-if="saring"
				variant="link"
				color="neutral"
				size="sm"
				icon="lucide:x"
				@click="saring = null">
				Tampilkan semua
			</UButton>
		</div>

		<!-- ===== Matriks: mapel × kelas ===== -->
		<PanelCard
			v-if="modeMatriks"
			title="Progres Nilai"
			icon="lucide:grid-3x3"
			description="Persen siswa yang sudah dinilai. Klik sel untuk detail.">
			<div class="overflow-x-auto">
				<table class="w-full text-sm">
					<thead class="text-xs text-muted">
						<tr>
							<th class="sticky left-0 bg-default p-2 text-left">
								Mapel
							</th>
							<th v-for="k in kolomKelas" :key="k.id" class="min-w-20 p-2 text-center font-medium">
								<p class="text-highlighted">
									{{ k.nama }}
								</p>
								<UBadge
									:color="warnaKelas(k)"
									variant="subtle"
									size="sm"
									class="mt-1 tabular-nums">
									{{ persenKelas(k) }}%
								</UBadge>
							</th>
						</tr>
					</thead>
					<tbody class="divide-y divide-default">
						<tr v-for="m in barisMapel" :key="m.kode">
							<td class="sticky left-0 bg-default p-2 font-medium whitespace-nowrap">
								{{ m.singkat }}
							</td>
							<td v-for="k in kolomKelas" :key="k.id" class="p-1 text-center">
								<template v-if="ambil(m.kode, k.id)">
									<UPopover>
										<button
											type="button"
											class="inline-flex w-16 cursor-pointer items-center justify-center gap-1 rounded py-1 text-xs font-semibold tabular-nums transition-opacity"
											:class="[GAYA[status(ambil(m.kode, k.id)!)].sel, lolos(ambil(m.kode, k.id)!) ? '' : 'opacity-25']"
											:aria-label="`${m.singkat} ${k.nama}: ${pct(ambil(m.kode, k.id)!)}% dinilai`"
										>
											{{ pct(ambil(m.kode, k.id)!) }}%
											<UIcon v-if="ambil(m.kode, k.id)!.terkunci" name="lucide:lock" class="size-3" />
										</button>
										<template #content>
											<div class="w-64 space-y-2 p-3 text-sm">
												<div>
													<p class="font-semibold">
														{{ m.singkat }} · {{ k.nama }}
													</p>
													<p class="text-xs text-muted">
														{{ ambil(m.kode, k.id)!.guru ?? "Belum ada pengampu" }}
													</p>
												</div>
												<UProgress :model-value="pct(ambil(m.kode, k.id)!)" :color="GAYA[status(ambil(m.kode, k.id)!)].warna" />
												<dl class="grid grid-cols-[1fr_auto] gap-x-3 gap-y-1 text-xs">
													<dt class="text-muted">
														Nilai terisi
													</dt>
													<dd class="text-right tabular-nums">
														{{ ambil(m.kode, k.id)!.terisi }}/{{ ambil(m.kode, k.id)!.siswa }}
													</dd>
													<dt class="text-muted">
														Deskripsi sesuai aturan
													</dt>
													<dd class="text-right tabular-nums">
														{{ ambil(m.kode, k.id)!.deskripsi }}/{{ ambil(m.kode, k.id)!.siswa }}
													</dd>
													<dt class="text-muted">
														Status kirim
													</dt>
													<dd class="text-right">
														{{ ambil(m.kode, k.id)!.terkunci ? `Terkirim ${ambil(m.kode, k.id)!.dikirim_at ?? ""}` : "Belum dikirim" }}
													</dd>
												</dl>
												<UButton
													v-if="isAdmin && ambil(m.kode, k.id)!.terkunci"
													block
													size="sm"
													variant="soft"
													icon="lucide:lock-open"
													@click="bukaKunci(ambil(m.kode, k.id)!)"
												>
													Buka Kunci
												</UButton>
											</div>
										</template>
									</UPopover>
								</template>
								<span v-else class="text-dimmed">–</span>
							</td>
						</tr>
					</tbody>
				</table>
				<p v-if="!barisMapel.length" class="py-8 text-center text-sm text-muted">
					Tidak ada mapel dengan status ini.
				</p>
			</div>
		</PanelCard>

		<!-- ===== Kartu: satu kelas (wali kelas) ===== -->
		<template v-else>
			<PanelCard
				v-for="k in kartuKelas"
				:key="k.id"
				:title="k.nama"
				icon="lucide:layers"
				:description="`${k.selesai}/${k.items.length} mapel lengkap`"
			>
				<template #actions>
					<UBadge :color="warnaKelas(k)" variant="subtle" class="tabular-nums">
						{{ persenKelas(k) }}%
					</UBadge>
				</template>
				<ul class="divide-y divide-default">
					<li v-for="r in k.tampil" :key="r.id" class="flex flex-wrap items-center gap-x-3 gap-y-1 py-2">
						<UIcon :name="GAYA[status(r)].icon" class="size-4 shrink-0" :class="GAYA[status(r)].teks" />
						<div class="min-w-0 flex-1 basis-32">
							<p class="truncate text-sm font-medium" :title="r.singkat">
								{{ r.singkat }}
							</p>
							<p class="truncate text-xs text-muted" :title="r.guru ?? undefined">
								{{ r.guru ?? "Belum ada pengampu" }}
							</p>
						</div>
						<div class="flex min-w-48 flex-2 items-center gap-3">
							<UProgress :model-value="pct(r)" :color="GAYA[status(r)].warna" class="flex-1" />
							<span class="shrink-0 text-right text-xs text-muted tabular-nums">
								{{ r.terisi }}/{{ r.siswa }} · {{ pct(r) }}%
							</span>
							<UTooltip v-if="r.terkunci" :text="`Dikirim guru ${r.dikirim_at ?? ''}${isAdmin ? ' — klik untuk buka kunci' : ''}`">
								<UButton
									icon="lucide:lock"
									size="xs"
									color="success"
									variant="ghost"
									:aria-label="`Buka kunci ${r.singkat} ${r.rombel}`"
									:disabled="!isAdmin"
									@click="bukaKunci(r)"
								/>
							</UTooltip>
							<UTooltip v-else text="Belum dikirim guru">
								<UIcon name="lucide:lock-open" class="mx-1.5 size-4 shrink-0 text-dimmed" />
							</UTooltip>
						</div>
					</li>
				</ul>
			</PanelCard>
			<p v-if="!kartuKelas.length" class="py-8 text-center text-sm text-muted">
				Tidak ada mapel dengan status ini.
			</p>
		</template>
	</div>
</template>
