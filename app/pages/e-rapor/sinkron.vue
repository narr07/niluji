<script setup lang="ts">
	import type { KelasSiap } from "~/composables/useGuruSync";

	// Laptop guru ↔ laptop admin. Belum login = tarik data pertama kali; sudah login = tarik ulang,
	// cek TP, dan kirim nilai per kelas (hanya kalau semua mapel di kelas itu lengkap).
	definePageMeta({ layout: false });

	const { session, user, login, waliRombel, loadWali } = useAuth();
	const { tarik, kesiapan, kirim, kirimWalas, kirimEkskul, serverUrl } = useGuruSync();
	const { load: loadMode, adminLaptop } = useAppMode();
	const db = useDb();
	const swal = useSwal();

	const form = reactive({ url: "", pin: "", username: "", password: "" });
	const busy = ref(false);
	const kelas = ref<KelasSiap[]>([]);
	const lastPull = ref<{ at: string, semesterId: string }>();

	async function refresh() {
		await loadMode();
		form.url = await serverUrl();
		form.username = (await db.getSetting("guru_username")) ?? form.username;
		const raw = await db.getSetting("last_pull");
		lastPull.value = raw ? JSON.parse(raw) : undefined;
		if (user.value?.ptk_id && session.value?.semesterId) {
			kelas.value = await kesiapan(user.value.ptk_id, session.value.semesterId);
			await loadWali();
			await loadWalas();
			await loadEkskul();
		}
	}

	// ===== Nilai ekskul yang dibina (boleh dikirim ulang) =====
	interface EkskulKirim { id: string, nama: string, anggota: number, dinilai: number, terkirim?: string | null }
	const ekskul = ref<EkskulKirim[]>([]);
	async function loadEkskul() {
		const sem = session.value?.semesterId;
		const rows = await db.query<Omit<EkskulKirim, "terkirim">>(
			`SELECT r.rombongan_belajar_id AS id, r.nama,
				(SELECT COUNT(*) FROM anggota_rombel a WHERE a.rombongan_belajar_id = r.rombongan_belajar_id) AS anggota,
				(SELECT COUNT(*) FROM nilai_ekskul n WHERE n.rombongan_belajar_id = r.rombongan_belajar_id AND n.semester_id = ? AND n.predikat IS NOT NULL) AS dinilai
			FROM rombel r WHERE r.semester_id = ? AND r.jenis_rombel = '51' AND r.ptk_id = ? ORDER BY r.nama`,
			[sem, sem, user.value?.ptk_id]
		);
		ekskul.value = await Promise.all(rows.map(async r => ({ ...r, terkirim: await db.getSetting(`ekskul_terkirim:${r.id}`) })));
	}

	async function doKirimEkskul(x: EkskulKirim) {
		if (!form.pin || !form.password) {
			swal.error("PIN dan password diperlukan", "Minta admin membuka Sesi Online dan mengirim PIN-nya.");
			return;
		}
		const kurang = x.anggota - x.dinilai;
		if (kurang > 0 && !await swal.confirm(`Nilai ${x.nama} belum lengkap`, `${kurang} siswa belum dinilai. Tetap kirim yang sudah ada? Nanti bisa dikirim ulang.`, "Tetap Kirim"))
			return;
		busy.value = true;
		try {
			const r = await kirimEkskul({ pin: form.pin, password: form.password, ekskulId: x.id, semesterId: session.value!.semesterId! });
			await loadEkskul();
			swal.success("Terkirim", `Nilai ${r.ekskul} (${r.siswa} siswa) sudah diterima admin.`);
		}
		catch (e) {
			swal.error("Gagal kirim", pesanError(e));
		}
		finally {
			busy.value = false;
		}
	}

	// ===== Data wali kelas: kehadiran, catatan, kenaikan (boleh dikirim ulang) =====
	interface WalasKelas { id: string, nama: string, siswa: number, hadir: number, catatan: number, terkirim?: string | null }
	const walas = ref<WalasKelas[]>([]);
	async function loadWalas() {
		const sem = session.value?.semesterId;
		walas.value = await Promise.all(waliRombel.value.map(async (r) => {
			const c = await db.first<{ siswa: number, hadir: number, catatan: number }>(
				`SELECT COUNT(*) AS siswa,
					COUNT(CASE WHEN w.sakit IS NOT NULL AND w.izin IS NOT NULL AND w.alpa IS NOT NULL THEN 1 END) AS hadir,
					COUNT(CASE WHEN w.catatan IS NOT NULL THEN 1 END) AS catatan
				FROM anggota_rombel a LEFT JOIN rapor_siswa w ON w.peserta_didik_id = a.peserta_didik_id AND w.semester_id = ?
				WHERE a.rombongan_belajar_id = ?`,
				[sem, r.rombongan_belajar_id]
			);
			return { id: r.rombongan_belajar_id, nama: r.nama, siswa: c?.siswa ?? 0, hadir: c?.hadir ?? 0, catatan: c?.catatan ?? 0, terkirim: await db.getSetting(`walas_terkirim:${r.rombongan_belajar_id}`) };
		}));
	}

	async function doKirimWalas(w: WalasKelas) {
		if (!form.pin || !form.password) {
			swal.error("PIN dan password diperlukan", "Minta admin membuka Sesi Online dan mengirim PIN-nya.");
			return;
		}
		const kurang = w.siswa - Math.min(w.hadir, w.catatan);
		if (kurang > 0 && !await swal.confirm(`Data ${w.nama} belum lengkap`, `${kurang} siswa belum lengkap kehadiran/catatannya. Tetap kirim yang sudah ada? Nanti bisa dikirim ulang.`, "Tetap Kirim"))
			return;
		busy.value = true;
		try {
			const r = await kirimWalas({ pin: form.pin, password: form.password, rombelId: w.id, semesterId: session.value!.semesterId! });
			await loadWalas();
			swal.success("Terkirim", `Data wali kelas ${r.rombel} (${r.siswa} siswa) sudah diterima admin.`);
		}
		catch (e) {
			swal.error("Gagal kirim", pesanError(e));
		}
		finally {
			busy.value = false;
		}
	}
	onMounted(refresh);

	const tanpaTp = computed(() => kelas.value.flatMap(k => k.mapel.filter(m => !m.tp && !m.terkunci).map(m => `${m.singkat} ${k.rombel}`)));

	async function doTarik() {
		if (!form.url || !form.pin || !form.username || !form.password) {
			swal.error("Belum lengkap", "Isi link, PIN, username, dan password.");
			return;
		}
		if (adminLaptop.value) {
			swal.error("Ini laptop admin", "Tarik data hanya untuk laptop guru. Di laptop admin, guru cukup login dan langsung mengisi nilai.");
			return;
		}
		busy.value = true;
		const pertamaKali = !session.value;
		try {
			const r = await tarik({ ...form });
			if (pertamaKali) {
				const sekolah = await db.first<{ sekolah_id: string }>("SELECT sekolah_id FROM sekolah LIMIT 1");
				await login({ username: form.username, password: form.password, sekolahId: sekolah?.sekolah_id ?? null, semesterId: r.semesterId });
			}
			await refresh();
			await swal.success("Data berhasil ditarik", `Selamat datang, ${r.guru.nama}. Sekarang Anda bisa mengisi nilai tanpa internet.${tanpaTp.value.length ? `\n\nBelum ada TP dari admin untuk: ${tanpaTp.value.join(", ")}. Silakan isi TP sendiri dulu.` : ""}`);
			if (pertamaKali)
				await navigateTo("/");
		}
		catch (e) {
			swal.error("Gagal tarik data", pesanError(e));
		}
		finally {
			busy.value = false;
		}
	}

	async function doKirim(k: KelasSiap) {
		if (!form.pin || !form.password) {
			swal.error("PIN dan password diperlukan", "Minta admin membuka Sesi Online dan mengirim PIN-nya.");
			return;
		}
		const ok = await swal.confirm(
			`Kirim nilai ${k.rombel}?`,
			`${k.mapel.filter(m => !m.terkunci).map(m => m.singkat).join(", ")} akan dikirim ke admin dan terkunci. Kalau perlu revisi, minta admin membuka kunci.`,
			"Kirim"
		);
		if (!ok)
			return;
		busy.value = true;
		try {
			const r = await kirim({ pin: form.pin, password: form.password, rombelId: k.rombongan_belajar_id, ptkId: user.value!.ptk_id!, semesterId: session.value!.semesterId! });
			await refresh();
			swal.success("Terkirim", `${r.pembelajaran} mapel (${r.nilai} nilai) ${k.rombel} sudah diterima admin.`);
		}
		catch (e) {
			swal.error("Gagal kirim", pesanError(e));
		}
		finally {
			busy.value = false;
		}
	}

	function status(k: KelasSiap) {
		if (k.terkirim)
			return { label: "Terkirim", color: "success" as const, icon: "lucide:lock" };
		if (k.siap)
			return { label: "Siap dikirim", color: "primary" as const, icon: "lucide:send" };
		return { label: "Belum lengkap", color: "warning" as const, icon: "lucide:circle-dashed" };
	}

	watch(() => form.url, (v) => {
		// Tempel pesan WA utuh juga boleh: ambil link & PIN-nya.
		const link = v.match(/https?:\/\/\S+/)?.[0];
		const pin = v.match(/PIN\D{0,5}(\d{6})/i)?.[1];
		if (link && link !== v) {
			form.url = link.replace(/[.,]$/, "");
			if (pin)
				form.pin = pin;
		}
	});
