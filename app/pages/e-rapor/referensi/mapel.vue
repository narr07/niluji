<script setup lang="ts">
	// Satu-satunya tempat mengatur mapel rapor. Padanan e-Rapor: Data Mata Pelajaran (singkatan,
	// mapel transkrip, tambah mapel lokal) + Kelompok Mapel + Mapping Rapor (nama, kelompok, urutan).
	// Bedanya: diatur sekali untuk semua tingkat; fase menentukan mapel muncul di kelas mana.
	// Disimpan sebagai template, lalu langsung diterapkan ke semua kelas lewat Susun Pembelajaran.

	interface Pemakaian { kelas: number, daftar: string }

	const db = useDb();
	const swal = useSwal();
	const toast = useToast();
	const { session } = useAuth();
	const { getTemplate, saveTemplate, getKelompok, saveKelompok, susun } = usePembelajaranRapor();

	const loading = ref(true);
	const saving = ref(false);
	const tpl = ref<TemplateMapel[]>([]);
	const kelompok = ref<KelompokMapel[]>([]);
	const pakai = ref<Record<string, Pemakaian>>({});
	const awal = ref("");

	const FASE: Fase[] = ["A", "B", "C"];
	const snapshot = () => JSON.stringify([tpl.value, kelompok.value]);
	const dirty = computed(() => !loading.value && snapshot() !== awal.value);

	async function loadPemakaian() {
		const sem = session.value?.semesterId;
		if (!sem) {
			pakai.value = {};
			return;
		}
		const rows = await db.query<{ kode_mapel: string, kelas: number, daftar: string }>(
			`SELECT pr.kode_mapel, COUNT(*) AS kelas,
				(SELECT group_concat(nama, ', ') FROM (SELECT r.nama FROM pembelajaran_rapor p2 JOIN rombel r USING (rombongan_belajar_id)
					WHERE p2.kode_mapel = pr.kode_mapel AND p2.semester_id = ?1 ORDER BY CAST(r.tingkat AS INTEGER), r.nama)) AS daftar
			FROM pembelajaran_rapor pr WHERE pr.semester_id = ?1 GROUP BY pr.kode_mapel`,
			[sem]
		);
		pakai.value = Object.fromEntries(rows.map(r => [r.kode_mapel, { kelas: r.kelas, daftar: r.daftar }]));
	}

	// Referensi mapel Dapodik: kode yang tidak ada di sini (atau kode "Guru Kelas") tidak bisa
	// dikirim ke Dapodik sebagai mata evaluasi.
	const refNama = ref(new Map<string, string>());
	const kodeSah = (kode: string) => !refNama.value.size || (refNama.value.has(kode) && kode !== KODE_GURU_KELAS);
	const kodeTidakSah = computed(() => tpl.value.filter(m => m.aktif && !kodeSah(m.kode)));

	async function load() {
		loading.value = true;
		try {
			const ref = await db.query<{ mata_pelajaran_id: string, nama: string }>("SELECT mata_pelajaran_id, nama FROM mata_pelajaran");
			refNama.value = new Map(ref.map(r => [r.mata_pelajaran_id, r.nama]));
			const [t, k] = await Promise.all([getTemplate(), getKelompok(), loadPemakaian()]);
			tpl.value = t.map(m => ({ ...m, transkrip: m.transkrip !== false }));
			kelompok.value = k;
			awal.value = snapshot();
		}
		catch (e) {
			toast.add({ title: "Gagal memuat mata pelajaran", description: pesanError(e), color: "error" });
		}
		finally {
			loading.value = false;
		}
	}
	watch(() => session.value?.semesterId, load, { immediate: true });

	// ===== Pengelompokan tampilan =====
	const grup = computed(() => {
		const nama = kelompok.value.map(k => k.nama);
		const out = kelompok.value.map(k => ({ nama: k.nama, aktif: k.aktif, items: tpl.value.filter(m => m.kelompok === k.nama) }));
		const lepas = tpl.value.filter(m => !nama.includes(m.kelompok));
		if (lepas.length)
			out.push({ nama: "Tanpa kelompok", aktif: true, items: lepas });
		return out;
	});
	const kelompokItems = computed(() => kelompok.value.map(k => k.nama));

	// Drag & drop seperti niluji. Mapel diurutkan di dalam kelompoknya saja (urutan rapor per kelompok):
	// posisi mapel kelompok lain di daftar tidak berubah.
	const { pasang } = useDragUrut();

	function pindahMapel(namaKelompok: string, dari: number, ke: number) {
		const anggota = tpl.value.filter(m => m.kelompok === namaKelompok);
		const posisi = tpl.value.map((m, i) => (m.kelompok === namaKelompok ? i : -1)).filter(i => i >= 0);
		const urutBaru = pindahkan(anggota, dari, ke);
		const hasil = [...tpl.value];
		posisi.forEach((pos, i) => { hasil[pos] = urutBaru[i]!; });
		tpl.value = hasil;
	}

	function pindahKelompok(dari: number, ke: number) {
		kelompok.value = pindahkan(kelompok.value, dari, ke);
	}

	function toggleFase(m: TemplateMapel, f: Fase, on: boolean) {
		m.fase = on ? [...new Set([...m.fase, f])].sort() as Fase[] : m.fase.filter(x => x !== f);
	}

	async function hapusMapel(m: TemplateMapel) {
		const p = pakai.value[m.kode];
		if (!await swal.confirm(`Hapus ${m.nama} dari daftar?`, p ? `Dipakai di ${p.kelas} kelas. Setelah disimpan, mapel ini hilang dari rapor (kecuali yang sudah ada nilainya).` : "Mapel ini belum dipakai di kelas mana pun.", "Hapus"))
			return;
		tpl.value = tpl.value.filter(x => x !== m);
	}

	// ===== Kelompok =====
	function renameKelompok(i: number, baru: string) {
		const lama = kelompok.value[i]!.nama;
		kelompok.value[i]!.nama = baru;
		tpl.value.forEach((m) => {
			if (m.kelompok === lama)
				m.kelompok = baru;
		});
	}

	function tambahKelompok() {
		let n = kelompok.value.length + 1;
		while (kelompok.value.some(k => k.nama === `Kelompok ${n}`))
			n++;
		kelompok.value.push({ nama: `Kelompok ${n}`, aktif: true });
	}

	const jumlahDiKelompok = (nama: string) => tpl.value.filter(m => m.kelompok === nama).length;

	function hapusKelompok(i: number) {
		kelompok.value.splice(i, 1);
	}

	// ===== Validasi & simpan =====
	const masalah = computed(() => {
		const out: string[] = [];
		const namaKel = kelompok.value.map(k => k.nama.trim());
		if (namaKel.some(n => !n))
			out.push("Ada nama kelompok yang kosong");
		if (new Set(namaKel).size !== namaKel.length)
			out.push("Ada nama kelompok yang sama");
		for (const m of tpl.value.filter(x => x.aktif)) {
			const label = m.nama.trim() || m.kode;
			if (!m.nama.trim())
				out.push(`${label}: nama di rapor kosong`);
			if (!m.singkat.trim())
				out.push(`${label}: singkatan kosong`);
			else if (m.singkat.trim().length > SINGKAT_MAX)
				out.push(`${label}: singkatan lebih dari ${SINGKAT_MAX} karakter`);
			if (!m.fase.length)
				out.push(`${label}: belum dipilih fasenya`);
			if (!namaKel.includes(m.kelompok))
				out.push(`${label}: belum masuk kelompok`);
		}
		return out;
	});

	async function simpan() {
		if (masalah.value.length || saving.value)
			return;
		saving.value = true;
		try {
			await saveKelompok(kelompok.value.map(k => ({ ...k, nama: k.nama.trim() })));
			await saveTemplate(tpl.value.map(m => ({ ...m, nama: m.nama.trim(), singkat: m.singkat.trim() })));
			const sem = session.value?.semesterId;
			if (sem) {
				const r = await susun(sem);
				toast.add({ title: "Mapel disimpan & diterapkan", description: `${r.pembelajaran} pembelajaran untuk ${r.kelas} kelas disusun ulang.`, color: "success" });
			}
			else {
				toast.add({ title: "Mapel disimpan", description: "Akan diterapkan setelah Sinkron Dapodik.", color: "success" });
			}
			await load();
		}
		catch (e) {
			swal.error("Gagal menyimpan", pesanError(e));
		}
		finally {
			saving.value = false;
		}
	}

	async function batal() {
		if (dirty.value && !await swal.confirm("Buang perubahan?", "Perubahan mapel belum disimpan.", "Buang"))
			return;
		await load();
	}

	async function kembalikanBawaan() {
		if (!await swal.confirm("Kembalikan ke bawaan?", "Daftar mapel dan kelompok kembali ke susunan bawaan SD Kurikulum Merdeka. Belum tersimpan sampai Anda klik Simpan.", "Kembalikan"))
			return;
		tpl.value = structuredClone(DEFAULT_TEMPLATE).map(m => ({ ...m, transkrip: true }));
		kelompok.value = structuredClone(DEFAULT_KELOMPOK);
	}

	onBeforeRouteLeave(async () => {
		if (dirty.value)
			return await swal.confirm("Tinggalkan halaman?", "Perubahan mapel belum disimpan.", "Tinggalkan");
	});

	// ===== Tambah mapel dari referensi Dapodik =====
	const tambahOpen = ref(false);
	const refMapel = ref<{ label: string, value: string, nama: string }[]>([]);
	const baru = reactive({ kode: undefined as string | undefined, nama: "", singkat: "", kelompok: "", fase: ["A", "B", "C"] as Fase[] });

	async function openTambah() {
		try {
			const ada = new Set(tpl.value.map(m => m.kode));
			const rows = await db.query<{ mata_pelajaran_id: string, nama: string }>(
				"SELECT mata_pelajaran_id, nama FROM mata_pelajaran ORDER BY nama COLLATE NOCASE"
			);
			refMapel.value = rows.filter(r => !ada.has(r.mata_pelajaran_id) && r.mata_pelajaran_id !== KODE_GURU_KELAS)
				.map(r => ({ label: `${r.nama} · ${r.mata_pelajaran_id}`, value: r.mata_pelajaran_id, nama: r.nama }));
			Object.assign(baru, { kode: undefined, nama: "", singkat: "", kelompok: kelompok.value[0]?.nama ?? "", fase: ["A", "B", "C"] });
			tambahOpen.value = true;
		}
		catch (e) {
			toast.add({ title: "Gagal memuat referensi mapel", description: pesanError(e), color: "error" });
		}
	}

	// ===== Ganti kode Dapodik (mis. mapel lokal yang terlanjur memakai kode Guru Kelas) =====
	// Kode dipindah di semua tabel yang memakainya, jadi nilai, TP, dan transkrip yang sudah ada tetap aman.
	const ganti = reactive({ buka: false, mapel: undefined as TemplateMapel | undefined, kode: undefined as string | undefined, pilihan: [] as { label: string, value: string }[], proses: false });

	async function bukaGanti(m: TemplateMapel) {
		if (dirty.value) {
			toast.add({ title: "Simpan dulu perubahan mapel", description: "Ganti kode langsung tersimpan, jadi perubahan lain harus disimpan atau dibatalkan dulu.", color: "warning" });
			return;
		}
		// Kode tujuan tidak boleh sedang dipakai mapel lain (aktif) atau pembelajaran mana pun.
		const terpakai = new Set((await db.query<{ k: string }>("SELECT DISTINCT kode_mapel AS k FROM pembelajaran_rapor")).map(x => x.k));
		const mapelLain = new Set(tpl.value.filter(x => x !== m && x.aktif).map(x => x.kode));
		ganti.pilihan = [...refNama.value]
			.filter(([k]) => k !== KODE_GURU_KELAS && k !== m.kode && !terpakai.has(k) && !mapelLain.has(k))
			.sort((a, b) => a[1].localeCompare(b[1], "id"))
			.map(([k, n]) => ({ label: `${n} · ${k}`, value: k }));
		Object.assign(ganti, { buka: true, mapel: m, kode: undefined });
	}

	async function simpanGanti() {
		const m = ganti.mapel;
		const kode = ganti.kode;
		if (!m || !kode)
			return;
		const lama = m.kode;
		ganti.proses = true;
		try {
			await db.batch([
				{ sql: "DELETE FROM mapel_rapor WHERE kode = ?", params: [kode] },
				{ sql: "UPDATE mapel_rapor SET kode = ? WHERE kode = ?", params: [kode, lama] },
				{ sql: "UPDATE pembelajaran_rapor SET kode_mapel = ? WHERE kode_mapel = ?", params: [kode, lama] },
				{ sql: "UPDATE tujuan_pembelajaran SET kode_mapel = ? WHERE kode_mapel = ?", params: [kode, lama] },
				{ sql: "UPDATE nilai_transkrip SET kode_mapel = ? WHERE kode_mapel = ?", params: [kode, lama] }
			]);
			// Entri template lain yang memakai kode tujuan (pasti tidak aktif & tidak dipakai) dibuang.
			const baruTpl = tpl.value.filter(x => x === m || x.kode !== kode).map(x => (x === m ? { ...x, kode } : x));
			await saveTemplate(baruTpl);
			ganti.buka = false;
			toast.add({ title: `Kode ${m.nama} diganti`, description: `${lama} → ${kode} (${refNama.value.get(kode)}). Nilai dan TP tetap.`, color: "success" });
			await load();
		}
		catch (e) {
			swal.error("Gagal mengganti kode", pesanError(e));
		}
		finally {
			ganti.proses = false;
		}
	}

	watch(() => baru.kode, (kode) => {
		const r = refMapel.value.find(x => x.value === kode);
		if (r) {
			baru.nama = r.nama;
			baru.singkat = r.nama.slice(0, SINGKAT_MAX);
		}
	});

	const bisaTambah = computed(() => !!baru.kode && !!baru.nama.trim() && !!baru.singkat.trim()
		&& baru.singkat.trim().length <= SINGKAT_MAX && baru.fase.length > 0 && !!baru.kelompok);

	function tambah() {
		if (!bisaTambah.value)
			return;
		tpl.value.push({
			kode: baru.kode!,
			nama: baru.nama.trim(),
			singkat: baru.singkat.trim(),
			kelompok: baru.kelompok,
			fase: [...baru.fase],
			aktif: true,
			transkrip: true,
			lokal: true
		});
		tambahOpen.value = false;
		toast.add({ title: `${baru.nama} ditambahkan`, description: "Klik Simpan untuk menerapkan ke semua kelas.", color: "info" });
	}

	function toggleFaseBaru(f: Fase, on: boolean) {
		baru.fase = on ? [...new Set([...baru.fase, f])].sort() as Fase[] : baru.fase.filter(x => x !== f);
	}
