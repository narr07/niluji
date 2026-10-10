<script setup lang="ts">
	// Padanan e-Rapor: Data Kokurikuler → Tema + Kegiatan + Profil Lulusan + Kelompok, diringkas:
	// satu kegiatan = nama, tema, tujuan akhir, kelas yang mengikuti, dimensi profil lulusan.
	// Kelompok = kelas, koordinator = wali kelas (tidak perlu disusun manual).

	interface Baris extends KegiatanKoku { dinilai: number }

	const db = useDb();
	const toast = useToast();
	const swal = useSwal();
	const { session } = useAuth();

	const kegiatan = ref<Baris[]>([]);
	const tingkatAda = ref<number[]>([]);
	const loading = ref(true);

	async function load() {
		const sem = session.value?.semesterId;
		if (!sem) {
			kegiatan.value = [];
			loading.value = false;
			return;
		}
		loading.value = true;
		try {
			const [rows, tingkat] = await Promise.all([
				db.query<Omit<Baris, "tingkat" | "dimensi" | "subdimensi"> & { tingkat: string, dimensi: string, subdimensi: string }>(
					`SELECT k.id, k.nama, k.tema, k.tujuan, k.tingkat, k.dimensi, k.subdimensi,
						(SELECT COUNT(*) FROM nilai_kokurikuler n WHERE n.kegiatan_id = k.id AND n.capaian <> '{}') AS dinilai
					FROM kokurikuler_kegiatan k WHERE k.semester_id = ? ORDER BY k.urutan, k.id`,
					[sem]
				),
				db.query<{ t: number }>("SELECT DISTINCT CAST(tingkat AS INTEGER) AS t FROM rombel WHERE semester_id = ? AND jenis_rombel = '1' ORDER BY t", [sem])
			]);
			kegiatan.value = rows.map(r => ({ ...r, tingkat: JSON.parse(r.tingkat), dimensi: JSON.parse(r.dimensi), subdimensi: JSON.parse(r.subdimensi || "{}") }));
			tingkatAda.value = tingkat.map(t => t.t);
		}
		catch (e) {
			toast.add({ title: "Gagal memuat kegiatan kokurikuler", description: pesanError(e), color: "error" });
		}
		finally {
			loading.value = false;
		}
	}
	watch(() => session.value?.semesterId, load, { immediate: true });

	// Kelas yang belum punya kegiatan → peringatan (bagian kokurikuler rapornya akan kosong).
	const tanpaKegiatan = computed(() => tingkatAda.value.filter(t => !kegiatan.value.some(k => k.tingkat.includes(t))));
	const temaSaran = computed(() => [...new Set(kegiatan.value.map(k => k.tema).filter((t): t is string => !!t))]);

	// ===== Form tambah/ubah =====
	const buka = ref(false);
	const saving = ref(false);
	const dicoba = ref(false);
	// sub = teks subdimensi per dimensi, satu baris satu subdimensi.
	const form = reactive({ id: 0, nama: "", tema: "", tujuan: "", tingkat: [] as number[], dimensi: [] as string[], sub: {} as Record<string, string> });
	const errors = computed(() => ({
		nama: !form.nama.trim() ? "Wajib diisi" : undefined,
		tingkat: !form.tingkat.length ? "Pilih minimal satu kelas" : undefined,
		dimensi: !form.dimensi.length ? "Pilih minimal satu dimensi" : undefined
	}));
	const valid = computed(() => !Object.values(errors.value).some(Boolean));
	const err = (k: keyof typeof errors.value) => (dicoba.value ? errors.value[k] : undefined);

	function tambah() {
		Object.assign(form, { id: 0, nama: "", tema: temaSaran.value[0] ?? "", tujuan: "", tingkat: [...tanpaKegiatan.value], dimensi: [], sub: {} });
		dicoba.value = false;
		buka.value = true;
	}
	function ubah(k: Baris) {
		Object.assign(form, { id: k.id, nama: k.nama, tema: k.tema ?? "", tujuan: k.tujuan ?? "", tingkat: [...k.tingkat], dimensi: [...k.dimensi], sub: Object.fromEntries(Object.entries(k.subdimensi).map(([d, xs]) => [d, xs.join("\n")])) });
		dicoba.value = false;
		buka.value = true;
	}
	const toggle = <T,>(arr: T[], v: T) => (arr.includes(v) ? arr.filter(x => x !== v) : [...arr, v]);

	async function simpan() {
		dicoba.value = true;
		if (!valid.value || saving.value)
			return;
		saving.value = true;
		try {
			// Tujuan akhir diawali huruf kecil (panduan e-Rapor), tanpa titik di akhir.
			const tujuan = form.tujuan.trim().replace(/\.$/, "");
			const dimensi = DIMENSI.map(d => d.kode).filter(k => form.dimensi.includes(k));
			const sub = Object.fromEntries(dimensi
				.map(d => [d, (form.sub[d] ?? "").split(/\r?\n/).map(x => x.trim()).filter(Boolean)] as const)
				.filter(([, xs]) => xs.length));
			const params = [form.nama.trim(), form.tema.trim() || null, tujuan ? tujuan.charAt(0).toLowerCase() + tujuan.slice(1) : null,
				JSON.stringify([...form.tingkat].sort()), JSON.stringify(dimensi), JSON.stringify(sub)];
			if (form.id) {
				await db.execute("UPDATE kokurikuler_kegiatan SET nama = ?, tema = ?, tujuan = ?, tingkat = ?, dimensi = ?, subdimensi = ? WHERE id = ?", [...params, form.id]);
			}
			else {
				await db.execute(
					`INSERT INTO kokurikuler_kegiatan (semester_id, nama, tema, tujuan, tingkat, dimensi, subdimensi, urutan)
					VALUES (?, ?, ?, ?, ?, ?, ?, (SELECT COALESCE(MAX(urutan), 0) + 1 FROM kokurikuler_kegiatan WHERE semester_id = ?))`,
					[session.value?.semesterId, ...params, session.value?.semesterId]
				);
			}
			buka.value = false;
			toast.add({ title: form.id ? "Kegiatan diperbarui" : "Kegiatan ditambahkan", color: "success" });
			await load();
		}
		catch (e) {
			toast.add({ title: "Gagal menyimpan kegiatan", description: pesanError(e), color: "error" });
		}
		finally {
			saving.value = false;
		}
	}

	async function hapus(k: Baris) {
		if (!await swal.confirm(`Hapus kegiatan "${k.nama}"?`, k.dinilai ? `Capaian ${k.dinilai} siswa untuk kegiatan ini ikut terhapus.` : "Belum ada siswa yang dinilai.", "Hapus"))
			return;
		try {
			await db.batch([
				{ sql: "DELETE FROM nilai_kokurikuler WHERE kegiatan_id = ?", params: [k.id] },
				{ sql: "DELETE FROM kokurikuler_kegiatan WHERE id = ?", params: [k.id] }
			]);
			await load();
		}
		catch (e) {
			toast.add({ title: "Gagal menghapus", description: pesanError(e), color: "error" });
		}
	}
