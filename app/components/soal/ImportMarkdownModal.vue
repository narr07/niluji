<script lang="ts" setup>
	import { invoke } from "@tauri-apps/api/core";
	import { open as openDialog } from "@tauri-apps/plugin-dialog";
	import { parseSoalMarkdown, type BankSoalOnlineRow } from "~/composables/useBankSoalOnline";

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

	const filePath = ref("");
	const fileName = ref("");
	const fileDir = ref("");
	const rows = ref<BankSoalOnlineRow[]>([]);
	const selectedIds = ref(new Set<string>());
	const parsing = ref(false);
	const importing = ref(false);
	const errorMessage = ref("");
	const importError = ref("");
	const importedIds = ref(new Set<string>());

	const validCount = computed(() => rows.value.filter((r) => r.valid).length);
	const selectedCount = computed(() => selectedIds.value.size);

	watch(
		() => props.open,
		(isOpen) => {
			if (!isOpen) return;
			filePath.value = "";
			fileName.value = "";
			fileDir.value = "";
			rows.value = [];
			selectedIds.value = new Set();
			importedIds.value = new Set();
			errorMessage.value = "";
			importError.value = "";
		}
	);

	const loadFile = async (path: string) => {
		filePath.value = path;
		fileName.value = path.split(/[/\\]/).pop() ?? path;
		fileDir.value = path.slice(0, path.length - fileName.value.length - 1);

		errorMessage.value = "";
		parsing.value = true;
		rows.value = [];
		selectedIds.value = new Set();
		importedIds.value = new Set();
		try {
			const bytes = await invoke<number[]>("read_file_bytes", { path });
			const text = new TextDecoder("utf-8").decode(new Uint8Array(bytes));
			const { rows: parsedRows } = parseSoalMarkdown(text, "");
			rows.value = parsedRows;
			selectedIds.value = new Set(parsedRows.filter((r) => r.valid).map((r) => r.id));
		} catch (error) {
			errorMessage.value = error instanceof Error ? error.message : String(error);
		} finally {
			parsing.value = false;
		}
	};

	const pickFile = async () => {
		const path = await openDialog({ multiple: false, filters: [{ name: "Bank Soal Markdown", extensions: ["md"] }] });
		if (!path) return;
		await loadFile(path);
	};

	const { isDragging } = useFileDrop({
		isActive: () => props.open,
		accept: (path) => path.toLowerCase().endsWith(".md"),
		onDrop: (paths) => loadFile(paths[0]!)
	});

	const isSelected = (row: BankSoalOnlineRow) => selectedIds.value.has(row.id);
	const toggleRow = (row: BankSoalOnlineRow, value: boolean | "indeterminate") => {
		if (!row.valid || value === "indeterminate") return;
		const next = new Set(selectedIds.value);
		if (value) next.add(row.id);
		else next.delete(row.id);
		selectedIds.value = next;
	};

	// Gambar dirujuk relatif terhadap lokasi file .md di disk (folder "gambar" bersebelahan) —
	// persis struktur yang dihasilkan fitur Export Bank Soal, jadi file hasil export bisa
	// langsung diimpor lagi lewat sini tanpa perlu diunggah ke GitHub dulu.
	const imageCache = new Map<string, string>();
	const resolveOptionImage = async (namaFileGambar: string): Promise<string> => {
		const cached = imageCache.get(namaFileGambar);
		if (cached) return cached;

		const imagePath = `${fileDir.value}/gambar/${namaFileGambar}`;
		const bytes = await invoke<number[]>("read_file_bytes", { path: imagePath });
		const saved = await invoke<{ path: string }>("save_question_image_bytes", { fileName: namaFileGambar, bytes });
		imageCache.set(namaFileGambar, saved.path);
		return saved.path;
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
					if (row.nama_file_gambar) {
						imagePath = await resolveOptionImage(row.nama_file_gambar);
					}

					await invoke("create_question", {
						input: {
							subjectId: props.subjectId,
							class: props.kelas,
							jenis: props.jenis,
							questionText: row.soal,
							questionType: row.tipe === "pg" ? "multiple_choice" : "essay",
							image: imagePath,
							score: Number(row.skor) || 1,
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
					failed.push(`Baris ${row.rowNumber}: ${error instanceof Error ? error.message : String(error)}`);
				}
			}
		} finally {
			importing.value = false;
		}

		if (newlyImported.length) {
			importedIds.value = new Set([...importedIds.value, ...newlyImported]);
			toast.add({
				title: "Import selesai",
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
	<UModal v-model:open="openModel" title="Import dari File Markdown (.md)" :ui="{ content: 'max-w-4xl' }">
		<template #body>
			<div class="space-y-4">
				<p class="text-sm text-muted">
					Pilih file <code>.md</code> lokal — format persis seperti hasil Export Bank Soal (kalau ada gambar,
					pastikan folder <code>gambar/</code> ada di sebelah file ini). Cocok buat file yang sudah pernah
					diekspor sendiri atau dikirim guru lain lewat USB/chat, tanpa perlu lewat GitHub.
				</p>

				<div
					class="rounded-lg border-2 border-dashed p-6 text-center transition-colors"
					:class="isDragging ? 'border-primary bg-primary/5' : 'border-default'">
					<UIcon name="lucide:file-up" class="size-8 text-muted mx-auto mb-2" />
					<p class="text-sm text-muted mb-3">
						{{ isDragging ? "Lepas file di sini" : "Seret file .md ke sini, atau" }}
					</p>
					<UButton
						variant="soft"
						icon="lucide:file-text"
						:loading="parsing"
						@click="pickFile">
						{{ fileName || "Pilih File" }}
					</UButton>
				</div>

				<UAlert
					v-if="errorMessage"
					color="error"
					variant="subtle"
					title="Gagal memproses"
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
						<UButton
							icon="lucide:database"
							:loading="importing"
							:disabled="!selectedCount"
							@click="doImport">
							Impor ke Bank Soal ({{ selectedCount }})
						</UButton>
					</div>

					<div class="overflow-x-auto rounded-lg border border-default max-h-[28rem] overflow-y-auto">
						<table class="w-full text-sm">
							<thead class="bg-elevated text-left sticky top-0">
								<tr>
									<th class="p-2 w-10">
										Pilih
									</th>
									<th class="p-2 w-12">
										No
									</th>
									<th class="p-2 w-16">
										Tipe
									</th>
									<th class="p-2 min-w-64">
										Soal
									</th>
									<th class="p-2 min-w-32">
										Status
									</th>
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
									<td class="p-2 align-top text-muted">
										{{ row.rowNumber }}
									</td>
									<td class="p-2 align-top">
										<UBadge :color="row.tipe === 'pg' ? 'info' : 'neutral'" variant="subtle">
											{{ row.tipe === "pg" ? "PG" : "Esai" }}
										</UBadge>
										<UIcon v-if="row.nama_file_gambar" name="lucide:image" class="ml-1 size-3.5 inline" />
									</td>
									<td class="p-2 align-top max-w-sm">
										<div class="line-clamp-3">
											{{ row.soal || "(kosong)" }}
										</div>
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
										<p v-if="row.alasan_invalid" class="text-xs text-error mt-1">
											{{ row.alasan_invalid }}
										</p>
									</td>
								</tr>
							</tbody>
						</table>
					</div>
				</div>
			</div>
		</template>
	</UModal>
</template>
