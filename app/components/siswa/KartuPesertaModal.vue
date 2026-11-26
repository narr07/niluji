<script lang="ts" setup>
	import { invoke } from "@tauri-apps/api/core";

	interface StudentRecord {
		id: number
		nisn: string
		name: string
		class: string | null
		rombel?: string | null
		pin?: string
	}

	interface SchoolInfo {
		name: string
		npsn: string
		address: string
		principal: string
	}

	const props = defineProps<{
		open: boolean
		students: StudentRecord[]
		defaultClass?: string
	}>();

	const emit = defineEmits<{
		"update:open": [value: boolean]
		regenerated: []
	}>();

	const openModel = computed({
		get: () => props.open,
		set: (v) => emit("update:open", v)
	});

	const isTauri = () => typeof window !== "undefined" && !!(window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;
	const toast = useToast();

	const school = ref<SchoolInfo>({ name: "Sekolah", npsn: "", address: "", principal: "" });
	const filterClass = ref(props.defaultClass || "ALL");
	const regenerating = ref(false);
	const printing = ref(false);
	const legacyPinCount = ref(0);

	const loadLegacyPinCount = async () => {
		legacyPinCount.value = isTauri() ? await invoke<number>("count_legacy_student_pins") : 0;
	};

	const classOptions = computed(() => {
		const unique = [...new Set(props.students.map((s) => s.class).filter((c): c is string => !!c))].sort();
		return [{ label: "Semua Kelas", value: "ALL" }, ...unique.map((c) => ({ label: `Kelas ${c}`, value: c }))];
	});

	const filteredStudents = computed(() => {
		if (filterClass.value === "ALL") return props.students;
		return props.students.filter((s) => s.class === filterClass.value);
	});

	const pageCount = computed(() => Math.ceil(filteredStudents.value.length / KARTU_PER_HALAMAN));

	// Kartu sengaja tidak memuat alamat server atau QR-nya: alamat IP laptop pengawas bisa berubah
	// (router dinyalakan ulang, pindah jaringan), sehingga kartu yang sudah dicetak akan salah.
	// Siswa memakai alamat atau QR Akses Ujian yang ditampilkan pengawas dari Dashboard.
	const loadMeta = async () => {
		if (!isTauri()) {
			school.value = { name: "SDN Percobaan", npsn: "12345678", address: "", principal: "" };
			return;
		}
		school.value = await invoke<SchoolInfo>("get_school");
		await loadLegacyPinCount();
	};

	watch(
		() => props.open,
		(open) => {
			if (open) {
				filterClass.value = props.defaultClass || "ALL";
				void loadMeta();
			}
		},
		{ immediate: true }
	);

	// Peringatan "ya/tidak" dipasang sebagai modal bertingkat DI DALAM modal ini. Dialog konfirmasi
	// global (confirmDelete) dirender di luar modal yang sedang terbuka, sehingga tidak bisa diklik
	// dan Acak Ulang PIN terlihat tidak berfungsi.
	const confirmRegenOpen = ref(false);
	const regenTargetLabel = computed(() => (filterClass.value === "ALL" ? "semua kelas" : `Kelas ${filterClass.value}`));

	const askRegeneratePins = () => {
		if (!isTauri()) return;
		confirmRegenOpen.value = true;
	};

	const regeneratePins = async () => {
		regenerating.value = true;
		try {
			const count = await invoke<number>("regenerate_student_pins", {
				class: filterClass.value === "ALL" ? null : filterClass.value
			});
			confirmRegenOpen.value = false;
			toast.add({
				title: "PIN diperbarui",
				description: `${count} PIN siswa berhasil diacak ulang. Cetak ulang kartunya sebelum ujian.`,
				icon: "lucide:check",
				color: "success"
			});
			await loadLegacyPinCount();
			emit("regenerated");
		} catch (e) {
			toast.add({ title: "Gagal mengacak ulang PIN", description: e instanceof Error ? e.message : String(e), icon: "lucide:x", color: "error" });
		} finally {
			regenerating.value = false;
		}
	};

	// Kartu dicetak dari dokumen terpisah di iframe tersembunyi, bukan dari jendela aplikasi, supaya
	// yang tercetak hanya lembar kartu dengan ukuran A4 yang pasti.
	const printCards = () => {
		if (typeof window === "undefined" || !filteredStudents.value.length) return;
		printing.value = true;
		const frame = document.createElement("iframe");
		frame.setAttribute("aria-hidden", "true");
		frame.style.cssText = "position:fixed;right:0;bottom:0;width:0;height:0;border:0;";
		document.body.appendChild(frame);

		const cleanup = () => {
			frame.remove();
			printing.value = false;
		};
		const doc = frame.contentDocument;
		const win = frame.contentWindow;
		if (!doc || !win) {
			cleanup();
			return;
		}
		doc.open();
		doc.write(buildKartuPesertaHtml(filteredStudents.value, school.value.name || "NILUJI"));
		doc.close();
		win.addEventListener("afterprint", cleanup, { once: true });
		// Beri waktu dokumen selesai ditata sebelum dialog cetak dibuka.
		setTimeout(() => {
			win.focus();
			win.print();
			setTimeout(cleanup, 60_000);
		}, 250);
	};
</script>

<template>
	<UModal
		v-model:open="openModel"
		title="Kartu Peserta Ujian & PIN Siswa"
		description="Kartu berisi nama, NISN, kelas, dan PIN. Alamat ujian ditampilkan pengawas dari Dashboard."
		:ui="{ content: 'max-w-4xl' }">
		<template #body>
			<div class="space-y-4">
				<UAlert
					v-if="legacyPinCount"
					color="warning"
					variant="subtle"
					icon="lucide:shield-alert"
					:title="`${legacyPinCount} siswa masih memakai PIN versi lama`"
					description="PIN ini bisa ditebak dari NISN. Klik Acak Ulang PIN untuk semua kelas, lalu cetak ulang kartu sebelum ujian berikutnya." />

				<div class="flex flex-wrap items-center justify-between gap-3">
					<div class="flex items-center gap-2">
						<USelectMenu
							v-if="!defaultClass"
							v-model="filterClass"
							:items="classOptions"
							value-key="value"
							class="w-44" />
						<span class="text-sm text-muted">
							{{ filteredStudents.length }} kartu · {{ pageCount }} lembar A4
						</span>
					</div>
					<div class="flex items-center gap-2">
						<UButton
							variant="soft"
							color="warning"
							icon="lucide:key-round"
							:loading="regenerating"
							@click="askRegeneratePins">
							Acak Ulang PIN
						</UButton>
						<UButton
							color="primary"
							icon="lucide:printer"
							:loading="printing"
							:disabled="!filteredStudents.length"
							@click="printCards">
							Cetak Kartu
						</UButton>
					</div>
				</div>

				<p class="text-xs text-muted">
					Dicetak di A4, 10 kartu per lembar, berukuran sama (9,7 × 5,6 cm). Garis kotak dipakai bersama oleh kartu bersebelahan, jadi satu guntingan memisahkan dua kartu.
				</p>

				<div
					v-if="filteredStudents.length"
					class="grid grid-cols-1 sm:grid-cols-2 max-h-[65vh] overflow-y-auto border-t border-l border-default">
					<div
						v-for="student in filteredStudents"
						:key="student.id"
						class="border-r border-b border-default p-4 flex flex-col gap-2">
						<div>
							<p class="text-[11px] font-bold uppercase tracking-wider text-muted">
								Kartu Peserta Ujian
							</p>
							<p class="text-sm font-semibold text-highlighted truncate">
								{{ school.name || "NILUJI" }}
							</p>
						</div>
						<p class="text-lg font-bold text-highlighted leading-snug line-clamp-2">
							{{ student.name }}
						</p>
						<div class="flex gap-6 text-sm">
							<div>
								<p class="text-[11px] uppercase tracking-wider text-muted">
									NISN
								</p>
								<p class="font-mono font-semibold">
									{{ student.nisn }}
								</p>
							</div>
							<div>
								<p class="text-[11px] uppercase tracking-wider text-muted">
									Kelas
								</p>
								<p>{{ labelKelas(student.class, student.rombel) }}</p>
							</div>
						</div>
						<div>
							<p class="text-[11px] uppercase tracking-wider text-muted">
								PIN
							</p>
							<p class="font-mono text-2xl font-bold tracking-[0.2em] text-highlighted">
								{{ student.pin || "-" }}
							</p>
						</div>
					</div>
				</div>

				<div v-else class="text-center py-10 text-muted">
					Belum ada data siswa untuk dicetak.
				</div>

				<UModal
					v-model:open="confirmRegenOpen"
					:title="`Acak ulang PIN ${regenTargetLabel}?`"
					:dismissible="!regenerating"
					:ui="{ content: 'max-w-md', footer: 'justify-end' }">
					<template #body>
						<UAlert
							color="warning"
							variant="subtle"
							icon="lucide:triangle-alert"
							title="PIN lama tidak berlaku lagi"
							:description="`${filteredStudents.length} siswa akan mendapat PIN baru. Kartu yang sudah dibagikan harus dicetak ulang, atau siswa tidak bisa masuk ke ujian yang mewajibkan PIN.`" />
					</template>
					<template #footer>
						<UButton
							label="Tidak"
							color="neutral"
							variant="outline"
							:disabled="regenerating"
							@click="confirmRegenOpen = false" />
						<UButton
							label="Ya, acak ulang"
							color="warning"
							icon="lucide:key-round"
							:loading="regenerating"
							@click="regeneratePins" />
					</template>
				</UModal>
			</div>
		</template>
	</UModal>
</template>
