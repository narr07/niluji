<script lang="ts" setup>
	import { h, resolveComponent } from "vue";

	interface Subject {
		id: number;
		name: string;
		code: string | null;
	}

	interface Exam {
		id: number;
		subject: string;
		class: string | null;
		jenis: string | null;
		title: string;
		duration: number;
		scheduledAt: number | null;
		windowEnd: number | null;
		token: string;
		randomizePg: boolean;
		randomizeEssay: boolean;
	}

	const props = defineProps<{
		exams: Exam[];
		subjects: Subject[];
	}>();

	const emit = defineEmits<{
		edit: [exam: Exam];
		delete: [exam: Exam];
	}>();

	const subjectLabel = (name: string) => props.subjects.find((s) => s.name === name)?.code || name;

	const formatWindow = (start: number | null, end: number | null) => {
		if (!start) return "-";
		const startText = new Date(start * 1000).toLocaleString("id-ID", { dateStyle: "medium", timeStyle: "short" });
		if (!end) return startText;
		const endText = new Date(end * 1000).toLocaleTimeString("id-ID", { timeStyle: "short" });
		return `${startText} – ${endText}`;
	};

	const { copy, copied } = useClipboard();

	const expanded = ref({});

	const columns = [
		{
			id: "expand",
			cell: ({ row }: { row: { getIsExpanded: () => boolean; toggleExpanded: () => void } }) => {
				const UButton = resolveComponent("UButton");
				return h(UButton, {
					size: "xs",
					variant: "ghost",
					color: "neutral",
					icon: row.getIsExpanded() ? "lucide:chevron-down" : "lucide:chevron-right",
					onClick: () => row.toggleExpanded(),
				});
			},
		},
		{ accessorKey: "title", header: "Judul" },
		{ accessorKey: "class", header: "Kelas", cell: ({ row }: { row: { original: Exam } }) => row.original.class ?? "-" },
		{
			accessorKey: "subject",
			header: "Mata Pelajaran",
			cell: ({ row }: { row: { original: Exam } }) => subjectLabel(row.original.subject),
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
								onClick: () => navigateTo(hasilPath),
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
			},
		},
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
			<div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-4 p-3 bg-elevated/50 rounded-md text-sm">
				<div class="flex items-start gap-2">
					<UIcon name="i-lucide-layers" class="size-4 mt-0.5 text-muted shrink-0" />
					<div>
						<p class="text-dimmed text-[11px] font-medium uppercase tracking-wide">Jenis Soal</p>
						<p class="font-medium text-highlighted">{{ row.original.jenis ?? "-" }}</p>
					</div>
				</div>

				<div class="flex items-start gap-2">
					<UIcon name="i-lucide-calendar-clock" class="size-4 mt-0.5 text-muted shrink-0" />
					<div>
						<p class="text-dimmed text-[11px] font-medium uppercase tracking-wide">Waktu Pelaksanaan</p>
						<p class="font-medium text-highlighted">{{ formatWindow(row.original.scheduledAt, row.original.windowEnd) }}</p>
					</div>
				</div>

				<div class="flex items-start gap-2">
					<UIcon name="i-lucide-timer" class="size-4 mt-0.5 text-muted shrink-0" />
					<div>
						<p class="text-dimmed text-[11px] font-medium uppercase tracking-wide">Durasi</p>
						<p class="font-medium text-highlighted">{{ row.original.duration }} menit</p>
					</div>
				</div>

				<div class="flex items-start gap-2">
					<UIcon name="i-lucide-key-round" class="size-4 mt-0.5 text-muted shrink-0" />
					<div class="min-w-0">
						<p class="text-dimmed text-[11px] font-medium uppercase tracking-wide">Token</p>
						<button
							type="button"
							class="flex items-center gap-1.5 font-mono font-medium text-highlighted hover:text-primary transition-colors"
							@click="copy(row.original.token)"
						>
							{{ row.original.token }}
							<UIcon :name="copied ? 'i-lucide-check' : 'i-lucide-copy'" class="size-3.5 shrink-0" />
						</button>
					</div>
				</div>

				<div class="flex items-start gap-2">
					<UIcon name="i-lucide-shuffle" class="size-4 mt-0.5 text-muted shrink-0" />
					<div>
						<p class="text-dimmed text-[11px] font-medium uppercase tracking-wide">Acak Soal PG</p>
						<UBadge :color="row.original.randomizePg ? 'success' : 'neutral'" variant="subtle" size="sm">
							{{ row.original.randomizePg ? "Ya" : "Tidak" }}
						</UBadge>
					</div>
				</div>

				<div class="flex items-start gap-2">
					<UIcon name="i-lucide-shuffle" class="size-4 mt-0.5 text-muted shrink-0" />
					<div>
						<p class="text-dimmed text-[11px] font-medium uppercase tracking-wide">Acak Soal Esai</p>
						<UBadge :color="row.original.randomizeEssay ? 'success' : 'neutral'" variant="subtle" size="sm">
							{{ row.original.randomizeEssay ? "Ya" : "Tidak" }}
						</UBadge>
					</div>
				</div>
			</div>
		</template>
	</UTable>
</template>