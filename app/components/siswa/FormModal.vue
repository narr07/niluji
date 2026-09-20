<script lang="ts" setup>
	import { invoke } from "@tauri-apps/api/core";

	interface StudentRecord {
		id: number
		nisn: string
		name: string
		class: string | null
	}

	const props = defineProps<{
		open: boolean
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

	const form = reactive({ nisn: "", name: "" });
	const formError = ref("");
	const saving = ref(false);

	watch(
		() => [props.open, props.student] as const,
		([isOpen, student]) => {
			if (!isOpen) return;
			form.nisn = student?.nisn ?? "";
			form.name = student?.name ?? "";
			formError.value = "";
		},
		{ immediate: true }
	);

	const submitForm = async () => {
		formError.value = "";
		if (!form.nisn.trim() || !form.name.trim()) {
			formError.value = "NISN dan nama wajib diisi.";
			return;
		}

		saving.value = true;
		try {
			if (props.student) {
				await invoke("update_student", { id: props.student.id, nisn: form.nisn, name: form.name, class: props.kelas });
			} else {
				await invoke("create_student", { nisn: form.nisn, name: form.name, class: props.kelas });
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
					<UInput v-model="form.nisn" />
				</UFormField>
				<UFormField label="Nama">
					<UInput v-model="form.name" />
				</UFormField>
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
