<script setup lang="ts">
	interface Sekolah {
		npsn: string | null
		alamat: string | null
		kecamatan: string | null
		kabupaten_kota: string | null
		email: string | null
		kepsek: string | null
	}

	interface Task {
		label: string
		done: boolean
		to: string
		hint: string
	}

	const { session } = useAuth();
	const db = useDb();
	const toast = useToast();

	const sem = computed(() => session.value?.semesterId ?? "");
	const inputKey = computed(() => `input_nilai_dibuka:${sem.value}`);

	const loading = ref(true);
	const counts = ref({ guru: 0, siswa: 0, rombel: 0, pembelajaran: 0, online: 0 });
	const inputDibuka = ref(true);
	const savingInput = ref(false);
	const tasks = ref<Task[]>([]);
	const sekolah = ref<Sekolah>();

	const stats = computed(() => [
		{ title: "Siswa", icon: "lucide:users", value: counts.value.siswa, to: "/referensi/siswa" },
		{ title: "Guru", icon: "lucide:user-round", value: counts.value.guru, to: "/referensi/guru" },
		{ title: "Rombel", icon: "lucide:layers", value: counts.value.rombel, to: "/referensi/kelas" },
		{ title: "Mapel Rapor", icon: "lucide:book-open", value: counts.value.pembelajaran, to: "/referensi/pembelajaran" }
	]);
	const progress = computed(() => tasks.value.length ? Math.round(tasks.value.filter(t => t.done).length / tasks.value.length * 100) : 0);

	async function load() {
		const s = sem.value;
		loading.value = true;
		try {
			// Tabel ptk menumpuk dari semua semester yang pernah ditarik; hitung yang terdaftar di
			// tahun ajaran semester ini saja (tahun_ajaran_id dari getGtk, mis. "2026" untuk 20261).
			const tahun = s.slice(0, 4);
			const [guru, siswa, rombel, pembelajaran, online, ws, admin, guruUser, siswaUser, kepsek, gelar, flag, sek] = await Promise.all([
				db.scalar("SELECT COUNT(*) FROM ptk WHERE json_extract(raw, '$.tahun_ajaran_id') = ?", [tahun]),
				db.scalar(`SELECT COUNT(DISTINCT a.peserta_didik_id) FROM anggota_rombel a JOIN rombel r USING (rombongan_belajar_id)
					WHERE a.semester_id = ? AND r.jenis_rombel = '1'`, [s]),
				db.scalar("SELECT COUNT(*) FROM rombel WHERE semester_id = ? AND jenis_rombel = '1'", [s]),
				db.scalar("SELECT COUNT(*) FROM pembelajaran_rapor WHERE semester_id = ?", [s]),
				db.scalar("SELECT COUNT(*) FROM users WHERE online = 1"),
				db.scalar("SELECT COUNT(*) FROM webservice"),
				db.scalar("SELECT COUNT(*) FROM users WHERE level = 'admin'"),
				db.scalar("SELECT COUNT(*) FROM users WHERE level = 'guru'"),
				db.scalar("SELECT COUNT(*) FROM users WHERE level = 'siswa'"),
				db.scalar("SELECT COUNT(*) FROM sekolah WHERE kepsek_ptk_id IS NOT NULL"),
				db.scalar("SELECT COUNT(*) FROM ptk WHERE COALESCE(gelar_depan, '') <> '' OR COALESCE(gelar_belakang, '') <> ''"),
				db.getSetting(inputKey.value),
				db.first<Sekolah>(
					`SELECT s.npsn, s.alamat, s.kecamatan, s.kabupaten_kota, s.email, g.nama AS kepsek
					FROM sekolah s LEFT JOIN ptk g ON g.ptk_id = s.kepsek_ptk_id WHERE s.sekolah_id = ?`,
					[session.value?.sekolahId]
				)
			]);
			counts.value = { guru, siswa, rombel, pembelajaran, online };
			inputDibuka.value = flag !== "0";
			sekolah.value = sek;

			// Padanan "Status Kerja Administrator" di e-Rapor, sebagai checklist persiapan semester.
			tasks.value = [
				{ label: "Simpan koneksi Web Service", done: ws > 0, to: "/e-rapor/dapodik", hint: "Sinkron Dapodik" },
				{ label: `Tarik data Dapodik ${semesterLabel(s)}`, done: rombel > 0, to: "/e-rapor/dapodik", hint: "Sinkron Dapodik" },
				{ label: "Susun pembelajaran rapor", done: pembelajaran > 0, to: "/e-rapor/referensi/pembelajaran", hint: "Data Referensi → Pembelajaran" },
				{ label: "Tambah administrator lain", done: admin > 1, to: "/e-rapor/pengguna", hint: "Data Referensi → Pengguna" },
				{ label: "Buat akun guru & siswa", done: guru > 0 && siswa > 0 && guruUser >= guru && siswaUser >= siswa, to: "/e-rapor/pengguna", hint: "Generate Semua Pengguna" },
				{ label: "Tentukan kepala sekolah", done: kepsek > 0, to: "/e-rapor/referensi/sekolah", hint: "Data Referensi → Sekolah" },
				{ label: "Lengkapi gelar guru", done: gelar > 0, to: "/e-rapor/referensi/guru", hint: "Data Referensi → Guru" }
			];
		}
		catch (e) {
			toast.add({ title: "Gagal memuat dashboard", description: pesanError(e), color: "error" });
		}
		finally {
			loading.value = false;
		}
	}

	// Optimistic: saklar langsung berubah, dikembalikan kalau gagal disimpan.
	async function toggleInput(v: boolean) {
		const sebelum = inputDibuka.value;
		inputDibuka.value = v;
		savingInput.value = true;
		try {
			await db.setSetting(inputKey.value, v ? "1" : "0");
			toast.add({ title: v ? "Input nilai dibuka untuk guru" : "Input nilai ditutup", color: v ? "success" : "warning" });
		}
		catch (e) {
			inputDibuka.value = sebelum;
			toast.add({ title: "Gagal mengubah status input nilai", description: pesanError(e), color: "error" });
		}
		finally {
			savingInput.value = false;
		}
	}

	watch(sem, load, { immediate: true });

	const sekolahRows = computed(() => {
		const s = sekolah.value;
		if (!s)
			return [];
		return [
			{ icon: "lucide:hash", label: "NPSN", value: s.npsn },
			{ icon: "lucide:map-pin", label: "Alamat", value: [s.alamat, s.kecamatan, s.kabupaten_kota].filter(Boolean).join(", ") },
			{ icon: "lucide:user-check", label: "Kepala Sekolah", value: s.kepsek },
			{ icon: "lucide:mail", label: "Email", value: s.email }
		];
	});