</script>

<template>
	<!-- Belum login: kartu tarik data pertama kali -->
	<NuxtLayout v-if="!session" name="auth">
		<UCard class="w-full max-w-sm" :ui="{ body: 'sm:p-6' }">
			<div v-if="adminLaptop" class="flex flex-col items-center gap-3 text-center">
				<UIcon name="lucide:server" class="size-10 text-primary" />
				<h1 class="text-xl font-bold">
					Ini laptop admin
				</h1>
				<p class="text-sm text-muted">
					Tarik data hanya untuk <b>laptop guru</b>. Di laptop admin, guru cukup login dan langsung mengisi nilai, karena datanya sudah ada di sini.
				</p>
				<p class="text-sm text-muted">
					Untuk melayani laptop guru, buka menu <b>Sesi Online</b> sebagai admin.
				</p>
				<UButton to="/e-rapor/login" variant="soft" icon="lucide:arrow-left">
					Kembali ke halaman masuk
				</UButton>
			</div>
			<form v-else class="flex flex-col gap-4" @submit.prevent="doTarik">
				<div class="flex flex-col items-center gap-1 text-center">
					<UIcon name="lucide:laptop" class="size-10 text-primary" />
					<h1 class="text-xl font-bold">
						Laptop Guru
					</h1>
					<p class="text-sm text-muted">
						Tarik data kelas dari laptop admin. Setelah itu nilai bisa diisi tanpa internet.
					</p>
				</div>
				<UFormField label="Link dari admin" help="Boleh tempel pesan WA dari admin utuh">
					<UInput v-model="form.url" icon="lucide:link" placeholder="https://….trycloudflare.com" />
				</UFormField>
				<UFormField label="PIN sesi">
					<UInput
						v-model="form.pin"
						icon="lucide:hash"
						placeholder="6 angka"
						inputmode="numeric" />
				</UFormField>
				<UFormField label="Username">
					<UInput v-model="form.username" icon="lucide:user" placeholder="NIP / username guru" />
				</UFormField>
				<UFormField label="Password">
					<UInput v-model="form.password" icon="lucide:lock" type="password" />
				</UFormField>
				<UButton
					type="submit"
					block
					size="lg"
					variant="solid"
					icon="lucide:download"
					:loading="busy">
					Tarik Data
				</UButton>
				<NuxtLink to="/e-rapor/login" class="text-center text-xs text-muted hover:text-default">
					Kembali ke halaman masuk
				</NuxtLink>
			</form>
		</UCard>
	</NuxtLayout>

	<!-- Sudah login (mode guru): tarik ulang, cek TP, kirim nilai -->
	<NuxtLayout v-else name="default">
		<ErPage id="sinkron" title="Sinkron ke Admin">
			<template #right>
				<span v-if="lastPull" class="text-xs text-muted">Terakhir tarik: {{ lastPull.at }}</span>
			</template>

			<div class="space-y-6">
				<PanelCard title="Koneksi ke Laptop Admin" icon="lucide:link" description="Minta admin membuka Sesi Online, lalu masukkan PIN yang dikirim admin">
					<div class="grid gap-3 md:grid-cols-[1fr_140px_200px_auto] md:items-end">
						<UFormField label="Link">
							<UInput v-model="form.url" icon="lucide:link" placeholder="https://….trycloudflare.com" />
						</UFormField>
						<UFormField label="PIN sesi">
							<UInput v-model="form.pin" icon="lucide:hash" inputmode="numeric" />
						</UFormField>
						<UFormField label="Password Anda">
							<UInput v-model="form.password" icon="lucide:lock" type="password" />
						</UFormField>
						<UButton
							icon="lucide:download"
							variant="soft"
							:loading="busy"
							@click="doTarik">
							Tarik Ulang
						</UButton>
					</div>
				</PanelCard>

				<UAlert
					v-if="tanpaTp.length"
					color="warning"
					variant="subtle"
					icon="lucide:list-plus"
					title="Belum ada TP dari admin"
					:description="`${tanpaTp.join(', ')}. Isi TP sendiri dulu, TP Anda ikut terkirim ke admin bersama nilai.`"
					:actions="[{ label: 'Isi TP', to: '/e-rapor/tujuan-pembelajaran', icon: 'lucide:arrow-right' }]"
				/>

				<PanelCard
					v-for="w in walas"
					:key="w.id"
					:title="`Data Wali Kelas ${w.nama}`"
					icon="lucide:clipboard-check"
					description="Kehadiran, catatan wali kelas, kenaikan, dan kokurikuler. Boleh dikirim ulang setiap ada perubahan."
				>
					<template #actions>
						<UBadge
							v-if="w.terkirim"
							color="success"
							variant="subtle"
							icon="lucide:check">
							Terkirim {{ w.terkirim }}
						</UBadge>
					</template>
					<div class="flex flex-wrap items-center gap-3">
						<UBadge :color="w.hadir >= w.siswa ? 'success' : 'warning'" variant="subtle" class="tabular-nums">
							Kehadiran {{ w.hadir }}/{{ w.siswa }}
						</UBadge>
						<UBadge :color="w.catatan >= w.siswa ? 'success' : 'warning'" variant="subtle" class="tabular-nums">
							Catatan {{ w.catatan }}/{{ w.siswa }}
						</UBadge>
						<UButton
							to="/e-rapor/walas"
							variant="link"
							size="sm"
							icon="lucide:pencil">
							Isi di Kelas Saya
						</UButton>
						<UButton
							class="ms-auto"
							icon="lucide:send"
							variant="solid"
							:loading="busy"
							@click="doKirimWalas(w)">
							Kirim Data {{ w.nama }}
						</UButton>
					</div>
				</PanelCard>

				<PanelCard
					v-for="x in ekskul"
					:key="x.id"
					:title="`Nilai Ekskul ${x.nama}`"
					icon="lucide:trophy"
					description="Predikat dan keterangan ekskul. Boleh dikirim ulang setiap ada perubahan."
				>
					<template #actions>
						<UBadge
							v-if="x.terkirim"
							color="success"
							variant="subtle"
							icon="lucide:check">
							Terkirim {{ x.terkirim }}
						</UBadge>
					</template>
					<div class="flex flex-wrap items-center gap-3">
						<UBadge :color="x.dinilai >= x.anggota ? 'success' : 'warning'" variant="subtle" class="tabular-nums">
							Dinilai {{ x.dinilai }}/{{ x.anggota }}
						</UBadge>
						<UButton
							to="/e-rapor/guru/ekskul"
							variant="link"
							size="sm"
							icon="lucide:pencil">
							Isi Nilai Ekskul
						</UButton>
						<UButton
							class="ms-auto"
							icon="lucide:send"
							variant="solid"
							:loading="busy"
							@click="doKirimEkskul(x)">
							Kirim {{ x.nama }}
						</UButton>
					</div>
				</PanelCard>

				<div class="grid gap-4 xl:grid-cols-2">
					<PanelCard
						v-for="k in kelas"
						:key="k.rombongan_belajar_id"
						:title="k.rombel"
						icon="lucide:layers">
						<template #actions>
							<UBadge :color="status(k).color" variant="subtle" :icon="status(k).icon">
								{{ status(k).label }}
							</UBadge>
						</template>
						<ul class="divide-y divide-default">
							<li v-for="m in k.mapel" :key="m.id" class="flex items-center gap-3 py-2 text-sm">
								<UIcon
									:name="m.terkunci ? 'lucide:lock' : m.lengkap >= m.siswa ? 'lucide:circle-check' : 'lucide:circle-dashed'"
									class="size-4 shrink-0"
									:class="m.terkunci || m.lengkap >= m.siswa ? 'text-success' : 'text-warning'"
								/>
								<span class="w-28 shrink-0 font-medium">{{ m.singkat }}</span>
								<UProgress :model-value="m.siswa ? m.lengkap / m.siswa * 100 : 0" size="sm" class="flex-1" />
								<span class="w-36 shrink-0 text-right text-xs text-muted">
									<template v-if="m.terkunci">Terkirim {{ m.dikirim_at }}</template>
									<template v-else>{{ m.lengkap }}/{{ m.siswa }} lengkap{{ m.tp ? "" : " · tanpa TP" }}</template>
								</span>
							</li>
						</ul>
						<div class="mt-3 flex items-center justify-between gap-2">
							<p class="text-xs text-muted">
								Lengkap = nilai + minimal 1 TP tercapai (✓) dan 1 TP perlu bantuan (!) untuk setiap siswa.
							</p>
							<UButton
								icon="lucide:send"
								variant="solid"
								:disabled="!k.siap"
								:loading="busy"
								@click="doKirim(k)"
							>
								Kirim {{ k.rombel }}
							</UButton>
						</div>
					</PanelCard>
				</div>
			</div>
		</ErPage>
	</NuxtLayout>
</template>
