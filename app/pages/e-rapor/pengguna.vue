<script setup lang="ts">
	import type { DropdownMenuItem, TableColumn } from "@nuxt/ui";
	import { invoke } from "@tauri-apps/api/core";

	interface UserRow {
		id: number
		username: string
		nama: string
		email: string | null
		level: UserLevel
		aktif: number
		online: number
		last_login: string | null
	}

	// Password bawaan sama dengan e-Rapor SD (Panduan hal. 16) supaya guru/siswa tidak bingung.
	const DEFAULT_PASS: Record<"guru" | "siswa", string> = { guru: "@dikdasmen123456*", siswa: "erapor2025*" };
	// Password bebas, asal tidak kosong (sama dengan aturan di Rust: db::cek_password).

	const db = useDb();
	const swal = useSwal();
	const toast = useToast();
	const { user: me, session } = useAuth();

	const tab = ref<UserLevel>("admin");
	const tabs = [
		{ label: "Administrator", value: "admin" },
		{ label: "Guru", value: "guru" },
		{ label: "Siswa", value: "siswa" }
	];
	const loading = ref(true);
	const users = ref<UserRow[]>([]);
	const search = ref("");
	const generating = ref(false);

	async function load() {
		try {
			users.value = await db.query<UserRow>(
				"SELECT id, username, nama, email, level, aktif, online, last_login FROM users ORDER BY level, nama COLLATE NOCASE"
			);
		}
		catch (e) {
			toast.add({ title: "Gagal memuat pengguna", description: pesanError(e), color: "error" });
		}
		finally {
			loading.value = false;
		}
	}
	onMounted(load);

	const filtered = computed(() => {
		const q = search.value.toLowerCase().trim();
		return users.value.filter(u => u.level === tab.value
			&& (!q || u.username.toLowerCase().includes(q) || u.nama.toLowerCase().includes(q)));
	});
	const jumlahPerLevel = computed(() => Object.fromEntries(tabs.map(t => [t.value, users.value.filter(u => u.level === t.value).length])));
	const tabItems = computed(() => tabs.map(t => ({ ...t, badge: String(jumlahPerLevel.value[t.value] ?? 0) })));

	// ===== Checklist =====
	const dipilih = ref(new Set<number>());
	watch(tab, () => { dipilih.value = new Set(); });

	const terpilih = computed(() => users.value.filter(u => dipilih.value.has(u.id)));
	const semuaDipilih = computed(() => filtered.value.length > 0 && filtered.value.every(u => dipilih.value.has(u.id)));
	const sebagianDipilih = computed(() => !semuaDipilih.value && filtered.value.some(u => dipilih.value.has(u.id)));

	function pilih(id: number, on: boolean) {
		const s = new Set(dipilih.value);
		if (on)
			s.add(id);
		else
			s.delete(id);
		dipilih.value = s;
	}

	function pilihSemua(on: boolean) {
		const s = new Set(dipilih.value);
		filtered.value.forEach(u => (on ? s.add(u.id) : s.delete(u.id)));
		dipilih.value = s;
	}

	// ===== Atur password (satu atau banyak pengguna) =====
	const pwOpen = ref(false);
	const pwSaving = ref(false);
	const pwTarget = ref<UserRow[]>([]);
	const pw = reactive({ mode: "manual" as "manual" | "bawaan", password: "", ulang: "", tampil: false, dicoba: false });

	const bisaBawaan = computed(() => pwTarget.value.length > 0 && pwTarget.value.every(u => u.level !== "admin"));
	const pwError = computed(() => {
		if (pw.mode === "bawaan")
			return {};
		return {
			password: !pw.password ? "Wajib diisi" : undefined,
			ulang: pw.ulang !== pw.password ? "Tidak sama dengan password" : undefined
		};
	});
	const pwValid = computed(() => !pwError.value.password && !pwError.value.ulang);

	function openPassword(target: UserRow[]) {
		pwTarget.value = target;
		Object.assign(pw, { mode: "manual", password: "", ulang: "", tampil: false, dicoba: false });
		pwOpen.value = true;
	}

	async function simpanPassword() {
		pw.dicoba = true;
		if (!pwValid.value || pwSaving.value)
			return;
		pwSaving.value = true;
		try {
			const items = pwTarget.value.map(u => ({
				userId: u.id,
				password: pw.mode === "manual" ? pw.password : DEFAULT_PASS[u.level as "guru" | "siswa"]
			}));
			const n = await invoke<number>("users_set_password", { items });
			pwOpen.value = false;
			dipilih.value = new Set();
			toast.add({
				title: `Password ${n} pengguna diganti`,
				description: pw.mode === "manual" ? "Berikan password baru ke pengguna yang bersangkutan." : "Kembali ke password bawaan e-Rapor.",
				color: "success"
			});
		}
		catch (e) {
			toast.add({ title: "Gagal mengganti password", description: pesanError(e), color: "error" });
		}
		finally {
			pwSaving.value = false;
		}
	}

	async function setAktifBanyak(aktif: boolean) {
		const target = terpilih.value.filter(u => u.id !== me.value?.id);
		if (!target.length)
			return;
		try {
			await db.execute(
				`UPDATE users SET aktif = ?, online = 0 WHERE id IN (${target.map(() => "?").join(",")})`,
				[aktif ? 1 : 0, ...target.map(u => u.id)]
			);
			dipilih.value = new Set();
			await load();
			toast.add({ title: `${target.length} pengguna ${aktif ? "diaktifkan" : "dinonaktifkan"}`, color: "success" });
		}
		catch (e) {
			toast.add({ title: "Gagal mengubah status", description: pesanError(e), color: "error" });
		}
	}

	// ===== Generate username =====
	const slug = (s: string) => s.toLowerCase().replace(/[^a-z0-9]/g, "");

	function uniqueName(base: string, taken: Set<string>) {
		let name = base || "user";
		let n = 2;
		while (taken.has(name))
			name = `${base}${n++}`;
		taken.add(name);
		return name;
	}

	async function takenUsernames() {
		return new Set((await db.query<{ username: string }>("SELECT username FROM users")).map(u => u.username.toLowerCase()));
	}

	// Guru: username NIP, kalau kosong nama tanpa spasi. Siswa: NISN, kalau kosong nama.
	// `taken` dibagi antara guru & siswa saat Generate Semua, supaya username tidak saling bentrok
	// (mis. guru tanpa NIP dan siswa tanpa NISN yang namanya sama).
	async function buildGuruSiswa(level: "guru" | "siswa", onlyIds?: string[], taken?: Set<string>) {
		taken ??= await takenUsernames();
		if (level === "guru") {
			const rows = await db.query<{ ptk_id: string, nama: string, nip: string | null }>(
				"SELECT ptk_id, nama, nip FROM ptk WHERE ptk_id NOT IN (SELECT ptk_id FROM users WHERE ptk_id IS NOT NULL) ORDER BY nama COLLATE NOCASE"
			);
			return rows.filter(r => !onlyIds || onlyIds.includes(r.ptk_id)).map(r => ({
				username: uniqueName(r.nip?.trim() || slug(r.nama), taken),
				password: DEFAULT_PASS.guru,
				nama: r.nama,
				level,
				ptkId: r.ptk_id
			}));
		}
		const rows = await db.query<{ peserta_didik_id: string, nama: string, nisn: string | null }>(
			`SELECT DISTINCT s.peserta_didik_id, s.nama, s.nisn FROM siswa_rapor s
			JOIN anggota_rombel a ON a.peserta_didik_id = s.peserta_didik_id AND a.semester_id = ?
			WHERE s.peserta_didik_id NOT IN (SELECT peserta_didik_id FROM users WHERE peserta_didik_id IS NOT NULL)
			ORDER BY s.nama COLLATE NOCASE`,
			[session.value?.semesterId]
		);
		return rows.filter(r => !onlyIds || onlyIds.includes(r.peserta_didik_id)).map(r => ({
			username: uniqueName(r.nisn?.trim() || slug(r.nama), taken),
			password: DEFAULT_PASS.siswa,
			nama: r.nama,
			level,
			pesertaDidikId: r.peserta_didik_id
		}));
	}

	async function generateAll() {
		if (!await swal.confirm("Generate Semua Pengguna?", `Akun dibuat untuk semua guru dan siswa yang belum punya akun.\nPassword bawaan guru: ${DEFAULT_PASS.guru}\nPassword bawaan siswa: ${DEFAULT_PASS.siswa}\n\nPassword bisa diganti nanti lewat checklist → Atur Password.`, "Generate"))
			return;
		generating.value = true;
		try {
			const taken = await takenUsernames();
			const list = [...await buildGuruSiswa("guru", undefined, taken), ...await buildGuruSiswa("siswa", undefined, taken)];
			if (!list.length) {
				toast.add({ title: "Tidak ada yang perlu dibuat", description: "Semua guru dan siswa sudah punya akun.", color: "info" });
				return;
			}
			const r = await invoke<{ created: number, skipped: string[] }>("users_create", { users: list });
			await load();
			if (r.skipped.length)
				await swal.fire({ type: "warning", title: `${r.created} akun dibuat, ${r.skipped.length} dilewati`, text: `Sudah ada akun dengan username ini: ${r.skipped.join(", ")}` });
			else
				toast.add({ title: `${r.created} akun guru & siswa dibuat`, color: "success" });
		}
		catch (e) {
			toast.add({ title: "Gagal membuat akun", description: pesanError(e), color: "error" });
		}
		finally {
			generating.value = false;
		}
	}

	// ===== Tambah pengguna =====
	// Kesalahan isian tampil langsung di bawah kolomnya (bukan popup yang bisa tertutup modal).
	const addOpen = ref(false);
	const saving = ref(false);
	const add = reactive({
		level: "admin" as UserLevel,
		username: "",
		nama: "",
		email: "",
		refId: undefined as string | undefined,
		pwMode: "manual" as "manual" | "bawaan",
		password: "",
		ulang: "",
		tampil: false,
		dicoba: false
	});
	const refOptions = ref<{ label: string, value: string }[]>([]);

	watch(() => add.level, async (level) => {
		add.refId = undefined;
		add.pwMode = level === "admin" ? "manual" : "bawaan";
		if (level === "admin") {
			refOptions.value = [];
			return;
		}
		try {
			const rows = await buildGuruSiswa(level);
			// Level sudah diganti lagi selama menunggu: buang hasil lama.
			if (add.level !== level)
				return;
			refOptions.value = rows.map(r => ({ label: `${r.nama} (${r.username})`, value: "ptkId" in r ? r.ptkId! : r.pesertaDidikId! }));
		}
		catch (e) {
			toast.add({ title: "Gagal memuat daftar nama", description: pesanError(e), color: "error" });
		}
	});

	function openAdd() {
		Object.assign(add, { level: tab.value, username: "", nama: "", email: "", refId: undefined, pwMode: tab.value === "admin" ? "manual" : "bawaan", password: "", ulang: "", tampil: false, dicoba: false });
		addOpen.value = true;
	}

	const addError = computed(() => {
		const err: Record<string, string | undefined> = {};
		if (add.level === "admin") {
			const u = add.username.trim().toLowerCase();
			if (!u)
				err.username = "Wajib diisi";
			else if (/\s/.test(u))
				err.username = "Tanpa spasi";
			else if (users.value.some(x => x.username.toLowerCase() === u))
				err.username = "Username sudah dipakai";
			if (!add.nama.trim())
				err.nama = "Wajib diisi";
		}
		else if (!add.refId) {
			err.refId = "Pilih nama dulu";
		}
		if (add.pwMode === "manual") {
			if (!add.password)
				err.password = "Wajib diisi";
			if (add.ulang !== add.password)
				err.ulang = "Tidak sama dengan password";
		}
		return err;
	});
	const addValid = computed(() => Object.values(addError.value).every(v => !v));
	const tampilError = (k: string) => (add.dicoba ? addError.value[k] : undefined);

	async function saveAdd() {
		add.dicoba = true;
		if (!addValid.value || saving.value)
			return;
		saving.value = true;
		try {
			let list: { username: string, password: string, nama: string, level: string, email?: string | null }[];
			if (add.level === "admin") {
				list = [{ username: add.username.trim(), password: add.password, nama: add.nama.trim(), email: add.email.trim() || null, level: "admin" }];
			}
			else {
				list = await buildGuruSiswa(add.level, [add.refId!]);
				if (add.pwMode === "manual")
					list = list.map(x => ({ ...x, password: add.password }));
			}
			const r = await invoke<{ created: number, skipped: string[] }>("users_create", { users: list });
			if (!r.created)
				throw new Error(`Sudah punya akun / username dipakai: ${r.skipped.join(", ")}`);
			addOpen.value = false;
			tab.value = add.level;
			await load();
			toast.add({
				title: `Akun ${list[0]!.nama} dibuat`,
				description: `Username: ${list[0]!.username}${add.pwMode === "bawaan" && add.level !== "admin" ? ` · Password bawaan: ${DEFAULT_PASS[add.level]}` : ""}`,
				color: "success"
			});
		}
		catch (e) {
			toast.add({ title: "Gagal menambah pengguna", description: pesanError(e), color: "error" });
		}
		finally {
			saving.value = false;
		}
	}

	// ===== Aksi per baris =====
	function randomPassword() {
		const chars = "abcdefghjkmnpqrstuvwxyz23456789";
		return Array.from(crypto.getRandomValues(new Uint32Array(8)), n => chars[n % chars.length]).join("");
	}

	async function resetAcak(u: UserRow) {
		if (!await swal.confirm("Buat password acak?", `Password ${u.nama} diganti password acak dari sistem.`, "Buat"))
			return;
		try {
			const pass = randomPassword();
			await invoke("user_set_password", { userId: u.id, password: pass });
			await swal.success("Password diganti", `Username: ${u.username}\nPassword baru: ${pass}\n\nCatat dan berikan ke pengguna.`);
		}
		catch (e) {
			toast.add({ title: "Gagal mengganti password", description: pesanError(e), color: "error" });
		}
	}

	async function toggleAktif(u: UserRow) {
		try {
			await db.execute("UPDATE users SET aktif = ?, online = 0 WHERE id = ?", [u.aktif ? 0 : 1, u.id]);
			await load();
		}
		catch (e) {
			toast.add({ title: "Gagal mengubah status", description: pesanError(e), color: "error" });
		}
	}

	async function resetLogin(u: UserRow) {
		try {
			await db.execute("UPDATE users SET online = 0 WHERE id = ?", [u.id]);
			await load();
		}
		catch (e) {
			toast.add({ title: "Gagal reset login", description: pesanError(e), color: "error" });
		}
	}

	async function hapus(u: UserRow) {
		if (u.level === "admin" && users.value.filter(x => x.level === "admin").length <= 1) {
			toast.add({ title: "Tidak bisa dihapus", description: "Harus ada minimal satu administrator.", color: "warning" });
			return;
		}
		const catatan = u.level === "admin"
			? ""
			: "\nNilai dan data rapornya tidak ikut terhapus. Akunnya bisa dibuat lagi lewat Generate Semua Pengguna.";
		if (!await swal.confirm("Hapus akun?", `Akun ${u.username} (${u.nama}) akan dihapus.${catatan}`, "Hapus"))
			return;
		try {
			await db.execute("DELETE FROM users WHERE id = ?", [u.id]);
			await load();
		}
		catch (e) {
			toast.add({ title: "Gagal menghapus", description: pesanError(e), color: "error" });
		}
	}

	function actions(u: UserRow): DropdownMenuItem[] {
		const self = u.id === me.value?.id;
		return [
			{ label: "Atur Password", icon: "lucide:key-round", onSelect: () => openPassword([u]) },
			{ label: "Password Acak", icon: "lucide:shuffle", onSelect: () => resetAcak(u) },
			{ label: u.aktif ? "Nonaktifkan" : "Aktifkan", icon: u.aktif ? "lucide:user-x" : "lucide:user-check", disabled: self, onSelect: () => toggleAktif(u) },
			...(u.online && !self ? [{ label: "Reset Login", icon: "lucide:log-out", onSelect: () => resetLogin(u) }] : []),
			{ label: "Hapus Pengguna", icon: "lucide:trash-2", color: "error" as const, disabled: self, onSelect: () => hapus(u) }
		];
	}

	const columns: TableColumn<UserRow>[] = [
		{ id: "pilih", size: 40 },
		{ id: "no", header: "No", cell: ({ row }) => row.index + 1, size: 48 },
		{ accessorKey: "username", header: "Username", meta: { class: { td: "font-mono" } } },
		{ accessorKey: "nama", header: "Nama Pengguna", meta: { class: { td: "font-medium" } } },
		{ accessorKey: "email", header: "e-mail", cell: ({ row }) => atauStrip(row.original.email) },
		{ accessorKey: "aktif", header: "Status" },
		{ accessorKey: "last_login", header: "Login Terakhir", cell: ({ row }) => (row.original.last_login ? waktuIndo(row.original.last_login) : "-") },
		{ accessorKey: "online", header: "Online" },
		{ id: "opsi", header: "" }
	];
