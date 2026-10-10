<script setup lang="ts">
	import type { Mengajar } from "~/composables/useMengajar";
	import type { SheetMapel, SheetSiswa } from "~/utils/nilaiExcel";
	import { invoke } from "@tauri-apps/api/core";

	// Gabungan menu e-Rapor: Input Nilai Rapor + Import Nilai + Nilai Tersimpan + Deskripsi Tersimpan.
	// Kelas langsung terpilih, mapel jadi tab, nilai & centang TP satu baris, tersimpan otomatis.

	const db = useDb();
	const toast = useToast();
	const swal = useSwal();
	const { list, loading, inputDibuka, semesterKe: smt } = useMengajar();

	const rombelItems = computed(() => {
		const seen = new Map<string, string>();
		list.value.forEach(m => seen.set(m.rombongan_belajar_id, m.rombel));
		return [...seen].map(([value, label]) => ({ value, label }));
	});
	const rombelId = ref<string>();
	watch(rombelItems, (items) => {
		if (!items.some(i => i.value === rombelId.value))
			rombelId.value = items[0]?.value;
	}, { immediate: true });

	const mapelDiRombel = computed(() => list.value.filter(m => m.rombongan_belajar_id === rombelId.value));
	const prId = ref<number>();
	watch(mapelDiRombel, (items) => {
		if (!items.some(i => i.id === prId.value))
			prId.value = items[0]?.id;
	}, { immediate: true });

	const progress = ref<Record<number, number>>({});
	// Terkunci = sudah dikirim ke admin; hanya bisa diubah lagi kalau admin membuka kunci.
	const bisaEdit = computed(() => inputDibuka.value && !current.value?.terkunci);
	const jumlahSiswa = computed(() => data.value?.siswa.length ?? 0);
	const tabs = computed(() => mapelDiRombel.value.map(m => ({
		label: m.singkat,
		value: m.id,
		badge: m.terkunci ? "terkirim" : jumlahSiswa.value ? `${progress.value[m.id] ?? 0}/${jumlahSiswa.value}` : undefined,
		icon: m.terkunci ? "lucide:lock" : undefined
	})));

	// ===== Data =====
	async function loadMapel(m: Mengajar): Promise<SheetMapel> {
		const [tps, siswa, nilai] = await Promise.all([
			db.query<{ id: number, deskripsi: string }>(
				"SELECT id, deskripsi FROM tujuan_pembelajaran WHERE kode_mapel = ? AND tingkat = ? AND semester = ? ORDER BY urutan, id",
				[m.kode_mapel, m.tingkat, smt.value]
			),
			db.query<{ peserta_didik_id: string, nisn: string | null, nama: string }>(
				`SELECT p.peserta_didik_id, p.nisn, p.nama FROM anggota_rombel a JOIN peserta_didik p USING (peserta_didik_id)
				WHERE a.rombongan_belajar_id = ? ORDER BY p.nama COLLATE NOCASE`,
				[m.rombongan_belajar_id]
			),
			db.query<{ peserta_didik_id: string, nilai: number | null, tp_optimal: string, tp_perlu: string }>(
				"SELECT peserta_didik_id, nilai, tp_optimal, tp_perlu FROM nilai_rapor WHERE pembelajaran_rapor_id = ?",
				[m.id]
			)
		]);
		const byId = new Map(nilai.map(n => [n.peserta_didik_id, n]));
		return {
			prId: m.id,
			singkat: m.singkat,
			mapel: m.mapel,
			tps,
			siswa: siswa.map((s) => {
				const n = byId.get(s.peserta_didik_id);
				return {
					pesertaDidikId: s.peserta_didik_id,
					nisn: s.nisn,
					nama: s.nama,
					nilai: n?.nilai ?? null,
					optimal: n ? JSON.parse(n.tp_optimal) : [],
					perlu: n ? JSON.parse(n.tp_perlu) : []
				};
			})
		};
	}

	const data = ref<SheetMapel>();
	const current = computed(() => list.value.find(m => m.id === prId.value));

	async function loadProgress() {
		const ids = mapelDiRombel.value.map(m => m.id);
		if (!ids.length)
			return;
		const rows = await db.query<{ id: number, n: number }>(
			`SELECT pembelajaran_rapor_id AS id, COUNT(*) AS n FROM nilai_rapor WHERE nilai IS NOT NULL AND tp_optimal <> '[]' AND tp_perlu <> '[]'
			AND pembelajaran_rapor_id IN (${ids.map(() => "?").join(",")}) GROUP BY pembelajaran_rapor_id`,
			ids
		);
		progress.value = Object.fromEntries(rows.map(r => [r.id, r.n]));
	}

	watch(current, async (m) => {
		data.value = m ? await loadMapel(m) : undefined;
		await loadProgress();
	}, { immediate: true });

	// ===== Simpan otomatis (per siswa, ditunda sebentar) =====
	const pending = new Map<string, ReturnType<typeof setTimeout>>();
	const saveState = ref<"idle" | "saving" | "saved" | "error">("idle");

	function upsert(pr: number, s: SheetSiswa) {
		return {
			sql: `INSERT INTO nilai_rapor (pembelajaran_rapor_id, peserta_didik_id, nilai, tp_optimal, tp_perlu, updated_at)
				VALUES (?, ?, ?, ?, ?, datetime('now','localtime'))
				ON CONFLICT (pembelajaran_rapor_id, peserta_didik_id) DO UPDATE SET
					nilai = excluded.nilai, tp_optimal = excluded.tp_optimal, tp_perlu = excluded.tp_perlu, updated_at = excluded.updated_at`,
			params: [pr, s.pesertaDidikId, s.nilai, JSON.stringify(s.optimal), JSON.stringify(s.perlu)]
		};
	}

	function queueSave(s: SheetSiswa) {
		const pr = data.value?.prId;
		if (!pr)
			return;
		const key = `${pr}:${s.pesertaDidikId}`;
		clearTimeout(pending.get(key));
		saveState.value = "saving";
		pending.set(key, setTimeout(async () => {
			pending.delete(key);
			try {
				const st = upsert(pr, s);
				await db.execute(st.sql, st.params);
				if (!pending.size) {
					saveState.value = "saved";
					loadProgress();
				}
			}
			catch (e) {
				saveState.value = "error";
				toast.add({ title: "Gagal menyimpan", description: pesanError(e), color: "error" });
			}
		}, 500));
	}

	function setNilai(s: SheetSiswa, raw: string) {
		const n = raw.trim() === "" ? null : Math.round(Number(raw));
		if (n !== null && (!Number.isFinite(n) || n < 0 || n > 100)) {
			toast.add({ title: "Nilai harus 0–100", color: "warning" });
			return;
		}
		s.nilai = n;
		queueSave(s);
	}

	type TpState = "none" | "optimal" | "perlu";
	const stateOf = (s: SheetSiswa, id: number): TpState => s.optimal.includes(id) ? "optimal" : s.perlu.includes(id) ? "perlu" : "none";

	function setTp(s: SheetSiswa, id: number, st: TpState) {
		s.optimal = s.optimal.filter(x => x !== id);
		s.perlu = s.perlu.filter(x => x !== id);
		if (st === "optimal")
			s.optimal.push(id);
		if (st === "perlu")
			s.perlu.push(id);
	}

	const NEXT: Record<TpState, TpState> = { none: "optimal", optimal: "perlu", perlu: "none" };

	function cycleTp(s: SheetSiswa, id: number) {
		if (!bisaEdit.value)
			return;
		setTp(s, id, NEXT[stateOf(s, id)]);
		queueSave(s);
	}

	// Klik judul kolom TP: semua siswa ikut status berikutnya dari siswa pertama.
	function cycleKolom(id: number) {
		if (!bisaEdit.value || !data.value?.siswa.length)
			return;
		const next = NEXT[stateOf(data.value.siswa[0]!, id)];
		for (const s of data.value.siswa) {
			setTp(s, id, next);
			queueSave(s);
		}
	}

	// Baris yang belum memenuhi aturan deskripsi (minimal 1 ✓ dan 1 !). Baris tanpa nilai & centang
	// sama sekali dianggap "belum diisi", bukan salah.
	const salahDeskripsi = (s: SheetSiswa) => (s.nilai !== null || s.optimal.length || s.perlu.length) && !deskripsiSah(s.optimal, s.perlu);
	const jumlahSalah = computed(() => data.value?.siswa.filter(salahDeskripsi).length ?? 0);

	function deskripsi(s: SheetSiswa) {
		const tps = data.value?.tps ?? [];
		const text = (ids: number[]) => tps.filter(t => ids.includes(t.id)).map(t => t.deskripsi);
		const d = deskripsiCapaian(text(s.optimal), text(s.perlu));
		return [d.capai, d.perlu].filter(Boolean).join(" ");
	}

	// Enter = pindah ke nilai siswa berikutnya, seperti di Excel.
	function nextRow(e: KeyboardEvent, i: number) {
		e.preventDefault();
		(document.querySelector(`[data-nilai="${i + 1}"]`) as HTMLInputElement | null)?.focus();
	}

	// ===== Excel =====
	const rombelNama = computed(() => rombelItems.value.find(r => r.value === rombelId.value)?.label ?? "");
	const fileInput = ref<HTMLInputElement>();
	const busy = ref(false);

	async function exportExcel() {
		busy.value = true;
		try {
			const sheets = await Promise.all(mapelDiRombel.value.map(loadMapel));
			const bytes = buildWorkbook(rombelNama.value, sheets);
			const path = await invoke<string>("save_to_downloads", { fileName: `Nilai Rapor ${rombelNama.value}.xlsx`, bytes: Array.from(bytes) });
			toast.add({
				title: "File Excel tersimpan di Downloads",
				description: path,
				color: "success",
				actions: [{ label: "Buka Folder", onClick: () => { invoke("reveal_file", { path }); } }]
			});
		}
		catch (e) {
			swal.error("Gagal export", pesanError(e));
		}
		finally {
			busy.value = false;
		}
	}

	async function importExcel(e: Event) {
		const file = (e.target as HTMLInputElement).files?.[0];
		(e.target as HTMLInputElement).value = "";
		if (!file)
			return;
		busy.value = true;
		try {
			const sheets = await Promise.all(mapelDiRombel.value.map(loadMapel));
			const hasil = readWorkbook(await file.arrayBuffer(), sheets);
			const st = hasil.flatMap(h => h.siswa.map(s => upsert(h.prId, s)));
			if (!st.length)
				throw new Error("Tidak ada data yang cocok. Pastikan file berasal dari tombol Export di halaman ini.");
			await db.batch(st);
			if (current.value)
				data.value = await loadMapel(current.value);
			await loadProgress();
			const salah = hasil.flatMap(h => h.siswa).filter(s => (s.nilai !== null || s.optimal.length || s.perlu.length) && !deskripsiSah(s.optimal, s.perlu)).length;
			if (salah)
				swal.fire({ type: "warning", title: "Import selesai, ada yang perlu dilengkapi", text: `${st.length} nilai dari ${hasil.length} mapel tersimpan.\n\n${salah} baris belum memenuhi aturan deskripsi: ${ATURAN_DESKRIPSI}. Baris itu ditandai merah di tabel.` });
			else
				swal.success("Import selesai", `${st.length} nilai dari ${hasil.length} mapel tersimpan.`);
		}
		catch (err) {
			swal.error("Gagal import", err instanceof Error ? err.message : pesanError(err));
		}
		finally {
			busy.value = false;
		}
	}
