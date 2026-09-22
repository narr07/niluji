<script lang="ts" setup>
	import { invoke } from "@tauri-apps/api/core";
	import type { HasilSessionRow } from "~/composables/useHasilExport";

	definePageMeta({
		hideFromNav: true
	});

	type SessionProgress = HasilSessionRow & { sessionId: number };

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

	const { exportOneXlsx, exportStudentDetailXlsx, exportStudentDetailZip } = useHasilExport();

	const exportAll = async () => {
		await exportOneXlsx(`hasil-kelas-${kelas.value}-${pelajaran.value}-${jenis.value}.xlsx`, sessions.value);
	};

	// ---------- Export per siswa (satu siswa terpilih) — rincian per soal, bukan cuma skor akhir ----------

	const studentExportOpen = ref(false);
	const studentOptions = computed(() => [...new Set(sessions.value.map((s) => s.studentName))].sort());
	const studentToExport = ref("");

	const exportStudent = async () => {
		if (!studentToExport.value) return;
		const rows = sessions.value.filter((s) => s.studentName === studentToExport.value);
		await exportStudentDetailXlsx(
			`hasil-${studentToExport.value}-${kelas.value}-${pelajaran.value}-${jenis.value}.xlsx`,
			rows.map((s) => ({ sessionId: s.sessionId, jenis: s.jenis }))
		);
		studentExportOpen.value = false;
	};

	// ---------- Export per siswa, semua siswa sekaligus (rincian per soal, satu file .xlsx per siswa, dibundel .zip) ----------

	const exportAllStudents = async () => {
		const byStudent = new Map<string, SessionProgress[]>();
		for (const s of sessions.value) {
			if (!byStudent.has(s.studentName)) byStudent.set(s.studentName, []);
			byStudent.get(s.studentName)!.push(s);
		}
		const students = [...byStudent.entries()].map(([name, rows]) => ({
			name,
			sessions: rows.map((s) => ({ sessionId: s.sessionId, jenis: s.jenis }))
		}));
		await exportStudentDetailZip(`hasil-per-siswa-${kelas.value}-${pelajaran.value}-${jenis.value}.zip`, students);
	};

	const exportMenuItems = [[
		{ label: "Export Semua", icon: "lucide:download", onSelect: exportAll },
		{ label: "Export per Siswa...", icon: "lucide:user", onSelect: () => (studentExportOpen.value = true) },
		{ label: "Export per Siswa (Semua, .zip)", icon: "lucide:users", onSelect: exportAllStudents }
	]];

	const resetSession = async (sessionId: number, studentName: string) => {
		const ok = await confirmDelete({
			title: `Reset ujian ${studentName}?`,
			description: "Semua jawaban yang sudah diisi akan dihapus.",
			confirmLabel: "Reset"
		});
		if (!ok) return;
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
				<UButton
					icon="lucide:download"
					variant="soft"
					trailing-icon="lucide:chevron-down"
					:disabled="sessions.length === 0">
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
					<USelectMenu
						v-model="studentToExport"
						:items="studentOptions"
						placeholder="Pilih siswa"
						class="w-full" />
				</UFormField>
				<UButton
					icon="lucide:download"
					block
					:disabled="!studentToExport"
					@click="exportStudent">
					Export
				</UButton>
			</div>
		</template>
	</UModal>
</template>