</script>

<template>
	<ErPage id="kokurikuler" title="Kegiatan Kokurikuler">
		<template #right>
			<UButton icon="lucide:plus" @click="tambah">
				Tambah Kegiatan
			</UButton>
		</template>

		<div class="space-y-4">
			<UAlert
				color="neutral"
				variant="subtle"
				icon="lucide:info"
				description="Lebih ringkas dari e-Rapor: kelompok kokurikuler = kelas, koordinatornya wali kelas. Wali kelas menilai capaian tiap dimensi di menu Kokurikuler, lalu deskripsi rapor dibuat otomatis."
			/>
			<UAlert
				v-if="!loading && tanpaKegiatan.length"
				color="warning"
				variant="subtle"
				icon="lucide:triangle-alert"
				:title="`Kelas ${tanpaKegiatan.join(', ')} belum punya kegiatan`"
				description="Bagian kokurikuler di rapor kelas tersebut akan kosong."
			/>

			<div v-if="loading" class="space-y-2">
				<USkeleton v-for="i in 3" :key="i" class="h-24 w-full" />
			</div>
			<div v-else-if="!kegiatan.length" class="flex flex-col items-center gap-3 py-16 text-center text-muted">
				<UIcon name="lucide:notebook" class="size-12" />
				<p>Belum ada kegiatan kokurikuler di semester ini.</p>
				<UButton icon="lucide:plus" variant="soft" @click="tambah">
					Tambah Kegiatan
				</UButton>
			</div>
			<div v-else class="grid gap-4 xl:grid-cols-2">
				<PanelCard
					v-for="k in kegiatan"
					:key="k.id"
					:title="k.nama"
					icon="lucide:notebook"
					:description="[k.tema && `Tema: ${k.tema}`, `Kelas ${k.tingkat.join(', ')}`, `${k.dinilai} siswa dinilai`].filter(Boolean).join(' · ')"
				>
					<template #actions>
						<UButton
							icon="lucide:pencil"
							variant="ghost"
							size="sm"
							:aria-label="`Ubah ${k.nama}`"
							@click="ubah(k)" />
						<UButton
							icon="lucide:trash-2"
							variant="ghost"
							color="error"
							size="sm"
							:aria-label="`Hapus ${k.nama}`"
							@click="hapus(k)" />
					</template>
					<p v-if="k.tujuan" class="mb-3 text-sm">
						<span class="text-muted">Tujuan akhir:</span> {{ k.tujuan }}
					</p>
					<div class="flex flex-wrap gap-1">
						<UBadge
							v-for="d in k.dimensi"
							:key="d"
							variant="subtle"
							size="sm">
							{{ namaDimensi(d) }}{{ k.subdimensi[d]?.length ? ` (${k.subdimensi[d].length} subdimensi)` : "" }}
						</UBadge>
					</div>
				</PanelCard>
			</div>
		</div>

		<UModal v-model:open="buka" :title="form.id ? 'Ubah Kegiatan' : 'Tambah Kegiatan'" :ui="{ content: 'max-w-xl' }">
			<template #body>
				<form class="space-y-4" @submit.prevent="simpan">
					<UFormField label="Nama kegiatan" :error="err('nama')" required>
						<UInput
							v-model="form.nama"
							placeholder="mis. Pasar Jajanan Sehat"
							class="w-full"
							autofocus />
					</UFormField>
					<UFormField label="Tema" help="Boleh sama untuk beberapa kegiatan">
						<UInputMenu
							v-model="form.tema"
							:items="temaSaran"
							create-item
							placeholder="mis. Gaya Hidup Berkelanjutan"
							class="w-full"
							@create="form.tema = $event" />
					</UFormField>
					<UFormField label="Tujuan akhir" help="Diawali huruf kecil, mis. &quot;merancang dan menjual makanan sehat&quot;">
						<UTextarea
							v-model="form.tujuan"
							:rows="2"
							autoresize
							class="w-full" />
					</UFormField>
					<UFormField label="Kelas yang mengikuti" :error="err('tingkat')" required>
						<div class="flex flex-wrap gap-2">
							<UButton
								v-for="t in (tingkatAda.length ? tingkatAda : [1, 2, 3, 4, 5, 6])"
								:key="t"
								size="sm"
								:variant="form.tingkat.includes(t) ? 'solid' : 'outline'"
								:color="form.tingkat.includes(t) ? 'primary' : 'neutral'"
								:aria-pressed="form.tingkat.includes(t)"
								@click="form.tingkat = toggle(form.tingkat, t)"
							>
								Kelas {{ t }}
							</UButton>
						</div>
					</UFormField>
					<UFormField label="Dimensi profil lulusan yang dikuatkan" :error="err('dimensi')" required>
						<div class="grid gap-2 sm:grid-cols-2">
							<UCheckbox
								v-for="d in DIMENSI"
								:key="d.kode"
								:label="d.nama"
								:model-value="form.dimensi.includes(d.kode)"
								@update:model-value="form.dimensi = toggle(form.dimensi, d.kode)"
							/>
						</div>
					</UFormField>
					<UFormField
						v-if="form.dimensi.length"
						label="Subdimensi (opsional)"
						help="Satu baris satu subdimensi, salin dari panduan kokurikuler / e-Rapor. Kosong = dimensinya dinilai langsung."
					>
						<div class="space-y-2">
							<div v-for="d in DIMENSI.filter(x => form.dimensi.includes(x.kode))" :key="d.kode">
								<p class="mb-1 text-xs font-medium">
									{{ d.nama }}
								</p>
								<UTextarea
									v-model="form.sub[d.kode]"
									:rows="1"
									autoresize
									:placeholder="`Subdimensi ${d.singkat}, satu per baris`"
									class="w-full" />
							</div>
						</div>
					</UFormField>
				</form>
			</template>
			<template #footer>
				<div class="flex w-full justify-end gap-2">
					<UButton variant="ghost" color="neutral" @click="buka = false">
						Batal
					</UButton>
					<UButton icon="lucide:save" :loading="saving" @click="simpan">
						Simpan
					</UButton>
				</div>
			</template>
		</UModal>
	</ErPage>
</template>
