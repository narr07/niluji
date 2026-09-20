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
	}

	defineProps<{
		sessions: SessionProgress[]
	}>();

	const emit = defineEmits<{
		detail: [sessionId: number]
		reset: [sessionId: number, studentName: string]
	}>();

	const statusText = (s: SessionProgress) => {
		if (s.submittedAt) return "Selesai";
		if (s.pgTotal > 0 && s.pgAnswered >= s.pgTotal && s.totalQuestions > s.pgTotal) return "PG selesai, lanjut esai";
		if (s.pgTotal > 0 && s.pgAnswered < s.pgTotal) return `Mengerjakan PG (${s.pgAnswered}/${s.pgTotal})`;
		return "Sedang mengerjakan";
	};

	const columns = [
		{ accessorKey: "studentName", header: "Nama" },
		{ accessorKey: "studentNisn", header: "NISN" },
		{ accessorKey: "examTitle", header: "Ujian" },
		{
			accessorKey: "answeredCount",
			header: "Progress",
			cell: ({ row }: { row: { original: SessionProgress } }) =>
				`${row.original.answeredCount} / ${row.original.totalQuestions}`
		},
		{
			id: "status",
			header: "Status",
			cell: ({ row }: { row: { original: SessionProgress } }) => statusText(row.original)
		},
		{
			accessorKey: "score",
			header: "Nilai",
			cell: ({ row }: { row: { original: SessionProgress } }) =>
				row.original.score === null ? "-" : row.original.score.toFixed(0)
		},
		{
			id: "actions",
			header: "Aksi",
			cell: ({ row }: { row: { original: SessionProgress } }) => {
				const UButton = resolveComponent("UButton");
				return h("div", { class: "flex gap-2" }, [
					h(UButton, {
						size: "xs",
						variant: "soft",
						icon: "lucide:eye",
						onClick: () => emit("detail", row.original.sessionId)
					}, () => "Detail"),
					h(UButton, {
						size: "xs",
						variant: "soft",
						color: "error",
						icon: "lucide:rotate-ccw",
						onClick: () => emit("reset", row.original.sessionId, row.original.studentName)
					}, () => "Reset")
				]);
			}
		}
	];
</script>

<template>
	<UTable :data="sessions" :columns="columns">
		<template #empty>
			<div class="text-center py-10 text-muted">
				Belum ada siswa yang mengerjakan ujian ini.
			</div>
		</template>
	</UTable>
</template>
