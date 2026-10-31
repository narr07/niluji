<script lang="ts" setup>
	import { h, resolveComponent } from "vue";

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
		essayScore?: number | null
		finalScore?: number | null
		extraMinutes?: number
		deviceLocked?: boolean
	}

	defineProps<{
		sessions: SessionProgress[]
	}>();

	const emit = defineEmits<{
		detail: [sessionId: number]
		reset: [sessionId: number, studentName: string]
		unlockDevice: [sessionId: number, studentName: string]
		addTime: [sessionId: number, studentName: string, minutes: number]
		forceSubmit: [sessionId: number, studentName: string]
	}>();

	const statusInfo = (s: SessionProgress): { text: string, color: "success" | "warning" | "neutral" } => {
		if (s.submittedAt) return { text: "Selesai", color: "success" };
		if (s.pgTotal > 0 && s.pgAnswered >= s.pgTotal && s.totalQuestions > s.pgTotal) {
			return { text: "PG selesai, lanjut esai", color: "warning" };
		}
		if (s.pgTotal > 0 && s.pgAnswered < s.pgTotal) {
			return { text: `Mengerjakan PG (${s.pgAnswered}/${s.pgTotal})`, color: "neutral" };
		}
		return { text: "Sedang mengerjakan", color: "neutral" };
	};

	const columns = [
		{
			accessorKey: "studentName",
			header: "Nama",
			cell: ({ row }: { row: { original: SessionProgress } }) => {
				const UAvatar = resolveComponent("UAvatar");
				const UBadge = resolveComponent("UBadge");
				return h("div", { class: "flex items-center gap-2" }, [
					h(UAvatar, { text: row.original.studentName.slice(0, 2).toUpperCase(), size: "xs" }),
					h("div", {}, [
						h("div", { class: "flex items-center gap-1.5" }, [
							h("p", { class: "font-medium text-highlighted" }, row.original.studentName),
							row.original.extraMinutes
								? h(UBadge, { color: "info", variant: "subtle", size: "xs" }, () => `+${row.original.extraMinutes}m`)
								: null
						]),
						h("p", { class: "text-xs text-dimmed" }, row.original.studentNisn)
					])
				]);
			}
		},
		{ accessorKey: "examTitle", header: "Ujian" },
		{
			accessorKey: "answeredCount",
			header: "Progress",
			cell: ({ row }: { row: { original: SessionProgress } }) => {
				const UProgress = resolveComponent("UProgress");
				const pct = row.original.totalQuestions > 0 ? (row.original.answeredCount / row.original.totalQuestions) * 100 : 0;
				return h("div", { class: "flex items-center gap-2 w-32" }, [
					h(UProgress, { modelValue: pct, size: "sm", class: "flex-1" }),
					h("span", { class: "text-xs text-muted shrink-0 w-10 text-right" }, `${row.original.answeredCount}/${row.original.totalQuestions}`)
				]);
			}
		},
		{
			id: "status",
			header: "Status",
			cell: ({ row }: { row: { original: SessionProgress } }) => {
				const UBadge = resolveComponent("UBadge");
				const info = statusInfo(row.original);
				return h(UBadge, { color: info.color, variant: "subtle", size: "sm" }, () => info.text);
			}
		},
		{
			accessorKey: "score",
			header: "Nilai PG",
			cell: ({ row }: { row: { original: SessionProgress } }) =>
				row.original.score === null || row.original.score === undefined
					? h("span", { class: "text-dimmed" }, "-")
					: h("span", { class: "font-medium text-default" }, row.original.score.toFixed(0))
		},
		{
			accessorKey: "essayScore",
			header: "Nilai Esai",
			cell: ({ row }: { row: { original: SessionProgress } }) =>
				row.original.essayScore === null || row.original.essayScore === undefined
					? h("span", { class: "text-dimmed" }, "-")
					: h("span", { class: "font-medium text-default" }, row.original.essayScore.toFixed(0))
		},
		{
			accessorKey: "finalScore",
			header: "Nilai Akhir",
			cell: ({ row }: { row: { original: SessionProgress } }) => {
				const val = row.original.finalScore ?? row.original.score;
				return val === null || val === undefined
					? h("span", { class: "text-dimmed" }, "-")
					: h("span", { class: "font-bold text-highlighted" }, val.toFixed(0));
			}
		},
		{
			id: "actions",
			header: "",
			cell: ({ row }: { row: { original: SessionProgress } }) => {
				const UButton = resolveComponent("UButton");
				const UTooltip = resolveComponent("UTooltip");
				const s = row.original;
				const mk = (
					icon: string,
					text: string,
					color: "neutral" | "error" | "warning" | "info" | "success",
					onClick: () => void,
					disabled = false
				) => h(UTooltip, { text }, () => h(UButton, { size: "xs", variant: "ghost", color, icon, disabled, onClick }));

				return h("div", { class: "flex justify-end gap-0.5" }, [
					mk("i-lucide-eye", "Detail & Koreksi", "neutral", () => emit("detail", s.sessionId)),
					!s.submittedAt
						? mk("i-lucide-clock-plus", "Tambah waktu +10 menit", "info", () => emit("addTime", s.sessionId, s.studentName, 10))
						: null,
					!s.submittedAt && s.deviceLocked
						? mk("i-lucide-smartphone", "Buka kunci perangkat (pindah HP/PC)", "warning", () => emit("unlockDevice", s.sessionId, s.studentName))
						: null,
					!s.submittedAt
						? mk("i-lucide-check-square", "Paksa selesai & kumpulkan", "success", () => emit("forceSubmit", s.sessionId, s.studentName))
						: null,
					mk("i-lucide-rotate-ccw", "Reset progres", "error", () => emit("reset", s.sessionId, s.studentName))
				]);
			}
		}
	];
</script>

<template>
	<UTable :data="sessions" :columns="columns" :column-pinning="{ right: ['actions'] }">
		<template #empty>
			<div class="flex flex-col items-center gap-2 py-10 text-muted">
				<UIcon name="i-lucide-users" class="size-6" />
				<span>Belum ada siswa yang mengerjakan ujian ini.</span>
			</div>
		</template>
	</UTable>
</template>
