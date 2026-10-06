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

	const route = useRoute();
	const kelas = computed(() => route.params.kelas as string);

	const breadcrumbItems = computed(() => [
		{ label: "Hasil Ujian", icon: "lucide:bar-chart-3", to: "/hasil" },
		{ label: `Kelas ${kelas.value}` }
	]);

	type SessionWithId = HasilSessionRow & { sessionId: number };

	const allSubjects = ref<Subject[]>([]);
	const allSessions = ref<SessionWithId[]>([]);
	const sessions = computed(() => allSessions.value.filter((s) => s.class === kelas.value));
	const { loading, subjectsIn, ujianIn } = useHasilTerlaksana();

	// Hanya mapel yang punya ujian TERLAKSANA di kelas ini (lihat useHasilTerlaksana), dengan
	// jumlah jenis ujian yang terlaksana — bukan semua jenis yang terdaftar di Bank Soal.
	const subjects = computed(() => {
		const terlaksana = subjectsIn(kelas.value);
		return allSubjects.value.filter((s) => terlaksana.has(s.name));
	});
	const jenisCounts = computed(() => {
		const counts: Record<string, number> = {};
		for (const s of subjects.value) counts[s.name] = new Set(ujianIn(kelas.value, s.name).map((u) => u.jenis)).size;
		return counts;
	});

	onMounted(async () => {
		[allSubjects.value, allSessions.value] = await Promise.all([
			invoke<Subject[]>("list_subjects"),
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
			<div v-if="loading" class="grid grid-cols-[repeat(auto-fill,minmax(13rem,1fr))] gap-3">
				<USkeleton v-for="i in 4" :key="i" class="h-32 rounded-lg" />
			</div>
			<HasilPelajaranGrid
				v-else-if="subjects.length"
				:kelas="kelas"
				:subjects="subjects"
				:jenis-counts="jenisCounts" />
			<UEmpty
				v-else
				icon="i-lucide-inbox"
				title="Belum ada ujian yang terlaksana di kelas ini"
				description="Mata pelajaran muncul di sini setelah ujiannya dibuat di Kelola Ujian, soalnya tersedia di Bank Soal, dan waktu ujiannya sudah dimulai."
				:actions="[{ label: 'Buka Kelola Ujian', icon: 'i-lucide-calendar-clock', to: '/ujian', color: 'neutral', variant: 'outline' }]" />
		</template>
	</UDashboardPanel>
</template>
