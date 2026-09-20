<script lang="ts" setup>
	import { invoke } from "@tauri-apps/api/core";
	import { moveArrayElement, useSortable } from "@vueuse/integrations/useSortable";

	interface SubjectRecord {
		id: number
		name: string
		code: string | null
	}

	const errorMessage = ref("");
	const subjects = ref<SubjectRecord[]>([]);
	const editingId = ref<number | null>(null);
	const form = reactive({ name: "", code: "" });
	const saving = ref(false);
	const modalOpen = ref(false);
	const listEl = useTemplateRef<HTMLElement>("listEl");

	const load = async () => {
		subjects.value = await invoke<SubjectRecord[]>("list_subjects");
	};

	const openCreate = () => {
		editingId.value = null;
		form.name = "";
		form.code = "";
		errorMessage.value = "";
		modalOpen.value = true;
	};

	const openEdit = (s: SubjectRecord) => {
		editingId.value = s.id;
		form.name = s.name;
		form.code = s.code ?? "";
		errorMessage.value = "";
		modalOpen.value = true;
	};

	const submit = async () => {
		errorMessage.value = "";
		if (!form.name.trim()) {
			errorMessage.value = "Nama mata pelajaran wajib diisi.";
			return;
		}
		saving.value = true;
		try {
			const code = form.code.trim() || null;
			if (editingId.value) {
				await invoke("update_subject", { id: editingId.value, name: form.name, code });
			} else {
				await invoke("create_subject", { name: form.name, code });
			}
			modalOpen.value = false;
			await load();
		} catch (e) {
			errorMessage.value = e instanceof Error ? e.message : String(e);
		} finally {
			saving.value = false;
		}
	};

	const remove = async (s: SubjectRecord) => {
		if (!confirm(`Hapus mata pelajaran "${s.name}"? Semua soal & ujian untuk mata pelajaran ini juga akan terhapus.`)) return;
		await invoke("delete_subject", { id: s.id });
		await load();
	};

	// Urutan di sini nentuin urutan mata pelajaran di semua menu lain yang nampilin daftar
	// pelajaran (Bank Soal, Kelola Ujian, dst) — bukan cuma tampilan di halaman ini saja.
	// forceFallback: true karena WebView Tauri sering nggak negosiasi native HTML5 drag-and-drop
	// dengan benar (muncul kursor 🚫 permanen) — ini bikin SortableJS pakai simulasi drag sendiri
	// (mouse/touch based) alih-alih native browser DnD.
	//
	// onUpdate custom di sini WAJIB manggil moveArrayElement sendiri — kalau nggak, useSortable
	// nggak akan nyusun ulang array `subjects` sama sekali (default onUpdate-nya ke-override
	// karena object spread), jadi drag cuma keliatan jalan di DOM doang tapi datanya gak berubah.
	const orderDirty = ref(false);
	const savingOrder = ref(false);

	useSortable(listEl, subjects, {
		handle: "[data-drag-handle]",
		animation: 150,
		forceFallback: true,
		onUpdate: (e) => {
			moveArrayElement(subjects, e.oldIndex!, e.newIndex!, e);
			orderDirty.value = true;
		}
	});

	const saveOrder = async () => {
		savingOrder.value = true;
		try {
			await invoke("reorder_subjects", { orderedIds: subjects.value.map((s) => s.id) });
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
				Tambah Mata Pelajaran
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
			<div v-for="s in subjects" :key="s.id" class="flex items-center gap-3 p-3 bg-default">
				<span
					data-drag-handle
					draggable="false"
					class="cursor-grab text-muted shrink-0 touch-none select-none"
					style="-webkit-user-drag: none;">
					<UIcon name="lucide:grip-vertical" draggable="false" class="size-5 block pointer-events-none" />
				</span>
				<UBadge v-if="s.code" variant="subtle" color="neutral">
					{{ s.code }}
				</UBadge>
				<span class="flex-1 truncate">{{ s.name }}</span>
				<UButton
					size="xs"
					variant="soft"
					icon="lucide:pencil"
					@click="openEdit(s)">
					Edit
				</UButton>
				<UButton
					size="xs"
					variant="soft"
					color="error"
					icon="lucide:trash-2"
					@click="remove(s)">
					Hapus
				</UButton>
			</div>
		</div>
		<div v-if="!subjects.length" class="text-center py-10 text-muted">
			Belum ada mata pelajaran.
		</div>

		<UModal v-model:open="modalOpen" :title="editingId ? 'Edit Mata Pelajaran' : 'Tambah Mata Pelajaran'">
			<template #body>
				<form class="space-y-4" @submit.prevent="submit">
					<UFormField label="Kode" description="Singkatan, biar tampilan di menu lain tidak panjang">
						<UInput v-model="form.code" placeholder="Contoh: MTK" />
					</UFormField>
					<UFormField label="Mata Pelajaran" description="Nama lengkap">
						<UInput v-model="form.name" placeholder="Contoh: Matematika" />
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
