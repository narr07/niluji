<script lang="ts" setup>
	import { invoke } from "@tauri-apps/api/core";
	import { moveArrayElement, useSortable } from "@vueuse/integrations/useSortable";

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
	const listEl = useTemplateRef<HTMLElement>("listEl");

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
			errorMessage.value = "Nama kelas wajib diisi.";
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
		const ok = await confirmDelete({
			title: `Hapus kelas "${c.title}"?`,
			description: "Data siswa/soal/ujian yang sudah memakai kelas ini tidak ikut terhapus."
		});
		if (!ok) return;
		await invoke("delete_class", { id: c.id });
		await load();
	};

	// Urutan di sini nentuin urutan kelas di semua menu lain yang nampilin daftar kelas (Bank
	// Soal, Kelola Ujian, Siswa, dst) — bukan cuma tampilan di halaman ini saja. Pola sama persis
	// dengan Pelajaran.vue: forceFallback: true karena WebView Tauri sering nggak negosiasi
	// native HTML5 drag-and-drop dengan benar, dan onUpdate custom WAJIB manggil
	// moveArrayElement sendiri supaya array `classes` ikut tersusun ulang.
	const orderDirty = ref(false);
	const savingOrder = ref(false);

	useSortable(listEl, classes, {
		handle: "[data-drag-handle]",
		animation: 150,
		forceFallback: true,
		onUpdate: (e) => {
			moveArrayElement(classes, e.oldIndex!, e.newIndex!, e);
			orderDirty.value = true;
		}
	});

	const saveOrder = async () => {
		savingOrder.value = true;
		try {
			await invoke("reorder_classes", { orderedIds: classes.value.map((c) => c.id) });
			orderDirty.value = false;
		} finally {
			savingOrder.value = false;
		}
	};

	onMounted(load);
</script>

<template>
	<div class="space-y-4">
		<div class="flex items-center gap-3">
			<UButton icon="lucide:plus" @click="openCreate">
				Tambah Kelas
			</UButton>

			<UButton
				v-if="orderDirty"
				icon="lucide:save"
				color="success"
				:loading="savingOrder"
				@click="saveOrder">
				Simpan Urutan
			</UButton>
			<span v-if="orderDirty" class="text-sm text-muted">
				Urutan berubah, belum disimpan
			</span>
		</div>

		<div ref="listEl" class="divide-y divide-default rounded-lg border border-default empty:hidden">
			<div v-for="c in classes" :key="c.id" class="flex items-center gap-3 p-3 bg-default">
				<span
					data-drag-handle
					draggable="false"
					class="cursor-grab text-muted shrink-0 touch-none select-none"
					style="-webkit-user-drag: none;">
					<UIcon name="lucide:grip-vertical" draggable="false" class="size-5 block pointer-events-none" />
				</span>
				<UBadge v-if="c.description" variant="subtle" color="neutral">
					{{ c.description }}
				</UBadge>
				<span class="flex-1 truncate">{{ c.title }}</span>
				<UButton
					size="xs"
					variant="soft"
					icon="lucide:pencil"
					@click="openEdit(c)">
					Edit
				</UButton>
				<UButton
					size="xs"
					variant="soft"
					color="error"
					icon="lucide:trash-2"
					@click="remove(c)">
					Hapus
				</UButton>
			</div>
		</div>
		<div v-if="!classes.length" class="text-center py-10 text-muted">
			Belum ada data kelas.
		</div>

		<UModal v-model:open="modalOpen" :title="editingId ? 'Edit Kelas' : 'Tambah Kelas'">
			<template #body>
				<form class="space-y-4" @submit.prevent="submit">
					<UFormField label="Kode Kelas" description="Singkatan/kode, opsional (mis. IVA)">
						<UInput v-model="form.description" placeholder="Contoh: IVA" />
					</UFormField>
					<UFormField label="Nama Kelas" description="Nama bebas, dipakai di seluruh menu lain">
						<UInput v-model="form.title" placeholder="Contoh: 4" />
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
