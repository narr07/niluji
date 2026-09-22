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
		randomizePg: boolean
		randomizeEssay: boolean
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

	// Kolom dibatasi yang penting saja (Judul/Kelas/Mapel/Aksi) supaya tabel tidak melebar jauh
	// ke kanan — detail lain (jenis soal, jadwal, durasi, token) dipindah ke baris expand yang
	// muncul kalau barisnya diklik.
	const expanded = ref({});

	const columns = [
		{
			id: "expand",
			cell: ({ row }: { row: { getIsExpanded: () => boolean, toggleExpanded: () => void } }) => {
				const UButton = resolveComponent("UButton");
				return h(UButton, {
					size: "xs",
					variant: "ghost",
					color: "neutral",
					icon: row.getIsExpanded() ? "lucide:chevron-down" : "lucide:chevron-right",
					onClick: () => row.toggleExpanded()
				});
			}
		},
		{ accessorKey: "title", header: "Judul" },
		{ accessorKey: "class", header: "Kelas", cell: ({ row }: { row: { original: Exam } }) => row.original.class ?? "-" },
		{
			accessorKey: "subject",
			header: "Mata Pelajaran",
			cell: ({ row }: { row: { original: Exam } }) => subjectLabel(row.original.subject)
		},
		{
			id: "actions",
			header: "Aksi",
			cell: ({ row }: { row: { original: Exam } }) => {
				const UButton = resolveComponent("UButton");
				const buttons = [];
				if (row.original.class) {
					const hasilPath = row.original.jenis
						? `/hasil/${row.original.class}/${encodeURIComponent(row.original.subject)}/${encodeURIComponent(row.original.jenis)}`
						: `/hasil/${row.original.class}/${encodeURIComponent(row.original.subject)}`;
					buttons.push(
						h(
							UButton,
							{
								size: "xs",
								variant: "soft",
								color: "neutral",
								icon: "lucide:bar-chart-3",
								onClick: () => navigateTo(hasilPath)
							},
							() => "Lihat Hasil"
						)
					);
				}
				buttons.push(
					h(UButton, { size: "xs", variant: "soft", icon: "lucide:pencil", onClick: () => emit("edit", row.original) }, () => "Edit"),
					h(
						UButton,
						{ size: "xs", variant: "soft", color: "error", icon: "lucide:trash-2", onClick: () => emit("delete", row.original) },
						() => "Hapus"
					)
				);
				return h("div", { class: "flex gap-2" }, buttons);
			}
		}
	];
</script>

<template>
	<UTable v-model:expanded="expanded" :data="exams" :columns="columns">
		<template #empty>
			<div class="text-center py-10 text-muted">
				Belum ada ujian dibuat.
			</div>
		</template>

		<template #expanded="{ row }">
			<div class="grid gap-3 sm:grid-cols-3 p-2 text-sm">
				<div>
					<p class="text-muted text-xs">
						Jenis Soal
					</p>
					<p>{{ row.original.jenis ?? "-" }}</p>
				</div>
				<div>
					<p class="text-muted text-xs">
						Waktu Pelaksanaan
					</p>
					<p>{{ formatWindow(row.original.scheduledAt, row.original.windowEnd) }}</p>
				</div>
				<div>
					<p class="text-muted text-xs">
						Durasi
					</p>
					<p>{{ row.original.duration }} menit</p>
				</div>
				<div>
					<p class="text-muted text-xs">
						Token
					</p>
					<p>{{ row.original.token }}</p>
				</div>
				<div>
					<p class="text-muted text-xs">
						Acak Soal PG
					</p>
					<p>{{ row.original.randomizePg ? "Ya" : "Tidak" }}</p>
				</div>
				<div>
					<p class="text-muted text-xs">
						Acak Soal Esai
					</p>
					<p>{{ row.original.randomizeEssay ? "Ya" : "Tidak" }}</p>
				</div>
			</div>
		</template>
	</UTable>
</template>
