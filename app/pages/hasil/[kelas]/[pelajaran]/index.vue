<script lang="ts" setup>
	import { invoke } from "@tauri-apps/api/core";

	definePageMeta({
		hideFromNav: true
	});

	interface SessionProgress {
		sessionId: number
		studentName: string
		studentNisn: string
		examTitle: string
		class: string | null
		subject: string
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

	const allSessions = ref<SessionProgress[]>([]);
	const sessions = computed(() => allSessions.value.filter((s) => s.class === kelas.value && s.subject === pelajaran.value));

	const refresh = async () => {
		allSessions.value = await invoke<SessionProgress[]>("list_exam_sessions");
	};

	const downloadCsv = (filename: string, rows: (string | number)[][]) => {
		const csv = rows.map((r) => r.map((v) => `"${String(v).replace(/"/g, "\"\"")}"`).join(",")).join("\n");
		const blob = new Blob([csv], { type: "text/csv;charset=utf-8;" });
		const url = URL.createObjectURL(blob);
		const a = document.createElement("a");
		a.href = url;
		a.download = filename;
		a.click();
		URL.revokeObjectURL(url);
	};

	const statusText = (s: SessionProgress) => {
		if (s.submittedAt) return "Selesai";
		if (s.pgTotal > 0 && s.pgAnswered >= s.pgTotal && s.totalQuestions > s.pgTotal) return "PG selesai, lanjut esai";
		if (s.pgTotal > 0 && s.pgAnswered < s.pgTotal) return `Mengerjakan PG (${s.pgAnswered}/${s.pgTotal})`;
		return "Sedang mengerjakan";
	};

	const exportAll = () => {
		downloadCsv(`hasil-kelas-${kelas.value}-${pelajaran.value}.csv`, [
			["Nama", "NISN", "Ujian", "Progress", "Status", "Nilai"],
			...sessions.value.map((s) => [
				s.studentName,
				s.studentNisn,
				s.examTitle,
				`${s.answeredCount}/${s.totalQuestions}`,
				statusText(s),
				s.score === null ? "-" : s.score.toFixed(0)
			])
		]);
	};

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
			<UButton
				icon="lucide:download"
				variant="soft"
				:disabled="sessions.length === 0"
				@click="exportAll">
				Export Semua
			</UButton>
		</div>

		<HasilTable
			:sessions="sessions"
			@detail="(id) => navigateTo(`/hasil/sesi/${id}`)"
			@reset="resetSession" />
	</div>
</template>
