<script lang="ts" setup>
	import { invoke } from "@tauri-apps/api/core";

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

	interface SekolahInfo {
		sekolah_id: string
		nama: string
		npsn: string
	}

	interface SemesterInfo {
		semester_id: string
		nama: string
		tahun_ajaran: string
	}

	interface RombelInfo {
		rombongan_belajar_id: string
		nama_rombel: string
		tingkat: string
		total_siswa: number
		siswa_ada_nisn: number
	}

	interface SiswaDapodikRow {
		nisn: string
		name: string
		tingkat: string
		nama_rombel: string
	}

	const isTauri = () => typeof window !== "undefined" && !!(window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;

	const toast = useToast();
	const db = useDb();

	const loading = ref(false);
	const importing = ref(false);
	const errorMessage = ref("");

	const sekolah = ref<SekolahInfo | null>(null);
	const semester = ref<SemesterInfo | null>(null);
	const rombels = ref<RombelInfo[]>([]);
	const selectedRombelIds = ref<Set<string>>(new Set());
	const formatKelas = ref<"tingkat" | "nama_rombel">("tingkat");
	const skipNoNisn = ref(true);

	const totalSiswaTerpilih = computed(() => {
		let total = 0;
		for (const r of rombels.value) {
			if (selectedRombelIds.value.has(r.rombongan_belajar_id)) {
				total += skipNoNisn.value ? r.siswa_ada_nisn : r.total_siswa;
			}
		}
		return total;
	});

	const totalSiswaTanpaNisn = computed(() => {
		let total = 0;
		for (const r of rombels.value) {
			if (selectedRombelIds.value.has(r.rombongan_belajar_id)) {
				total += (r.total_siswa - r.siswa_ada_nisn);
			}
		}
		return total;
	});

	const isAllSelected = computed(() => rombels.value.length > 0 && selectedRombelIds.value.size === rombels.value.length);

	const toggleSelectAll = () => {
		if (isAllSelected.value) {
			selectedRombelIds.value.clear();
		} else {
			selectedRombelIds.value = new Set(rombels.value.map(r => r.rombongan_belajar_id));
		}
	};

	const toggleRombel = (id: string) => {
		const next = new Set(selectedRombelIds.value);
		if (next.has(id)) {
			next.delete(id);
		} else {
			next.add(id);
		}
		selectedRombelIds.value = next;
	};

	const loadEraporData = async () => {
		errorMessage.value = "";
		loading.value = true;
		sekolah.value = null;
		semester.value = null;
		rombels.value = [];
		selectedRombelIds.value.clear();

		if (!isTauri()) {
			// Mock data untuk mode pratinjau browser
			sekolah.value = { sekolah_id: "demo", nama: "SD NEGERI 01 MERDEKA", npsn: "10203040" };
			semester.value = { semester_id: "20241", nama: "2024/2025 Ganjil", tahun_ajaran: "2024/2025" };
			rombels.value = [
				{ rombongan_belajar_id: "r1", nama_rombel: "Kelas 1", tingkat: "1", total_siswa: 25, siswa_ada_nisn: 25 },
				{ rombongan_belajar_id: "r2", nama_rombel: "Kelas 2", tingkat: "2", total_siswa: 27, siswa_ada_nisn: 27 },
				{ rombongan_belajar_id: "r3", nama_rombel: "Kelas 3", tingkat: "3", total_siswa: 26, siswa_ada_nisn: 26 },
				{ rombongan_belajar_id: "r4", nama_rombel: "Kelas 4", tingkat: "4", total_siswa: 28, siswa_ada_nisn: 28 },
				{ rombongan_belajar_id: "r5", nama_rombel: "Kelas 5", tingkat: "5", total_siswa: 30, siswa_ada_nisn: 30 },
				{ rombongan_belajar_id: "r6", nama_rombel: "Kelas 6", tingkat: "6", total_siswa: 29, siswa_ada_nisn: 29 }
			];
			selectedRombelIds.value = new Set(["r4", "r5", "r6"]);
			loading.value = false;
			return;
		}

		try {
			// 1. Ambil info sekolah
			const sek = await db.first<SekolahInfo>("SELECT sekolah_id, nama, npsn FROM sekolah LIMIT 1");
			sekolah.value = sek ?? null;

			// 2. Ambil semester aktif
			let sem = await db.first<SemesterInfo>("SELECT semester_id, nama, tahun_ajaran FROM semester WHERE aktif = 1 LIMIT 1");
			if (!sem) {
				sem = await db.first<SemesterInfo>("SELECT semester_id, nama, tahun_ajaran FROM semester ORDER BY semester_id DESC LIMIT 1");
			}
			semester.value = sem ?? null;

			if (!sem) {
				loading.value = false;
				return;
			}

			// 3. Ambil daftar rombel beserta jumlah siswa & jumlah yang memiliki NISN
			const list = await db.query<RombelInfo>(
				`SELECT 
					r.rombongan_belajar_id,
					r.nama AS nama_rombel,
					r.tingkat,
					COUNT(ar.anggota_rombel_id) AS total_siswa,
					COUNT(CASE WHEN pd.nisn IS NOT NULL AND TRIM(pd.nisn) != '' THEN 1 END) AS siswa_ada_nisn
				FROM rombel r
				LEFT JOIN anggota_rombel ar ON ar.rombongan_belajar_id = r.rombongan_belajar_id AND ar.semester_id = r.semester_id
				LEFT JOIN peserta_didik pd ON pd.peserta_didik_id = ar.peserta_didik_id
				WHERE r.semester_id = ?
				GROUP BY r.rombongan_belajar_id, r.nama, r.tingkat
				ORDER BY CAST(r.tingkat AS INTEGER), r.nama`,
				[sem.semester_id]
			);

			rombels.value = list;

			// Otomatis centang kelas ujian 4, 5, 6 jika ada, atau seluruh rombel
			const defaultSelected = new Set<string>();
			for (const r of list) {
				if (["4", "5", "6"].includes(String(r.tingkat).trim()) || list.length <= 3) {
					defaultSelected.add(r.rombongan_belajar_id);
				}
			}
			if (defaultSelected.size === 0) {
				for (const r of list) defaultSelected.add(r.rombongan_belajar_id);
			}
			selectedRombelIds.value = defaultSelected;
		} catch (e) {
			errorMessage.value = e instanceof Error ? e.message : String(e);
		} finally {
			loading.value = false;
		}
	};

	watch(
		() => props.open,
		(isOpen) => {
			if (isOpen) {
				loadEraporData();
			}
		}
	);

	const doImport = async () => {
		if (selectedRombelIds.value.size === 0) {
			toast.add({ title: "Pilih Rombel", description: "Pilih minimal 1 rombongan belajar.", color: "warning" });
			return;
		}

		if (!isTauri()) {
			toast.add({ title: "Mode Pratinjau", description: "Impor dijalankan di aplikasi desktop.", color: "info" });
			openModel.value = false;
			return;
		}

		importing.value = true;
		errorMessage.value = "";

		try {
			const idList = Array.from(selectedRombelIds.value);
			const placeholders = idList.map(() => "?").join(",");

			const rows = await db.query<SiswaDapodikRow>(
				`SELECT 
					pd.nisn,
					pd.nama AS name,
					r.tingkat,
					r.nama AS nama_rombel
				FROM anggota_rombel ar
				JOIN peserta_didik pd ON pd.peserta_didik_id = ar.peserta_didik_id
				JOIN rombel r ON r.rombongan_belajar_id = ar.rombongan_belajar_id AND r.semester_id = ar.semester_id
				WHERE ar.semester_id = ? AND r.rombongan_belajar_id IN (${placeholders})
				ORDER BY CAST(r.tingkat AS INTEGER), r.nama, pd.nama`,
				[semester.value?.semester_id, ...idList]
			);

			const validRows = rows.filter(r => {
				const hasNisn = Boolean(r.nisn && r.nisn.trim());
				const hasName = Boolean(r.name && r.name.trim());
				return skipNoNisn.value ? (hasNisn && hasName) : hasName;
			});

			if (validRows.length === 0) {
				throw new Error("Tidak ada siswa valid untuk diimpor. Pastikan siswa memiliki NISN.");
			}

			const payload = validRows.map(r => {
				const assignedClass = formatKelas.value === "tingkat"
					? (r.tingkat || r.nama_rombel).trim()
					: r.nama_rombel.trim();

				return {
					nisn: (r.nisn || "").trim(),
					name: r.name.trim(),
					class: assignedClass,
					school: sekolah.value?.nama?.trim()
				};
			});

			const summary = await invoke<{ studentsImported: number }>("import_students_rows", {
				rows: payload
			});

			toast.add({
				title: "Tarik Data Berhasil",
				description: `${summary.studentsImported} siswa dari ${selectedRombelIds.value.size} rombel berhasil ditarik ke CBT.`,
				icon: "lucide:check-circle",
				color: "success"
			});

			emit("imported");
			openModel.value = false;
		} catch (e) {
			errorMessage.value = e instanceof Error ? e.message : String(e);
		} finally {
			importing.value = false;
		}
	};
</script>

<template>
	<UModal v-model:open="openModel" title="Tarik Data Siswa dari e-Rapor / Dapodik" :ui="{ content: 'max-w-2xl' }">
		<template #body>
			<div class="space-y-4">
				<p class="text-xs text-muted">
					Mengambil data siswa resmi langsung dari database e-Rapor SD lokal yang sudah tersinkronisasi dengan Dapodik. Proses ini berlangsung 100% offline dan instan.
				</p>

				<!-- Loading State -->
				<div v-if="loading" class="py-12 flex flex-col items-center justify-center gap-3">
					<UIcon name="lucide:loader-2" class="size-8 animate-spin text-primary" />
					<p class="text-sm text-muted">Membaca data siswa dari e-Rapor...</p>
				</div>

				<!-- Database Kosong / Belum Sinkron -->
				<div v-else-if="!rombels.length" class="rounded-xl border border-warning/30 bg-warning/5 p-5 text-center space-y-3">
					<div class="mx-auto flex size-12 items-center justify-center rounded-full bg-warning/10 text-warning">
						<UIcon name="lucide:database-zap" class="size-6" />
					</div>
					<div>
						<h3 class="font-semibold text-sm">Data e-Rapor Masih Kosong</h3>
						<p class="text-xs text-muted max-w-md mx-auto mt-1">
							Belum ada data sekolah atau rombel di database e-Rapor lokal. Silakan hubungkan dengan Web Service Dapodik terlebih dahulu.
						</p>
					</div>
					<div class="pt-2">
						<UButton
							to="/e-rapor/sinkron"
							icon="lucide:refresh-cw"
							color="primary"
							size="sm"
							@click="openModel = false"
						>
							Buka Halaman Sinkron Dapodik
						</UButton>
					</div>
				</div>

				<!-- Data Tersedia -->
				<div v-else class="space-y-4">
					<!-- Info Box Sekolah & Semester -->
					<div class="rounded-lg border border-default bg-elevated/50 p-3.5 flex flex-wrap items-center justify-between gap-3 text-xs">
						<div class="space-y-0.5">
							<div class="font-semibold text-sm flex items-center gap-2">
								<UIcon name="lucide:school" class="size-4 text-primary" />
								<span>{{ sekolah?.nama || "Sekolah Belum Dinamai" }}</span>
								<UBadge v-if="sekolah?.npsn" variant="subtle" size="xs">
									NPSN: {{ sekolah.npsn }}
								</UBadge>
							</div>
							<div class="text-muted flex items-center gap-1.5">
								<UIcon name="lucide:calendar" class="size-3.5" />
								<span>Semester: {{ semester?.nama || semester?.semester_id || "-" }}</span>
							</div>
						</div>
						<div class="text-right">
							<div class="text-xs text-muted">Total Terpilih</div>
							<div class="text-base font-bold text-primary">{{ totalSiswaTerpilih }} Siswa</div>
						</div>
					</div>

					<!-- Pengaturan Format Nama Kelas -->
					<div class="rounded-lg border border-default p-3 space-y-2">
						<label class="text-xs font-semibold block text-muted">Format Nama Kelas di CBT:</label>
						<div class="grid grid-cols-1 sm:grid-cols-2 gap-2 text-xs">
							<label
								class="flex items-center gap-2.5 p-2 rounded-md border cursor-pointer transition-colors"
								:class="formatKelas === 'tingkat' ? 'border-primary bg-primary/5 font-medium' : 'border-default hover:bg-elevated'"
							>
								<input
									v-model="formatKelas"
									type="radio"
									value="tingkat"
									class="text-primary" >
								<div>
									<div>Tingkat Angka Saja</div>
									<div class="text-[11px] text-muted">Contoh: "4", "5", "6"</div>
								</div>
							</label>

							<label
								class="flex items-center gap-2.5 p-2 rounded-md border cursor-pointer transition-colors"
								:class="formatKelas === 'nama_rombel' ? 'border-primary bg-primary/5 font-medium' : 'border-default hover:bg-elevated'"
							>
								<input
									v-model="formatKelas"
									type="radio"
									value="nama_rombel"
									class="text-primary" >
								<div>
									<div>Nama Rombel Lengkap</div>
									<div class="text-[11px] text-muted">Contoh: "Kelas 4A", "5B", "Kelas 6"</div>
								</div>
							</label>
						</div>
					</div>

					<!-- Checklist Rombongan Belajar -->
					<div class="space-y-2">
						<div class="flex items-center justify-between text-xs">
							<span class="font-semibold text-muted">Pilih Rombongan Belajar (Kelas):</span>
							<UButton
								variant="link"
								size="xs"
								:label="isAllSelected ? 'Batal Pilih Semua' : 'Pilih Semua Rombel'"
								@click="toggleSelectAll"
							/>
						</div>

						<div class="grid grid-cols-2 sm:grid-cols-3 gap-2 max-h-48 overflow-y-auto p-1">
							<div
								v-for="r in rombels"
								:key="r.rombongan_belajar_id"
								class="flex items-center justify-between p-2.5 rounded-lg border text-xs cursor-pointer select-none transition-colors"
								:class="selectedRombelIds.has(r.rombongan_belajar_id) ? 'border-primary bg-primary/5' : 'border-default hover:bg-elevated/40'"
								@click="toggleRombel(r.rombongan_belajar_id)"
							>
								<div class="flex items-center gap-2 truncate">
									<UCheckbox
										:model-value="selectedRombelIds.has(r.rombongan_belajar_id)"
										@click.stop
										@update:model-value="toggleRombel(r.rombongan_belajar_id)"
									/>
									<span class="font-medium truncate">{{ r.nama_rombel }}</span>
								</div>
								<UBadge variant="subtle" size="xs" color="neutral">
									{{ r.total_siswa }}
								</UBadge>
							</div>
						</div>
					</div>

					<!-- Warning jika ada siswa tanpa NISN -->
					<div v-if="totalSiswaTanpaNisn > 0" class="flex items-center justify-between gap-2 p-2.5 rounded-lg bg-warning/10 text-xs text-warning border border-warning/20">
						<div class="flex items-center gap-2">
							<UIcon name="lucide:alert-circle" class="size-4 shrink-0" />
							<span>Ditemukan {{ totalSiswaTanpaNisn }} siswa tanpa NISN di rombel yang dipilih.</span>
						</div>
						<label class="flex items-center gap-1.5 text-[11px] font-medium cursor-pointer shrink-0">
							<input v-model="skipNoNisn" type="checkbox" class="rounded text-primary" >
							<span>Lewati tanpa NISN</span>
						</label>
					</div>

					<!-- Error Message -->
					<UAlert
						v-if="errorMessage"
						color="error"
						variant="subtle"
						title="Gagal Menarik Siswa"
						:description="errorMessage"
					/>
				</div>
			</div>
		</template>

		<template #footer>
			<div class="flex items-center justify-between w-full">
				<UButton
					variant="ghost"
					color="neutral"
					label="Batal"
					@click="openModel = false"
				/>

				<UButton
					v-if="rombels.length > 0"
					icon="lucide:download"
					color="primary"
					:loading="importing"
					:disabled="totalSiswaTerpilih === 0 || loading"
					@click="doImport"
				>
					Tarik {{ totalSiswaTerpilih }} Siswa ke CBT
				</UButton>
			</div>
		</template>
	</UModal>
</template>
