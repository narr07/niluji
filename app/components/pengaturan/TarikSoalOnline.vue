<script lang="ts" setup>
	import { invoke } from "@tauri-apps/api/core";
	import { fetch as tauriFetch } from "@tauri-apps/plugin-http";
	import {
		checkBankSoalConnection,
		fetchBankSoalMarkdown,
		listJenisFiles,
		subjectFolderCode,
		type BankSoalOnlineRow,
		type JenisFileOption,
		type OnlineSubject
	} from "~/composables/useBankSoalOnline";

	const emit = defineEmits<{
		selected: [rows: BankSoalOnlineRow[]]
	}>();

	const toast = useToast();
	const { classes } = useBankSoalOnline();
	const subjects = ref<OnlineSubject[]>([]);
	const selectedClass = ref("");
	const selectedSubject = ref("");
	const jenisFiles = ref<JenisFileOption[]>([]);
	const selectedJenisFile = ref("");
	const loadingJenisFiles = ref(false);
	const rows = ref<BankSoalOnlineRow[]>([]);
	const sourceFile = ref("");
	const essayFile = ref("");
	const sourceFolder = ref("");
	const loadingSubjects = ref(false);
	const loadingRows = ref(false);
	const checkingConnection = ref(false);
	const errorMessage = ref("");
	const selectedRows = ref(new Set<string>());
	const connectionStatus = ref<"checking" | "online" | "offline">("checking");
	const connectionMessage = ref("Memeriksa koneksi internet...");
	const connectionCheckedAt = ref<Date>();
	const jenisSoal = ref("");
	const scopeOptions = [
		{ label: "Kelas & mata pelajaran ini saja", value: "narrow" as const },
		{ label: "Semua pelajaran di kelas ini saja", value: "class" as const },
		{ label: "Semua kelas, mata pelajaran ini saja", value: "subject" as const },
		{ label: "Semua kelas & semua mata pelajaran", value: "global" as const }
	];
	const scope = ref<"narrow" | "class" | "subject" | "global">("narrow");
	const importing = ref(false);
	const importError = ref("");
	const importedIds = ref(new Set<string>());

	// nama_file_gambar di Excel kadang cuma nama dasarnya (mis. "gambar11", tanpa ".jpg")
	// sementara file aslinya di repo tetap punya ekstensi — jadi ekstensi ditebak dengan
	// coba beberapa kandidat kalau kolomnya sendiri tidak menyebutkan ekstensi.
	const IMAGE_EXTENSIONS = ["jpg", "jpeg", "png", "gif", "webp"];
	const hasKnownExtension = (name: string) => /\.(jpg|jpeg|png|gif|webp)$/i.test(name);

	const numericClass = computed(() => selectedClass.value.replace(/^kelas_/, ""));
	const selectedSubjectId = computed(() => subjects.value.find((s) => subjectFolderCode(s) === selectedSubject.value)?.id);

	const subjectItems = computed(() => subjects.value.map((subject) => ({
		label: subject.code ? `${subject.code} — ${subject.name}` : subject.name,
		value: subjectFolderCode(subject)
	})));
	const jenisFileItems = computed(() => jenisFiles.value.map((j) => ({ label: j.label, value: j.fileName })));
	const canFetch = computed(() =>
		Boolean(selectedClass.value && selectedSubject.value && selectedJenisFile.value && connectionStatus.value === "online")
	);
	const validCount = computed(() => rows.value.filter((row) => row.valid).length);
	const selectedCount = computed(() => selectedRows.value.size);

	const loadSubjects = async () => {
		loadingSubjects.value = true;
		try {
			subjects.value = await invoke<OnlineSubject[]>("list_subjects");
		} catch (error) {
			errorMessage.value = error instanceof Error ? error.message : String(error);
		} finally {
			loadingSubjects.value = false;
		}
	};

	const refreshConnection = async () => {
		checkingConnection.value = true;
		connectionStatus.value = "checking";
		connectionMessage.value = "Memeriksa koneksi internet...";
		try {
			const status = await checkBankSoalConnection();
			connectionStatus.value = status.connected ? "online" : "offline";
			connectionMessage.value = status.message;
			connectionCheckedAt.value = status.checkedAt;
			return status.connected;
		} finally {
			checkingConnection.value = false;
		}
	};

	const selectClass = (value: string) => {
		selectedClass.value = value;
		selectedSubject.value = "";
		selectedJenisFile.value = "";
		jenisFiles.value = [];
		rows.value = [];
		errorMessage.value = "";
	};

	const loadJenisFiles = async () => {
		selectedJenisFile.value = "";
		jenisFiles.value = [];
		if (!selectedClass.value || !selectedSubject.value) return;
		loadingJenisFiles.value = true;
		try {
			jenisFiles.value = await listJenisFiles(selectedClass.value, selectedSubject.value);
		} catch (error) {
			errorMessage.value = error instanceof Error ? error.message : String(error);
		} finally {
			loadingJenisFiles.value = false;
		}
	};

	const selectSubject = (value: string) => {
		selectedSubject.value = value;
		rows.value = [];
		errorMessage.value = "";
		void loadJenisFiles();
	};

	const fetchRows = async () => {
		if (!canFetch.value) return;
		loadingRows.value = true;
		errorMessage.value = "";
		importError.value = "";
		rows.value = [];
		selectedRows.value = new Set();
		importedIds.value = new Set();
		try {
			const isConnected = await refreshConnection();
			if (!isConnected) return;
			const result = await fetchBankSoalMarkdown(selectedClass.value, selectedSubject.value, selectedJenisFile.value);
			rows.value = result.rows;
			sourceFile.value = result.fileName;
			essayFile.value = "";
			sourceFolder.value = result.folderPath;
			jenisSoal.value = result.jenis;
			selectedRows.value = new Set(result.rows.filter((row) => row.valid).map((row) => row.id));
		} catch (error) {
			errorMessage.value = error instanceof Error ? error.message : String(error);
		} finally {
			loadingRows.value = false;
		}
	};

	const isSelected = (row: BankSoalOnlineRow) => selectedRows.value.has(row.id);

	const toggleRow = (row: BankSoalOnlineRow, value: boolean | "indeterminate") => {
		if (!row.valid || value === "indeterminate") return;
		const next = new Set(selectedRows.value);
		if (value) next.add(row.id);
		else next.delete(row.id);
		selectedRows.value = next;
	};

	// Beberapa baris bisa merujuk nama_file_gambar yang sama (disengaja, bukan duplikat) — gambar
	// hanya diunduh & disimpan sekali per pathRelatifGambar, baris lain memakai ulang hasilnya.
	const importSelected = async () => {
		importError.value = "";
		if (!jenisSoal.value.trim()) {
			importError.value = "Isi \"Jenis Soal\" dulu sebelum impor (misalnya \"UTS Semester 1\").";
			return;
		}
		if (!selectedSubjectId.value) {
			importError.value = "Mata pelajaran terpilih tidak ditemukan di database.";
			return;
		}

		const selected = rows.value.filter((row) => selectedRows.value.has(row.id) && !importedIds.value.has(row.id));
		if (!selected.length) return;

		importing.value = true;

		// Daftarkan jenisnya dulu ke registry (biar muncul sebagai kartu di Bank Soal, bukan
		// cuma nempel sebagai teks di tiap soal) — kalau jenis+scope yang sama sudah ada,
		// itu tidak masalah, lanjut saja pakai yang sudah ada.
		try {
			await invoke("create_question_type", {
				name: jenisSoal.value.trim(),
				description: null,
				class: (scope.value === "narrow" || scope.value === "class") ? numericClass.value : null,
				subjectId: (scope.value === "narrow" || scope.value === "subject") ? selectedSubjectId.value : null
			});
		} catch {
			// Sudah ada — tidak masalah.
		}

		const imageCache = new Map<string, string>();
		const newlyImported: string[] = [];
		const failed: string[] = [];

		try {
			for (const row of selected) {
				try {
					let imagePath: string | undefined;
					if (row.pathRelatifGambar && row.urlGambar) {
						imagePath = imageCache.get(row.pathRelatifGambar);
						if (!imagePath) {
							const candidates = hasKnownExtension(row.nama_file_gambar)
								? [{ url: row.urlGambar, ext: row.nama_file_gambar.split(".").pop()! }]
								: IMAGE_EXTENSIONS.map((ext) => ({ url: `${row.urlGambar}.${ext}`, ext }));

							let found: { bytes: number[], ext: string } | undefined;
							for (const candidate of candidates) {
								const response = await tauriFetch(candidate.url);
								if (response.ok) {
									found = { bytes: Array.from(new Uint8Array(await response.arrayBuffer())), ext: candidate.ext };
									break;
								}
							}
							if (!found) throw new Error(`Gagal mengunduh gambar ${row.nama_file_gambar} (sudah dicoba ${candidates.length} kemungkinan ekstensi)`);

							const saved = await invoke<{ path: string }>("save_question_image_bytes", {
								fileName: hasKnownExtension(row.nama_file_gambar) ? row.nama_file_gambar : `${row.nama_file_gambar}.${found.ext}`,
								bytes: found.bytes
							});
							imagePath = saved.path;
							imageCache.set(row.pathRelatifGambar, imagePath);
						}
					}

					await invoke("create_question", {
						input: {
							subjectId: selectedSubjectId.value,
							class: numericClass.value,
							jenis: jenisSoal.value.trim(),
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
			emit("selected", rows.value.filter((row) => newlyImported.includes(row.id)));
		}

		if (newlyImported.length) {
			toast.add({
				title: "Impor selesai",
				description: `${newlyImported.length} soal berhasil ditambahkan ke Bank Soal${failed.length ? `, ${failed.length} gagal` : ""}.`,
				icon: "lucide:check",
				color: failed.length ? "warning" : "success"
			});
		}
		if (failed.length) {
			importError.value = failed.slice(0, 5).join("\n") + (failed.length > 5 ? `\n...dan ${failed.length - 5} baris lainnya gagal.` : "");
		}
	};

	onMounted(() => {
		void loadSubjects();
		void refreshConnection();
	});
</script>

<template>
	<div class="space-y-6">
		<div>
			<h2 class="text-lg font-semibold">Tarik Soal Online</h2>
			<p class="text-sm text-muted mt-1">
				Ambil dan periksa bank soal dari repo GitHub NILUJI sebelum digunakan di CBT offline.
			</p>
		</div>

		<div class="flex flex-wrap items-center gap-3 rounded-lg border border-default bg-elevated/40 px-4 py-3">
			<UIcon
				:name="connectionStatus === 'online' ? 'lucide:wifi' : connectionStatus === 'offline' ? 'lucide:wifi-off' : 'lucide:loader-circle'"
				:class="['size-5', connectionStatus === 'online' ? 'text-success' : connectionStatus === 'offline' ? 'text-error' : 'animate-spin text-muted']" />
			<div class="flex-1 text-sm">
				<p :class="connectionStatus === 'online' ? 'text-success' : connectionStatus === 'offline' ? 'text-error' : 'text-muted'">
					{{ connectionMessage }}
				</p>
				<p v-if="connectionCheckedAt" class="mt-0.5 text-xs text-muted">
					Diperiksa pukul {{ connectionCheckedAt.toLocaleTimeString('id-ID') }}
				</p>
			</div>
			<UButton
				variant="soft"
				icon="lucide:refresh-cw"
				:loading="checkingConnection"
				@click="refreshConnection">
				Refresh koneksi
			</UButton>
		</div>

		<div class="grid gap-4 md:grid-cols-4 items-end">
			<UFormField label="Kelas">
				<USelectMenu
					:model-value="selectedClass"
					:items="classes"
					value-key="value"
					placeholder="Pilih kelas"
					@update:model-value="selectClass" />
			</UFormField>

			<UFormField label="Mata Pelajaran" description="Folder memakai kode mata pelajaran dari database.">
				<USelectMenu
					:model-value="selectedSubject"
					:items="subjectItems"
					value-key="value"
					:disabled="!selectedClass || loadingSubjects"
					placeholder="Pilih mata pelajaran"
					@update:model-value="selectSubject" />
			</UFormField>

			<UFormField label="Jenis Ujian" :description="selectedSubject && !loadingJenisFiles && !jenisFileItems.length ? 'Belum ada file jenis ujian di folder ini.' : undefined">
				<USelectMenu
					v-model="selectedJenisFile"
					:items="jenisFileItems"
					value-key="value"
					:loading="loadingJenisFiles"
					:disabled="!selectedSubject"
					placeholder="Pilih jenis ujian" />
			</UFormField>

			<UButton
				icon="lucide:cloud-download"
				:loading="loadingRows"
				:disabled="!canFetch"
				@click="fetchRows">
				Ambil &amp; Preview
			</UButton>
		</div>

		<UAlert
			v-if="errorMessage"
			color="error"
			variant="subtle"
			title="Gagal mengambil bank soal"
			:description="errorMessage" />

		<div v-if="rows.length" class="space-y-4">
			<div class="flex flex-wrap items-end justify-between gap-3">
				<div class="text-sm text-muted">
					Sumber: <code>{{ sourceFolder }}/{{ sourceFile }}</code> · Jenis: <UBadge color="neutral" variant="subtle">{{ jenisSoal }}</UBadge>
					<span class="mx-2">·</span>
					{{ rows.length }} baris, {{ validCount }} valid, {{ rows.length - validCount }} invalid
				</div>
				<UButton
					icon="lucide:database"
					:loading="importing"
					:disabled="!selectedCount"
					@click="importSelected">
					Impor ke Bank Soal ({{ selectedCount }})
				</UButton>
			</div>

			<UFormField label="Berlaku untuk">
				<URadioGroup v-model="scope" orientation="horizontal" :items="scopeOptions" />
			</UFormField>

			<UAlert
				v-if="importError"
				color="error"
				variant="subtle"
				title="Sebagian/semua impor gagal"
				:description="importError" />

			<div class="overflow-x-auto rounded-lg border border-default">
				<table class="w-full text-sm">
					<thead class="bg-elevated text-left">
						<tr>
							<th class="p-3 w-12">Pilih</th>
							<th class="p-3 w-16">Baris</th>
							<th class="p-3 w-20">Tipe</th>
							<th class="p-3 min-w-80">Soal</th>
							<th class="p-3 min-w-64">Status</th>
							<th class="p-3 min-w-48">Gambar</th>
						</tr>
					</thead>
					<tbody class="divide-y divide-default">
						<tr v-for="row in rows" :key="row.id" :class="row.valid ? '' : 'bg-error/5'">
							<td class="p-3 align-top">
								<UCheckbox
									:model-value="isSelected(row)"
									:disabled="!row.valid || importedIds.has(row.id)"
									@update:model-value="(value) => toggleRow(row, value)" />
							</td>
							<td class="p-3 align-top text-muted">{{ row.rowNumber }}</td>
							<td class="p-3 align-top">
								<UBadge :color="row.tipe === 'pg' ? 'info' : 'neutral'" variant="subtle">
									{{ row.tipe === 'pg' ? 'PG' : 'Esai' }}
								</UBadge>
							</td>
							<td class="p-3 align-top max-w-xl">
								<div class="line-clamp-3">{{ row.soal || "(kosong)" }}</div>
							</td>
							<td class="p-3 align-top">
								<UBadge :color="row.valid ? 'success' : 'error'" variant="subtle">
									{{ row.valid ? "✓ Valid" : "✕ Invalid" }}
								</UBadge>
								<UBadge
									v-if="importedIds.has(row.id)"
									color="info"
									variant="subtle"
									class="ml-1">
									Diimpor
								</UBadge>
								<p v-if="row.alasan_invalid" class="mt-1 text-xs text-error">{{ row.alasan_invalid }}</p>
							</td>
							<td class="p-3 align-top">
								<div v-if="row.nama_file_gambar" class="flex items-start gap-2">
									<UIcon name="lucide:image" class="size-4 mt-0.5 shrink-0" />
									<span class="break-all">{{ row.nama_file_gambar }}</span>
								</div>
								<span v-else class="text-muted">—</span>
							</td>
						</tr>
					</tbody>
				</table>
			</div>
		</div>

		<div v-else-if="!loadingRows" class="rounded-lg border border-dashed border-default p-10 text-center text-muted">
			<UIcon name="lucide:cloud-download" class="size-8 mb-2" />
			<p>Pilih kelas dan mata pelajaran, lalu ambil bank soal untuk melihat preview.</p>
		</div>
	</div>
</template>
