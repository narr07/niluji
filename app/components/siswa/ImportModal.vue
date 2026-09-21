<script lang="ts" setup>
	import { invoke } from "@tauri-apps/api/core";
	import { open as openDialog } from "@tauri-apps/plugin-dialog";

	const props = defineProps<{
		open: boolean
	}>();

	const emit = defineEmits<{
		"update:open": [value: boolean]
		imported: []
	}>();

	const openModel = computed({
		get: () => props.open,
		set: (value) => emit("update:open", value)
	});

	interface ImportRow {
		id: number
		nisn: string
		name: string
		class: string
	}

	const toast = useToast();

	const filePath = ref("");
	const fileName = ref("");
	const rows = ref<ImportRow[]>([]);
	const selectedIds = ref(new Set<number>());
	const parsing = ref(false);
	const importing = ref(false);
	const errorMessage = ref("");

	const isValid = (row: ImportRow) => Boolean(row.nisn.trim() && row.name.trim());
	const validCount = computed(() => rows.value.filter(isValid).length);
	const selectedCount = computed(() => selectedIds.value.size);

	watch(
		() => props.open,
		(isOpen) => {
			if (!isOpen) return;
			filePath.value = "";
			fileName.value = "";
			rows.value = [];
			selectedIds.value = new Set();
			errorMessage.value = "";
		}
	);

	const pickFile = async () => {
		const path = await openDialog({ multiple: false, filters: [{ name: "Siswa", extensions: ["csv", "xlsx", "xls"] }] });
		if (!path) return;
		filePath.value = path;
		fileName.value = path.split(/[/\\]/).pop() ?? path;

		errorMessage.value = "";
		parsing.value = true;
		rows.value = [];
		selectedIds.value = new Set();
		try {
			const parsed = await invoke<{ nisn: string, name: string, class?: string }[]>("parse_students_file", { path });
			rows.value = parsed.map((r, i) => ({ id: i + 1, nisn: r.nisn, name: r.name, class: r.class ?? "" }));
			selectedIds.value = new Set(rows.value.filter(isValid).map((r) => r.id));
		} catch (error) {
			errorMessage.value = error instanceof Error ? error.message : String(error);
		} finally {
			parsing.value = false;
		}
	};

	const isSelected = (row: ImportRow) => selectedIds.value.has(row.id);
	const toggleRow = (row: ImportRow, value: boolean | "indeterminate") => {
		if (!isValid(row) || value === "indeterminate") return;
		const next = new Set(selectedIds.value);
		if (value) next.add(row.id);
		else next.delete(row.id);
		selectedIds.value = next;
	};

	// ---------- Edit satu baris (buat benerin hasil baca file yang salah/kosong, atau isi
	// baris yang ditambah manual) — pola sama seperti Import dari Word di Bank Soal ----------

	const editRowOpen = ref(false);
	const editRowId = ref<number | null>(null);
	const editForm = reactive({ nisn: "", name: "", class: "" });

	const openEditRow = (row: ImportRow) => {
		editRowId.value = row.id;
		editForm.nisn = row.nisn;
		editForm.name = row.name;
		editForm.class = row.class;
		editRowOpen.value = true;
	};

	const saveEditRow = () => {
		const row = rows.value.find((r) => r.id === editRowId.value);
		if (!row) return;

		if (!editForm.nisn.trim() || !editForm.name.trim()) {
			toast.add({ title: "NISN dan Nama wajib diisi", color: "error", icon: "lucide:x" });
			return;
		}

		row.nisn = editForm.nisn.trim();
		row.name = editForm.name.trim();
		row.class = editForm.class.trim();

		selectedIds.value = new Set([...selectedIds.value, row.id]);
		editRowOpen.value = false;
	};

	// ---------- Tambah baris manual ----------

	const addManualRow = () => {
		const row: ImportRow = { id: rows.value.length ? Math.max(...rows.value.map((r) => r.id)) + 1 : 1, nisn: "", name: "", class: "" };
		rows.value = [...rows.value, row];
		openEditRow(row);
	};

	const doImport = async () => {
		errorMessage.value = "";
		const selected = rows.value.filter((r) => selectedIds.value.has(r.id) && isValid(r));
		if (!selected.length) return;

		importing.value = true;
		try {
			const summary = await invoke<{ studentsImported: number }>("import_students_rows", {
				rows: selected.map((r) => ({ nisn: r.nisn.trim(), name: r.name.trim(), class: r.class.trim() || undefined }))
			});
			toast.add({
				title: "Import selesai",
				description: `${summary.studentsImported} siswa berhasil diimpor/diperbarui.`,
				icon: "lucide:check",
				color: "success"
			});
			emit("imported");
			openModel.value = false;
		} catch (error) {
			errorMessage.value = error instanceof Error ? error.message : String(error);
		} finally {
			importing.value = false;
		}
	};
