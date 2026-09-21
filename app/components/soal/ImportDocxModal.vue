<script lang="ts" setup>
	import { invoke } from "@tauri-apps/api/core";
	import { open as openDialog } from "@tauri-apps/plugin-dialog";
	import { parseKunciJawabanDocx, parseNaskahSoalDocx, type DocxImportRow, type KunciJawabanResult } from "~/composables/useDocxSoalImport";

	const props = defineProps<{
		open: boolean
		kelas: string
		jenis: string
		subjectId: number | undefined
	}>();

	const emit = defineEmits<{
		"update:open": [value: boolean]
		saved: []
	}>();

	const openModel = computed({
		get: () => props.open,
		set: (value) => emit("update:open", value)
	});

	const toast = useToast();

	const soalPath = ref("");
	const soalName = ref("");
	const kunciPath = ref("");
	const kunciName = ref("");
	const pgCount = ref<number | undefined>();
	const essayCount = ref<number | undefined>();
	const kunciResult = ref<KunciJawabanResult | null>(null);
	const rows = ref<DocxImportRow[]>([]);
	const selectedIds = ref(new Set<string>());
	const parsing = ref(false);
	const importing = ref(false);
	const errorMessage = ref("");
	const importError = ref("");
	const importedIds = ref(new Set<string>());

	const validCount = computed(() => rows.value.filter((r) => r.valid).length);
	const selectedCount = computed(() => selectedIds.value.size);
	const canProcess = computed(() => Boolean(soalPath.value && kunciPath.value && pgCount.value && essayCount.value));

	watch(
		() => props.open,
		(isOpen) => {
			if (!isOpen) return;
			soalPath.value = "";
			soalName.value = "";
			kunciPath.value = "";
			kunciName.value = "";
			pgCount.value = undefined;
			essayCount.value = undefined;
			kunciResult.value = null;
			rows.value = [];
			selectedIds.value = new Set();
			importedIds.value = new Set();
			errorMessage.value = "";
			importError.value = "";
		}
	);

	const pickSoal = async () => {
		const path = await openDialog({ multiple: false, filters: [{ name: "Naskah Soal", extensions: ["docx"] }] });
		if (!path) return;
		soalPath.value = path;
		soalName.value = path.split(/[/\\]/).pop() ?? path;
	};

	const pickKunci = async () => {
		const path = await openDialog({ multiple: false, filters: [{ name: "Kunci Jawaban", extensions: ["docx"] }] });
		if (!path) return;
		kunciPath.value = path;
		kunciName.value = path.split(/[/\\]/).pop() ?? path;
		errorMessage.value = "";
		try {
			const bytes = new Uint8Array(await invoke<number[]>("read_file_bytes", { path }));
			kunciResult.value = await parseKunciJawabanDocx(bytes);
			pgCount.value = kunciResult.value.detectedPgCount;
			essayCount.value = kunciResult.value.detectedEssayCount;
		} catch (error) {
			errorMessage.value = error instanceof Error ? error.message : String(error);
		}
	};

	const processFiles = async () => {
		if (!canProcess.value || !kunciResult.value) return;
		parsing.value = true;
		errorMessage.value = "";
		rows.value = [];
		selectedIds.value = new Set();
		importedIds.value = new Set();
		try {
			const bytes = new Uint8Array(await invoke<number[]>("read_file_bytes", { path: soalPath.value }));
			const result = await parseNaskahSoalDocx(bytes, pgCount.value!, essayCount.value!, kunciResult.value);
			rows.value = result;
			selectedIds.value = new Set(result.filter((r) => r.valid).map((r) => r.id));
		} catch (error) {
			errorMessage.value = error instanceof Error ? error.message : String(error);
		} finally {
			parsing.value = false;
		}
	};

	const isSelected = (row: DocxImportRow) => selectedIds.value.has(row.id);
	const toggleRow = (row: DocxImportRow, value: boolean | "indeterminate") => {
		if (!row.valid || value === "indeterminate") return;
		const next = new Set(selectedIds.value);
		if (value) next.add(row.id);
		else next.delete(row.id);
		selectedIds.value = next;
	};

	// ---------- Edit satu baris (buat benerin hasil deteksi yang salah/"Cek manual", atau
	// isi baris yang ditambah manual) ----------

	const editRowOpen = ref(false);
	const editRowId = ref<string | null>(null);
	const editForm = reactive({
		tipe: "pg" as "pg" | "esai",
		soal: "",
		pilihan_a: "",
		pilihan_b: "",
		pilihan_c: "",
		pilihan_d: "",
		kunci_jawaban: "",
		catatanJawabanEsai: ""
	});

	const openEditRow = (row: DocxImportRow) => {
		editRowId.value = row.id;
		editForm.tipe = row.tipe;
		editForm.soal = row.soal;
		editForm.pilihan_a = row.pilihan_a;
		editForm.pilihan_b = row.pilihan_b;
		editForm.pilihan_c = row.pilihan_c;
		editForm.pilihan_d = row.pilihan_d;
		editForm.kunci_jawaban = row.kunci_jawaban;
		editForm.catatanJawabanEsai = row.catatanJawabanEsai ?? "";
		editRowOpen.value = true;
	};

	const saveEditRow = () => {
		const row = rows.value.find((r) => r.id === editRowId.value);
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
		row.kunci_jawaban = editForm.kunci_jawaban.trim();
		row.catatanJawabanEsai = editForm.catatanJawabanEsai.trim() || undefined;
		row.valid = true;
		row.alasanInvalid = undefined;

		selectedIds.value = new Set([...selectedIds.value, row.id]);
		editRowOpen.value = false;
	};

	// ---------- Tambah baris manual (buat nambah soal di luar yang kedetek dari file, mis. file
	// cuma ada 20 tapi guru mau 30 total) ----------

	let manualCounter = 0;
	const addManualRow = () => {
		manualCounter += 1;
		const row: DocxImportRow = {
			id: `manual-${manualCounter}`,
			number: rows.value.length + 1,
			tipe: "pg",
			soal: "",
			pilihan_a: "",
			pilihan_b: "",
			pilihan_c: "",
			pilihan_d: "",
			kunci_jawaban: "",
			valid: false,
			alasanInvalid: "Soal manual, belum diisi"
		};
		rows.value = [...rows.value, row];
		openEditRow(row);
	};

	const doImport = async () => {
		if (!props.subjectId) {
			importError.value = "Mata pelajaran tidak ditemukan.";
			return;
		}
		importError.value = "";
		importing.value = true;
		const selected = rows.value.filter((r) => selectedIds.value.has(r.id) && !importedIds.value.has(r.id));
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
							optionE: undefined,
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
	<UModal v-model:open="openModel" title="Import dari Naskah Soal Word" :ui="{ content: 'max-w-4xl' }">
		<template #body>
			<div class="space-y-4">
				<p class="text-sm text-muted">
					Pilih file naskah soal (untuk dicetak) dan file kunci jawabannya — keduanya format
					<code>.docx</code>. Sistem akan mencocokkan soal dengan jawabannya otomatis berdasarkan nomor.
				</p>

				<div class="grid gap-4 sm:grid-cols-2">
					<UFormField label="File Naskah Soal (.docx)">
						<UButton variant="soft" icon="lucide:file-text" @click="pickSoal">
							{{ soalName || "Pilih File" }}
						</UButton>
					</UFormField>
					<UFormField label="File Kunci Jawaban (.docx)">
						<UButton variant="soft" icon="lucide:key" @click="pickKunci">
							{{ kunciName || "Pilih File" }}
						</UButton>
					</UFormField>
				</div>

				<div v-if="kunciPath" class="grid gap-4 sm:grid-cols-3 items-end">
					<UFormField label="Jumlah Soal PG" :description="kunciResult?.detectedPgCount ? 'Terdeteksi otomatis' : 'Tidak terdeteksi, isi manual'">
						<UInputNumber v-model="pgCount" :min="0" />
					</UFormField>
					<UFormField label="Jumlah Soal Esai" :description="kunciResult?.detectedEssayCount ? 'Terdeteksi otomatis' : 'Tidak terdeteksi, isi manual'">
						<UInputNumber v-model="essayCount" :min="0" />
					</UFormField>
					<UButton
						icon="lucide:scan-search"
						:loading="parsing"
						:disabled="!canProcess"
						@click="processFiles">
						Proses & Preview
					</UButton>
				</div>

				<UAlert
					v-if="errorMessage"
					color="error"
					variant="subtle"
					title="Gagal memproses file"
					:description="errorMessage" />
				<UAlert
					v-if="importError"
					color="error"
					variant="subtle"
					title="Sebagian/semua impor gagal"
					:description="importError" />

				<div v-if="rows.length" class="space-y-3">
					<div class="flex flex-wrap items-center justify-between gap-3">
						<div class="text-sm text-muted">
							{{ rows.length }} soal, {{ validCount }} valid, {{ rows.length - validCount }} perlu dicek manual
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
						<table class="w-full text-sm">
							<thead class="bg-elevated text-left sticky top-0">
								<tr>
									<th class="p-2 w-10">Pilih</th>
									<th class="p-2 w-12">No</th>
									<th class="p-2 w-16">Tipe</th>
									<th class="p-2 min-w-64">Soal</th>
									<th class="p-2 min-w-48">Pilihan (kunci ditandai)</th>
									<th class="p-2 min-w-40">Status</th>
									<th class="p-2 w-16">Aksi</th>
								</tr>
							</thead>
							<tbody class="divide-y divide-default">
								<tr v-for="row in rows" :key="row.id" :class="row.valid ? '' : 'bg-error/5'">
									<td class="p-2 align-top">
										<UCheckbox
											:model-value="isSelected(row)"
											:disabled="!row.valid || importedIds.has(row.id)"
											@update:model-value="(v) => toggleRow(row, v)" />
									</td>
									<td class="p-2 align-top text-muted">{{ row.number }}</td>
									<td class="p-2 align-top">
										<UBadge :color="row.tipe === 'pg' ? 'info' : 'neutral'" variant="subtle">
											{{ row.tipe === "pg" ? "PG" : "Esai" }}
										</UBadge>
										<UIcon v-if="row.imageFileName" name="lucide:image" class="ml-1 size-3.5 inline" />
									</td>
									<td class="p-2 align-top max-w-sm">
										<div class="line-clamp-3">{{ row.soal || "(kosong)" }}</div>
									</td>
									<td class="p-2 align-top">
										<ul v-if="row.tipe === 'pg'" class="space-y-0.5">
											<li
												v-for="(opt, idx) in [row.pilihan_a, row.pilihan_b, row.pilihan_c, row.pilihan_d]"
												:key="idx"
												:class="row.kunci_jawaban.trim().toUpperCase() === 'ABCD'[idx] ? 'text-success font-medium' : ''">
												{{ "abcd"[idx] }}. {{ opt || "(kosong)" }}
											</li>
										</ul>
										<span v-else class="text-muted line-clamp-2">{{ row.catatanJawabanEsai || "-" }}</span>
									</td>
									<td class="p-2 align-top">
										<UBadge :color="row.valid ? 'success' : 'error'" variant="subtle">
											{{ row.valid ? "Valid" : "Cek manual" }}
										</UBadge>
										<UBadge
											v-if="importedIds.has(row.id)"
											color="info"
											variant="subtle"
											class="ml-1">
											Diimpor
										</UBadge>
										<p v-if="row.alasanInvalid" class="text-xs text-error mt-1">{{ row.alasanInvalid }}</p>
									</td>
									<td class="p-2 align-top">
										<UButton size="xs" variant="soft" icon="lucide:pencil" @click="openEditRow(row)">
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
					</div>
					<UFormField label="Kunci Jawaban">
						<USelectMenu v-model="editForm.kunci_jawaban" :items="['A', 'B', 'C', 'D']" placeholder="Pilih kunci" />
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
