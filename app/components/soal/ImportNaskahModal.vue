<script lang="ts" setup>
	import type { NaskahImportSummary, SoalImportRow } from "~/composables/useNaskahImport";
	import { invoke } from "@tauri-apps/api/core";
	import { open as openDialog } from "@tauri-apps/plugin-dialog";
	import { importSoalFromDocx } from "~/composables/useDocxSoalImport";
	import { importSoalFromPdf } from "~/composables/usePdfSoalImport";

	// Import naskah soal dari Word atau PDF. Keduanya memakai pengurai yang sama, jadi alurnya
	// sama: pilih naskah (wajib) dan kunci (opsional), periksa pratinjau, lalu simpan.
	const props = defineProps<{
		open: boolean
		format: "docx" | "pdf"
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
	const isWord = computed(() => props.format === "docx");
	const extension = computed(() => `.${props.format}`);

	const soalPath = ref("");
	const kunciPath = ref("");
	const rows = ref<SoalImportRow[]>([]);
	const summary = ref<NaskahImportSummary | null>(null);
	const parsing = ref(false);
	const errorMessage = ref("");

	const fileName = (path: string) => path.split(/[/\\]/).pop() ?? path;

	const reset = () => {
		rows.value = [];
		summary.value = null;
		errorMessage.value = "";
	};

	watch(
		() => props.open,
		(isOpen) => {
			if (!isOpen) return;
			soalPath.value = "";
			kunciPath.value = "";
			reset();
		}
	);

	const pickFile = async (title: string) => {
		const path = await openDialog({ multiple: false, filters: [{ name: title, extensions: [props.format] }] });
		return typeof path === "string" ? path : "";
	};

	const pickSoal = async () => {
		const path = await pickFile("Naskah Soal");
		if (!path) return;
		soalPath.value = path;
		reset();
	};

	const pickKunci = async () => {
		const path = await pickFile("Kunci Jawaban");
		if (!path) return;
		kunciPath.value = path;
		reset();
	};

	const clearKunci = () => {
		kunciPath.value = "";
		reset();
	};

	const { isDragging } = useFileDrop({
		isActive: () => props.open,
		accept: (path) => path.toLowerCase().endsWith(extension.value),
		onDrop: (paths) => {
			for (const path of paths) {
				if (!soalPath.value) soalPath.value = path;
				else if (!kunciPath.value && path !== soalPath.value) kunciPath.value = path;
			}
			reset();
		}
	});

	const notifyTemplateDownload = (name: string) => {
		toast.add({ title: "Template diunduh", description: `Contoh ${name} sedang diunduh.`, icon: "lucide:download", color: "success" });
	};

	const readBytes = async (path: string) => new Uint8Array(await invoke<number[]>("read_file_bytes", { path }));

	const processFiles = async () => {
		if (!soalPath.value) return;
		parsing.value = true;
		reset();
		try {
			const naskah = await readBytes(soalPath.value);
			const kunci = kunciPath.value ? await readBytes(kunciPath.value) : undefined;
			const result = isWord.value ? await importSoalFromDocx(naskah, kunci) : await importSoalFromPdf(naskah, kunci);
			rows.value = result.rows;
			summary.value = result.summary;
		} catch (error) {
			errorMessage.value = error instanceof Error ? error.message : String(error);
		} finally {
			parsing.value = false;
		}
	};

	const summaryLines = computed(() => {
		const s = summary.value;
		if (!s) return [];
		const source = s.pages ? ` dari ${s.pages} halaman` : "";
		const lines = [`${s.pgCount} soal PG dan ${s.essayCount} soal esai terbaca${source}.`];
		if (s.twoColumnPages) lines.push(`Tata letak dua kolom dikenali di ${s.twoColumnPages} halaman.`);
		const sources = [
			s.kunciFromMarks && `${s.kunciFromMarks} dari tanda di naskah`,
			s.kunciFromFile && `${s.kunciFromFile} dari file kunci`,
			s.kunciFromNaskah && `${s.kunciFromNaskah} dari bagian kunci di akhir naskah`
		].filter(Boolean);
		if (sources.length) lines.push(`Kunci PG ditemukan: ${sources.join(", ")}.`);
		if (s.images) lines.push(`${s.images} gambar soal ikut diambil.`);
		return lines;
	});

	const warningLines = computed(() => {
		const s = summary.value;
		if (!s) return [];
		const lines: string[] = [];
		if (s.expectedPg !== undefined && s.expectedPg !== s.pgCount) {
			lines.push(`Kunci jawaban menyebut ${s.expectedPg} soal PG, tetapi yang terbaca ${s.pgCount}.`);
		}
		if (s.expectedEssay !== undefined && s.expectedEssay !== s.essayCount) {
			lines.push(`Kunci jawaban menyebut ${s.expectedEssay} soal uraian, tetapi yang terbaca ${s.essayCount}.`);
		}
		if (s.kunciFileEmpty) lines.push("File kunci jawaban tidak berisi pasangan nomor dan jawaban yang bisa dibaca.");
		if (s.pgWithoutKunci) lines.push(`${s.pgWithoutKunci} soal PG belum punya kunci. Pilih kuncinya di kolom Kunci.`);
		if (s.missingNumbers.length) {
			const list = s.missingNumbers.slice(0, 10).join(", ") + (s.missingNumbers.length > 10 ? ", ..." : "");
			lines.push(`Nomor soal ${list} tidak terbaca. Tambahkan lewat Tambah Baris Manual bila perlu.`);
		}
		if (s.imagesUnsupported) lines.push(`${s.imagesUnsupported} gambar berformat selain JPG/PNG/GIF/WEBP (misalnya EMF dari Word) tidak bisa diambil.`);
		if (s.imagesFailed) lines.push(`${s.imagesFailed} gambar soal gagal diambil. Tambahkan gambarnya lewat Edit di Bank Soal setelah impor.`);
		return lines;
	});
</script>

<template>
	<UModal v-model:open="openModel" :title="isWord ? 'Import dari Naskah Soal Word' : 'Import dari PDF Soal'" :ui="{ content: 'max-w-4xl' }">
		<template #body>
			<div class="space-y-4">
				<p v-if="isWord" class="text-sm text-muted">
					Pilih file naskah soal Word (<code>.docx</code>) dan, bila ada, file kunci jawabannya. Nomor soal, opsi a-e
					(termasuk penomoran otomatis Word dan opsi di dalam tabel), bagian PG/uraian, bacaan bersama, dan gambar
					soal dikenali otomatis.
				</p>
				<p v-else class="text-sm text-muted">
					Pilih PDF naskah soal yang berisi teks, misalnya hasil <strong>Save as PDF</strong> dari Word. Nomor soal,
					opsi a-e, bagian PG/uraian, bacaan bersama, tata letak dua kolom, dan gambar soal dikenali otomatis.
				</p>
				<p class="text-sm text-muted -mt-2">
					Kunci jawaban boleh berupa file terpisah, bagian kunci di akhir naskah, tanda bintang (<code>*c.</code>)
					di naskah, atau dipilih sendiri di tabel pratinjau. Semua berjalan tanpa internet.
				</p>
				<p v-if="isWord" class="text-sm text-muted -mt-2">
					Belum tahu formatnya? Unduh contoh:
					<a
						href="/templates/naskah-soal-word/contoh-naskah-soal.docx"
						download
						class="text-primary underline"
						@click="notifyTemplateDownload('Naskah Soal')">Contoh Naskah Soal.docx</a>
					·
					<a
						href="/templates/naskah-soal-word/contoh-kunci-jawaban.docx"
						download
						class="text-primary underline"
						@click="notifyTemplateDownload('Kunci Jawaban')">Contoh Kunci Jawaban.docx</a>
				</p>

				<div
					class="rounded-lg border-2 border-dashed p-4 transition-colors"
					:class="isDragging ? 'border-primary bg-primary/5' : 'border-default'">
					<p class="text-xs text-muted text-center mb-3">
						{{ isDragging ? "Lepas file di sini" : `Seret 1-2 file ${extension} ke sini (naskah dulu, lalu kunci), atau pilih manual di bawah` }}
					</p>
					<div class="grid gap-4 sm:grid-cols-2">
						<UFormField :label="`File Naskah Soal (${extension})`" required>
							<UButton variant="soft" icon="lucide:file-text" @click="pickSoal">
								{{ soalPath ? fileName(soalPath) : "Pilih File" }}
							</UButton>
						</UFormField>
						<UFormField :label="`File Kunci Jawaban (${extension})`" hint="Opsional">
							<div class="flex items-center gap-1">
								<UButton variant="soft" icon="lucide:key" @click="pickKunci">
									{{ kunciPath ? fileName(kunciPath) : "Pilih File" }}
								</UButton>
								<UButton
									v-if="kunciPath"
									variant="ghost"
									color="neutral"
									icon="lucide:x"
									aria-label="Hapus file kunci"
									@click="clearKunci" />
							</div>
						</UFormField>
					</div>
				</div>

				<div class="flex justify-end">
					<UButton
						icon="lucide:scan-search"
						:loading="parsing"
						:disabled="!soalPath"
						@click="processFiles">
						Proses &amp; Preview
					</UButton>
				</div>

				<UAlert
					v-if="errorMessage"
					color="error"
					variant="subtle"
					title="Gagal memproses file"
					:description="errorMessage" />

				<UAlert
					v-if="summaryLines.length"
					color="info"
					variant="subtle"
					icon="lucide:scan-text"
					title="Hasil pembacaan naskah">
					<template #description>
						<ul class="list-disc ps-4 space-y-0.5">
							<li v-for="line in summaryLines" :key="line">
								{{ line }}
							</li>
						</ul>
					</template>
				</UAlert>

				<UAlert
					v-if="summary?.autoFixes.length"
					color="neutral"
					variant="subtle"
					icon="lucide:wand-sparkles"
					title="Salah ketik di naskah yang dibetulkan otomatis">
					<template #description>
						<p class="mb-1">
							Periksa soal-soal ini di tabel sebelum disimpan.
						</p>
						<ul class="list-disc ps-4 space-y-0.5">
							<li v-for="line in summary.autoFixes" :key="line">
								{{ line }}
							</li>
						</ul>
					</template>
				</UAlert>

				<UAlert
					v-if="warningLines.length"
					color="warning"
					variant="subtle"
					icon="lucide:alert-triangle"
					title="Perlu dicek">
					<template #description>
						<ul class="list-disc ps-4 space-y-0.5">
							<li v-for="line in warningLines" :key="line">
								{{ line }}
							</li>
						</ul>
					</template>
				</UAlert>

				<SoalImportReview
					v-if="rows.length"
					:rows="rows"
					:kelas="kelas"
					:jenis="jenis"
					:subject-id="subjectId"
					@saved="emit('saved')" />
			</div>
		</template>
	</UModal>
</template>
