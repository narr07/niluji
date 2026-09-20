<script lang="ts" setup>
	import { invoke } from "@tauri-apps/api/core";
	import { h, resolveComponent } from "vue";

	interface ClassRecord {
		id: number
		title: string
		description: string | null
	}

	const errorMessage = ref("");
	const classes = ref<ClassRecord[]>([]);
	const editingId = ref<number | null>(null);
	const form = reactive({ title: "", description: "" });
	const saving = ref(false);
	const modalOpen = ref(false);

	const load = async () => {
		classes.value = await invoke<ClassRecord[]>("list_classes_full");
	};

	const openCreate = () => {
		editingId.value = null;
		form.title = "";
		form.description = "";
		errorMessage.value = "";
		modalOpen.value = true;
	};

	const openEdit = (c: ClassRecord) => {
		editingId.value = c.id;
		form.title = c.title;
		form.description = c.description ?? "";
		errorMessage.value = "";
		modalOpen.value = true;
	};

	const submit = async () => {
		errorMessage.value = "";
		if (!form.title.trim()) {
			errorMessage.value = "Judul kelas wajib diisi.";
			return;
		}
		saving.value = true;
		try {
			const description = form.description.trim() || null;
			if (editingId.value) {
				await invoke("update_class", { id: editingId.value, title: form.title, description });
			} else {
				await invoke("create_class", { title: form.title, description });
			}
			modalOpen.value = false;
			await load();
		} catch (e) {
			errorMessage.value = e instanceof Error ? e.message : String(e);
		} finally {
			saving.value = false;
		}
	};

	const remove = async (c: ClassRecord) => {
		if (!confirm(`Hapus kelas "${c.title}"? Data siswa/soal/ujian yang sudah memakai kelas ini tidak ikut terhapus.`)) return;
		await invoke("delete_class", { id: c.id });
		await load();
	};

	const columns = [
		{ accessorKey: "title", header: "Judul" },
		{ accessorKey: "description", header: "Deskripsi", cell: ({ row }: { row: { original: ClassRecord } }) => row.original.description ?? "-" },
		{
			id: "actions",
			header: "Aksi",
			cell: ({ row }: { row: { original: ClassRecord } }) => {
				const UButton = resolveComponent("UButton");
				return h("div", { class: "flex gap-2" }, [
					h(UButton, { size: "xs", variant: "soft", icon: "lucide:pencil", onClick: () => openEdit(row.original) }, () => "Edit"),
					h(
						UButton,
						{ size: "xs", variant: "soft", color: "error", icon: "lucide:trash-2", onClick: () => remove(row.original) },
						() => "Hapus"
					)
				]);
			}
		}
	];

	onMounted(load);
</script>

<template>
	<div class="space-y-4">
		<UButton icon="lucide:plus" @click="openCreate">
			Tambah Kelas
		</UButton>

		<UTable :data="classes" :columns="columns">
			<template #empty>
				<div class="text-center py-10 text-muted">
					Belum ada data kelas.
				</div>
			</template>
		</UTable>

		<UModal v-model:open="modalOpen" :title="editingId ? 'Edit Kelas' : 'Tambah Kelas'">
			<template #body>
				<form class="space-y-4" @submit.prevent="submit">
					<UFormField label="Judul">
						<UInput v-model="form.title" placeholder="Contoh: 4" />
					</UFormField>
					<UFormField label="Deskripsi">
						<UInput v-model="form.description" placeholder="Contoh: Kelas 4" />
					</UFormField>

					<UAlert
						v-if="errorMessage"
						color="error"
						variant="subtle"
						:title="errorMessage" />

					<UButton type="submit" block :loading="saving">
						{{ editingId ? "Simpan Perubahan" : "Tambah" }}
					</UButton>
				</form>
			</template>
		</UModal>
	</div>
</template>