</script>

<template>
	<ErPage id="nilai" title="Nilai Rapor">
		<template #right>
			<span class="text-xs text-muted">
				<template v-if="saveState === 'saving'">Menyimpan…</template>
				<template v-else-if="saveState === 'saved'">Tersimpan otomatis</template>
			</span>
			<USelect
				v-if="rombelItems.length > 1"
				v-model="rombelId"
				:items="rombelItems"
				class="w-36" />
			<UButton
				icon="lucide:file-down"
				variant="subtle"
				:loading="busy"
				:disabled="!mapelDiRombel.length"
				@click="exportExcel">
				Export
			</UButton>
			<UButton
				icon="lucide:upload"
				variant="soft"
				:disabled="!inputDibuka || !mapelDiRombel.length"
				@click="fileInput?.click()">
				Import
			</UButton>
			<input
				ref="fileInput"
				type="file"
				accept=".xlsx"
				class="hidden"
				@change="importExcel">
		</template>

		<div v-if="!loading && !list.length" class="flex flex-col items-center gap-3 py-24 text-center text-muted">
			<UIcon name="lucide:book-x" class="size-12" />
			<p>Anda belum mengampu pembelajaran di semester ini. Minta admin menyusun pembelajaran (Data Referensi → Pembelajaran).</p>
		</div>

		<div v-else class="space-y-4">
			<UAlert
				v-if="!inputDibuka"
				color="warning"
				variant="subtle"
				icon="lucide:lock"
				title="Input nilai sedang ditutup administrator"
				description="Nilai hanya bisa dilihat."
			/>

			<UTabs v-model="prId" :items="tabs" :content="false" />

			<UAlert
				v-if="current?.terkunci"
				color="success"
				variant="subtle"
				icon="lucide:lock"
				title="Nilai mapel ini sudah dikirim ke admin"
				description="Terkunci. Kalau perlu revisi, minta admin membuka kunci, lalu tarik data ulang."
			/>

			<UAlert
				v-if="data && !data.tps.length"
				color="info"
				variant="subtle"
				icon="lucide:list-plus">
				<template #description>
					Belum ada Tujuan Pembelajaran {{ data.singkat }} kelas {{ current?.tingkat }}. Nilai tetap bisa diisi, tapi deskripsi butuh TP.
					<NuxtLink to="/e-rapor/tujuan-pembelajaran" class="font-medium underline">
						Isi TP dulu
					</NuxtLink>
				</template>
			</UAlert>

			<UAlert
				v-if="data && data.tps.length === 1"
				color="warning"
				variant="subtle"
				icon="lucide:list-plus"
				title="Mapel ini baru punya 1 TP"
				:description="`Deskripsi wajib memuat ${ATURAN_DESKRIPSI}, jadi butuh minimal 2 TP. Tambahkan TP dulu.`"
			/>
			<UAlert
				v-else-if="jumlahSalah"
				color="error"
				variant="subtle"
				icon="lucide:circle-alert"
				:title="`${jumlahSalah} siswa belum sesuai aturan deskripsi`"
				:description="`Setiap siswa wajib punya ${ATURAN_DESKRIPSI}. Baris yang belum sesuai ditandai merah; mapel ini belum bisa dikirim sebelum semuanya sesuai.`"
			/>

			<div v-if="data" class="overflow-x-auto rounded-md border border-default">
				<table class="w-full text-sm">
					<thead class="bg-elevated/50 text-xs text-muted">
						<tr>
							<th class="w-10 p-2 text-left">
								No
							</th>
							<th class="min-w-48 p-2 text-left">
								Nama
							</th>
							<th class="w-20 p-2 text-center">
								Nilai
							</th>
							<th v-for="(tp, i) in data.tps" :key="tp.id" class="w-12 p-1 text-center">
								<UTooltip :text="`${tp.deskripsi} — klik untuk ubah satu kelas`">
									<button class="cursor-pointer rounded px-1.5 py-0.5 font-semibold hover:bg-elevated" @click="cycleKolom(tp.id)">
										TP{{ i + 1 }}
									</button>
								</UTooltip>
							</th>
							<th class="min-w-72 p-2 text-left">
								Deskripsi
							</th>
						</tr>
					</thead>
					<tbody class="divide-y divide-default">
						<tr
							v-for="(s, i) in data.siswa"
							:key="s.pesertaDidikId"
							class="hover:bg-elevated/30"
							:class="{ 'bg-error/5': salahDeskripsi(s) }">
							<td class="p-2 text-muted">
								{{ i + 1 }}
							</td>
							<td class="p-2">
								{{ s.nama }}
							</td>
							<td class="p-1 text-center">
								<input
									:data-nilai="i"
									type="number"
									min="0"
									max="100"
									inputmode="numeric"
									:value="s.nilai ?? ''"
									:disabled="!bisaEdit"
									class="w-16 rounded border border-default bg-default px-2 py-1 text-center focus:border-primary focus:outline-none"
									@change="setNilai(s, ($event.target as HTMLInputElement).value)"
									@keydown.enter="nextRow($event, i)"
								>
							</td>
							<td v-for="tp in data.tps" :key="tp.id" class="p-1 text-center">
								<button
									class="size-7 cursor-pointer rounded text-xs font-bold transition"
									:class="{
										'bg-success/15 text-success': stateOf(s, tp.id) === 'optimal',
										'bg-warning/15 text-warning': stateOf(s, tp.id) === 'perlu',
										'text-dimmed hover:bg-elevated': stateOf(s, tp.id) === 'none'
									}"
									:disabled="!bisaEdit"
									@click="cycleTp(s, tp.id)"
								>
									{{ stateOf(s, tp.id) === 'optimal' ? '✓' : stateOf(s, tp.id) === 'perlu' ? '!' : '·' }}
								</button>
							</td>
							<td class="p-2 text-xs text-muted">
								<p v-if="salahDeskripsi(s)" class="mb-0.5 flex items-center gap-1 font-medium text-error">
									<UIcon name="lucide:circle-alert" class="size-3.5 shrink-0" />
									{{ !s.optimal.length && !s.perlu.length ? "Belum ada centang TP" : !s.optimal.length ? "Belum ada TP tercapai (✓)" : "Belum ada TP perlu bantuan (!)" }}
								</p>
								{{ deskripsi(s) || "—" }}
							</td>
						</tr>
					</tbody>
				</table>
			</div>

			<p v-if="data?.tps.length" class="text-xs text-muted">
				Klik kotak TP: sekali <span class="font-bold text-success">✓</span> tercapai optimal, dua kali <span class="font-bold text-warning">!</span> perlu bantuan, tiga kali kosong.
				Klik judul TP untuk mengubah satu kelas sekaligus. Enter di kolom nilai pindah ke siswa berikutnya.
				Aturan deskripsi: setiap siswa wajib punya <b>minimal 1 <span class="text-success">✓</span> dan 1 <span class="text-warning">!</span></b>.
			</p>
		</div>
	</ErPage>
</template>
