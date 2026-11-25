<script lang="ts" setup>
	import type { TableColumn } from "@nuxt/ui";
	import type { SoalImportRow } from "~/composables/useNaskahImport";
	import { invoke } from "@tauri-apps/api/core";

	// Tabel pratinjau hasil import naskah (Word maupun PDF): pilih baris, rapikan, lalu simpan ke Bank Soal.
	const props = defineProps<{
		rows: SoalImportRow[]
		kelas: string
		jenis: string
		subjectId: number | undefined
	}>();

	const emit = defineEmits<{
		saved: []
	}>();

	const toast = useToast();
	const LETTERS = ["A", "B", "C", "D", "E"] as const;

	const localRows = ref<SoalImportRow[]>([]);
	const selectedIds = ref(new Set<string>());
	const importedIds = ref(new Set<string>());
	const importing = ref(false);
	const importError = ref("");

	watch(
		() => props.rows,
		(rows) => {
			localRows.value = rows.map((r) => ({ ...r }));
			selectedIds.value = new Set(rows.filter((r) => r.valid).map((r) => r.id));
			importedIds.value = new Set();
			importError.value = "";
		},
		{ immediate: true }
	);

	const validCount = computed(() => localRows.value.filter((r) => r.valid).length);
	const selectedCount = computed(() => [...selectedIds.value].filter((id) => !importedIds.value.has(id)).length);

	const columns: TableColumn<SoalImportRow>[] = [
		{ id: "select", header: "Pilih" },
		{ accessorKey: "number", header: "No" },
		{ id: "tipe", header: "Tipe" },
		{ id: "soal", header: "Soal" },
		{ id: "pilihan", header: "Pilihan (kunci ditandai)" },
		{ id: "kunci", header: "Kunci" },
		{ id: "status", header: "Status" },
		{ id: "aksi", header: "Aksi" }
	];

	const optionsOf = (row: SoalImportRow) => [row.pilihan_a, row.pilihan_b, row.pilihan_c, row.pilihan_d, row.pilihan_e];
	const kunciItems = (row: SoalImportRow): string[] => {
		const filled = LETTERS.filter((_, idx) => optionsOf(row)[idx]?.trim());
		return filled.length ? filled : [...LETTERS];
	};

	const setSelected = (row: SoalImportRow, selected: boolean) => {
		const next = new Set(selectedIds.value);
		if (selected) next.add(row.id);
		else next.delete(row.id);
		selectedIds.value = next;
	};

	const revalidate = (row: SoalImportRow) => {
		const reasons: string[] = [];
		if (!row.soal.trim()) reasons.push("teks soal kosong");
		if (row.tipe === "pg") {
			const options = optionsOf(row);
			if (!options[0]?.trim() || !options[1]?.trim()) reasons.push("pilihan A dan B wajib diisi");
			const idx = LETTERS.indexOf(row.kunci_jawaban.trim().toUpperCase() as (typeof LETTERS)[number]);
			if (idx === -1) reasons.push("kunci jawaban belum dipilih");
			else if (!options[idx]?.trim()) reasons.push("kunci jawaban tidak cocok dengan pilihan yang ada");
		}
		row.valid = reasons.length === 0;
		row.alasanInvalid = reasons.length ? reasons.join("; ") : undefined;
		setSelected(row, row.valid && !importedIds.value.has(row.id));
	};

	const setKunci = (row: SoalImportRow, value: string | undefined) => {
		row.kunci_jawaban = value ?? "";
		revalidate(row);
	};

	const isSelected = (row: SoalImportRow) => selectedIds.value.has(row.id);
	const toggleRow = (row: SoalImportRow, value: boolean | "indeterminate") => {
		if (!row.valid || value === "indeterminate") return;
		setSelected(row, value);
	};

	const editRowOpen = ref(false);
	const editRowId = ref<string | null>(null);
	const editForm = reactive({
		tipe: "pg" as "pg" | "esai",
		soal: "",
		pilihan_a: "",
		pilihan_b: "",
		pilihan_c: "",
		pilihan_d: "",
		pilihan_e: "",
		kunci_jawaban: "",
		catatanJawabanEsai: ""
	});

	const openEditRow = (row: SoalImportRow) => {
		editRowId.value = row.id;
		editForm.tipe = row.tipe;
		editForm.soal = row.soal;
		editForm.pilihan_a = row.pilihan_a;
		editForm.pilihan_b = row.pilihan_b;
		editForm.pilihan_c = row.pilihan_c;
		editForm.pilihan_d = row.pilihan_d;
		editForm.pilihan_e = row.pilihan_e;
		editForm.kunci_jawaban = row.kunci_jawaban;
		editForm.catatanJawabanEsai = row.catatanJawabanEsai ?? "";
		editRowOpen.value = true;
	};

	const saveEditRow = () => {
		const row = localRows.value.find((r) => r.id === editRowId.value);
		if (!row) return;

		if (!editForm.soal.trim()) {
			toast.add({ title: "Soal tidak boleh kosong", color: "error", icon: "lucide:x" });
			return;
		}
		if (editForm.tipe === "pg" && (!editForm.pilihan_a.trim() || !editForm.pilihan_b.trim() || !editForm.kunci_jawaban.trim())) {
			toast.add({ title: "Pilihan A & B serta kunci jawaban wajib diisi untuk soal PG", color: "error", icon: "lucide:x" });
			return;
		}

		row.tipe = editForm.tipe;
		row.soal = editForm.soal.trim();
		row.pilihan_a = editForm.pilihan_a.trim();
		row.pilihan_b = editForm.pilihan_b.trim();
		row.pilihan_c = editForm.pilihan_c.trim();
		row.pilihan_d = editForm.pilihan_d.trim();
		row.pilihan_e = editForm.pilihan_e.trim();
		row.kunci_jawaban = editForm.kunci_jawaban.trim();
		row.catatanJawabanEsai = editForm.catatanJawabanEsai.trim() || undefined;
		revalidate(row);
		if (!row.valid) {
			toast.add({ title: "Baris belum valid", description: row.alasanInvalid, color: "warning", icon: "lucide:alert-triangle" });
		}
		editRowOpen.value = false;
	};

	let manualCounter = 0;
	const addManualRow = () => {
		manualCounter += 1;
		const row: SoalImportRow = {
			id: `manual-${manualCounter}`,
			number: localRows.value.length + 1,
			tipe: "pg",
			soal: "",
			pilihan_a: "",
			pilihan_b: "",
			pilihan_c: "",
			pilihan_d: "",
			pilihan_e: "",
			kunci_jawaban: "",
			valid: false,
			alasanInvalid: "Soal manual, belum diisi"
		};
		localRows.value = [...localRows.value, row];
		openEditRow(row);
	};

	const doImport = async () => {
		if (!props.subjectId) {
			importError.value = "Mata pelajaran tidak ditemukan.";
			return;
		}
		importError.value = "";
		importing.value = true;
		const selected = localRows.value.filter((r) => selectedIds.value.has(r.id) && !importedIds.value.has(r.id));
		const newlyImported: string[] = [];
		const failed: string[] = [];

		try {
			for (const row of selected) {
				try {
					let imagePath: string | undefined;
					if (row.imageBytes && row.imageFileName) {
						const saved = await invoke<{ path: string }>("save_question_image_bytes", {
							fileName: row.imageFileName,
							bytes: Array.from(row.imageBytes)
						});
						imagePath = saved.path;
					}

					await invoke("create_question", {
						input: {
							subjectId: props.subjectId,
							class: props.kelas,
							jenis: props.jenis,
							questionText: row.soal,
							questionType: row.tipe === "pg" ? "multiple_choice" : "essay",
							image: imagePath,
							score: 1,
							optionA: row.tipe === "pg" ? row.pilihan_a : "",
							optionB: row.tipe === "pg" ? row.pilihan_b : "",
							optionC: row.tipe === "pg" ? (row.pilihan_c || undefined) : undefined,
							optionD: row.tipe === "pg" ? (row.pilihan_d || undefined) : undefined,
							optionE: row.tipe === "pg" ? (row.pilihan_e || undefined) : undefined,
							correctOption: row.tipe === "pg" ? row.kunci_jawaban.trim().toUpperCase() : ""
						}
					});
					newlyImported.push(row.id);
				} catch (error) {
					failed.push(`Nomor ${row.number}: ${error instanceof Error ? error.message : String(error)}`);
				}
			}
		} finally {
			importing.value = false;
		}

		if (newlyImported.length) {
			importedIds.value = new Set([...importedIds.value, ...newlyImported]);
			toast.add({
				title: "Impor selesai",
				description: `${newlyImported.length} soal berhasil ditambahkan${failed.length ? `, ${failed.length} gagal` : ""}.`,
				icon: "lucide:check",
				color: failed.length ? "warning" : "success"
			});
			emit("saved");
		}
		if (failed.length) {
			importError.value = failed.slice(0, 5).join("\n") + (failed.length > 5 ? `\n...dan ${failed.length - 5} lainnya gagal.` : "");
		}
	};
