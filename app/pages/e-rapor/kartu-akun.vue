<script setup lang="ts">
	// Cetak Kartu Akun (padanan e-Rapor): kartu login berisi username + password bawaan, per kelas
	// (siswa) atau semua guru, siap dicetak lalu digunting. Password tersimpan sebagai hash, jadi
	// kartu menampilkan password BAWAAN; kalau admin sudah menggantinya, pakai yang baru.

	interface Kartu { username: string, nama: string, sub: string }

	const db = useDb();
	const toast = useToast();
	const { session } = useAuth();

	const DEFAULT_PASS = { guru: "@dikdasmen123456*", siswa: "erapor2025*" };

	const sekolah = ref("");
	const tahun = computed(() => semesterLabel(session.value?.semesterId));
	const kelasList = ref<{ value: string, label: string }[]>([]);
	// "guru" = semua guru; selain itu = rombongan_belajar_id
	const pilihan = ref<string>("guru");
	const kartu = ref<Kartu[]>([]);
	const loading = ref(true);

	const level = computed<"guru" | "siswa">(() => (pilihan.value === "guru" ? "guru" : "siswa"));
	const password = computed(() => DEFAULT_PASS[level.value]);

	async function load() {
		loading.value = true;
		try {
			sekolah.value = (await db.first<{ nama: string }>("SELECT nama FROM sekolah LIMIT 1"))?.nama ?? "";
			const sem = session.value?.semesterId;
			kelasList.value = sem
				? await db.query("SELECT rombongan_belajar_id AS value, nama AS label FROM rombel WHERE semester_id = ? AND jenis_rombel = '1' ORDER BY CAST(tingkat AS INTEGER), nama", [sem])
				: [];
			await loadKartu();
		}
		catch (e) {
			toast.add({ title: "Gagal memuat data", description: pesanError(e), color: "error" });
		}
		finally {
			loading.value = false;
		}
	}

	async function loadKartu() {
		if (pilihan.value === "guru") {
			kartu.value = (await db.query<{ username: string, nama: string }>(
				"SELECT u.username, g.nama FROM users u JOIN ptk g ON g.ptk_id = u.ptk_id WHERE u.level = 'guru' ORDER BY g.nama COLLATE NOCASE"
			)).map(r => ({ username: r.username, nama: r.nama, sub: "Guru" }));
		}
		else {
			kartu.value = (await db.query<{ username: string, nama: string, rombel: string }>(
				`SELECT u.username, s.nama, r.nama AS rombel FROM users u
				JOIN siswa_rapor s ON s.peserta_didik_id = u.peserta_didik_id
				JOIN anggota_rombel a ON a.peserta_didik_id = u.peserta_didik_id
				JOIN rombel r ON r.rombongan_belajar_id = a.rombongan_belajar_id
				WHERE u.level = 'siswa' AND a.rombongan_belajar_id = ? ORDER BY s.nama COLLATE NOCASE`,
				[pilihan.value]
			)).map(r => ({ username: r.username, nama: r.nama, sub: r.rombel }));
		}
	}

	watch(() => session.value?.semesterId, load, { immediate: true });
	watch(pilihan, loadKartu);

	const pilihanItems = computed(() => [
		{ value: "guru", label: "Semua Guru" },
		...kelasList.value.map(k => ({ value: k.value, label: k.label }))
	]);

	function cetak() {
		window.print();
	}
</script>

<template>
	<ErPage id="kartu-akun" title="Cetak Kartu Akun">
		<template #right>
			<USelect
				v-model="pilihan"
				:items="pilihanItems"
				icon="lucide:users"
				class="w-48 print:hidden" />
			<UButton
				icon="lucide:printer"
				:disabled="!kartu.length"
				class="print:hidden"
				@click="cetak">
				Cetak {{ kartu.length }} Kartu
			</UButton>
		</template>

		<div class="space-y-4">
			<UAlert
				color="info"
				variant="subtle"
				icon="lucide:info"
				class="print:hidden"
				:description="`Kartu menampilkan password bawaan (${password}). Kalau password sudah diganti lewat menu Pengguna, berikan password yang baru ke pemiliknya.`"
			/>

			<div v-if="loading" class="grid grid-cols-2 gap-3 sm:grid-cols-3">
				<USkeleton v-for="i in 9" :key="i" class="h-28 w-full" />
			</div>
			<div v-else-if="!kartu.length" class="py-16 text-center text-sm text-muted print:hidden">
				Belum ada akun {{ pilihan === "guru" ? "guru" : "siswa di kelas ini" }}. Buat dulu di menu Pengguna.
			</div>

			<div v-else class="kartu-grid">
				<KartuItem
					v-for="(k, i) in kartu"
					:key="i"
					:sekolah="sekolah"
					:sub="`${k.sub} · ${tahun}`"
					:nama="k.nama"
					:username="k.username"
					:password="password" />
			</div>
		</div>

		<!-- Hanya tampil saat mencetak, tanpa sidebar/navbar (.cetak-root global di main.css). -->
		<Teleport to="body">
			<div class="cetak-root">
				<div class="kartu-grid kartu-grid-cetak">
					<KartuItem
						v-for="(k, i) in kartu"
						:key="i"
						:sekolah="sekolah"
						:sub="`${k.sub} · ${tahun}`"
						:nama="k.nama"
						:username="k.username"
						:password="password" />
				</div>
			</div>
		</Teleport>
	</ErPage>
</template>

<style>
	.kartu-grid { display: grid; grid-template-columns: repeat(2, 1fr); gap: 10px; }
	@media (min-width: 640px) { .kartu-grid { grid-template-columns: repeat(3, 1fr); } }

	/* Salinan cetak memakai .cetak-root global (main.css) untuk menyembunyikan sidebar/navbar.
	   @page margin 0 + padding di dalam = tanggal/URL browser tidak ikut tercetak. */
	@media print {
		@page { margin: 0; }
		.cetak-root { padding: 10mm; }
		.cetak-root .kartu-grid-cetak { display: grid; grid-template-columns: repeat(3, 1fr); gap: 5mm; }
	}
</style>
