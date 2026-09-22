<script lang="ts" setup>
	import { h, resolveComponent } from "vue";

	interface QuestionSummary {
		id: number;
		subject: string;
		class: string | null;
		questionText: string;
		questionType: string;
		image: string | null;
		score: number;
		optionCount: number;
	}

	const props = defineProps<{
		questions: QuestionSummary[];
		loading: boolean;
	}>();

	const emit = defineEmits<{
		edit: [id: number];
		delete: [question: QuestionSummary];
		deleteMany: [ids: number[]];
		preview: [id: number];
	}>();

	const typeFilterItems = [
		{ label: "Semua Tipe", value: "all", icon: "i-lucide-list" },
		{ label: "Pilihan Ganda", value: "multiple_choice", icon: "i-lucide-circle-dot" },
		{ label: "Esai", value: "essay", icon: "i-lucide-pen-line" },
	];
	const typeFilter = ref(typeFilterItems[0]);

	const filteredQuestions = computed(() =>
		typeFilter.value?.value === "all" ? props.questions : props.questions.filter((q) => q.questionType === typeFilter.value?.value)
	);

	const selected = ref<number[]>([]);
	const allSelected = computed(() => filteredQuestions.value.length > 0 && selected.value.length === filteredQuestions.value.length);
	const someSelected = computed(() => selected.value.length > 0 && !allSelected.value);

	const toggleAll = (value: boolean) => {
		selected.value = value ? filteredQuestions.value.map((q) => q.id) : [];
	};
	const toggleOne = (id: number, value: boolean) => {
		selected.value = value ? [...selected.value, id] : selected.value.filter((v) => v !== id);
	};

	watch(filteredQuestions, () => {
		selected.value = selected.value.filter((id) => filteredQuestions.value.some((q) => q.id === id));
	});

	const confirmDeleteMany = async () => {
		const ok = await confirmDelete({ title: `Hapus ${selected.value.length} soal terpilih?` });
		if (!ok) return;
		emit("deleteMany", selected.value);
		selected.value = [];
	};

	const columns = [
		{ id: "select", header: "", enableSorting: false },
		{
			id: "no",
			header: "No",
			cell: ({ row }: { row: { index: number } }) => row.index + 1,
		},
		{
			accessorKey: "questionType",
			header: "Tipe",
			cell: ({ row }: { row: { original: QuestionSummary } }) => {
				const UBadge = resolveComponent("UBadge");
				const isEssay = row.original.questionType === "essay";
				return h(UBadge, { color: isEssay ? "warning" : "primary", variant: "subtle", size: "sm" }, () => (isEssay ? "Esai" : "PG"));
			},
		},
		{
			accessorKey: "questionText",
			header: "Soal",
			cell: ({ row }: { row: { original: QuestionSummary } }) => {
				const plain = stripSoalMarkdown(row.original.questionText);
				return h("span", { class: "block max-w-sm truncate", title: plain }, plain);
			},
		},
		{
			id: "image",
			header: "Gambar",
			cell: ({ row }: { row: { original: QuestionSummary } }) => {
				const UIcon = resolveComponent("UIcon");
				return row.original.image
					? h(UIcon, { name: "i-lucide-image", class: "size-4 text-primary", title: "Soal ini punya gambar" })
					: h("span", { class: "text-dimmed" }, "-");
			},
		},
		{ accessorKey: "optionCount", header: "Opsi" },
		{ accessorKey: "score", header: "Skor" },
		{
			id: "actions",
			header: "",
			cell: ({ row }: { row: { original: QuestionSummary } }) => {
				const UButton = resolveComponent("UButton");
				const UTooltip = resolveComponent("UTooltip");
				const mk = (icon: string, text: string, color: "neutral" | "error", onClick: () => void) =>
					h(UTooltip, { text }, () =>
						h(UButton, {
							size: "xs",
							variant: "ghost",
							color,
							icon,
							onClick,
						})
					);
				return h("div", { class: "flex justify-end gap-0.5" }, [
					mk("i-lucide-eye", "Preview", "neutral", () => emit("preview", row.original.id)),
					mk("i-lucide-pencil", "Edit", "neutral", () => emit("edit", row.original.id)),
					mk("i-lucide-trash-2", "Hapus", "error", () => emit("delete", row.original)),
				]);
			},
		},
	];
</script>

<template>
	<div class="space-y-3">
		<div class="flex items-center justify-between gap-3">
			<USelectMenu
				v-model="typeFilter"
				:items="typeFilterItems"
				icon="i-lucide-filter"
				class="w-fit min-w-40" />
			<p class="text-sm text-muted">
				{{ filteredQuestions.length }} soal
			</p>
		</div>

		<UTable
			:data="filteredQuestions"
			:columns="columns"
			:loading="loading"
			:column-pinning="{ right: ['actions'] }"
			sticky
			class="max-h-[32rem]"
			:ui="{ th: 'bg-default', td: 'bg-default' }"
		>
			<template #select-header>
				<UCheckbox :model-value="allSelected" :indeterminate="someSelected" @update:model-value="toggleAll" />
			</template>
			<template #select-cell="{ row }">
				<UCheckbox
					:model-value="selected.includes(row.original.id)"
					@update:model-value="(v: boolean) => toggleOne(row.original.id, v)" />
			</template>
			<template #empty>
				<div class="flex flex-col items-center gap-2 py-10 text-muted">
					<UIcon name="i-lucide-inbox" class="size-6" />
					<span>Belum ada soal di kelas & mata pelajaran ini.</span>
				</div>
			</template>
		</UTable>

		<Transition
			enter-active-class="transition duration-200 ease-out"
			enter-from-class="opacity-0 translate-y-3"
			enter-to-class="opacity-100 translate-y-0"
			leave-active-class="transition duration-150 ease-in"
			leave-to-class="opacity-0 translate-y-3"
		>
			<UCard
				v-if="selected.length"
				class="fixed bottom-6 left-1/2 -translate-x-1/2 z-50 shadow-lg ring-1 ring-default"
				:ui="{ body: 'flex items-center gap-3 py-2 px-4 sm:p-2' }"
			>
				<UBadge color="primary" variant="subtle" size="lg">
					{{ selected.length }}
				</UBadge>
				<span class="text-sm font-medium">Soal dipilih</span>
				<USeparator orientation="vertical" class="h-4" />
				<div class="flex items-center gap-2">
					<UButton
						variant="ghost"
						color="neutral"
						size="sm"
						@click="selected = []">
						Batal
					</UButton>
					<UButton
						variant="soft"
						color="error"
						size="sm"
						icon="i-lucide-trash-2"
						@click="confirmDeleteMany">
						Hapus
					</UButton>
				</div>
			</UCard>
		</Transition>
	</div>
</template>