</script>

<template>
	<div class="space-y-3">
		<UAlert
			v-if="importError"
			color="error"
			variant="subtle"
			title="Sebagian/semua impor gagal"
			:description="importError" />

		<div class="flex flex-wrap items-center justify-between gap-3">
			<div class="text-sm text-muted">
				{{ localRows.length }} soal, {{ validCount }} valid, {{ localRows.length - validCount }} perlu dicek manual
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
					Impor ke Bank Soal ({{ selectedCount }})
				</UButton>
			</div>
		</div>

		<div class="overflow-x-auto rounded-lg border border-default max-h-[28rem] overflow-y-auto">
			<UTable :data="localRows" :columns="columns">
				<template #select-cell="{ row }">
					<UCheckbox
						:model-value="isSelected(row.original)"
						:disabled="!row.original.valid || importedIds.has(row.original.id)"
						@update:model-value="(v) => toggleRow(row.original, v)" />
				</template>
				<template #tipe-cell="{ row }">
					<UBadge :color="row.original.tipe === 'pg' ? 'info' : 'neutral'" variant="subtle">
						{{ row.original.tipe === "pg" ? "PG" : "Esai" }}
					</UBadge>
					<UIcon v-if="row.original.imageFileName" name="lucide:image" class="ml-1 size-3.5 inline" />
				</template>
				<template #soal-cell="{ row }">
					<div class="line-clamp-3 max-w-sm whitespace-normal">{{ row.original.soal || "(kosong)" }}</div>
				</template>
				<template #pilihan-cell="{ row }">
					<ul v-if="row.original.tipe === 'pg'" class="space-y-0.5 text-xs">
						<li
							v-for="(opt, idx) in optionsOf(row.original).filter((value, position) => position < 4 || value)"
							:key="idx"
							:class="row.original.kunci_jawaban.trim().toUpperCase() === 'ABCDE'[idx] ? 'text-success font-medium' : ''">
							{{ "abcde"[idx] }}. {{ opt || "(kosong)" }}
						</li>
					</ul>
					<span v-else class="text-muted line-clamp-2 text-xs">{{ row.original.catatanJawabanEsai || "-" }}</span>
				</template>
				<template #kunci-cell="{ row }">
					<USelect
						v-if="row.original.tipe === 'pg'"
						:model-value="row.original.kunci_jawaban.trim().toUpperCase() || undefined"
						:items="kunciItems(row.original)"
						:disabled="importedIds.has(row.original.id)"
						placeholder="-"
						size="xs"
						class="w-16"
						@update:model-value="(v) => setKunci(row.original, v)" />
					<span v-else class="text-muted text-xs">-</span>
				</template>
				<template #status-cell="{ row }">
					<UBadge :color="row.original.valid ? 'success' : 'error'" variant="subtle">
						{{ row.original.valid ? "Valid" : "Cek manual" }}
					</UBadge>
					<UBadge
						v-if="importedIds.has(row.original.id)"
						color="info"
						variant="subtle"
						class="ml-1">
						Diimpor
					</UBadge>
					<p v-if="row.original.alasanInvalid" class="text-xs text-error mt-1">{{ row.original.alasanInvalid }}</p>
				</template>
				<template #aksi-cell="{ row }">
					<UButton
						size="xs"
						variant="soft"
						icon="lucide:pencil"
						:disabled="importedIds.has(row.original.id)"
						@click="openEditRow(row.original)">
						Edit
					</UButton>
				</template>
			</UTable>
		</div>
	</div>

	<UModal v-model:open="editRowOpen" title="Edit Soal" :ui="{ content: 'max-w-xl' }">
		<template #body>
			<form class="space-y-4" @submit.prevent="saveEditRow">
				<UFormField label="Tipe Soal">
					<URadioGroup
						v-model="editForm.tipe"
						orientation="horizontal"
						:items="[{ label: 'Pilihan Ganda', value: 'pg' }, { label: 'Esai', value: 'esai' }]" />
				</UFormField>

				<UFormField label="Pertanyaan">
					<UTextarea v-model="editForm.soal" :rows="3" class="w-full" />
				</UFormField>

				<template v-if="editForm.tipe === 'pg'">
					<div class="grid gap-3 sm:grid-cols-2">
						<UFormField label="Pilihan A">
							<UInput v-model="editForm.pilihan_a" class="w-full" />
						</UFormField>
						<UFormField label="Pilihan B">
							<UInput v-model="editForm.pilihan_b" class="w-full" />
						</UFormField>
						<UFormField label="Pilihan C (opsional)">
							<UInput v-model="editForm.pilihan_c" class="w-full" />
						</UFormField>
						<UFormField label="Pilihan D (opsional)">
							<UInput v-model="editForm.pilihan_d" class="w-full" />
						</UFormField>
						<UFormField label="Pilihan E (opsional)">
							<UInput v-model="editForm.pilihan_e" class="w-full" />
						</UFormField>
					</div>
					<UFormField label="Kunci Jawaban">
						<USelectMenu v-model="editForm.kunci_jawaban" :items="['A', 'B', 'C', 'D', 'E']" placeholder="Pilih kunci" />
					</UFormField>
				</template>
				<UFormField v-else label="Catatan Jawaban Esai (opsional)">
					<UTextarea v-model="editForm.catatanJawabanEsai" :rows="2" class="w-full" />
				</UFormField>

				<UButton type="submit" block>
					Simpan Baris Ini
				</UButton>
			</form>
		</template>
	</UModal>
</template>