</script>

<template>
	<ErPage id="ref-mapel" title="Mata Pelajaran">
		<template #right>
			<UBadge
				v-if="dirty"
				color="warning"
				variant="subtle"
				icon="lucide:circle-dot">
				Belum disimpan
			</UBadge>
			<UButton
				icon="lucide:plus"
				variant="soft"
				:disabled="loading || saving"
				@click="openTambah">
				Tambah Mapel
			</UButton>
			<UButton
				v-if="dirty"
				color="neutral"
				variant="ghost"
				:disabled="saving"
				@click="batal">
				Batal
			</UButton>
			<UButton
				icon="lucide:save"
				variant="solid"
				:loading="saving"
				:disabled="!dirty || !!masalah.length"
				@click="simpan">
				Simpan & Terapkan
			</UButton>
		</template>

		<div v-if="loading" class="space-y-4">
			<USkeleton class="h-32 w-full" />
			<USkeleton class="h-80 w-full" />
		</div>

		<div v-else class="space-y-4">
			<UAlert
				v-if="kodeTidakSah.length"
				color="warning"
				variant="subtle"
				icon="lucide:link-2-off"
				:title="`${kodeTidakSah.length} mapel memakai kode yang bukan mapel Dapodik`"
				:description="`${kodeTidakSah.map(m => m.nama).join(', ')}. Nama di rapor boleh apa saja, tapi kode harus kode mata pelajaran dari referensi Dapodik supaya nilai bisa dikirim. Klik &quot;Ubah kode&quot; di mapel tersebut; nilai yang sudah ada tetap aman.`"
			/>
			<UAlert
				v-if="masalah.length"
				color="error"
				variant="subtle"
				icon="lucide:triangle-alert"
				title="Lengkapi dulu sebelum disimpan"
				:description="masalah.join(' · ')"
			/>

			<PanelCard title="Kelompok Mapel" icon="lucide:layers" description="Seret ⋮⋮ untuk mengatur urutan kelompok di rapor">
				<template #actions>
					<UButton
						size="sm"
						variant="ghost"
						icon="lucide:plus"
						:disabled="saving"
						@click="tambahKelompok">
						Tambah Kelompok
					</UButton>
				</template>
				<ul :ref="el => pasang(el, pindahKelompok)" class="divide-y divide-default">
					<li v-for="(k, i) in kelompok" :key="i" class="flex items-center gap-2 bg-default py-2">
						<span
							data-drag-handle
							draggable="false"
							class="shrink-0 cursor-grab touch-none text-muted select-none"
							style="-webkit-user-drag: none;"
							:aria-label="`Seret untuk mengurutkan ${k.nama}`"
						>
							<UIcon name="lucide:grip-vertical" class="pointer-events-none block size-5" />
						</span>
						<span class="w-5 text-right text-xs text-dimmed tabular-nums">{{ i + 1 }}</span>
						<UInput
							:model-value="k.nama"
							size="sm"
							class="flex-1"
							:aria-label="`Nama kelompok ${i + 1}`"
							:color="!k.nama.trim() ? 'error' : 'primary'"
							:disabled="saving"
							@update:model-value="v => renameKelompok(i, String(v))"
						/>
						<span class="w-20 text-right text-xs text-muted">{{ jumlahDiKelompok(k.nama) }} mapel</span>
						<USwitch
							v-model="k.aktif"
							size="sm"
							:aria-label="`Kelompok ${k.nama} aktif`"
							:disabled="saving" />
						<div class="flex">
							<UTooltip :text="jumlahDiKelompok(k.nama) ? 'Pindahkan dulu mapelnya ke kelompok lain' : 'Hapus kelompok'">
								<UButton
									icon="lucide:trash-2"
									size="xs"
									color="error"
									variant="ghost"
									:disabled="!!jumlahDiKelompok(k.nama) || kelompok.length <= 1 || saving"
									:aria-label="`Hapus kelompok ${k.nama}`"
									@click="hapusKelompok(i)"
								/>
							</UTooltip>
						</div>
					</li>
				</ul>
			</PanelCard>

			<PanelCard
				v-for="g in grup"
				:key="g.nama"
				:title="g.nama"
				icon="lucide:book-open"
				:description="`${g.items.filter(m => m.aktif).length} dari ${g.items.length} mapel dipakai${g.aktif ? '' : ' · kelompok tidak aktif'}`"
			>
				<div class="overflow-x-auto">
					<table class="w-full text-sm">
						<thead class="text-left text-xs text-muted">
							<tr>
								<th class="w-16 p-2">
									<span class="sr-only">Urutan</span>
								</th>
								<th class="w-14 p-2">
									Pakai
								</th>
								<th class="min-w-56 p-2">
									Nama di Rapor
								</th>
								<th class="w-40 p-2">
									Singkatan
								</th>
								<th class="p-2 text-center">
									Fase
								</th>
								<th class="w-44 p-2">
									Kelompok
								</th>
								<th class="w-20 p-2 text-center">
									Transkrip
								</th>
								<th class="w-24 p-2 text-right">
									Dipakai
								</th>
								<th class="w-10 p-2" />
							</tr>
						</thead>
						<tbody :ref="el => pasang(el, (dari, ke) => pindahMapel(g.nama, dari, ke))" class="divide-y divide-default">
							<tr
								v-for="(m, i) in g.items"
								:key="m.kode"
								class="bg-default"
								:class="m.aktif ? '' : 'opacity-50'">
								<td class="p-2">
									<div class="flex items-center gap-1.5">
										<span
											data-drag-handle
											draggable="false"
											class="shrink-0 cursor-grab touch-none text-muted select-none"
											style="-webkit-user-drag: none;"
											:aria-label="`Seret untuk mengurutkan ${m.nama}`"
										>
											<UIcon name="lucide:grip-vertical" class="pointer-events-none block size-5" />
										</span>
										<span class="text-xs text-dimmed tabular-nums">{{ i + 1 }}</span>
									</div>
								</td>
								<td class="p-2">
									<USwitch
										v-model="m.aktif"
										size="sm"
										:aria-label="`Pakai ${m.nama}`"
										:disabled="saving" />
								</td>
								<td class="p-2">
									<UInput
										v-model="m.nama"
										size="sm"
										class="w-full"
										:aria-label="`Nama di rapor ${m.kode}`"
										:color="m.aktif && !m.nama.trim() ? 'error' : 'primary'"
										:disabled="saving"
									/>
									<p class="mt-0.5 flex flex-wrap items-center gap-x-1 text-xs text-dimmed">
										<span class="font-mono tabular-nums">Kode Dapodik {{ m.kode }}</span>
										<span>· {{ refNama.get(m.kode) ?? "tidak ada di referensi" }}</span>
										<span v-if="m.lokal">· ditambahkan</span>
										<UButton
											size="xs"
											variant="link"
											color="neutral"
											icon="lucide:pencil"
											class="p-0"
											:disabled="saving"
											:aria-label="`Ubah kode Dapodik ${m.nama}`"
											@click="bukaGanti(m)"
										>
											Ubah kode
										</UButton>
									</p>
									<p v-if="!kodeSah(m.kode)" class="mt-0.5 flex items-center gap-1 text-xs text-error">
										<UIcon name="lucide:circle-alert" class="size-3.5 shrink-0" />
										{{ m.kode === KODE_GURU_KELAS ? "Kode Guru Kelas, bukan mata pelajaran" : "Bukan kode mapel Dapodik" }}: nilainya tidak bisa dikirim ke Dapodik.
									</p>
								</td>
								<td class="p-2">
									<UInput
										v-model="m.singkat"
										size="sm"
										class="w-full"
										:maxlength="SINGKAT_MAX"
										:aria-label="`Singkatan ${m.nama}`"
										:color="m.aktif && (!m.singkat.trim() || m.singkat.trim().length > SINGKAT_MAX) ? 'error' : 'primary'"
										:disabled="saving"
									/>
									<p class="mt-0.5 text-right text-xs tabular-nums" :class="m.singkat.length >= SINGKAT_MAX ? 'text-warning' : 'text-dimmed'">
										{{ m.singkat.length }}/{{ SINGKAT_MAX }}
									</p>
								</td>
								<td class="p-2">
									<div class="flex justify-center gap-2">
										<UCheckbox
											v-for="f in FASE"
											:key="f"
											:label="f"
											:disabled="saving"
											:model-value="m.fase.includes(f)"
											@update:model-value="v => toggleFase(m, f, !!v)"
										/>
									</div>
								</td>
								<td class="p-2">
									<USelect
										v-model="m.kelompok"
										:items="kelompokItems"
										size="sm"
										class="w-full"
										:aria-label="`Kelompok ${m.nama}`"
										:disabled="saving" />
								</td>
								<td class="p-2 text-center">
									<USwitch
										v-model="m.transkrip"
										size="sm"
										:aria-label="`${m.nama} ikut transkrip ijazah`"
										:disabled="saving" />
								</td>
								<td class="p-2 text-right text-xs tabular-nums">
									<UTooltip v-if="pakai[m.kode]" :text="pakai[m.kode]!.daftar">
										<span>{{ pakai[m.kode]!.kelas }} kelas</span>
									</UTooltip>
									<span v-else class="text-dimmed">–</span>
								</td>
								<td class="p-2">
									<UButton
										v-if="m.lokal"
										icon="lucide:trash-2"
										size="xs"
										color="error"
										variant="ghost"
										:aria-label="`Hapus ${m.nama}`"
										:disabled="saving"
										@click="hapusMapel(m)"
									/>
								</td>
							</tr>
						</tbody>
					</table>
				</div>
			</PanelCard>

			<div class="flex items-center justify-between gap-2 text-xs text-muted">
				<p>
					Seret ⋮⋮ untuk mengurutkan mapel di dalam kelompoknya. Fase A = Kelas 1–2, Fase B = Kelas 3–4, Fase C = Kelas 5–6. Mapel yang ada di Dapodik (mis. PAI, PJOK) tetap diampu guru dari Dapodik; mapel lain diampu guru kelas.
				</p>
				<UButton
					size="xs"
					color="neutral"
					variant="ghost"
					icon="lucide:rotate-ccw"
					:disabled="saving"
					@click="kembalikanBawaan">
					Kembalikan bawaan
				</UButton>
			</div>
		</div>
	</ErPage>

	<UModal v-model:open="tambahOpen" title="Tambah Mata Pelajaran" description="Pilih dari referensi mapel Dapodik supaya kodenya cocok saat kirim nilai.">
		<template #body>
			<form id="form-tambah-mapel" class="flex flex-col gap-3" @submit.prevent="tambah">
				<UFormField label="Mapel referensi Dapodik" :help="refMapel.length ? `${refMapel.length} mapel tersedia` : 'Referensi kosong. Jalankan Sinkron Dapodik dulu.'">
					<USelectMenu
						v-model="baru.kode"
						:items="refMapel"
						value-key="value"
						virtualize
						placeholder="Cari nama mapel…"
						class="w-full"
					/>
				</UFormField>
				<UFormField label="Nama di rapor">
					<UInput v-model="baru.nama" class="w-full" />
				</UFormField>
				<div class="grid grid-cols-2 gap-3">
					<UFormField label="Singkatan" :help="`${baru.singkat.length}/${SINGKAT_MAX} karakter`">
						<UInput v-model="baru.singkat" :maxlength="SINGKAT_MAX" class="w-full" />
					</UFormField>
					<UFormField label="Kelompok">
						<USelect v-model="baru.kelompok" :items="kelompokItems" class="w-full" />
					</UFormField>
				</div>
				<UFormField label="Fase">
					<div class="flex gap-4">
						<UCheckbox
							v-for="f in FASE"
							:key="f"
							:label="`Fase ${f}`"
							:model-value="baru.fase.includes(f)"
							@update:model-value="v => toggleFaseBaru(f, !!v)"
						/>
					</div>
				</UFormField>
			</form>
		</template>
		<template #footer>
			<div class="flex w-full justify-end gap-2">
				<UButton color="neutral" variant="outline" @click="tambahOpen = false">
					Batal
				</UButton>
				<UButton
					type="submit"
					form="form-tambah-mapel"
					variant="solid"
					icon="lucide:plus"
					:disabled="!bisaTambah">
					Tambahkan
				</UButton>
			</div>
		</template>
	</UModal>

	<UModal v-model:open="ganti.buka" title="Ubah Kode Dapodik" :description="ganti.mapel ? `${ganti.mapel.nama} · sekarang ${ganti.mapel.kode}` : undefined">
		<template #body>
			<div class="space-y-3">
				<UFormField label="Kode mata pelajaran Dapodik" help="Pilih mapel yang dipakai sekolah di Dapodik. Nama di rapor tidak berubah.">
					<USelectMenu
						v-model="ganti.kode"
						:items="ganti.pilihan"
						value-key="value"
						virtualize
						placeholder="Cari nama atau kode, mis. Seni"
						class="w-full"
					/>
				</UFormField>
				<UAlert
					color="info"
					variant="subtle"
					icon="lucide:shield-check"
					description="Kode dipindah sekaligus di pembelajaran, TP, nilai, dan transkrip, jadi isian guru yang sudah ada tidak hilang."
				/>
			</div>
		</template>
		<template #footer>
			<div class="flex w-full justify-end gap-2">
				<UButton variant="ghost" color="neutral" @click="ganti.buka = false">
					Batal
				</UButton>
				<UButton
					icon="lucide:replace"
					:loading="ganti.proses"
					:disabled="!ganti.kode"
					@click="simpanGanti">
					Simpan Kode
				</UButton>
			</div>
		</template>
	</UModal>
</template>
