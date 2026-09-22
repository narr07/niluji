<script lang="ts" setup>
	import { h, resolveComponent } from "vue";

	interface SessionProgress {
		sessionId: number;
		studentName: string;
		studentNisn: string;
		examTitle: string;
		class: string | null;
		subject: string;
		totalQuestions: number;
		answeredCount: number;
		pgTotal: number;
		pgAnswered: number;
		submittedAt: number | null;
		score: number | null;
	}

	defineProps<{
		sessions: SessionProgress[];
	}>();

	const emit = defineEmits<{
		detail: [sessionId: number];
		reset: [sessionId: number, studentName: string];
	}>();

	const statusInfo = (s: SessionProgress): { text: string; color: "success" | "warning" | "neutral" } => {
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
				return h("div", { class: "flex items-center gap-2" }, [
					h(UAvatar, { text: row.original.studentName.slice(0, 2).toUpperCase(), size: "xs" }),
					h("div", {}, [
						h("p", { class: "font-medium text-highlighted" }, row.original.studentName),
						h("p", { class: "text-xs text-dimmed" }, row.original.studentNisn),
					]),
				]);
			},
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
					h("span", { class: "text-xs text-muted shrink-0 w-10 text-right" }, `${row.original.answeredCount}/${row.original.totalQuestions}`),
				]);
			},
		},
		{
			id: "status",
			header: "Status",
			cell: ({ row }: { row: { original: SessionProgress } }) => {
				const UBadge = resolveComponent("UBadge");
				const info = statusInfo(row.original);
				return h(UBadge, { color: info.color, variant: "subtle", size: "sm" }, () => info.text);
			},
		},
		{
			accessorKey: "score",
			header: "Nilai",
			cell: ({ row }: { row: { original: SessionProgress } }) =>
				row.original.score === null
					? h("span", { class: "text-dimmed" }, "-")
					: h("span", { class: "font-semibold text-highlighted" }, row.original.score.toFixed(0)),
		},
		{
			id: "actions",
			header: "",
			cell: ({ row }: { row: { original: SessionProgress } }) => {
				const UButton = resolveComponent("UButton");
				const UTooltip = resolveComponent("UTooltip");
				const mk = (icon: string, text: string, color: "neutral" | "error", onClick: () => void) =>
					h(UTooltip, { text }, () => h(UButton, { size: "xs", variant: "ghost", color, icon, onClick }));
				return h("div", { class: "flex justify-end gap-0.5" }, [
					mk("i-lucide-eye", "Detail", "neutral", () => emit("detail", row.original.sessionId)),
					mk("i-lucide-rotate-ccw", "Reset progres", "error", () => emit("reset", row.original.sessionId, row.original.studentName)),
				]);
			},
		},
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