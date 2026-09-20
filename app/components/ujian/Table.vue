<script lang="ts" setup>
	import { h, resolveComponent } from "vue";

	interface Subject {
		id: number
		name: string
		code: string | null
	}

	interface Exam {
		id: number
		subject: string
		class: string | null
		jenis: string | null
		title: string
		duration: number
		scheduledAt: number | null
		windowEnd: number | null
		token: string
	}

	const props = defineProps<{
		exams: Exam[]
		subjects: Subject[]
	}>();

	const emit = defineEmits<{
		edit: [exam: Exam]
		delete: [exam: Exam]
	}>();

	const subjectLabel = (name: string) => props.subjects.find((s) => s.name === name)?.code || name;

	const formatWindow = (start: number | null, end: number | null) => {
		if (!start) return "-";
		const startText = new Date(start * 1000).toLocaleString("id-ID", { dateStyle: "medium", timeStyle: "short" });
		if (!end) return startText;
		const endText = new Date(end * 1000).toLocaleTimeString("id-ID", { timeStyle: "short" });
		return `${startText} – ${endText}`;
	};

	const columns = [
		{ accessorKey: "title", header: "Judul" },
		{ accessorKey: "class", header: "Kelas", cell: ({ row }: { row: { original: Exam } }) => row.original.class ?? "-" },
		{
			accessorKey: "subject",
			header: "Mata Pelajaran",
			cell: ({ row }: { row: { original: Exam } }) => subjectLabel(row.original.subject)
		},
		{ accessorKey: "jenis", header: "Jenis Soal", cell: ({ row }: { row: { original: Exam } }) => row.original.jenis ?? "-" },
		{
			accessorKey: "scheduledAt",
			header: "Waktu Pelaksanaan",
			cell: ({ row }: { row: { original: Exam } }) => formatWindow(row.original.scheduledAt, row.original.windowEnd)
		},
		{ accessorKey: "duration", header: "Durasi (menit)" },
		{ accessorKey: "token", header: "Token" },
		{
			id: "actions",
			header: "Aksi",
			cell: ({ row }: { row: { original: Exam } }) => {
				const UButton = resolveComponent("UButton");
				return h("div", { class: "flex gap-2" }, [
					h(UButton, { size: "xs", variant: "soft", icon: "lucide:pencil", onClick: () => emit("edit", row.original) }, () => "Edit"),
					h(
						UButton,
						{ size: "xs", variant: "soft", color: "error", icon: "lucide:trash-2", onClick: () => emit("delete", row.original) },
						() => "Hapus"
					)
				]);
			}
		}
	];
</script>

<template>
	<UTable :data="exams" :columns="columns">
		<template #empty>
			<div class="text-center py-10 text-muted">
				Belum ada ujian dibuat.
			</div>
		</template>
	</UTable>
</template>
