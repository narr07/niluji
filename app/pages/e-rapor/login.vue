<script setup lang="ts">
	import { invoke, isTauri } from "@tauri-apps/api/core";

	definePageMeta({ layout: "auth" });

	const { login, session, logout } = useAuth();
	const db = useDb();
	const toast = useToast();

	const form = reactive({
		username: "",
		password: "",
		sekolahId: undefined as string | undefined,
		semesterId: undefined as string | undefined
	});
	const showPass = ref(false);
	const loading = ref(false);
	const showReset = ref(false);

	// Lupa password (seperti e-Rapor): username + nama lengkap + email harus sama dengan data akun.
	const reset = reactive({ username: "", nama: "", email: "", password: "", ulang: "", proses: false, galat: "" });
	async function kirimReset() {
		reset.galat = "";
		if (!reset.username || !reset.nama || !reset.email || !reset.password) {
			reset.galat = "Semua kolom wajib diisi.";
			return;
		}
		if (reset.password !== reset.ulang) {
			reset.galat = "Ulangi password baru tidak sama.";
			return;
		}
		reset.proses = true;
		try {
			await invoke("auth_reset_password", { username: reset.username, nama: reset.nama, email: reset.email, password: reset.password });
			showReset.value = false;
			form.username = reset.username;
			form.password = "";
			Object.assign(reset, { nama: "", email: "", password: "", ulang: "" });
			toast.add({ title: "Password berhasil diganti", description: "Silakan masuk dengan password baru.", color: "success" });
		}
		catch (e) {
			reset.galat = pesanError(e);
		}
		finally {
			reset.proses = false;
		}
	}
	const sekolahList = ref<{ label: string, value: string }[]>([]);
	const semesterList = ref<{ label: string, value: string }[]>([]);
	const inTauri = isTauri();
	const { load: loadMode, isGuruMode, adminLaptop } = useAppMode();

	onMounted(async () => {
		if (session.value)
			return navigateTo("/e-rapor");
		if (!inTauri)
			return;
		await loadMode();
		const [sekolah, semester] = await Promise.all([
			db.query<{ sekolah_id: string, nama: string }>("SELECT sekolah_id, nama FROM sekolah ORDER BY nama"),
			db.query<{ semester_id: string }>("SELECT semester_id FROM semester ORDER BY semester_id DESC")
		]);
		sekolahList.value = sekolah.map(s => ({ label: s.nama, value: s.sekolah_id }));
		semesterList.value = semester.map(s => ({ label: semesterLabel(s.semester_id), value: s.semester_id }));
		form.sekolahId = sekolahList.value[0]?.value;
		// Bawaan: semester berjalan kalau sudah ditarik, kalau tidak semester terbaru.
		const cur = currentSemesterId();
		form.semesterId = semesterList.value.find(s => s.value === cur)?.value ?? semesterList.value[0]?.value;
	});

	async function submit() {
		if (!form.username || !form.password) {
			toast.add({ title: "Username dan password wajib diisi", color: "warning" });
			return;
		}
		loading.value = true;
		try {
			const u = await login({ ...form, sekolahId: form.sekolahId ?? null, semesterId: form.semesterId ?? null });
			// Laptop guru hanya untuk guru; data admin tidak ada di sini.
			if (isGuruMode.value && u.level !== "guru") {
				await logout();
				toast.add({ title: "Ini laptop guru", description: "Akun admin/siswa hanya bisa dipakai di laptop admin.", color: "warning" });
				return;
			}
			// Guru & siswa butuh sekolah + semester (data mereka per semester). Admin boleh masuk
			// tanpa keduanya, karena justru admin yang menarik data pertama kali.
			if (u.level !== "admin" && (!form.sekolahId || !form.semesterId)) {
				await logout();
				toast.add({ title: "Belum ada data sekolah/semester", description: "Minta administrator mengambil data Dapodik dulu.", color: "warning" });
				return;
			}
			toast.add({ title: "Sukses", description: `Selamat datang, ${u.nama}`, color: "success", icon: "lucide:circle-check" });
			await navigateTo("/e-rapor");
		}
		catch (e) {
			toast.add({ title: "Gagal masuk", description: pesanError(e), color: "error" });
		}
		finally {
			loading.value = false;
		}
	}
</script>

