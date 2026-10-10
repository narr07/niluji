<script setup lang="ts">
	interface Sekolah {
		sekolah_id: string
		kepsek_ptk_id: string | null
		raw: string | null
	}

	interface Ptk {
		ptk_id: string
		nama: string
		nip: string | null
		jabatan_ptk: string | null
		gelar_depan: string | null
		gelar_belakang: string | null
	}

	const db = useDb();
	const swal = useSwal();
	const toast = useToast();
	const { session } = useAuth();

	const loading = ref(true);
	const saving = ref(false);
	const sekolah = ref<Sekolah>();
	const ptkList = ref<Ptk[]>([]);
	const kepsek = ref<string>();

	// raw = JSON utuh dari Dapodik; kalau rusak, tampilkan kosong daripada halaman gagal dirender.
	const raw = computed<Record<string, any>>(() => {
		try {
			return sekolah.value?.raw ? JSON.parse(sekolah.value.raw) : {};
		}
		catch {
			return {};
		}
	});

	const guruItems = computed(() => ptkList.value.map(p => ({ label: p.jabatan_ptk ? `${p.nama} — ${p.jabatan_ptk}` : p.nama, value: p.ptk_id })));

	// Dihitung dari daftar guru yang sudah dimuat: langsung jadi pratinjau saat memilih, tanpa query lagi.
	const kepsekInfo = computed(() => ptkList.value.find(p => p.ptk_id === kepsek.value));
	const kepsekNama = computed(() => (kepsekInfo.value ? namaLengkap(kepsekInfo.value) : ""));
	const berubah = computed(() => !!kepsek.value && kepsek.value !== sekolah.value?.kepsek_ptk_id);

	async function load() {
		loading.value = true;
		try {
			const id = session.value?.sekolahId;
			const [s, ptk] = await Promise.all([
				id
					? db.first<Sekolah>("SELECT sekolah_id, kepsek_ptk_id, raw FROM sekolah WHERE sekolah_id = ?", [id])
					: db.first<Sekolah>("SELECT sekolah_id, kepsek_ptk_id, raw FROM sekolah LIMIT 1"),
				db.query<Ptk>("SELECT ptk_id, nama, nip, jabatan_ptk, gelar_depan, gelar_belakang FROM ptk ORDER BY nama COLLATE NOCASE")
			]);
			sekolah.value = s;
			ptkList.value = ptk;
			kepsek.value = s?.kepsek_ptk_id ?? undefined;
		}
		catch (e) {
			toast.add({ title: "Gagal memuat data sekolah", description: pesanError(e), color: "error" });
		}
		finally {
			loading.value = false;
		}
	}

	async function saveKepsek() {
		if (!sekolah.value || !kepsek.value)
			return;
		saving.value = true;
		try {
			await db.execute("UPDATE sekolah SET kepsek_ptk_id = ? WHERE sekolah_id = ?", [kepsek.value, sekolah.value.sekolah_id]);
			sekolah.value.kepsek_ptk_id = kepsek.value;
			swal.success("Berhasil", "Kepala sekolah disimpan. Gelar diatur di Data Guru.");
		}
		catch (e) {
			swal.error("Gagal menyimpan", pesanError(e));
		}
		finally {
			saving.value = false;
		}
	}

	onMounted(load);

	const fields = computed(() => {
		const r = raw.value;
		return [
			{ label: "Nama Sekolah", value: r.nama },
			{ label: "NPSN", value: r.npsn },
			{ label: "NSS", value: r.nss },
			{ label: "Bentuk Pendidikan", value: r.bentuk_pendidikan_id_str },
			{ label: "Status", value: r.status_sekolah_str },
			{ label: "Alamat", value: r.alamat_jalan },
			{ label: "RT / RW", value: `${r.rt ?? "-"} / ${r.rw ?? "-"}` },
			{ label: "Desa/Kelurahan", value: r.desa_kelurahan },
			{ label: "Kecamatan", value: r.kecamatan },
			{ label: "Kabupaten/Kota", value: r.kabupaten_kota },
			{ label: "Provinsi", value: r.provinsi },
			{ label: "Kode Pos", value: r.kode_pos },
			{ label: "Telepon", value: r.nomor_telepon },
			{ label: "Email", value: r.email },
			{ label: "Website", value: r.website }
		];
	});
</script>

<template>
	<ErPage id="ref-sekolah" title="Sekolah">
		<template #right>
			<DapodikNote />
		</template>

		<div v-if="loading" class="grid gap-4 lg:grid-cols-[1fr_380px]">
			<USkeleton class="h-96" />
			<USkeleton class="h-56" />
		</div>

		<div v-else class="grid gap-4 lg:grid-cols-[1fr_380px]">
			<PanelCard title="Data Sekolah" icon="lucide:school">
				<dl v-if="sekolah" class="grid grid-cols-1 gap-x-4 gap-y-1 text-sm sm:grid-cols-[180px_1fr] sm:gap-y-2">
					<template v-for="f in fields" :key="f.label">
						<dt class="text-muted">
							{{ f.label }}
						</dt>
						<dd class="mb-2 font-medium text-highlighted sm:mb-0">
							{{ f.value || "-" }}
						</dd>
					</template>
				</dl>
				<p v-else class="text-sm text-muted">
					Belum ada data sekolah. Mulai dari menu Sinkron Dapodik.
				</p>
			</PanelCard>

			<PanelCard title="Kepala Sekolah" icon="lucide:user-check">
				<div class="flex flex-col gap-3">
					<p class="text-sm text-muted">
						Web Service Dapodik tidak mengirim data kepala sekolah, jadi pilih dari daftar guru.
					</p>
					<USelectMenu
						v-model="kepsek"
						:items="guruItems"
						value-key="value"
						placeholder="Pilih kepala sekolah"
						class="w-full"
						:disabled="!sekolah"
					/>
					<UUser
						v-if="kepsekInfo"
						:name="kepsekNama"
						:description="`NIP. ${kepsekInfo.nip || '-'}`"
						:avatar="{ alt: kepsekInfo.nama }"
						class="rounded border border-default p-3"
					/>
					<UButton
						variant="solid"
						icon="lucide:save"
						:loading="saving"
						:disabled="!berubah"
						class="self-end"
						@click="saveKepsek"
					>
						Simpan
					</UButton>
				</div>
			</PanelCard>
		</div>
	</ErPage>
</template>