</script>

<template>
	<ErPage id="home" title="Dashboard">
		<template #title>
			<div class="flex items-center gap-2">
				<UIcon name="lucide:file-badge" class="size-6 shrink-0 text-primary" />
				<span class="font-bold">{{ session?.sekolahNama ?? "Belum ada sekolah" }}</span>
			</div>
		</template>
		<template #right>
			<UBadge color="success" variant="subtle" class="flex items-center gap-1.5">
				<span class="inline-block size-1.5 rounded-full bg-success" />
				{{ counts.online }} online
			</UBadge>
			<USwitch
				:model-value="inputDibuka"
				:disabled="!sem || loading"
				:loading="savingInput"
				label="Input nilai dibuka"
				@update:model-value="toggleInput"
			/>
		</template>

		<div v-if="loading" class="space-y-6">
			<USkeleton class="h-24 w-full" />
			<div class="grid gap-6 lg:grid-cols-2">
				<USkeleton class="h-72" />
				<USkeleton class="h-72" />
			</div>
		</div>

		<div v-else class="space-y-6">
			<StatsGrid :stats="stats" />

			<div class="grid gap-6 lg:grid-cols-2">
				<PanelCard title="Persiapan Semester" icon="lucide:list-checks" :description="`${progress}% selesai`">
					<UProgress :model-value="progress" class="mb-3" />
					<ul class="divide-y divide-default">
						<li v-for="t in tasks" :key="t.label">
							<NuxtLink :to="t.to" class="flex items-center gap-3 py-2.5 hover:text-highlighted">
								<UIcon
									:name="t.done ? 'lucide:circle-check' : 'lucide:circle'"
									class="size-5 shrink-0"
									:class="t.done ? 'text-success' : 'text-dimmed'"
									aria-hidden="true"
								/>
								<span class="sr-only">{{ t.done ? "Selesai:" : "Belum:" }}</span>
								<span class="flex-1 text-sm" :class="t.done ? 'text-muted line-through' : ''">{{ t.label }}</span>
								<span class="text-xs text-dimmed">{{ t.hint }}</span>
							</NuxtLink>
						</li>
					</ul>
				</PanelCard>

				<PanelCard title="Data Sekolah" icon="lucide:building-2">
					<dl v-if="sekolahRows.length" class="divide-y divide-default">
						<div v-for="row in sekolahRows" :key="row.label" class="flex items-start gap-3 py-3 first:pt-0 last:pb-0">
							<UIcon :name="row.icon" class="mt-0.5 size-4 shrink-0 text-muted" />
							<div class="grid min-w-0 flex-1 gap-1 sm:grid-cols-3 sm:gap-3">
								<dt class="text-sm text-muted">
									{{ row.label }}
								</dt>
								<dd class="text-sm font-medium wrap-break-word text-highlighted sm:col-span-2">
									{{ row.value || "-" }}
								</dd>
							</div>
						</div>
					</dl>
					<div v-else class="flex flex-col items-center gap-2 py-10 text-muted">
						<UIcon name="lucide:refresh-cw" class="size-6" />
						<span class="text-sm">Belum ada data. Mulai dari menu Sinkron Dapodik.</span>
						<UButton to="/e-rapor/dapodik" size="sm" icon="lucide:arrow-right">
							Sinkron Dapodik
						</UButton>
					</div>
				</PanelCard>
			</div>
		</div>
	</ErPage>
</template>
