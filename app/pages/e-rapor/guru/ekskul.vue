<script setup lang="ts">
	// Padanan e-Rapor: Input Nilai Ekstrakurikuler (khusus pembina). Satu tabel per ekskul,
	// predikat sekali klik, keterangan otomatis dari predikat (boleh ditulis sendiri), tersimpan otomatis.
	// Pembina: ekskul yang ia bina (Dapodik: rombel ekskul). Admin: semua ekskul.

	interface Anggota { pesertaDidikId: string, nama: string, kelas: string | null, predikat: Predikat | null, keterangan: string }

	const db = useDb();
	const toast = useToast();
	const swal = useSwal();
	const { user, session, waliRombel } = useAuth();
	// Wali kelas boleh mengisi nilai ekskul siswanya kalau pembina belum (seperti e-Rapor).
	const waliIds = computed(() => waliRombel.value.map(r => r.rombongan_belajar_id));
	const { isGuruMode, load: loadMode } = useAppMode();
	const isAdmin = computed(() => user.value?.level === "admin");

	// ===== Pilihan ekskul =====
	const ekskulList = ref<{ value: string, label: string, pembina: number }[]>([]);
	const ekskulId = ref<string>();
	const ekskulNama = computed(() => ekskulList.value.find(e => e.value === ekskulId.value)?.label ?? "");
	// Bukan pembina (wali kelas) → hanya siswa kelas yang ia walikan.
	const sebagaiWali = computed(() => !isAdmin.value && !ekskulList.value.find(e => e.value === ekskulId.value)?.pembina);

	async function loadEkskul() {
		const sem = session.value?.semesterId;
		const wali = waliIds.value;
		const phWali = wali.map(() => "?").join(",") || "NULL";
		ekskulList.value = !sem
			? []
			: isAdmin.value
				? await db.query(
					`SELECT rombongan_belajar_id AS value, nama AS label, 1 AS pembina FROM rombel
					WHERE semester_id = ? AND jenis_rombel = '51' ORDER BY nama COLLATE NOCASE`,
					[sem]
				)
				: await db.query(
					`SELECT e.rombongan_belajar_id AS value, e.nama AS label, (e.ptk_id = ?) AS pembina FROM rombel e
					WHERE e.semester_id = ? AND e.jenis_rombel = '51' AND (e.ptk_id = ? OR EXISTS (
						SELECT 1 FROM anggota_rombel a JOIN anggota_rombel k ON k.peserta_didik_id = a.peserta_didik_id
						WHERE a.rombongan_belajar_id = e.rombongan_belajar_id AND k.rombongan_belajar_id IN (${phWali})))
					ORDER BY e.nama COLLATE NOCASE`,
					[user.value?.ptk_id, sem, user.value?.ptk_id, ...wali]
				);
		if (!ekskulList.value.some(e => e.value === ekskulId.value))
			ekskulId.value = ekskulList.value[0]?.value;
	}

	// ===== Anggota & nilai =====
	const anggota = ref<Anggota[]>([]);
	const loading = ref(true);
	let permintaan = 0;

	async function load() {
		const nomor = ++permintaan;
		const sem = session.value?.semesterId;
		if (!ekskulId.value || !sem) {
			anggota.value = [];
			loading.value = false;
			return;
		}
		loading.value = true;
		try {
			const rows = await db.query<Anggota>(
				`SELECT s.peserta_didik_id AS pesertaDidikId, s.nama,
					(SELECT r.nama FROM anggota_rombel k JOIN rombel r USING (rombongan_belajar_id)
						WHERE k.peserta_didik_id = s.peserta_didik_id AND r.jenis_rombel = '1' AND r.semester_id = ? LIMIT 1) AS kelas,
					n.predikat, COALESCE(n.keterangan, '') AS keterangan
				FROM anggota_rombel a JOIN siswa_rapor s USING (peserta_didik_id)
				LEFT JOIN nilai_ekskul n ON n.semester_id = ? AND n.rombongan_belajar_id = a.rombongan_belajar_id AND n.peserta_didik_id = s.peserta_didik_id
				WHERE a.rombongan_belajar_id = ?
				${sebagaiWali.value ? `AND a.peserta_didik_id IN (SELECT peserta_didik_id FROM anggota_rombel WHERE rombongan_belajar_id IN (${waliIds.value.map(() => "?").join(",") || "NULL"}))` : ""}`,
				[sem, sem, ekskulId.value, ...(sebagaiWali.value ? waliIds.value : [])]
			);
			// Urut kelas lalu nama, supaya pembina bisa mengisi per kelas.
			rows.sort((a, b) => (a.kelas ?? "~").localeCompare(b.kelas ?? "~", "id", { numeric: true }) || a.nama.localeCompare(b.nama, "id"));
			if (nomor === permintaan)
				anggota.value = rows;
		}
		catch (e) {
			toast.add({ title: "Gagal memuat anggota ekskul", description: pesanError(e), color: "error" });
		}
		finally {
			if (nomor === permintaan)
				loading.value = false;
		}
	}

	async function init() {
		await loadMode();
		await loadEkskul();
		await load();
	}
	watch(() => session.value?.semesterId, init, { immediate: true });
	watch(ekskulId, async () => {
		await flush();
		kelasFilter.value = "*";
		await load();
	});

	// ===== Simpan otomatis =====
	const pending = new Map<string, { timer: ReturnType<typeof setTimeout>, run: () => Promise<void> }>();
	const saveState = ref<"idle" | "saving" | "saved" | "error">("idle");

	function queueSave(s: Anggota) {
		const key = s.pesertaDidikId;
		// Parameter dibekukan sekarang, supaya tetap ke ekskul yang benar walau pilihan diganti.
		const params = [session.value?.semesterId, ekskulId.value, s.pesertaDidikId, s.predikat, s.keterangan.trim() || null];
		clearTimeout(pending.get(key)?.timer);
		saveState.value = "saving";
		const run = async () => {
			pending.delete(key);
			try {
				await db.execute(
					`INSERT INTO nilai_ekskul (semester_id, rombongan_belajar_id, peserta_didik_id, predikat, keterangan, updated_at)
					VALUES (?, ?, ?, ?, ?, datetime('now','localtime'))
					ON CONFLICT (semester_id, rombongan_belajar_id, peserta_didik_id) DO UPDATE SET
						predikat = excluded.predikat, keterangan = excluded.keterangan, updated_at = excluded.updated_at`,
					params
				);
				if (!pending.size)
					saveState.value = "saved";
			}
			catch (e) {
				saveState.value = "error";
				toast.add({ title: `Gagal menyimpan ${s.nama}`, description: pesanError(e), color: "error" });
			}
		};
		pending.set(key, { timer: setTimeout(run, 500), run });
	}

	async function flush() {
		const semua = [...pending.values()];
		semua.forEach(p => clearTimeout(p.timer));
		await Promise.all(semua.map(p => p.run()));
	}
	onBeforeRouteLeave(flush);

	// Klik predikat yang sama lagi = batalkan.
	function setPredikat(s: Anggota, p: Predikat) {
		s.predikat = s.predikat === p ? null : p;
		queueSave(s);
	}
	function setKeterangan(s: Anggota, teks: string) {
		s.keterangan = teks.slice(0, KETERANGAN_MAX);
		queueSave(s);
	}

	// ===== Filter, ringkasan, isi cepat =====
	const kelasFilter = ref("*");
	const cari = ref("");
	const kelasItems = computed(() => [
		{ value: "*", label: "Semua kelas" },
		...[...new Set(anggota.value.map(a => a.kelas ?? "Tanpa kelas"))].map(k => ({ value: k, label: k }))
	]);
	const tampil = computed(() => {
		const q = cari.value.trim().toLowerCase();
		return anggota.value.filter(a =>
			(kelasFilter.value === "*" || (a.kelas ?? "Tanpa kelas") === kelasFilter.value)
			&& (!q || a.nama.toLowerCase().includes(q)));
	});
	const dinilai = computed(() => anggota.value.filter(a => a.predikat).length);

	async function isiKosong(p: Predikat) {
		const kosong = tampil.value.filter(a => !a.predikat);
		if (!kosong.length) {
			toast.add({ title: "Semua siswa di daftar ini sudah dinilai", color: "neutral" });
			return;
		}
		const label = PREDIKAT.find(x => x.value === p)!.label;
		const lingkup = kelasFilter.value === "*" ? "" : ` di ${kelasFilter.value}`;
		if (!await swal.confirm(`Beri predikat "${label}"?`, `${kosong.length} siswa${lingkup} yang belum dinilai diberi predikat ${label}. Yang sudah dinilai tidak diubah.`, "Isi"))
			return;
		kosong.forEach((s) => {
			s.predikat = p;
			queueSave(s);
		});
	}
	const isiKosongItems = PREDIKAT.map(p => ({ label: p.label, onSelect: () => isiKosong(p.value) }));
