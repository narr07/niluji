<script lang="ts" setup>
	import { invoke } from "@tauri-apps/api/core";
	import type { HasilSessionRow } from "~/composables/useHasilExport";

	definePageMeta({
		hideFromNav: true
	});

	type SessionProgress = HasilSessionRow & {
		sessionId: number
		extraMinutes?: number
		deviceLocked?: boolean
	};

	const route = useRoute();
	const toast = useToast();
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

	const unlockDevice = async (sessionId: number, studentName: string) => {
		await invoke("unlock_exam_session_device", { sessionId });
		toast.add({
			title: "Perangkat dibuka",
			description: `${studentName} kini dapat melanjutkan ujian dari perangkat baru.`,
			icon: "lucide:smartphone",
			color: "success"
		});
		await refresh();
	};

	const addTime = async (sessionId: number, studentName: string, minutes: number) => {
		const totalExtra = await invoke<number>("add_exam_session_time", { sessionId, minutes });
		toast.add({
			title: "Waktu ditambahkan",
			description: `${studentName} mendapat tambahan +${minutes} menit (total +${totalExtra} menit).`,
			icon: "lucide:clock-plus",
			color: "info"
		});
		await refresh();
	};

	const forceSubmit = async (sessionId: number, studentName: string) => {
		const ok = await confirmDelete({
			title: `Paksa kumpulkan ujian ${studentName}?`,
			description: "Sesi siswa ini akan langsung dikunci dan dinilai sesuai jawaban yang sudah masuk.",
			confirmLabel: "Paksa Selesai"
		});
		if (!ok) return;
		await invoke("force_submit_exam_session", { sessionId });
		toast.add({
			title: "Ujian dikumpulkan",
			description: `Sesi ujian ${studentName} telah diselesaikan oleh pengawas.`,
			icon: "lucide:check-circle",
			color: "success"
		});
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
			@reset="resetSession"
			@unlock-device="unlockDevice"
			@add-time="addTime"
			@force-submit="forceSubmit" />
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