</script>

<template>
	<UModal v-model:open="openModel" title="Import Siswa" :ui="{ content: 'max-w-3xl' }">
		<template #body>
			<div class="space-y-4">
				<p class="text-sm text-muted">
					Pilih file CSV atau Excel berisi data siswa (kolom NISN, Nama, Kelas). Data akan ditampilkan dulu
					sebagai preview yang bisa diperiksa dan diedit sebelum benar-benar disimpan.
				</p>

				<UFormField label="File Siswa (.csv / .xlsx)">
					<UButton
						variant="soft"
						icon="lucide:file-spreadsheet"
						:loading="parsing"
						@click="pickFile">
						{{ fileName || "Pilih File" }}
					</UButton>
				</UFormField>

				<UAlert
					v-if="errorMessage"
					color="error"
					variant="subtle"
					title="Gagal memproses"
					:description="errorMessage" />

				<div v-if="rows.length" class="space-y-3">
					<div class="flex flex-wrap items-center justify-between gap-3">
						<div class="text-sm text-muted">
							{{ rows.length }} baris, {{ validCount }} valid, {{ rows.length - validCount }} perlu dicek manual
						</div>
						<div class="flex gap-2">
							<UButton icon="lucide:plus" variant="soft" @click="addManualRow">
								Tambah Baris Manual
							</UButton>
							<UButton
								icon="lucide:database"
								:loading="importing"
								:disabled="!selectedCount"
								@click="doImport">
								Import Siswa ({{ selectedCount }})
							</UButton>
						</div>
					</div>

					<div class="overflow-x-auto rounded-lg border border-default max-h-[24rem] overflow-y-auto">
						<table class="w-full text-sm">
							<thead class="bg-elevated text-left sticky top-0">
								<tr>
									<th class="p-2 w-10">
										Pilih
									</th>
									<th class="p-2 min-w-32">
										NISN
									</th>
									<th class="p-2 min-w-48">
										Nama
									</th>
									<th class="p-2 min-w-24">
										Kelas
									</th>
									<th class="p-2 min-w-32">
										Status
									</th>
									<th class="p-2 w-16">
										Aksi
									</th>
								</tr>
							</thead>
							<tbody class="divide-y divide-default">
								<tr v-for="row in rows" :key="row.id" :class="isValid(row) ? '' : 'bg-error/5'">
									<td class="p-2 align-top">
										<UCheckbox
											:model-value="isSelected(row)"
											:disabled="!isValid(row)"
											@update:model-value="(v) => toggleRow(row, v)" />
									</td>
									<td class="p-2 align-top">
										{{ row.nisn || "(kosong)" }}
									</td>
									<td class="p-2 align-top">
										{{ row.name || "(kosong)" }}
									</td>
									<td class="p-2 align-top">
										{{ row.class || "-" }}
									</td>
									<td class="p-2 align-top">
										<UBadge :color="isValid(row) ? 'success' : 'error'" variant="subtle">
											{{ isValid(row) ? "Valid" : "Cek manual" }}
										</UBadge>
									</td>
									<td class="p-2 align-top">
										<UButton
											size="xs"
											variant="soft"
											icon="lucide:pencil"
											@click="openEditRow(row)">
											Edit
										</UButton>
									</td>
								</tr>
							</tbody>
						</table>
					</div>
				</div>
			</div>
		</template>
	</UModal>

	<UModal v-model:open="editRowOpen" title="Edit Siswa" :ui="{ content: 'max-w-md' }">
		<template #body>
			<form class="space-y-4" @submit.prevent="saveEditRow">
				<UFormField label="NISN">
					<UInput v-model="editForm.nisn" class="w-full" />
				</UFormField>
				<UFormField label="Nama">
					<UInput v-model="editForm.name" class="w-full" />
				</UFormField>
				<UFormField label="Kelas (opsional)">
					<UInput v-model="editForm.class" class="w-full" />
				</UFormField>

				<UButton type="submit" block>
					Simpan Baris Ini
				</UButton>
			</form>
		</template>
	</UModal>
</template>
