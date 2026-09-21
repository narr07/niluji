<script lang="ts" setup>
	import { invoke } from "@tauri-apps/api/core";
	import { save as saveDialog } from "@tauri-apps/plugin-dialog";
	import * as XLSX from "xlsx";

	definePageMeta({
		hideFromNav: true
	});

	const toast = useToast();

	interface SessionProgress {
		sessionId: number
		studentName: string
		studentNisn: string
		examTitle: string
		class: string | null
		subject: string
		jenis: string | null
		totalQuestions: number
		answeredCount: number
		pgTotal: number
		pgAnswered: number
		submittedAt: number | null
		score: number | null
	}

	const route = useRoute();
	const kelas = computed(() => route.params.kelas as string);
	const pelajaran = computed(() => decodeURIComponent(route.params.pelajaran as string));
	const jenis = computed(() => decodeURIComponent(route.params.jenis as string));

	const allSessions = ref<SessionProgress[]>([]);
	const sessions = computed(() =>
		allSessions.value.filter((s) => s.class === kelas.value && s.subject === pelajaran.value && s.jenis === jenis.value)
	);

	const refresh = async () => {
		allSessions.value = await invoke<SessionProgress[]>("list_exam_sessions");
	};

	// Pakai dialog save (bukan trigger download browser biasa) supaya guru pilih sendiri lokasi
	// filenya, dan kita bisa kasih tahu persis ke folder mana filenya tersimpan lewat toast.
	const downloadXlsx = async (defaultFilename: string, rows: (string | number)[][]) => {
		const path = await saveDialog({ defaultPath: defaultFilename, filters: [{ name: "Excel", extensions: ["xlsx"] }] });
		if (!path) return;

		const sheet = XLSX.utils.aoa_to_sheet(rows);
		const workbook = XLSX.utils.book_new();
		XLSX.utils.book_append_sheet(workbook, sheet, "Hasil");
		const bytes = XLSX.write(workbook, { type: "array", bookType: "xlsx" }) as ArrayBuffer;
		await invoke("write_file_bytes", { path, bytes: Array.from(new Uint8Array(bytes)) });

		toast.add({
			title: "Export selesai",
			description: `File disimpan di ${path}`,
			icon: "lucide:check",
			color: "success"
		});
	};

	const statusText = (s: SessionProgress) => {
		if (s.submittedAt) return "Selesai";
		if (s.pgTotal > 0 && s.pgAnswered >= s.pgTotal && s.totalQuestions > s.pgTotal) return "PG selesai, lanjut esai";
		if (s.pgTotal > 0 && s.pgAnswered < s.pgTotal) return `Mengerjakan PG (${s.pgAnswered}/${s.pgTotal})`;
		return "Sedang mengerjakan";
	};

	const toRow = (s: SessionProgress) => [
		s.studentName,
		s.studentNisn,
		s.examTitle,
		`${s.answeredCount}/${s.totalQuestions}`,
		statusText(s),
		s.score === null ? "-" : s.score.toFixed(0)
	];
	const headerRow = ["Nama", "NISN", "Ujian", "Progress", "Status", "Nilai"];

	const exportAll = async () => {
		await downloadXlsx(`hasil-kelas-${kelas.value}-${pelajaran.value}-${jenis.value}.xlsx`, [headerRow, ...sessions.value.map(toRow)]);
	};

	// ---------- Export per siswa ----------

	const studentExportOpen = ref(false);
	const studentOptions = computed(() => [...new Set(sessions.value.map((s) => s.studentName))].sort());
	const studentToExport = ref("");

	const exportStudent = async () => {
		if (!studentToExport.value) return;
		const rows = sessions.value.filter((s) => s.studentName === studentToExport.value);
		await downloadXlsx(`hasil-${studentToExport.value}-${kelas.value}-${pelajaran.value}-${jenis.value}.xlsx`, [headerRow, ...rows.map(toRow)]);
		studentExportOpen.value = false;
	};

	// ---------- Export bulanan ----------

	const monthExportOpen = ref(false);
	const monthToExport = ref("");

	const exportMonth = async () => {
		if (!monthToExport.value) return;
		const rows = sessions.value.filter((s) => {
			if (!s.submittedAt) return false;
			const d = new Date(s.submittedAt * 1000);
			const ym = `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}`;
			return ym === monthToExport.value;
		});
		await downloadXlsx(`hasil-${monthToExport.value}-${kelas.value}-${pelajaran.value}-${jenis.value}.xlsx`, [headerRow, ...rows.map(toRow)]);
		monthExportOpen.value = false;
	};

	const exportMenuItems = [[
		{ label: "Export Semua", icon: "lucide:download", onSelect: exportAll },
		{ label: "Export per Siswa...", icon: "lucide:user", onSelect: () => (studentExportOpen.value = true) },
		{ label: "Export Bulanan...", icon: "lucide:calendar", onSelect: () => (monthExportOpen.value = true) }
	]];

	const resetSession = async (sessionId: number, studentName: string) => {
		if (!confirm(`Reset ujian ${studentName}? Semua jawaban yang sudah diisi akan dihapus.`)) return;
		await invoke("reset_exam_session", { sessionId });
		await refresh();
	};

	let timer: ReturnType<typeof setInterval>;

	onMounted(() => {
		refresh();
		timer = setInterval(refresh, 3000);
	});

	onUnmounted(() => clearInterval(timer));
</script>

<template>
	<div>
		<div class="flex justify-end mb-4">
			<UDropdownMenu :items="exportMenuItems">
				<UButton icon="lucide:download" variant="soft" trailing-icon="lucide:chevron-down" :disabled="sessions.length === 0">
					Export
				</UButton>
			</UDropdownMenu>
		</div>

		<HasilTable
			:sessions="sessions"
			@detail="(id) => navigateTo(`/hasil/sesi/${id}`)"
			@reset="resetSession" />
	</div>

	<UModal v-model:open="studentExportOpen" title="Export per Siswa">
		<template #body>
			<div class="space-y-4">
				<UFormField label="Pilih Siswa">
					<USelectMenu v-model="studentToExport" :items="studentOptions" placeholder="Pilih siswa" class="w-full" />
				</UFormField>
				<UButton icon="lucide:download" block :disabled="!studentToExport" @click="exportStudent">
					Export
				</UButton>
			</div>
		</template>
	</UModal>

	<UModal v-model:open="monthExportOpen" title="Export Bulanan">
		<template #body>
			<div class="space-y-4">
				<UFormField label="Pilih Bulan" description="Berdasarkan tanggal siswa menyelesaikan ujian">
					<UInput v-model="monthToExport" type="month" class="w-full" />
				</UFormField>
				<UButton icon="lucide:download" block :disabled="!monthToExport" @click="exportMonth">
					Export
				</UButton>
			</div>
		</template>
	</UModal>
</template>
