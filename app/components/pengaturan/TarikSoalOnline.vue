<script lang="ts" setup>
	import { invoke } from "@tauri-apps/api/core";
	import { open as openDialog } from "@tauri-apps/plugin-dialog";
	import { open as openUrl } from "@tauri-apps/plugin-shell";
	import {
		checkBankSoalConnection,
		DEFAULT_BANK_SOAL_SOURCE,
		githubProvider,
		matchLocalSubject,
		resolveGithubSource,
		sourceRepoUrl,
		zipProviderFromFile,
		zipProviderFromLink,
		type BankSoalOnlineRow,
		type BankSoalProvider,
		type BankSoalSource,
		type JenisFileOption,
		type OnlineSubject
	} from "~/composables/useBankSoalOnline";

	const emit = defineEmits<{
		selected: [rows: BankSoalOnlineRow[]]
	}>();

	const toast = useToast();
	const { downloadTemplate } = useTemplateDownload();
	const SOURCE_STORAGE_KEY = "niluji:tarik-soal-sumber";

	// Sumber bank soal (semua berstruktur sama: kelas_*/{mapel}/{jenis}.md + gambar/):
	// - default: repo GitHub bawaan NILUJI
	// - github:  repo GitHub lain dari operator kecamatan
	// - link:    file .zip yang dibagikan lewat Google Drive / Dropbox / link unduhan lain
	// - local:   file .zip di komputer/flashdisk — satu-satunya yang jalan tanpa internet
	// Link/path terakhir diingat, jadi cukup diisi sekali.
	type SourceMode = "default" | "github" | "link" | "local";
	const sourceMode = ref<SourceMode>("default");
	const githubLinkInput = ref("");
	const githubSource = ref<BankSoalSource | null>(null);
	const zipLinkInput = ref("");
	const localZipPath = ref("");
	const sourceError = ref("");
	const resolvingSource = ref(false);
	const provider = shallowRef<BankSoalProvider | null>(null);
	const needsInternet = computed(() => sourceMode.value !== "local");
	const sourceItems = [
		{ label: "NILUJI (bawaan)", value: "default", description: "Repo GitHub narr07/niluji." },
		{ label: "Repo GitHub lain", value: "github", description: "Repo dari operator kecamatan." },
		{ label: "Link file ZIP", value: "link", description: "Google Drive, Dropbox, atau link unduhan lain." },
		{ label: "File ZIP lokal", value: "local", description: "Dari komputer/flashdisk — tanpa internet." }
	];

	const subjects = ref<OnlineSubject[]>([]);
	const onlineClasses = ref<{ label: string, value: string }[]>([]);
	const loadingClasses = ref(false);
	const subjectFolders = ref<string[]>([]);
	const loadingSubjectFolders = ref(false);
	const selectedClass = ref("");
	const selectedSubject = ref("");
	// Mapel lokal tujuan impor — otomatis dicocokkan dari nama folder, bisa diganti manual kalau
	// repo kecamatan lain memakai nama folder yang berbeda dari kode mapel di aplikasi ini.
	const targetSubjectId = ref<number>();
	const jenisFiles = ref<JenisFileOption[]>([]);
	const selectedJenisFile = ref("");
	const loadingJenisFiles = ref(false);
	const rows = ref<BankSoalOnlineRow[]>([]);
	const sourceFile = ref("");
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

	const numericClass = computed(() => selectedClass.value.replace(/^kelas_/i, ""));

	const subjectItems = computed(() => subjectFolders.value.map((folder) => {
		const match = matchLocalSubject(folder, subjects.value);
		return { label: match ? `${folder} — ${match.name}` : folder, value: folder };
	}));
	const localSubjectItems = computed(() => subjects.value.map((s) => ({ label: s.code ? `${s.code} — ${s.name}` : s.name, value: s.id })));
	const jenisFileItems = computed(() => jenisFiles.value.map((j) => ({ label: j.label, value: j.fileName })));
	const canFetch = computed(() =>
		Boolean(provider.value && selectedClass.value && selectedSubject.value && selectedJenisFile.value && (!needsInternet.value || connectionStatus.value === "online"))
	);
	const validCount = computed(() => rows.value.filter((row) => row.valid).length);
	const selectedCount = computed(() => selectedRows.value.size);

	interface SavedSource {
		mode?: SourceMode
		githubLink?: string
		githubSource?: BankSoalSource | null
		zipLink?: string
		localZipPath?: string
	}

	const persistSource = () => {
		try {
			const saved: SavedSource = {
				mode: sourceMode.value,
				githubLink: githubLinkInput.value.trim(),
				githubSource: githubSource.value,
				zipLink: zipLinkInput.value.trim(),
				localZipPath: localZipPath.value
			};
			localStorage.setItem(SOURCE_STORAGE_KEY, JSON.stringify(saved));
		} catch {
			// Abaikan kalau localStorage tidak bisa diakses — pilihan sumber cuma tidak diingat.
		}
	};

	const restoreSource = () => {
		try {
			const saved = JSON.parse(localStorage.getItem(SOURCE_STORAGE_KEY) ?? "null") as SavedSource | null;
			if (!saved) return;
			if (saved.githubSource?.owner && saved.githubSource.repo) githubSource.value = saved.githubSource;
			githubLinkInput.value = saved.githubLink ?? "";
			zipLinkInput.value = saved.zipLink ?? "";
			localZipPath.value = saved.localZipPath ?? "";
			if (saved.mode) sourceMode.value = saved.mode;
		} catch {
			// Data lama rusak/tidak bisa dibaca — mulai dari sumber bawaan.
		}
	};

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

	const selectClass = async (value: string) => {
		selectedClass.value = value;
		selectedSubject.value = "";
		selectedJenisFile.value = "";
		jenisFiles.value = [];
		subjectFolders.value = [];
		rows.value = [];
		errorMessage.value = "";
		if (!value || !provider.value) return;
		loadingSubjectFolders.value = true;
		try {
			subjectFolders.value = await provider.value.listSubjects(value);
		} catch (error) {
			errorMessage.value = error instanceof Error ? error.message : String(error);
		} finally {
			loadingSubjectFolders.value = false;
		}
	};

	const loadClasses = async () => {
		onlineClasses.value = [];
		await selectClass("");
		if (!provider.value) return;
		loadingClasses.value = true;
		try {
			onlineClasses.value = await provider.value.listClasses();
			if (!onlineClasses.value.length) errorMessage.value = `Tidak ada folder kelas_* di ${provider.value.label}.`;
		} catch (error) {
			errorMessage.value = error instanceof Error ? error.message : String(error);
		} finally {
			loadingClasses.value = false;
		}
	};

	// Siapkan provider untuk sumber yang dipilih. `interactive` = dipicu tombol (tampilkan toast
	// sukses); saat dipulihkan otomatis waktu halaman dibuka, cukup diam-diam.
	const activateSource = async (interactive = false) => {
		provider.value = null;
		sourceError.value = "";
		onlineClasses.value = [];
		await selectClass("");
		if (needsInternet.value && connectionStatus.value !== "online") return;

		resolvingSource.value = true;
		try {
			if (sourceMode.value === "default") {
				provider.value = githubProvider(DEFAULT_BANK_SOAL_SOURCE);
			} else if (sourceMode.value === "github") {
				if (interactive) githubSource.value = await resolveGithubSource(githubLinkInput.value);
				if (githubSource.value) provider.value = githubProvider(githubSource.value);
			} else if (sourceMode.value === "link") {
				if (zipLinkInput.value.trim()) provider.value = await zipProviderFromLink(zipLinkInput.value);
			} else if (localZipPath.value) {
				provider.value = await zipProviderFromFile(localZipPath.value);
			}
			persistSource();
		} catch (error) {
			sourceError.value = error instanceof Error ? error.message : String(error);
		} finally {
			resolvingSource.value = false;
		}

		if (!provider.value) return;
		await loadClasses();
		if (interactive && onlineClasses.value.length) {
			toast.add({ title: "Sumber bisa dipakai", description: `${provider.value.label} — ${onlineClasses.value.length} folder kelas ditemukan.`, icon: "lucide:check", color: "success" });
		}
	};

	// Dipulihkan SEBELUM watch dipasang, supaya pemulihan tidak memicu activateSource dua kali
	// (sekali dari watch, sekali dari onMounted).
	restoreSource();
	watch(sourceMode, () => void activateSource());

	const pickLocalZip = async () => {
		const path = await openDialog({ multiple: false, filters: [{ name: "Bank Soal (ZIP)", extensions: ["zip"] }] });
		if (!path) return;
		localZipPath.value = path;
		await activateSource(true);
	};

	const loadJenisFiles = async () => {
		selectedJenisFile.value = "";
		jenisFiles.value = [];
		if (!provider.value || !selectedClass.value || !selectedSubject.value) return;
		loadingJenisFiles.value = true;
		try {
			jenisFiles.value = await provider.value.listJenis(selectedClass.value, selectedSubject.value);
		} catch (error) {
			errorMessage.value = error instanceof Error ? error.message : String(error);
		} finally {
			loadingJenisFiles.value = false;
		}
	};

	const selectSubject = (value: string) => {
		selectedSubject.value = value;
		targetSubjectId.value = matchLocalSubject(value, subjects.value)?.id;
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
			if (!provider.value) return;
			const result = await provider.value.readMarkdown(selectedClass.value, selectedSubject.value, selectedJenisFile.value);
			rows.value = result.rows;
			sourceFile.value = result.fileName;
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
		if (!targetSubjectId.value) {
			importError.value = "Pilih dulu mata pelajaran tujuan di aplikasi ini (\"Simpan ke Mata Pelajaran\").";
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
				subjectId: (scope.value === "narrow" || scope.value === "subject") ? targetSubjectId.value : null
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
					if (row.pathRelatifGambar && provider.value) {
						imagePath = imageCache.get(row.pathRelatifGambar);
						if (!imagePath) {
							const found = await provider.value.readImage(row.pathRelatifGambar);
							if (!found) throw new Error(`Gambar ${row.nama_file_gambar} tidak ditemukan di sumber`);

							const saved = await invoke<{ path: string }>("save_question_image_bytes", {
								fileName: found.fileName,
								bytes: Array.from(found.bytes)
							});
							imagePath = saved.path;
							imageCache.set(row.pathRelatifGambar, imagePath);
						}
					}

					await invoke("create_question", {
						input: {
							subjectId: targetSubjectId.value,
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

	const guideSteps = [
		{
			title: "Siapkan soal di NILUJI",
			body: "Operator kecamatan memasukkan soal ke Bank Soal seperti biasa (impor Word/Excel/Markdown atau ketik manual), per kelas, mapel, dan jenis ujian."
		},
		{
			title: "Export ke folder bernama db-soal",
			body: "Buka Pengaturan → Fitur → Export Bank Soal (pakai PIN), pilih folder bernama db-soal sebagai tujuan. Ulangi untuk tiap jenis ujian. Hasilnya otomatis tersusun: db-soal/kelas_4/mtk/nama-ujian.md beserta folder gambar/."
		},
		{
			title: "Bagikan dengan salah satu cara di bawah",
			body: "Pilih yang paling mudah untuk kecamatan Anda — isinya sama, hanya cara mengirimnya yang beda."
		},
		{
			title: "Sekolah memilih sumber yang sesuai",
			body: "Di bagian Sumber Bank Soal, sekolah memilih jenis sumber yang sama, menempelkan link (atau memilih file ZIP), lalu klik \"Pakai\". Link disimpan, jadi cukup sekali."
		}
	];

	const shareMethods = [
		{
			icon: "lucide:github",
			title: "GitHub",
			source: "Repo GitHub lain",
			steps: [
				"Buat akun di github.com, klik New repository, pilih Public, centang \"Add a README file\".",
				"Di repo, klik Add file → Upload files, seret folder db-soal, lalu Commit changes.",
				"Bagikan link repo, mis. https://github.com/nama-akun/bank-soal-kecamatan."
			]
		},
		{
			icon: "lucide:hard-drive",
			title: "Google Drive / Dropbox",
			source: "Link file ZIP",
			steps: [
				"Klik kanan folder db-soal → Kirim ke → Folder terkompresi (zip), jadi db-soal.zip.",
				"Unggah db-soal.zip ke Google Drive (bukan foldernya — harus file .zip).",
				"Klik kanan file → Bagikan → Akses umum: \"Siapa saja yang memiliki link\" → Salin link."
			]
		},
		{
			icon: "lucide:usb",
			title: "Flashdisk / WhatsApp",
			source: "File ZIP lokal",
			steps: [
				"Buat db-soal.zip seperti cara Google Drive.",
				"Kirim file zip lewat flashdisk, WhatsApp, atau jaringan sekolah.",
				"Sekolah cukup memilih file zip-nya — tidak perlu internet sama sekali."
			]
		}
	];

	onMounted(async () => {
		void loadSubjects();
		await refreshConnection();
		await activateSource();
	});
</script>

<template>
	<div class="space-y-6">
		<div>
			<h2 class="text-lg font-semibold">Tarik Soal Online</h2>
			<p class="text-sm text-muted mt-1">
				Ambil dan periksa bank soal dari repo GitHub sebelum digunakan di CBT offline.
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
				@click="refreshConnection().then(() => !provider && activateSource())">
				Refresh koneksi
			</UButton>
		</div>

		<div class="space-y-4 rounded-lg border border-default p-4">
			<UFormField label="Sumber Bank Soal">
				<URadioGroup
					v-model="sourceMode"
					:items="sourceItems"
					variant="table"
					orientation="horizontal"
					:ui="{ fieldset: 'w-full flex-wrap', item: 'flex-1 min-w-48' }" />
			</UFormField>

			<UFormField
				v-if="sourceMode === 'github'"
				label="Link Repo GitHub"
				description="Tempel link repo dari operator kecamatan, mis. https://github.com/nama-akun/bank-soal-kecamatan">
				<div class="flex gap-2">
					<UInput
						v-model="githubLinkInput"
						placeholder="https://github.com/nama-akun/nama-repo"
						icon="lucide:link"
						class="flex-1"
						@keydown.enter="activateSource(true)" />
					<UButton
						icon="lucide:check"
						:loading="resolvingSource"
						:disabled="!githubLinkInput.trim() || connectionStatus !== 'online'"
						@click="activateSource(true)">
						Pakai Link
					</UButton>
				</div>
			</UFormField>

			<UFormField
				v-else-if="sourceMode === 'link'"
				label="Link File ZIP"
				description="Link berbagi file db-soal.zip dari Google Drive atau Dropbox, atau link unduhan langsung lainnya.">
				<div class="flex gap-2">
					<UInput
						v-model="zipLinkInput"
						placeholder="https://drive.google.com/file/d/.../view?usp=sharing"
						icon="lucide:link"
						class="flex-1"
						@keydown.enter="activateSource(true)" />
					<UButton
						icon="lucide:download"
						:loading="resolvingSource"
						:disabled="!zipLinkInput.trim() || connectionStatus !== 'online'"
						@click="activateSource(true)">
						Pakai Link
					</UButton>
				</div>
			</UFormField>

			<UFormField
				v-else-if="sourceMode === 'local'"
				label="File ZIP Bank Soal"
				description="File db-soal.zip dari operator kecamatan (flashdisk, WhatsApp, dll.).">
				<div class="flex flex-wrap items-center gap-2">
					<UButton
						icon="lucide:folder-open"
						variant="soft"
						:loading="resolvingSource"
						@click="pickLocalZip">
						Pilih File ZIP
					</UButton>
					<span v-if="localZipPath" class="text-sm text-muted break-all">{{ localZipPath }}</span>
				</div>
			</UFormField>

			<UAlert
				v-if="sourceError"
				color="error"
				variant="subtle"
				icon="lucide:triangle-alert"
				title="Sumber tidak bisa dipakai"
				:description="sourceError" />

			<p v-if="needsInternet && connectionStatus === 'offline'" class="text-sm text-warning">
				Sumber ini butuh internet. Tanpa internet, gunakan "File ZIP lokal".
			</p>

			<div v-if="provider" class="flex flex-wrap items-center gap-x-2 text-sm text-muted">
				<UIcon name="lucide:circle-check" class="size-4 text-success" />
				Sedang memakai <code>{{ provider.label }}</code>
				<UButton
					v-if="sourceMode === 'default' || (sourceMode === 'github' && githubSource)"
					variant="link"
					color="info"
					icon="lucide:external-link"
					class="px-0"
					@click="openUrl(sourceRepoUrl(sourceMode === 'default' ? DEFAULT_BANK_SOAL_SOURCE : githubSource!))">
					Buka di GitHub
				</UButton>
			</div>
		</div>

		<UCollapsible class="rounded-lg border border-default">
			<UButton
				variant="ghost"
				color="neutral"
				icon="lucide:book-open"
				trailing-icon="lucide:chevron-down"
				block
				class="justify-start px-4 py-3 group"
				:ui="{ trailingIcon: 'ms-auto group-data-[state=open]:rotate-180 transition-transform' }">
				Panduan untuk operator kecamatan: menyiapkan & membagikan bank soal
			</UButton>

			<template #content>
				<div class="space-y-4 border-t border-default px-4 py-4">
					<p class="text-sm text-muted">
						Dipakai di kecamatan lain? Operator kecamatan cukup menyiapkan satu folder db-soal berisi bank soal,
						lalu membagikannya. Tiap sekolah memilih sumber itu, lalu memilih kelas, mapel, dan jenis ujian sendiri.
					</p>

					<ol class="space-y-3">
						<li v-for="(step, i) in guideSteps" :key="step.title" class="flex gap-3">
							<span class="flex size-6 shrink-0 items-center justify-center rounded-full bg-primary/10 text-xs font-semibold text-primary">
								{{ i + 1 }}
							</span>
							<div class="text-sm flex-1">
								<p class="font-medium">{{ step.title }}</p>
								<p class="text-muted">{{ step.body }}</p>
								<div v-if="i === 2" class="mt-3 grid gap-3 md:grid-cols-3">
									<div v-for="method in shareMethods" :key="method.title" class="rounded-md border border-default p-3">
										<p class="flex items-center gap-2 font-medium">
											<UIcon :name="method.icon" class="size-4" />
											{{ method.title }}
										</p>
										<ol class="mt-2 list-decimal space-y-1 pl-4 text-xs text-muted">
											<li v-for="s in method.steps" :key="s">{{ s }}</li>
										</ol>
										<p class="mt-2 text-xs">
											Sekolah memilih: <UBadge size="sm" variant="subtle">{{ method.source }}</UBadge>
										</p>
									</div>
								</div>
							</div>
						</li>
					</ol>

					<div class="grid gap-4 md:grid-cols-2">
						<pre class="overflow-x-auto rounded-md border border-default bg-elevated/50 p-3 text-xs leading-relaxed">nama-repo/
└── db-soal/
    ├── kelas_4/
    │   ├── mtk/
    │   │   ├── sts-matematika-2026.md
    │   │   └── gambar/
    │   │       └── soal-12.jpg
    │   └── ipas/
    │       └── sts-ipas-2026.md
    ├── kelas_5/
    └── kelas_6/</pre>
						<div class="space-y-2 text-sm text-muted">
							<p>
								<span class="font-medium text-default">Nama folder kelas</span> harus <code>kelas_4</code>, <code>kelas_5</code>, dst.
							</p>
							<p>
								<span class="font-medium text-default">Nama folder mapel</span> sebaiknya kode mapel huruf kecil
								(MTK → <code>mtk</code>, B.Indo → <code>b-indo</code>) supaya otomatis cocok. Kalau berbeda, sekolah
								tinggal memilih mapel tujuan secara manual saat impor.
							</p>
							<p>
								<span class="font-medium text-default">Satu file .md = satu jenis ujian.</span> Format paling aman: hasil Export dari NILUJI.
								File bisa juga ditulis manual mengikuti contoh format.
							</p>
						</div>
					</div>

					<div class="flex flex-wrap gap-2">
						<UButton
							icon="lucide:download"
							variant="soft"
							@click="downloadTemplate('/templates/contoh-soal-markdown.md', 'contoh-soal-markdown.md', 'Contoh format Markdown')">
							Unduh Contoh Format (.md)
						</UButton>
						<UButton
							icon="lucide:external-link"
							variant="outline"
							color="neutral"
							@click="openUrl('https://github.com/new')">
							Buat Repo di GitHub
						</UButton>
					</div>
				</div>
			</template>
		</UCollapsible>

		<div class="grid gap-4 md:grid-cols-4 items-end">
			<UFormField label="Kelas">
				<USelectMenu
					:model-value="selectedClass"
					:items="onlineClasses"
					value-key="value"
					:loading="loadingClasses"
					:disabled="!provider"
					placeholder="Pilih kelas"
					@update:model-value="selectClass" />
			</UFormField>

			<UFormField label="Mata Pelajaran" :description="selectedClass && !loadingSubjectFolders && !subjectItems.length ? 'Belum ada folder mapel di kelas ini.' : undefined">
				<USelectMenu
					:model-value="selectedSubject"
					:items="subjectItems"
					value-key="value"
					:loading="loadingSubjectFolders"
					:disabled="!selectedClass"
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
			<div class="text-sm text-muted">
				Sumber: <code>{{ sourceFolder }}/{{ sourceFile }}</code> · Jenis: <UBadge color="neutral" variant="subtle">{{ jenisSoal }}</UBadge>
				<span class="mx-2">·</span>
				{{ rows.length }} baris, {{ validCount }} valid, {{ rows.length - validCount }} invalid
			</div>

			<div class="grid gap-4 md:grid-cols-3 items-end">
				<UFormField
					label="Simpan ke Mata Pelajaran"
					:description="targetSubjectId ? 'Dicocokkan otomatis dari nama folder — ganti kalau keliru.' : 'Folder ini tidak cocok dengan mapel mana pun — pilih manual.'">
					<USelectMenu
						v-model="targetSubjectId"
						:items="localSubjectItems"
						value-key="value"
						:loading="loadingSubjects"
						placeholder="Pilih mata pelajaran"
						class="w-full" />
				</UFormField>
				<UFormField label="Jenis Soal">
					<UInput v-model="jenisSoal" class="w-full" />
				</UFormField>
				<UButton
					icon="lucide:database"
					:loading="importing"
					:disabled="!selectedCount || !targetSubjectId"
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
			<p>Pilih kelas, mata pelajaran, dan jenis ujian, lalu ambil bank soal untuk melihat preview.</p>
		</div>
	</div>
</template>
