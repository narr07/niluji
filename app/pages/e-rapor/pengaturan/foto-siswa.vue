<script setup lang="ts">
	// Padanan e-Rapor: Foto Siswa. Per kelas; klik kartu untuk unggah satu foto, atau pilih banyak file
	// sekaligus yang namanya NISN/NIS (mis. 3194701026.jpg) supaya langsung cocok ke siswanya.
	// Foto dipotong 3:4 dan dikecilkan (±30 KB) lalu disimpan di database (ikut backup).

	interface Siswa { id: string, nama: string, nisn: string | null, nipd: string | null, foto: string | null }

	const db = useDb();
	const toast = useToast();
	const swal = useSwal();
	const { session } = useAuth();

	const kelasList = ref<{ value: string, label: string }[]>([]);
	const rombelId = ref<string>();
	const siswa = ref<Siswa[]>([]);
	const loading = ref(true);
	const proses = ref(false);
	let permintaan = 0;

	async function loadKelas() {
		kelasList.value = await db.query(
			`SELECT rombongan_belajar_id AS value, nama AS label FROM rombel WHERE semester_id = ? AND jenis_rombel = '1'
			ORDER BY CAST(tingkat AS INTEGER), nama`,
			[session.value?.semesterId]
		);
		if (!kelasList.value.some(k => k.value === rombelId.value))
			rombelId.value = kelasList.value[0]?.value;
	}

	async function load() {
		const nomor = ++permintaan;
		if (!rombelId.value) {
			siswa.value = [];
			loading.value = false;
			return;
		}
		loading.value = true;
		try {
			const rows = await db.query<Siswa>(
				`SELECT s.peserta_didik_id AS id, s.nama, s.nisn, s.nipd, f.foto FROM anggota_rombel a JOIN siswa_rapor s USING (peserta_didik_id)
				LEFT JOIN foto_siswa f USING (peserta_didik_id) WHERE a.rombongan_belajar_id = ? ORDER BY s.nama COLLATE NOCASE`,
				[rombelId.value]
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
		try {
			await loadKelas();
			await load();
		}
		catch (e) {
			toast.add({ title: "Gagal memuat kelas", description: pesanError(e), color: "error" });
			loading.value = false;
		}
	}
	watch(() => session.value?.semesterId, init, { immediate: true });
	watch(rombelId, load);

	const adaFoto = computed(() => siswa.value.filter(s => s.foto).length);

	async function simpanFoto(s: Siswa, file: Blob) {
		const foto = await fotoSiswa(file);
		await db.execute(
			`INSERT INTO foto_siswa (peserta_didik_id, foto, updated_at) VALUES (?, ?, datetime('now','localtime'))
			ON CONFLICT (peserta_didik_id) DO UPDATE SET foto = excluded.foto, updated_at = excluded.updated_at`,
			[s.id, foto]
		);
		s.foto = foto;
	}

	// ===== Satu siswa =====
	const satuInput = ref<HTMLInputElement>();
	const target = ref<Siswa>();
	function pilihSatu(s: Siswa) {
		target.value = s;
		satuInput.value?.click();
	}
	async function unggahSatu(e: Event) {
		const el = e.target as HTMLInputElement;
		const file = el.files?.[0];
		el.value = "";
		if (!file || !target.value)
			return;
		proses.value = true;
		try {
			await simpanFoto(target.value, file);
		}
		catch (err) {
			toast.add({ title: "Gagal menyimpan foto", description: pesanError(err), color: "error" });
		}
		finally {
			proses.value = false;
		}
	}

	async function hapus(s: Siswa) {
		if (!await swal.confirm(`Hapus foto ${s.nama}?`, "Pelengkap rapor dicetak dengan kotak pas foto kosong.", "Hapus"))
			return;
		try {
			await db.execute("DELETE FROM foto_siswa WHERE peserta_didik_id = ?", [s.id]);
			s.foto = null;
		}
		catch (e) {
			toast.add({ title: "Gagal menghapus foto", description: pesanError(e), color: "error" });
		}
	}

	// ===== Banyak sekaligus: nama file = NISN atau NIS =====
	const banyakInput = ref<HTMLInputElement>();
	async function unggahBanyak(e: Event) {
		const el = e.target as HTMLInputElement;
		const files = [...(el.files ?? [])];
		el.value = "";
		if (!files.length)
			return;
		proses.value = true;
		const cocok: string[] = [];
		const tidak: string[] = [];
		try {
			for (const f of files) {
				const kunci = f.name.replace(/\.[^.]+$/, "").trim();
				const s = siswa.value.find(x => (x.nisn && x.nisn === kunci) || (x.nipd && x.nipd === kunci));
				if (!s) {
					tidak.push(f.name);
					continue;
				}
				await simpanFoto(s, f);
				cocok.push(s.nama);
			}
			await swal.fire({
				type: tidak.length ? "warning" : "success",
				title: `${cocok.length} foto tersimpan`,
				text: tidak.length ? `${tidak.length} file tidak cocok dengan NISN/NIS siswa ${kelasList.value.find(k => k.value === rombelId.value)?.label}:\n${tidak.slice(0, 10).join(", ")}${tidak.length > 10 ? ", …" : ""}` : ""
			});
		}
		catch (err) {
			toast.add({ title: "Gagal menyimpan foto", description: pesanError(err), color: "error" });
		}
		finally {
			proses.value = false;
		}
	}
</script>

<template>
	<ErPage id="foto-siswa" title="Foto Siswa">
		<template #right>
			<UButton
				icon="lucide:images"
				variant="soft"
				:loading="proses"
				:disabled="!siswa.length"
				@click="banyakInput?.click()">
				Unggah Banyak
			</UButton>
		</template>

		<input
			ref="satuInput"
			type="file"
			accept="image/png,image/jpeg"
			class="hidden"
			aria-label="Pilih foto siswa"
			@change="unggahSatu">
		<input
			ref="banyakInput"
			type="file"
			accept="image/png,image/jpeg"
			multiple
			class="hidden"
			aria-label="Pilih banyak foto"
			@change="unggahBanyak">

		<div v-if="!loading && !kelasList.length" class="flex flex-col items-center gap-3 py-24 text-center text-muted">
			<UIcon name="lucide:camera-off" class="size-12" />
			<p>Belum ada data kelas di semester ini.</p>
		</div>

		<div v-else class="space-y-4">
			<div class="flex flex-wrap items-center gap-2">
				<USelect
					v-model="rombelId"
					:items="kelasList"
					icon="lucide:layers"
					aria-label="Kelas"
					class="w-36" />
				<UBadge
					class="ms-auto tabular-nums"
					variant="subtle"
					:color="siswa.length && adaFoto >= siswa.length ? 'success' : adaFoto ? 'warning' : 'neutral'"
				>
					Ada foto {{ adaFoto }}/{{ siswa.length }}
				</UBadge>
			</div>
			<p class="text-xs text-muted">
				Klik kartu untuk unggah satu foto. <b>Unggah Banyak</b>: pilih semua foto sekelas sekaligus, beri nama file sesuai NISN atau NIS siswa (mis. <code>3194701026.jpg</code>).
				Foto dipotong 3:4 dari tengah secara otomatis.
			</p>

			<div v-if="loading" class="grid grid-cols-2 gap-3 sm:grid-cols-4 lg:grid-cols-6">
				<USkeleton v-for="i in 12" :key="i" class="aspect-[3/4] w-full" />
			</div>
			<div v-else class="grid grid-cols-2 gap-3 sm:grid-cols-4 lg:grid-cols-6">
				<div v-for="s in siswa" :key="s.id" class="group relative">
					<button
						type="button"
						class="block w-full cursor-pointer overflow-hidden rounded-md border border-default text-left hover:border-primary"
						:aria-label="`${s.foto ? 'Ganti' : 'Unggah'} foto ${s.nama}`"
						:disabled="proses"
						@click="pilihSatu(s)"
					>
						<img
							v-if="s.foto"
							:src="s.foto"
							:alt="`Foto ${s.nama}`"
							class="aspect-[3/4] w-full object-cover">
						<div v-else class="flex aspect-[3/4] w-full flex-col items-center justify-center gap-1 bg-elevated text-muted">
							<UIcon name="lucide:user-round" class="size-10" />
							<span class="text-xs">Unggah</span>
						</div>
						<div class="p-2">
							<p class="truncate text-xs font-medium" :title="s.nama">
								{{ s.nama }}
							</p>
							<p class="text-xs text-muted tabular-nums">
								{{ s.nisn ?? s.nipd ?? "–" }}
							</p>
						</div>
					</button>
					<UButton
						v-if="s.foto"
						icon="lucide:trash-2"
						size="xs"
						color="error"
						variant="solid"
						class="absolute top-1 right-1 opacity-0 group-hover:opacity-100 focus-visible:opacity-100"
						:aria-label="`Hapus foto ${s.nama}`"
						@click="hapus(s)"
					/>
				</div>
			</div>
		</div>
	</ErPage>
</template>
