<script lang="ts" setup>
	import { invoke } from "@tauri-apps/api/core";
	import type { HasilSessionRow } from "~/composables/useHasilExport";

	definePageMeta({
		hideFromNav: true
	});

	interface Subject {
		id: number
		name: string
		code: string | null
	}

	interface QuestionTypeRecord {
		id: number
		name: string
		class: string | null
		subjectId: number | null
	}

	const route = useRoute();
	const kelas = computed(() => route.params.kelas as string);

	const breadcrumbItems = computed(() => [
		{ label: "Hasil Ujian", icon: "lucide:bar-chart-3", to: "/hasil" },
		{ label: `Kelas ${kelas.value}` }
	]);

	type SessionWithId = HasilSessionRow & { sessionId: number };

	const subjects = ref<Subject[]>([]);
	const questionTypes = ref<QuestionTypeRecord[]>([]);
	const allSessions = ref<SessionWithId[]>([]);
	const sessions = computed(() => allSessions.value.filter((s) => s.class === kelas.value));

	onMounted(async () => {
		[subjects.value, questionTypes.value, allSessions.value] = await Promise.all([
			invoke<Subject[]>("list_subjects"),
			// Tanpa class/subjectId, backend mengembalikan SEMUA jenis ujian terdaftar (tidak
			// difilter) — pencocokan per pelajaran+kelas dilakukan di HasilPelajaranGrid.
			invoke<QuestionTypeRecord[]>("list_question_types", { class: null, subjectId: null }),
			invoke<SessionWithId[]>("list_exam_sessions")
		]);
	});

	// ---------- Export tingkat kelas: gabungan semua mapel + jenis ujian di kelas ini ----------

	const { exportZipOfXlsx, exportStudentDetailZip, safeName } = useHasilExport();

	const exportAllZip = async () => {
		const byGroup = new Map<string, HasilSessionRow[]>();
		for (const s of sessions.value) {
			const key = `${s.subject} - ${s.jenis ?? "Tanpa Jenis"}`;
			if (!byGroup.has(key)) byGroup.set(key, []);
			byGroup.get(key)!.push(s);
		}
		const files = [...byGroup.entries()].map(([name, rows]) => ({ name: safeName(name), sessions: rows }));
		await exportZipOfXlsx(`hasil-kelas-${kelas.value}.zip`, files);
	};

	const exportAllStudentsZip = async () => {
		const byStudent = new Map<string, SessionWithId[]>();
		for (const s of sessions.value) {
			if (!byStudent.has(s.studentName)) byStudent.set(s.studentName, []);
			byStudent.get(s.studentName)!.push(s);
		}
		const students = [...byStudent.entries()].map(([name, rows]) => ({
			name,
			sessions: rows.map((s) => ({ sessionId: s.sessionId, jenis: s.jenis }))
		}));
		await exportStudentDetailZip(`hasil-kelas-${kelas.value}-per-siswa.zip`, students);
	};

	const exportMenuItems = [[
		{ label: "Export Semua (per Mapel+Jenis, .zip)", icon: "lucide:download", onSelect: exportAllZip },
		{ label: "Export per Siswa (Semua, .zip)", icon: "lucide:users", onSelect: exportAllStudentsZip }
	]];
</script>

<template>
	<UDashboardPanel id="hasil-kelas">
		<template #header>
			<UDashboardNavbar :title="`Kelas ${kelas}`">
				<template #leading>
					<UDashboardSidebarCollapse />
					<UButton icon="lucide:arrow-left" variant="ghost" to="/hasil" />
				</template>

				<template #right>
					<UDropdownMenu :items="exportMenuItems">
						<UButton
							icon="lucide:download"
							variant="soft"
							trailing-icon="lucide:chevron-down"
							:disabled="sessions.length === 0">
							Export Kelas
						</UButton>
					</UDropdownMenu>
				</template>
			</UDashboardNavbar>
		</template>

		<template #body>
			<UBreadcrumb :items="breadcrumbItems" class="mb-4" />
			<HasilPelajaranGrid
				:kelas="kelas"
				:subjects="subjects"
				:question-types="questionTypes"
				:sessions="sessions" />
		</template>
	</UDashboardPanel>
</template>
