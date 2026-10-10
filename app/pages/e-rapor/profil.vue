<script setup lang="ts">
	import { invoke } from "@tauri-apps/api/core";

	const { user, session, headerSemester, waliRombel } = useAuth();
	const swal = useSwal();
	const toast = useToast();

	// Email dipakai untuk "Lupa password?" di halaman masuk.
	const email = ref(user.value?.email ?? "");
	const simpanEmail = ref(false);
	async function ubahEmail() {
		simpanEmail.value = true;
		try {
			await useDb().execute("UPDATE users SET email = ? WHERE id = ?", [email.value.trim() || null, user.value?.id]);
			if (session.value)
				session.value = { ...session.value, user: { ...session.value.user, email: email.value.trim() || null } };
			toast.add({ title: "Email tersimpan", color: "success" });
		}
		catch (e) {
			toast.add({ title: "Gagal menyimpan email", description: pesanError(e), color: "error" });
		}
		finally {
			simpanEmail.value = false;
		}
	}

	const pass = reactive({ old: "", baru: "", ulang: "" });
	const saving = ref(false);

	async function ubahPassword() {
		if (pass.baru !== pass.ulang) {
			swal.error("Gagal", "Ulangi password baru tidak sama");
			return;
		}
		saving.value = true;
		try {
			await invoke("auth_change_password", { userId: user.value?.id, oldPassword: pass.old, newPassword: pass.baru });
			Object.assign(pass, { old: "", baru: "", ulang: "" });
			swal.success("Berhasil", "Password berhasil diubah");
		}
		catch (e) {
			swal.error("Gagal", pesanError(e));
		}
		finally {
			saving.value = false;
		}
	}

	const info = computed(() => [
		["Nama", user.value?.nama],
		["Username", user.value?.username],
		["Level", user.value ? LEVEL_LABEL[user.value.level] : ""],
		["Wali Kelas", waliRombel.value.map(r => r.nama).join(", ") || "-"],
		["Sekolah", session.value?.sekolahNama ?? "-"],
		["Semester", headerSemester.value],
		["Login Terakhir", user.value?.last_login ?? "-"]
	]);
</script>

<template>
	<ErPage id="profil" title="Profil">
		<div class="grid gap-4 lg:grid-cols-2">
			<PanelCard title="Profil Pengguna">
				<div class="flex items-center gap-4">
					<UAvatar :alt="user?.nama" size="3xl" />
					<dl class="grid grid-cols-[120px_1fr] gap-x-3 gap-y-1.5 text-sm">
						<template v-for="[k, v] in info" :key="k">
							<dt class="text-muted">
								{{ k }}
							</dt>
							<dd>{{ v }}</dd>
						</template>
					</dl>
				</div>
			</PanelCard>
	
			<PanelCard title="Email" description="Dipakai untuk reset sendiri lewat &quot;Lupa password?&quot; di halaman masuk.">
				<form class="flex gap-2" @submit.prevent="ubahEmail">
					<UInput
						v-model="email"
						type="email"
						icon="lucide:mail"
						placeholder="nama@sekolah.sch.id"
						class="flex-1"
						aria-label="Email" />
					<UButton type="submit" icon="lucide:save" :loading="simpanEmail">
						Simpan
					</UButton>
				</form>
			</PanelCard>

			<PanelCard title="Ubah Password">
				<form class="flex flex-col gap-3" @submit.prevent="ubahPassword">
					<UFormField label="Password Lama">
						<UInput v-model="pass.old" type="password" />
					</UFormField>
					<UFormField label="Password Baru">
						<UInput v-model="pass.baru" type="password" />
					</UFormField>
					<UFormField label="Ulangi Password Baru">
						<UInput v-model="pass.ulang" type="password" />
					</UFormField>
					<UButton
						type="submit"
						variant="solid"
						icon="lucide:save"
						:loading="saving"
						class="self-end"
					>
						Simpan
					</UButton>
				</form>
			</PanelCard>
		</div>
	</ErPage>
</template>