<template>
	<UCard class="w-full max-w-sm" :ui="{ body: 'sm:p-6' }">
		<form class="flex flex-col gap-4" @submit.prevent="submit">
			<div class="flex flex-col items-center gap-1 text-center">
				<UIcon name="lucide:file-badge" class="size-10 text-primary" />
				<h1 class="text-xl font-bold">
					e-Rapor SD
				</h1>
				<p class="text-sm text-muted">
					Masuk untuk memulai aplikasi
				</p>
			</div>

			<UAlert
				v-if="!inTauri"
				color="warning"
				variant="subtle"
				description="Aplikasi harus dibuka lewat jendela desktop (bun run tauri:dev), bukan browser."
			/>

			<UFormField label="Username">
				<UInput
					v-model="form.username"
					icon="lucide:user"
					placeholder="NIP / NISN / username"
					autofocus />
			</UFormField>
			<UFormField label="Password">
				<UInput
					v-model="form.password"
					icon="lucide:lock"
					:type="showPass ? 'text' : 'password'"
					placeholder="Kata sandi">
					<template #trailing>
						<UButton
							color="neutral"
							variant="link"
							size="sm"
							:icon="showPass ? 'lucide:eye-off' : 'lucide:eye'"
							@click="showPass = !showPass"
						/>
					</template>
				</UInput>
			</UFormField>
			<!-- Satu sekolah = tidak perlu dipilih (hampir selalu begitu di aplikasi desktop). -->
			<UFormField v-if="sekolahList.length !== 1" label="Sekolah">
				<USelect
					v-model="form.sekolahId"
					:items="sekolahList"
					placeholder="Belum ada sekolah"
					class="w-full" />
			</UFormField>
			<UFormField label="Semester">
				<USelect
					v-model="form.semesterId"
					:items="semesterList"
					placeholder="Belum ada semester"
					class="w-full" />
			</UFormField>

			<UButton
				type="submit"
				block
				size="lg"
				variant="solid"
				icon="lucide:log-in"
				:loading="loading"
			>
				Masuk
			</UButton>

			<div class="flex items-center justify-between text-xs">
				<button type="button" class="cursor-pointer text-muted hover:text-default" @click="showReset = true">
					Lupa password?
				</button>
				<NuxtLink v-if="!adminLaptop" to="/e-rapor/sinkron" class="font-medium text-primary hover:underline">
					{{ isGuruMode ? "Tarik data dari admin" : "Laptop guru? Tarik data" }}
				</NuxtLink>
			</div>

			<div class="text-center pt-2">
				<NuxtLink to="/" class="text-xs text-muted hover:text-primary">
					← Kembali ke Ujian CBT
				</NuxtLink>
			</div>
		</form>

		<UModal v-model:open="showReset" title="Lupa Password" description="Isi persis sama dengan data akun Anda (email diisi di menu Profil).">
			<template #body>
				<form id="form-reset" class="space-y-3" @submit.prevent="kirimReset">
					<UFormField label="Username">
						<UInput v-model="reset.username" icon="lucide:user" class="w-full" />
					</UFormField>
					<UFormField label="Nama lengkap">
						<UInput v-model="reset.nama" class="w-full" />
					</UFormField>
					<UFormField label="Email">
						<UInput
							v-model="reset.email"
							type="email"
							icon="lucide:mail"
							class="w-full" />
					</UFormField>
					<div class="grid grid-cols-2 gap-3">
						<UFormField label="Password baru">
							<UInput v-model="reset.password" type="password" class="w-full" />
						</UFormField>
						<UFormField label="Ulangi">
							<UInput v-model="reset.ulang" type="password" class="w-full" />
						</UFormField>
					</div>
					<UAlert
						v-if="reset.galat"
						color="error"
						variant="subtle"
						icon="lucide:circle-x"
						:description="reset.galat" />
					<details class="text-xs text-muted">
						<summary class="cursor-pointer">
							Semua admin lupa password dan tidak punya email?
						</summary>
						<p class="mt-1">
							Di laptop admin, tutup aplikasi, buat file kosong bernama <code>RESET-ADMIN.txt</code> di folder data aplikasi
							(<code>%APPDATA%\com.narr07.nuxt-erapor</code>), lalu buka aplikasi lagi. Akun <code>administrator</code> kembali ke
							password bawaan <code>administrator</code>. Segera ganti setelah masuk.
						</p>
					</details>
				</form>
			</template>
			<template #footer>
				<div class="flex w-full justify-end gap-2">
					<UButton variant="ghost" color="neutral" @click="showReset = false">
						Batal
					</UButton>
					<UButton
						type="submit"
						form="form-reset"
						icon="lucide:key-round"
						:loading="reset.proses">
						Ganti Password
					</UButton>
				</div>
			</template>
		</UModal>
	</UCard>
</template>
