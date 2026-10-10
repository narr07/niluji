<script setup lang="ts">
	// Gabungan Input Kelengkapan e-Rapor (Kehadiran, Catatan Wali Kelas, Kenaikan Kelas) jadi satu tabel
	// per kelas, tersimpan otomatis. Wali kelas: kelas yang ia walikan. Admin: semua kelas.

	const db = useDb();
	const toast = useToast();
	const swal = useSwal();
	const { user, session, waliRombel } = useAuth();
	const { isGuruMode, load: loadMode } = useAppMode();
	const isAdmin = computed(() => user.value?.level === "admin");
	const genap = computed(() => semesterKe(session.value?.semesterId) === 2);

	// ===== Pilihan kelas =====
	interface Kelas { value: string, label: string, tingkat: number }
	const kelasList = ref<Kelas[]>([]);
	const rombelId = ref<string>();
	const kelas = computed(() => kelasList.value.find(k => k.value === rombelId.value));

	async function loadKelas() {
		const sem = session.value?.semesterId;
		const ids = isAdmin.value ? null : waliRombel.value.map(r => r.rombongan_belajar_id);
		kelasList.value = !sem || (ids && !ids.length)
			? []
			: await db.query<Kelas>(
				`SELECT rombongan_belajar_id AS value, nama AS label, CAST(tingkat AS INTEGER) AS tingkat FROM rombel
				WHERE semester_id = ? AND jenis_rombel = '1' ${ids ? `AND rombongan_belajar_id IN (${ids.map(() => "?").join(",")})` : ""}
				ORDER BY CAST(tingkat AS INTEGER), nama`,
				[sem, ...(ids ?? [])]
			);
		if (!kelasList.value.some(k => k.value === rombelId.value))
			rombelId.value = kelasList.value[0]?.value;
	}

	// ===== Data siswa =====
	const siswa = ref<IsianWalas[]>([]);
	const loading = ref(true);
	let permintaan = 0;

	async function load() {
		const nomor = ++permintaan;
		const sem = session.value?.semesterId;
		if (!rombelId.value || !sem) {
			siswa.value = [];
			loading.value = false;
			return;
		}
		loading.value = true;
		try {
			const rows = await db.query<IsianWalas>(
				`SELECT s.peserta_didik_id AS pesertaDidikId, s.nama, s.nisn, s.jenis_kelamin AS jenisKelamin,
					w.sakit, w.izin, w.alpa, COALESCE(w.catatan, '') AS catatan, w.naik
				FROM anggota_rombel a JOIN siswa_rapor s USING (peserta_didik_id)
				LEFT JOIN rapor_siswa w ON w.peserta_didik_id = s.peserta_didik_id AND w.semester_id = ?
				WHERE a.rombongan_belajar_id = ? ORDER BY s.nama COLLATE NOCASE`,
				[sem, rombelId.value]
			);
			if (nomor === permintaan)
				siswa.value = rows;
		}
		catch (e) {
			toast.add({ title: "Gagal memuat siswa", description: pesanError(e), color: "error" });
		}
		finally {
			if (nomor === permintaan)
				loading.value = false;
		}
	}

	async function init() {
		await loadMode();
		await loadKelas();
		await load();
	}
	watch(() => session.value?.semesterId, init, { immediate: true });
	watch(rombelId, async () => {
		await flush();
		await load();
	});

	// ===== Simpan otomatis (per siswa, ditunda sebentar seperti Nilai Rapor) =====
	const pending = new Map<string, { timer: ReturnType<typeof setTimeout>, run: () => Promise<void> }>();
	const saveState = ref<"idle" | "saving" | "saved" | "error">("idle");

	function simpanSql(s: IsianWalas) {
		return {
			sql: `INSERT INTO rapor_siswa (semester_id, peserta_didik_id, rombongan_belajar_id, sakit, izin, alpa, catatan, naik, updated_at)
				VALUES (?, ?, ?, ?, ?, ?, ?, ?, datetime('now','localtime'))
				ON CONFLICT (semester_id, peserta_didik_id) DO UPDATE SET rombongan_belajar_id = excluded.rombongan_belajar_id,
					sakit = excluded.sakit, izin = excluded.izin, alpa = excluded.alpa, catatan = excluded.catatan, naik = excluded.naik,
					updated_at = excluded.updated_at`,
			params: [session.value?.semesterId, s.pesertaDidikId, rombelId.value, s.sakit, s.izin, s.alpa, s.catatan.trim() || null, s.naik]
		};
	}

	function queueSave(s: IsianWalas) {
		const key = s.pesertaDidikId;
		const st = simpanSql(s); // dibekukan sekarang, supaya tetap ke kelas yang benar walau kelas diganti
		clearTimeout(pending.get(key)?.timer);
		saveState.value = "saving";
		const run = async () => {
			pending.delete(key);
			try {
				await db.execute(st.sql, st.params);
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

	// Simpan yang masih tertunda sekarang juga (ganti kelas / tinggalkan halaman).
	async function flush() {
		const semua = [...pending.values()];
		semua.forEach(p => clearTimeout(p.timer));
		await Promise.all(semua.map(p => p.run()));
	}
	onBeforeRouteLeave(flush);

	type Hadir = "sakit" | "izin" | "alpa";
	const KOLOM_HADIR: { key: Hadir, label: string, judul: string }[] = [
		{ key: "sakit", label: "S", judul: "Sakit" },
		{ key: "izin", label: "I", judul: "Izin" },
		{ key: "alpa", label: "A", judul: "Tanpa keterangan" }
	];

	function setHadir(s: IsianWalas, k: Hadir, el: HTMLInputElement) {
		const raw = el.value.trim();
		const n = raw === "" ? null : Math.round(Number(raw));
		if (n !== null && (!Number.isFinite(n) || n < 0 || n > HARI_MAX)) {
			toast.add({ title: `Jumlah hari harus 0–${HARI_MAX}`, color: "warning" });
			el.value = s[k]?.toString() ?? "";
			return;
		}
		if (s[k] === n)
			return;
		s[k] = n;
		queueSave(s);
	}

	function setCatatan(s: IsianWalas, teks: string) {
		s.catatan = teks.slice(0, CATATAN_MAX);
		queueSave(s);
	}

	function setNaik(s: IsianWalas, v: number | null) {
		s.naik = v;
		queueSave(s);
	}

	// Enter = pindah ke baris berikutnya di kolom yang sama, seperti Excel.
	function turun(e: KeyboardEvent, kolom: string, i: number) {
		e.preventDefault();
		(document.querySelector(`[data-sel="${kolom}:${i + 1}"]`) as HTMLInputElement | null)?.focus();
	}

	const templateItems = (s: IsianWalas) => TEMPLATE_CATATAN.map(t => ({
		label: t.label,
		description: isiTemplate(t.teks, s.nama),
		onSelect: () => setCatatan(s, isiTemplate(t.teks, s.nama))
	}));

	// ===== Isi cepat satu kelas =====
	function hadirNol() {
		let n = 0;
		for (const s of siswa.value) {
			let ubah = false;
			for (const { key } of KOLOM_HADIR) {
				if (s[key] === null) {
					s[key] = 0;
					ubah = true;
				}
			}
			if (ubah) {
				n++;
				queueSave(s);
			}
		}
		toast.add({ title: n ? `Kehadiran kosong ${n} siswa diisi 0` : "Tidak ada kehadiran yang kosong", color: n ? "success" : "neutral" });
	}

	async function catatanMassal(teks: string, label: string) {
		const kosong = siswa.value.filter(s => !s.catatan.trim());
		if (!kosong.length) {
			toast.add({ title: "Semua siswa sudah punya catatan", color: "neutral" });
			return;
		}
		if (!await swal.confirm(`Isi catatan "${label}"?`, `${kosong.length} siswa yang catatannya masih kosong diisi kalimat ini (nama menyesuaikan). Catatan yang sudah ada tidak diubah.`, "Isi"))
			return;
		kosong.forEach(s => setCatatan(s, isiTemplate(teks, s.nama)));
	}
	const catatanMassalItems = computed(() => TEMPLATE_CATATAN.map(t => ({
		label: t.label,
		description: t.teks.replaceAll("{nama}", "…"),
		onSelect: () => catatanMassal(t.teks, t.label)
	})));

	function naikSemua() {
		const belum = siswa.value.filter(s => s.naik === null);
		belum.forEach(s => setNaik(s, 1));
		toast.add({ title: belum.length ? `${belum.length} siswa ditandai ${kelas.value && kelas.value.tingkat >= 6 ? "lulus" : "naik kelas"}` : "Semua siswa sudah diputuskan", color: belum.length ? "success" : "neutral" });
	}

	// ===== Ringkasan & cari =====
	const cari = ref("");
	const tampil = computed(() => {
		const q = cari.value.trim().toLowerCase();
		return q ? siswa.value.filter(s => s.nama.toLowerCase().includes(q) || s.nisn?.includes(q)) : siswa.value;
	});
	const ringkasan = computed(() => {
		const n = siswa.value.length;
		return [
			{ label: "Kehadiran", isi: siswa.value.filter(s => KOLOM_HADIR.every(k => s[k.key] !== null)).length, n },
			{ label: "Catatan", isi: siswa.value.filter(s => s.catatan.trim()).length, n },
			...(genap.value ? [{ label: "Kenaikan", isi: siswa.value.filter(s => s.naik !== null).length, n }] : [])
		];
	});
	const totalHadir = (s: IsianWalas) => (s.sakit ?? 0) + (s.izin ?? 0) + (s.alpa ?? 0);
</script>

<template>
	<ErPage id="walas" :title="isAdmin ? 'Kelengkapan Rapor' : 'Kelas Saya'">
		<template #right>
			<span class="text-xs text-muted" aria-live="polite">
				<template v-if="saveState === 'saving'">Menyimpan…</template>
				<template v-else-if="saveState === 'saved'">Tersimpan otomatis</template>
				<template v-else-if="saveState === 'error'">Ada yang gagal tersimpan</template>
			</span>
		</template>

		<div v-if="!loading && !kelasList.length" class="flex flex-col items-center gap-3 py-24 text-center text-muted">
			<UIcon name="lucide:users" class="size-12" />
			<p v-if="isAdmin">
				Belum ada data kelas di semester ini. Jalankan Sinkron Dapodik dulu.
			</p>
			<p v-else>
				Anda bukan wali kelas di semester ini.
			</p>
		</div>

		<div v-else class="space-y-4">
			<!-- Filter & ringkasan satu baris -->
			<div class="flex flex-wrap items-center gap-2">
				<USelect
					v-if="kelasList.length > 1"
					v-model="rombelId"
					:items="kelasList"
					icon="lucide:layers"
					aria-label="Kelas"
					class="w-36"
				/>
				<UBadge
					v-else-if="kelas"
					variant="subtle"
					color="neutral"
					size="lg"
					icon="lucide:layers">
					{{ kelas.label }}
				</UBadge>
				<UInput
					v-model="cari"
					icon="lucide:search"
					placeholder="Cari siswa…"
					aria-label="Cari siswa"
					class="w-48" />
				<div class="ms-auto flex flex-wrap items-center gap-2">
					<UBadge
						v-for="r in ringkasan"
						:key="r.label"
						variant="subtle"
						class="tabular-nums"
						:color="r.n && r.isi >= r.n ? 'success' : r.isi ? 'warning' : 'neutral'"
					>
						{{ r.label }} {{ r.isi }}/{{ r.n }}
					</UBadge>
				</div>
			</div>

			<UAlert
				v-if="isGuruMode"
				color="info"
				variant="subtle"
				icon="lucide:laptop"
				title="Tersimpan di laptop ini"
				description="Setelah selesai, kirim ke admin lewat menu Sinkron ke Admin → Data Wali Kelas."
				:actions="[{ label: 'Buka Sinkron', to: '/sinkron', icon: 'lucide:arrow-right', variant: 'soft' }]"
			/>

			<!-- Isi cepat -->
			<div class="flex flex-wrap items-center gap-2">
				<span class="text-xs text-muted">Isi cepat:</span>
				<UButton
					size="sm"
					variant="soft"
					icon="lucide:calendar-check"
					:disabled="!siswa.length"
					@click="hadirNol">
					Kehadiran kosong = 0
				</UButton>
				<UDropdownMenu :items="catatanMassalItems" :ui="{ content: 'w-80', itemDescription: 'whitespace-normal' }">
					<UButton
						size="sm"
						variant="soft"
						icon="lucide:message-square-text"
						trailing-icon="lucide:chevron-down"
						:disabled="!siswa.length">
						Catatan untuk yang kosong
					</UButton>
				</UDropdownMenu>
				<UButton
					v-if="genap"
					size="sm"
					variant="soft"
					icon="lucide:arrow-up-circle"
					:disabled="!siswa.length"
					@click="naikSemua">
					{{ kelas && kelas.tingkat >= 6 ? "Luluskan" : "Naikkan" }} yang belum diputuskan
				</UButton>
			</div>

			<div v-if="loading" class="space-y-2">
				<USkeleton v-for="i in 6" :key="i" class="h-12 w-full" />
			</div>

			<div v-else class="overflow-x-auto rounded-md border border-default">
				<table class="w-full text-sm">
					<thead class="bg-elevated/50 text-xs text-muted">
						<tr>
							<th rowspan="2" class="w-10 p-2 text-left">
								No
							</th>
							<th rowspan="2" class="sticky left-0 min-w-48 bg-elevated p-2 text-left">
								Nama
							</th>
							<th colspan="3" class="border-b border-default p-1 text-center">
								Ketidakhadiran (hari)
							</th>
							<th rowspan="2" class="min-w-80 p-2 text-left">
								Catatan Wali Kelas
							</th>
							<th v-if="genap" rowspan="2" class="w-48 p-2 text-left">
								{{ kelas && kelas.tingkat >= 6 ? "Kelulusan" : "Kenaikan Kelas" }}
							</th>
						</tr>
						<tr>
							<th v-for="k in KOLOM_HADIR" :key="k.key" class="w-14 p-1 text-center">
								<UTooltip :text="k.judul">
									<span>{{ k.label }}</span>
								</UTooltip>
							</th>
						</tr>
					</thead>
					<tbody class="divide-y divide-default">
						<tr v-for="(s, i) in tampil" :key="s.pesertaDidikId" class="align-top hover:bg-elevated/30">
							<td class="p-2 pt-3 text-muted tabular-nums">
								{{ i + 1 }}
							</td>
							<td class="sticky left-0 bg-default p-2">
								<p class="font-medium">
									{{ s.nama }}
								</p>
								<p class="text-xs text-muted">
									{{ s.nisn ?? "–" }}
									<template v-if="totalHadir(s)">
										· {{ totalHadir(s) }} hari absen
									</template>
								</p>
							</td>
							<td v-for="k in KOLOM_HADIR" :key="k.key" class="p-1 pt-2 text-center">
								<input
									:data-sel="`${k.key}:${i}`"
									type="number"
									min="0"
									:max="HARI_MAX"
									inputmode="numeric"
									:value="s[k.key] ?? ''"
									:aria-label="`${k.judul} ${s.nama}`"
									placeholder="–"
									class="w-12 rounded border border-default bg-default px-1 py-1 text-center tabular-nums placeholder:text-dimmed focus:border-primary focus:outline-none"
									@change="setHadir(s, k.key, $event.target as HTMLInputElement)"
									@keydown.enter="turun($event, k.key, i)"
								>
							</td>
							<td class="p-1">
								<div class="flex items-start gap-1">
									<UTextarea
										:model-value="s.catatan"
										:rows="1"
										autoresize
										:maxrows="5"
										:maxlength="CATATAN_MAX"
										placeholder="Tulis catatan atau pilih template →"
										:aria-label="`Catatan ${s.nama}`"
										class="flex-1"
										@update:model-value="setCatatan(s, String($event))"
									/>
									<UDropdownMenu :items="templateItems(s)" :ui="{ content: 'w-80', itemDescription: 'whitespace-normal' }">
										<UButton
											icon="lucide:sparkles"
											variant="ghost"
											size="sm"
											:aria-label="`Template catatan ${s.nama}`" />
									</UDropdownMenu>
								</div>
								<p v-if="s.catatan.length > CATATAN_MAX - 50" class="px-2 text-xs text-muted tabular-nums">
									{{ s.catatan.length }}/{{ CATATAN_MAX }}
								</p>
							</td>
							<td v-if="genap && kelas" class="p-1">
								<USelect
									:model-value="s.naik ?? undefined"
									:items="pilihanNaik(kelas.tingkat)"
									placeholder="Belum diputuskan"
									:color="s.naik === 0 ? 'error' : undefined"
									:aria-label="`Kenaikan ${s.nama}`"
									class="w-full"
									@update:model-value="setNaik(s, $event as number)"
								/>
							</td>
						</tr>
					</tbody>
				</table>
				<p v-if="!tampil.length" class="py-8 text-center text-sm text-muted">
					{{ cari ? "Tidak ada siswa yang cocok." : "Kelas ini belum punya anggota." }}
				</p>
			</div>

			<p class="text-xs text-muted">
				Kosong (–) berarti belum diisi; isi 0 kalau siswa selalu hadir. Enter di kolom kehadiran pindah ke siswa berikutnya.
				<template v-if="!genap">
					Kenaikan kelas diisi di semester genap.
				</template>
			</p>
		</div>
	</ErPage>
</template>
