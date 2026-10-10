<script setup lang="ts">
	import { invoke } from "@tauri-apps/api/core";

	// Padanan e-Rapor: Input Nilai Kokurikuler + Deskripsi Kokurikuler (koordinator), diringkas:
	// wali kelas = koordinator kelasnya. Capaian per dimensi sekali klik (B → C → M → kosong),
	// deskripsi rapor otomatis dari capaian (boleh ditulis sendiri). Admin: semua kelas.

	interface Siswa { id: string, nama: string, nisn: string | null, deskripsi: string }

	const db = useDb();
	const toast = useToast();
	const { user, session, waliRombel } = useAuth();
	const swal = useSwal();
	const { isGuruMode, load: loadMode } = useAppMode();
	const isAdmin = computed(() => user.value?.level === "admin");

	// ===== Kelas =====
	const kelasList = ref<{ value: string, label: string, tingkat: number }[]>([]);
	const rombelId = ref<string>();
	const kelas = computed(() => kelasList.value.find(k => k.value === rombelId.value));

	async function loadKelas() {
		const sem = session.value?.semesterId;
		const ids = isAdmin.value ? null : waliRombel.value.map(r => r.rombongan_belajar_id);
		kelasList.value = !sem || (ids && !ids.length)
			? []
			: await db.query(
				`SELECT rombongan_belajar_id AS value, nama AS label, CAST(tingkat AS INTEGER) AS tingkat FROM rombel
				WHERE semester_id = ? AND jenis_rombel = '1' ${ids ? `AND rombongan_belajar_id IN (${ids.map(() => "?").join(",")})` : ""}
				ORDER BY CAST(tingkat AS INTEGER), nama`,
				[sem, ...(ids ?? [])]
			);
		if (!kelasList.value.some(k => k.value === rombelId.value))
			rombelId.value = kelasList.value[0]?.value;
	}

	// ===== Data =====
	const kegiatan = ref<KegiatanKoku[]>([]);
	const kegiatanId = ref<number>();
	const keg = computed(() => kegiatan.value.find(k => k.id === kegiatanId.value) ?? kegiatan.value[0]);
	const siswa = ref<Siswa[]>([]);
	const nilai = ref<Record<string, Record<string, number>>>({}); // `${kegiatan}:${siswa}` → {dimensi: 1..3}
	const loading = ref(true);
	let permintaan = 0;

	async function load() {
		const nomor = ++permintaan;
		const sem = session.value?.semesterId;
		if (!rombelId.value || !sem || !kelas.value) {
			siswa.value = [];
			kegiatan.value = [];
			loading.value = false;
			return;
		}
		loading.value = true;
		try {
			const [keg, sis] = await Promise.all([
				db.query<Omit<KegiatanKoku, "tingkat" | "dimensi" | "subdimensi"> & { tingkat: string, dimensi: string, subdimensi: string }>(
					`SELECT id, nama, tema, tujuan, tingkat, dimensi, subdimensi FROM kokurikuler_kegiatan k
					WHERE semester_id = ? AND EXISTS (SELECT 1 FROM json_each(k.tingkat) WHERE value = ?) ORDER BY urutan, id`,
					[sem, kelas.value.tingkat]
				),
				db.query<Siswa>(
					`SELECT s.peserta_didik_id AS id, s.nama, s.nisn, COALESCE(w.kokurikuler, '') AS deskripsi
					FROM anggota_rombel a JOIN siswa_rapor s USING (peserta_didik_id)
					LEFT JOIN rapor_siswa w ON w.peserta_didik_id = s.peserta_didik_id AND w.semester_id = ?
					WHERE a.rombongan_belajar_id = ? ORDER BY s.nama COLLATE NOCASE`,
					[sem, rombelId.value]
				)
			]);
			const ids = keg.map(k => k.id);
			const n = ids.length && sis.length
				? await db.query<{ kegiatan_id: number, peserta_didik_id: string, capaian: string }>(
					`SELECT kegiatan_id, peserta_didik_id, capaian FROM nilai_kokurikuler WHERE kegiatan_id IN (${ids.map(() => "?").join(",")})
					AND peserta_didik_id IN (SELECT peserta_didik_id FROM anggota_rombel WHERE rombongan_belajar_id = ?)`,
					[...ids, rombelId.value]
				)
				: [];
			if (nomor !== permintaan)
				return;
			kegiatan.value = keg.map(k => ({ ...k, tingkat: JSON.parse(k.tingkat), dimensi: JSON.parse(k.dimensi), subdimensi: JSON.parse(k.subdimensi || "{}") }));
			siswa.value = sis;
			nilai.value = Object.fromEntries(n.map(x => [`${x.kegiatan_id}:${x.peserta_didik_id}`, JSON.parse(x.capaian)]));
			if (!kegiatan.value.some(k => k.id === kegiatanId.value))
				kegiatanId.value = kegiatan.value[0]?.id;
		}
		catch (e) {
			toast.add({ title: "Gagal memuat kokurikuler", description: pesanError(e), color: "error" });
		}
		finally {
			if (nomor === permintaan)
				loading.value = false;
		}
	}

	async function init() {
		try {
			await loadMode();
			await loadKelas();
			await load();
		}
		catch (e) {
			toast.add({ title: "Gagal memuat kokurikuler", description: pesanError(e), color: "error" });
		}
		finally {
			// Apa pun yang terjadi, jangan biarkan halaman tertahan di status memuat.
			if (!permintaan || !kelasList.value.length)
				loading.value = false;
		}
	}
	watch(() => session.value?.semesterId, init, { immediate: true });
	watch(rombelId, async () => {
		await flush();
		await load();
	});

	// ===== Simpan otomatis =====
	const pending = new Map<string, { timer: ReturnType<typeof setTimeout>, run: () => Promise<void> }>();
	const saveState = ref<"idle" | "saving" | "saved" | "error">("idle");

	function antre(key: string, sql: string, params: unknown[]) {
		clearTimeout(pending.get(key)?.timer);
		saveState.value = "saving";
		const run = async () => {
			pending.delete(key);
			try {
				await db.execute(sql, params);
				if (!pending.size)
					saveState.value = "saved";
			}
			catch (e) {
				saveState.value = "error";
				toast.add({ title: "Gagal menyimpan", description: pesanError(e), color: "error" });
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

	const capaianOf = (s: Siswa, d: string) => (keg.value ? nilai.value[`${keg.value.id}:${s.id}`]?.[d] : undefined);
	const NEXT: Record<number, number | undefined> = { 0: 1, 1: 2, 2: 3, 3: undefined };

	function setCapaian(s: Siswa, d: string, v: number | undefined) {
		const k = keg.value;
		if (!k)
			return;
		const key = `${k.id}:${s.id}`;
		const cur = { ...(nilai.value[key] ?? {}) };
		if (v)
			cur[d] = v;
		else
			Reflect.deleteProperty(cur, d);
		nilai.value[key] = cur;
		antre(`n:${key}`, `INSERT INTO nilai_kokurikuler (kegiatan_id, peserta_didik_id, capaian, updated_at) VALUES (?, ?, ?, datetime('now','localtime'))
			ON CONFLICT (kegiatan_id, peserta_didik_id) DO UPDATE SET capaian = excluded.capaian, updated_at = excluded.updated_at`, [k.id, s.id, JSON.stringify(cur)]);
	}
	const klik = (s: Siswa, d: string) => setCapaian(s, d, NEXT[capaianOf(s, d) ?? 0]);

	// Klik judul kolom: semua siswa ikut nilai berikutnya dari siswa pertama.
	function klikKolom(d: string) {
		const pertama = siswa.value[0];
		if (!pertama)
			return;
		const v = NEXT[capaianOf(pertama, d) ?? 0];
		siswa.value.forEach(s => setCapaian(s, d, v));
	}

	function setDeskripsi(s: Siswa, teks: string) {
		s.deskripsi = teks.slice(0, DESKRIPSI_KOKU_MAX);
		antre(`d:${s.id}`, `INSERT INTO rapor_siswa (semester_id, peserta_didik_id, rombongan_belajar_id, kokurikuler, updated_at)
			VALUES (?, ?, ?, ?, datetime('now','localtime'))
			ON CONFLICT (semester_id, peserta_didik_id) DO UPDATE SET kokurikuler = excluded.kokurikuler, updated_at = excluded.updated_at`,
			[session.value?.semesterId, s.id, rombelId.value, s.deskripsi.trim() || null]);
	}

	// Deskripsi otomatis dari semua kegiatan kelas ini (sama dengan yang tercetak kalau tidak ditulis sendiri).
	const otomatis = (s: Siswa) => deskripsiKokurikuler(s.nama, kegiatan.value.map(k => ({ kegiatan: k.nama, capaian: nilai.value[`${k.id}:${s.id}`] ?? {} })));

	// Kolom penilaian: tiap subdimensi, atau dimensinya langsung kalau subdimensi tidak diisi admin.
	const kolom = computed(() => (keg.value ? kolomKegiatan(keg.value) : []));
	const dinilai = computed(() => siswa.value.filter(s => kolom.value.length && kolom.value.every(c => capaianOf(s, c.kunci))).length);

	// ===== Excel (format nilai 1/2/3 seperti e-Rapor) =====
	const fileInput = ref<HTMLInputElement>();
	const busy = ref(false);
	const kegiatanExcel = () => kegiatan.value.map(k => ({ id: k.id, nama: k.nama, kolom: kolomKegiatan(k) }));
	async function exportExcel() {
		busy.value = true;
		try {
			const bytes = buildKokuWorkbook(kegiatanExcel(), siswa.value, nilai.value);
			const path = await invoke<string>("save_to_downloads", { fileName: `Kokurikuler ${kelas.value?.label ?? ""}.xlsx`, bytes: Array.from(bytes) });
			toast.add({ title: "File tersimpan di Downloads", description: path, color: "success", actions: [{ label: "Buka Folder", onClick: () => { invoke("reveal_file", { path }); } }] });
		}
		catch (e) {
			toast.add({ title: "Gagal membuat file", description: pesanError(e), color: "error" });
		}
		finally {
			busy.value = false;
		}
	}
	async function importExcel(e: Event) {
		const el = e.target as HTMLInputElement;
		const file = el.files?.[0];
		el.value = "";
		if (!file)
			return;
		busy.value = true;
		try {
			await flush();
			const { nilai: hasil, deskripsi, ditolak } = readKokuWorkbook(await file.arrayBuffer(), kegiatanExcel(), siswa.value);
			await db.batch([
				...hasil.map(h => ({
					sql: `INSERT INTO nilai_kokurikuler (kegiatan_id, peserta_didik_id, capaian, updated_at) VALUES (?, ?, ?, datetime('now','localtime'))
						ON CONFLICT (kegiatan_id, peserta_didik_id) DO UPDATE SET capaian = excluded.capaian, updated_at = excluded.updated_at`,
					params: [h.kegiatanId, h.siswaId, JSON.stringify(h.capaian)]
				})),
				...[...deskripsi].map(([id, d]) => ({
					sql: `INSERT INTO rapor_siswa (semester_id, peserta_didik_id, rombongan_belajar_id, kokurikuler) VALUES (?, ?, ?, ?)
						ON CONFLICT (semester_id, peserta_didik_id) DO UPDATE SET kokurikuler = excluded.kokurikuler`,
					params: [session.value?.semesterId, id, rombelId.value, d.slice(0, DESKRIPSI_KOKU_MAX)]
				}))
			]);
			await load();
			await swal.fire({
				type: ditolak.length ? "warning" : "success",
				title: "Import selesai",
				text: `${hasil.length} baris nilai, ${deskripsi.size} deskripsi tersimpan.${ditolak.length ? `\n\n${ditolak.length} isian ditolak:\n${ditolak.slice(0, 8).join("\n")}` : ""}`
			});
		}
		catch (err) {
			swal.error("Gagal import", pesanError(err));
		}
		finally {
			busy.value = false;
		}
	}
	const tabs = computed(() => kegiatan.value.map(k => ({ label: k.nama, value: k.id })));
</script>

<template>
	<ErPage id="walas-koku" :title="isAdmin ? 'Nilai Kokurikuler' : 'Kokurikuler'">
		<template #right>
			<span class="text-xs text-muted" aria-live="polite">
				<template v-if="saveState === 'saving'">Menyimpan…</template>
				<template v-else-if="saveState === 'saved'">Tersimpan otomatis</template>
				<template v-else-if="saveState === 'error'">Ada yang gagal tersimpan</template>
			</span>
		</template>

		<div v-if="!loading && !kelasList.length" class="flex flex-col items-center gap-3 py-24 text-center text-muted">
			<UIcon name="lucide:notebook-pen" class="size-12" />
			<p>{{ isAdmin ? "Belum ada data kelas di semester ini." : "Anda bukan wali kelas di semester ini." }}</p>
		</div>

		<div v-else class="space-y-4">
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
				<UBadge
					v-if="keg"
					class="ms-auto tabular-nums"
					variant="subtle"
					:color="siswa.length && dinilai >= siswa.length ? 'success' : dinilai ? 'warning' : 'neutral'"
				>
					Lengkap {{ dinilai }}/{{ siswa.length }}
				</UBadge>
			</div>

			<UAlert
				v-if="isGuruMode"
				color="info"
				variant="subtle"
				icon="lucide:laptop"
				title="Tersimpan di laptop ini"
				description="Ikut terkirim bersama Data Wali Kelas di menu Sinkron ke Admin."
			/>

			<div v-if="loading" class="space-y-2">
				<USkeleton v-for="i in 6" :key="i" class="h-11 w-full" />
			</div>

			<div v-else-if="!kegiatan.length" class="flex flex-col items-center gap-3 py-16 text-center text-muted">
				<UIcon name="lucide:notebook" class="size-12" />
				<p>Belum ada kegiatan kokurikuler untuk kelas {{ kelas?.tingkat }} di semester ini.</p>
				<UButton
					v-if="isAdmin"
					to="/e-rapor/kokurikuler"
					icon="lucide:plus"
					variant="soft">
					Tambah Kegiatan
				</UButton>
				<p v-else class="text-sm">
					Minta admin menambahkan kegiatan di menu Kegiatan Kokurikuler.
				</p>
			</div>

			<template v-else-if="keg">
				<div class="flex flex-wrap items-center gap-2">
					<UTabs
						v-if="kegiatan.length > 1"
						v-model="kegiatanId"
						:items="tabs"
						:content="false" />
					<div class="ms-auto flex gap-2">
						<UButton
							size="sm"
							variant="subtle"
							icon="lucide:file-down"
							:loading="busy"
							@click="exportExcel">
							Export Excel
						</UButton>
						<UButton
							size="sm"
							variant="soft"
							icon="lucide:upload"
							:disabled="busy"
							@click="fileInput?.click()">
							Import Excel
						</UButton>
						<input
							ref="fileInput"
							type="file"
							accept=".xlsx,.xls"
							class="hidden"
							aria-label="Pilih file Excel kokurikuler"
							@change="importExcel">
					</div>
				</div>
				<p class="text-sm text-muted">
					<template v-if="keg.tema">
						Tema <b class="text-default">{{ keg.tema }}</b> ·
					</template>
					<template v-if="keg.tujuan">
						Tujuan akhir: {{ keg.tujuan }}
					</template>
				</p>

				<div class="overflow-x-auto rounded-md border border-default">
					<table class="w-full text-sm">
						<thead class="bg-elevated/50 text-xs text-muted">
							<tr>
								<th class="w-10 p-2 text-left">
									No
								</th>
								<th class="sticky left-0 min-w-44 bg-elevated p-2 text-left">
									Nama
								</th>
								<th v-for="c in kolom" :key="c.kunci" class="w-24 p-1 text-center">
									<UTooltip :text="`${c.judul} — klik untuk mengisi satu kelas`">
										<button type="button" class="line-clamp-2 max-w-28 cursor-pointer rounded px-1.5 py-0.5 font-semibold hover:bg-elevated" @click="klikKolom(c.kunci)">
											{{ c.label }}
										</button>
									</UTooltip>
								</th>
								<th class="min-w-96 p-2 text-left">
									Deskripsi di rapor
								</th>
							</tr>
						</thead>
						<tbody class="divide-y divide-default">
							<tr v-for="(s, i) in siswa" :key="s.id" class="align-top hover:bg-elevated/30">
								<td class="p-2 pt-3 text-muted tabular-nums">
									{{ i + 1 }}
								</td>
								<td class="sticky left-0 bg-default p-2 pt-3 font-medium">
									{{ s.nama }}
								</td>
								<td v-for="col in kolom" :key="col.kunci" class="p-1 pt-2 text-center">
									<UButton
										size="xs"
										class="w-20 justify-center"
										:color="CAPAIAN.find(c => c.value === capaianOf(s, col.kunci))?.warna ?? 'neutral'"
										:variant="capaianOf(s, col.kunci) ? 'soft' : 'outline'"
										:aria-label="`${col.judul} ${s.nama}: ${CAPAIAN.find(c => c.value === capaianOf(s, col.kunci))?.label ?? 'belum dinilai'}`"
										@click="klik(s, col.kunci)"
									>
										{{ CAPAIAN.find(c => c.value === capaianOf(s, col.kunci))?.label ?? "–" }}
									</UButton>
								</td>
								<td class="p-1">
									<UTextarea
										:model-value="s.deskripsi"
										:rows="1"
										autoresize
										:maxrows="4"
										:maxlength="DESKRIPSI_KOKU_MAX"
										variant="ghost"
										:placeholder="otomatis(s) || 'Otomatis setelah capaian diisi'"
										:aria-label="`Deskripsi kokurikuler ${s.nama}`"
										class="w-full"
										@update:model-value="setDeskripsi(s, String($event))"
									/>
								</td>
							</tr>
						</tbody>
					</table>
				</div>
				<p class="text-xs text-muted">
					Klik kotak capaian: Berkembang → Cakap → Mahir → kosong. Klik judul kolom untuk mengisi satu kelas sekaligus. Export/Import Excel memakai angka 1 = Berkembang, 2 = Cakap, 3 = Mahir.
					Deskripsi kosong = kalimat otomatis (tulisan abu-abu) dari semua kegiatan; tulis sendiri kalau ingin mengubahnya.
				</p>
			</template>
		</div>
	</ErPage>
</template>
