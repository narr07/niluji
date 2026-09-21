<script lang="ts" setup>
	import { invoke } from "@tauri-apps/api/core";

	const toast = useToast();
	const errorMessage = ref("");
	const saving = ref(false);
	const form = reactive({ name: "", npsn: "", address: "", principal: "", exportPin: "" });

	const load = async () => {
		Object.assign(form, await invoke("get_school"));
	};

	const save = async () => {
		errorMessage.value = "";
		saving.value = true;
		try {
			await invoke("update_school", { info: { ...form } });
			toast.add({ title: "Data sekolah tersimpan", icon: "lucide:check", color: "success" });
		} catch (e) {
			errorMessage.value = e instanceof Error ? e.message : String(e);
		} finally {
			saving.value = false;
		}
	};

	onMounted(load);
</script>

<template>
	<UCard>
		<template #header>
			<span class="font-semibold">Data Sekolah</span>
		</template>

		<form class="grid gap-4 sm:grid-cols-2" @submit.prevent="save">
			<UFormField label="Nama Sekolah" class="sm:col-span-2">
				<UInput v-model="form.name" />
			</UFormField>
			<UFormField label="NPSN">
				<UInput v-model="form.npsn" />
			</UFormField>
			<UFormField label="Kepala Sekolah">
				<UInput v-model="form.principal" />
			</UFormField>
			<UFormField label="Alamat" class="sm:col-span-2">
				<UTextarea v-model="form.address" :rows="2" />
			</UFormField>
			<UFormField label="PIN Export Bank Soal" description="Diminta setiap kali ada yang mau export bank soal ke file, biar tidak sembarang orang bisa export.">
				<UInput v-model="form.exportPin" />
			</UFormField>
			<div class="sm:col-span-2">
				<UButton type="submit" :loading="saving">
					Simpan
				</UButton>
			</div>
		</form>

		<UAlert
			v-if="errorMessage"
			color="error"
			variant="subtle"
			class="mt-4"
			:title="errorMessage" />
	</UCard>
</template>
