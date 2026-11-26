<script lang="ts" setup>
	import { invoke } from "@tauri-apps/api/core";

	interface StudentRecord {
		id: number
		nisn: string
		name: string
		class: string | null
		rombel?: string | null
	}

	const props = defineProps<{
		open: boolean
		// Kelas halaman yang sedang dibuka; kosong di halaman "Kelas tidak terdaftar".
		kelas: string
		student: StudentRecord | null
	}>();

	const emit = defineEmits<{
		"update:open": [value: boolean]
		saved: []
	}>();

	const openModel = computed({
		get: () => props.open,
		set: (value) => emit("update:open", value)
	});

	const form = reactive({ nisn: "", name: "", class: "", rombel: "" });
	const classes = ref<string[]>([]);
	const formError = ref("");
	const saving = ref(false);

	const classItems = computed(() => classes.value.map((c) => ({ label: `Kelas ${c}`, value: c })));

	watch(
		() => [props.open, props.student] as const,
		async ([isOpen, student]) => {
			if (!isOpen) return;
			form.nisn = student?.nisn ?? "";
			form.name = student?.name ?? "";
			form.rombel = student?.rombel ?? "";
			formError.value = "";
			classes.value = await invoke<string[]>("list_classes");
			const current = student?.class ?? props.kelas;
			form.class = current && classes.value.includes(current) ? current : "";
		},
		{ immediate: true }
	);

	const submitForm = async () => {
		formError.value = "";
		if (!form.nisn.trim() || !form.name.trim()) {
			formError.value = "NISN dan nama wajib diisi.";
			return;
		}
		if (!form.class) {
			formError.value = "Pilih kelas siswa.";
			return;
		}

		saving.value = true;
		try {
			const payload = { nisn: form.nisn, name: form.name, class: form.class, rombel: form.rombel.trim() || null };
			if (props.student) {
				await invoke("update_student", { id: props.student.id, ...payload });
			} else {
				await invoke("create_student", payload);
			}
			openModel.value = false;
			emit("saved");
		} catch (e) {
			formError.value = e instanceof Error ? e.message : String(e);
		} finally {
			saving.value = false;
		}
	};
</script>

<template>
	<UModal v-model:open="openModel" :title="student ? 'Edit Siswa' : 'Tambah Siswa'">
		<template #body>
			<form class="space-y-4" @submit.prevent="submitForm">
				<UFormField label="NISN">
					<UInput v-model="form.nisn" class="w-full" />
				</UFormField>
				<UFormField label="Nama">
					<UInput v-model="form.name" class="w-full" />
				</UFormField>
				<div class="grid gap-4 sm:grid-cols-2">
					<UFormField label="Kelas" description="Dipakai untuk ujian dan bank soal.">
						<USelect
							v-model="form.class"
							:items="classItems"
							placeholder="Pilih kelas"
							class="w-full" />
					</UFormField>
					<UFormField label="Rombel" hint="Opsional" description="Contoh: IVA atau 4B.">
						<UInput v-model="form.rombel" placeholder="IVA" class="w-full" />
					</UFormField>
				</div>
				<UAlert
					v-if="formError"
					color="error"
					variant="subtle"
					:title="formError" />
				<UButton type="submit" block :loading="saving">
					Simpan
				</UButton>
			</form>
		</template>
	</UModal>
</template>