</script>

<template>
	<ErPage id="ekskul" title="Nilai Ekstrakurikuler">
		<template #right>
			<span class="text-xs text-muted" aria-live="polite">
				<template v-if="saveState === 'saving'">Menyimpan…</template>
				<template v-else-if="saveState === 'saved'">Tersimpan otomatis</template>
				<template v-else-if="saveState === 'error'">Ada yang gagal tersimpan</template>
			</span>
		</template>

		<div v-if="!loading && !ekskulList.length" class="flex flex-col items-center gap-3 py-24 text-center text-muted">
			<UIcon name="lucide:trophy" class="size-12" />
			<p v-if="isAdmin">
				Belum ada ekskul di semester ini. Isi rombel ekskul di Dapodik, lalu Sinkron Dapodik.
			</p>
			<p v-else>
				Anda bukan pembina ekskul dan siswa kelas Anda belum terdaftar di ekskul. Pembina & anggota ekskul diatur di Dapodik.
			</p>
		</div>

		<div v-else class="space-y-4">
			<div class="flex flex-wrap items-center gap-2">
				<USelect
					v-if="ekskulList.length > 1"
					v-model="ekskulId"
					:items="ekskulList"
					icon="lucide:trophy"
					aria-label="Ekstrakurikuler"
					class="w-44"
				/>
				<UBadge
					v-else
					variant="subtle"
					color="neutral"
					size="lg"
					icon="lucide:trophy">
					{{ ekskulNama }}
				</UBadge>
				<USelect
					v-model="kelasFilter"
					:items="kelasItems"
					icon="lucide:layers"
					aria-label="Kelas"
					class="w-40" />
				<UInput
					v-model="cari"
					icon="lucide:search"
					placeholder="Cari siswa…"
					aria-label="Cari siswa"
					class="w-48" />
				<UBadge
					class="ms-auto tabular-nums"
					variant="subtle"
					:color="anggota.length && dinilai >= anggota.length ? 'success' : dinilai ? 'warning' : 'neutral'"
				>
					Dinilai {{ dinilai }}/{{ anggota.length }}
				</UBadge>
			</div>

			<UAlert
				v-if="sebagaiWali"
				color="neutral"
				variant="subtle"
				icon="lucide:users"
				title="Anda mengisi sebagai wali kelas"
				description="Hanya siswa kelas Anda yang tampil. Nilai dari pembina tidak ditimpa: kiriman wali kelas hanya mengisi yang masih kosong di laptop admin."
			/>
			<UAlert
				v-if="isGuruMode"
				color="info"
				variant="subtle"
				icon="lucide:laptop"
				title="Tersimpan di laptop ini"
				description="Setelah selesai, kirim ke admin lewat menu Sinkron ke Admin."
				:actions="[{ label: 'Buka Sinkron', to: '/sinkron', icon: 'lucide:arrow-right', variant: 'soft' }]"
			/>

			<div class="flex flex-wrap items-center gap-2">
				<span class="text-xs text-muted">Isi cepat:</span>
				<UDropdownMenu :items="isiKosongItems">
					<UButton
						size="sm"
						variant="soft"
						icon="lucide:wand-sparkles"
						trailing-icon="lucide:chevron-down"
						:disabled="!tampil.length">
						Yang belum dinilai{{ kelasFilter === "*" ? "" : ` di ${kelasFilter}` }}
					</UButton>
				</UDropdownMenu>
			</div>

			<div v-if="loading" class="space-y-2">
				<USkeleton v-for="i in 6" :key="i" class="h-11 w-full" />
			</div>

			<div v-else class="overflow-x-auto rounded-md border border-default">
				<table class="w-full text-sm">
					<thead class="bg-elevated/50 text-xs text-muted">
						<tr>
							<th class="w-10 p-2 text-left">
								No
							</th>
							<th class="sticky left-0 min-w-48 bg-elevated p-2 text-left">
								Nama
							</th>
							<th class="w-24 p-2 text-left">
								Kelas
							</th>
							<th class="p-2 text-left">
								Predikat
							</th>
							<th class="min-w-80 p-2 text-left">
								Keterangan di rapor
							</th>
						</tr>
					</thead>
					<tbody class="divide-y divide-default">
						<tr v-for="(s, i) in tampil" :key="s.pesertaDidikId" class="hover:bg-elevated/30">
							<td class="p-2 text-muted tabular-nums">
								{{ i + 1 }}
							</td>
							<td class="sticky left-0 bg-default p-2 font-medium">
								{{ s.nama }}
							</td>
							<td class="p-2 text-muted">
								{{ s.kelas ?? "–" }}
							</td>
							<td class="p-1">
								<div class="flex gap-1" role="radiogroup" :aria-label="`Predikat ${s.nama}`">
									<UTooltip v-for="p in PREDIKAT" :key="p.value" :text="p.label">
										<UButton
											size="xs"
											class="w-9 justify-center"
											:color="s.predikat === p.value ? p.warna : 'neutral'"
											:variant="s.predikat === p.value ? 'solid' : 'outline'"
											role="radio"
											:aria-checked="s.predikat === p.value"
											:aria-label="p.label"
											@click="setPredikat(s, p.value)"
										>
											{{ p.value }}
										</UButton>
									</UTooltip>
								</div>
							</td>
							<td class="p-1">
								<UInput
									:model-value="s.keterangan"
									:maxlength="KETERANGAN_MAX"
									variant="ghost"
									class="w-full"
									:placeholder="keteranganEkskul(s.predikat, ekskulNama) || 'Pilih predikat dulu'"
									:aria-label="`Keterangan ${s.nama}`"
									@update:model-value="setKeterangan(s, String($event))"
								/>
							</td>
						</tr>
					</tbody>
				</table>
				<p v-if="!tampil.length" class="py-8 text-center text-sm text-muted">
					{{ cari || kelasFilter !== "*" ? "Tidak ada siswa yang cocok." : "Ekskul ini belum punya anggota." }}
				</p>
			</div>

			<p class="text-xs text-muted">
				Keterangan kosong = kalimat otomatis dari predikat (tulisan abu-abu). Tulis sendiri kalau ingin lebih spesifik. Klik predikat yang sama lagi untuk membatalkan.
			</p>
		</div>
	</ErPage>
</template>