</script>

<template>
	<ErPage id="pengguna" title="Pengguna">
		<template #right>
			<UButton
				to="/e-rapor/kartu-akun"
				icon="lucide:id-card"
				variant="ghost"
				color="neutral">
				Cetak Kartu Akun
			</UButton>
			<UButton icon="lucide:user-plus" variant="soft" @click="openAdd">
				Tambah Pengguna
			</UButton>
			<UButton icon="lucide:users" :loading="generating" @click="generateAll">
				Generate Semua Pengguna
			</UButton>
		</template>

		<div class="flex flex-col gap-3">
			<div class="flex flex-wrap items-center justify-between gap-2">
				<UTabs
					v-model="tab"
					:items="tabItems"
					:content="false"
					size="sm" />
				<UInput
					v-model="search"
					icon="lucide:search"
					placeholder="Cari username / nama"
					class="w-64" />
			</div>

			<div v-if="dipilih.size" class="flex flex-wrap items-center gap-2 rounded-md border border-primary/30 bg-primary/5 px-3 py-2">
				<span class="text-sm font-medium">{{ dipilih.size }} dipilih</span>
				<UButton
					size="sm"
					icon="lucide:key-round"
					variant="solid"
					@click="openPassword(terpilih)">
					Atur Password
				</UButton>
				<UButton
					size="sm"
					icon="lucide:user-check"
					color="neutral"
					variant="outline"
					@click="setAktifBanyak(true)">
					Aktifkan
				</UButton>
				<UButton
					size="sm"
					icon="lucide:user-x"
					color="neutral"
					variant="outline"
					@click="setAktifBanyak(false)">
					Nonaktifkan
				</UButton>
				<UButton
					size="sm"
					color="neutral"
					variant="ghost"
					class="ms-auto"
					@click="dipilih = new Set()">
					Batal pilih
				</UButton>
			</div>

			<UTable
				:data="filtered"
				:columns="columns"
				:loading="loading"
				empty="Belum ada pengguna"
				class="rounded border border-default">
				<template #pilih-header>
					<UCheckbox
						:model-value="sebagianDipilih ? 'indeterminate' : semuaDipilih"
						aria-label="Pilih semua"
						:disabled="!filtered.length"
						@update:model-value="v => pilihSemua(!!v)"
					/>
				</template>
				<template #pilih-cell="{ row }">
					<UCheckbox
						:model-value="dipilih.has(row.original.id)"
						:aria-label="`Pilih ${row.original.nama}`"
						@update:model-value="v => pilih(row.original.id, !!v)"
					/>
				</template>
				<template #aktif-cell="{ row }">
					<UBadge :color="row.original.aktif ? 'success' : 'neutral'" variant="subtle">
						{{ row.original.aktif ? "Aktif" : "Nonaktif" }}
					</UBadge>
				</template>
				<template #online-cell="{ row }">
					<UBadge :color="row.original.online ? 'success' : 'neutral'" variant="outline">
						{{ row.original.online ? "Online" : "Offline" }}
					</UBadge>
				</template>
				<template #opsi-cell="{ row }">
					<UDropdownMenu :items="actions(row.original)">
						<UButton
							size="xs"
							color="neutral"
							variant="outline"
							trailing-icon="lucide:chevron-down">
							Aksi
						</UButton>
					</UDropdownMenu>
				</template>
			</UTable>
			<p class="text-xs text-muted">
				{{ filtered.length }} pengguna · centang beberapa pengguna untuk mengganti password sekaligus
			</p>
		</div>
	</ErPage>

	<!-- Tambah pengguna -->
	<UModal v-model:open="addOpen" title="Tambah Pengguna" :dismissible="!saving">
		<template #body>
			<form id="form-tambah" class="flex flex-col gap-3" @submit.prevent="saveAdd">
				<UFormField label="Level Pengguna">
					<USelect
						v-model="add.level"
						:items="tabs"
						class="w-full"
						:disabled="saving" />
				</UFormField>

				<template v-if="add.level === 'admin'">
					<UFormField label="Username" :error="tampilError('username')">
						<UInput
							v-model="add.username"
							class="w-full font-mono"
							autocomplete="off"
							:disabled="saving" />
					</UFormField>
					<UFormField label="Nama Lengkap" :error="tampilError('nama')">
						<UInput v-model="add.nama" class="w-full" :disabled="saving" />
					</UFormField>
					<UFormField label="Email" hint="Opsional">
						<UInput
							v-model="add.email"
							type="email"
							class="w-full"
							:disabled="saving" />
					</UFormField>
				</template>
				<template v-else>
					<UFormField
						label="Pengguna"
						:error="tampilError('refId')"
						:help="refOptions.length ? undefined : `Semua ${add.level} sudah punya akun`"
					>
						<USelectMenu
							v-model="add.refId"
							:items="refOptions"
							value-key="value"
							placeholder="Pilih nama"
							class="w-full"
							:disabled="saving"
						/>
					</UFormField>
					<UFormField label="Password">
						<URadioGroup
							v-model="add.pwMode"
							orientation="horizontal"
							:disabled="saving"
							:items="[{ label: `Bawaan (${DEFAULT_PASS[add.level as 'guru' | 'siswa']})`, value: 'bawaan' }, { label: 'Isi manual', value: 'manual' }]"
						/>
					</UFormField>
				</template>

				<template v-if="add.pwMode === 'manual'">
					<div class="grid gap-3 sm:grid-cols-2">
						<UFormField label="Password" :error="tampilError('password')">
							<UInput
								v-model="add.password"
								:type="add.tampil ? 'text' : 'password'"
								autocomplete="new-password"
								class="w-full"
								:disabled="saving">
								<template #trailing>
									<UButton
										color="neutral"
										variant="link"
										size="sm"
										:icon="add.tampil ? 'lucide:eye-off' : 'lucide:eye'"
										:aria-label="add.tampil ? 'Sembunyikan password' : 'Tampilkan password'"
										@click="add.tampil = !add.tampil"
									/>
								</template>
							</UInput>
						</UFormField>
						<UFormField label="Ulangi Password" :error="tampilError('ulang')">
							<UInput
								v-model="add.ulang"
								:type="add.tampil ? 'text' : 'password'"
								autocomplete="new-password"
								class="w-full"
								:disabled="saving" />
						</UFormField>
					</div>
				</template>
			</form>
		</template>
		<template #footer>
			<div class="flex w-full justify-end gap-2">
				<UButton
					color="neutral"
					variant="outline"
					:disabled="saving"
					@click="addOpen = false">
					Batal
				</UButton>
				<UButton
					type="submit"
					form="form-tambah"
					variant="solid"
					icon="lucide:save"
					:loading="saving">
					Simpan
				</UButton>
			</div>
		</template>
	</UModal>

	<!-- Atur password satu / banyak pengguna -->
	<UModal
		v-model:open="pwOpen"
		title="Atur Password"
		:description="pwTarget.length === 1 ? `${pwTarget[0]!.nama} (${pwTarget[0]!.username})` : `${pwTarget.length} pengguna dipilih`"
		:dismissible="!pwSaving"
	>
		<template #body>
			<form id="form-password" class="flex flex-col gap-3" @submit.prevent="simpanPassword">
				<UFormField v-if="bisaBawaan" label="Password baru">
					<URadioGroup
						v-model="pw.mode"
						orientation="horizontal"
						:disabled="pwSaving"
						:items="[{ label: 'Isi manual', value: 'manual' }, { label: 'Kembalikan ke bawaan', value: 'bawaan' }]"
					/>
				</UFormField>

				<template v-if="pw.mode === 'manual'">
					<UFormField label="Password" :error="pw.dicoba ? pwError.password : undefined" help="Dipakai untuk semua pengguna yang dipilih.">
						<UInput
							v-model="pw.password"
							:type="pw.tampil ? 'text' : 'password'"
							autocomplete="new-password"
							class="w-full"
							autofocus
							:disabled="pwSaving">
							<template #trailing>
								<UButton
									color="neutral"
									variant="link"
									size="sm"
									:icon="pw.tampil ? 'lucide:eye-off' : 'lucide:eye'"
									:aria-label="pw.tampil ? 'Sembunyikan password' : 'Tampilkan password'"
									@click="pw.tampil = !pw.tampil"
								/>
							</template>
						</UInput>
					</UFormField>
					<UFormField label="Ulangi Password" :error="pw.dicoba ? pwError.ulang : undefined">
						<UInput
							v-model="pw.ulang"
							:type="pw.tampil ? 'text' : 'password'"
							autocomplete="new-password"
							class="w-full"
							:disabled="pwSaving" />
					</UFormField>
				</template>
				<UAlert
					v-else
					color="info"
					variant="subtle"
					icon="lucide:info"
					:description="`Guru: ${DEFAULT_PASS.guru} · Siswa: ${DEFAULT_PASS.siswa}`"
				/>

				<UAlert
					v-if="pwTarget.some(u => u.id === me?.id)"
					color="warning"
					variant="subtle"
					icon="lucide:triangle-alert"
					description="Termasuk akun Anda sendiri. Gunakan password baru ini saat login berikutnya."
				/>

				<div v-if="pwTarget.length > 1" class="max-h-40 overflow-y-auto rounded border border-default px-3 py-2 text-xs text-muted">
					{{ pwTarget.map(u => u.nama).join(", ") }}
				</div>
			</form>
		</template>
		<template #footer>
			<div class="flex w-full justify-end gap-2">
				<UButton
					color="neutral"
					variant="outline"
					:disabled="pwSaving"
					@click="pwOpen = false">
					Batal
				</UButton>
				<UButton
					type="submit"
					form="form-password"
					variant="solid"
					icon="lucide:save"
					:loading="pwSaving">
					Simpan Password
				</UButton>
			</div>
		</template>
	</UModal>
</template>
