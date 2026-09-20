<script lang="ts" setup>
	import { h, resolveComponent } from "vue";

	interface QuestionSummary {
		id: number
		subject: string
		class: string | null
		questionText: string
		questionType: string
		image: string | null
		score: number
		optionCount: number
	}

	const props = defineProps<{
		questions: QuestionSummary[]
		loading: boolean
	}>();

	const emit = defineEmits<{
		edit: [id: number]
		delete: [question: QuestionSummary]
	}>();

	const typeFilterItems = [
		{ label: "Semua Tipe", value: "all" },
		{ label: "Pilihan Ganda", value: "multiple_choice" },
		{ label: "Esai", value: "essay" }
	];
	const typeFilter = ref(typeFilterItems[0]);

	// Nomor di sini cuma nomor tampilan (biar gampang nunjuk "soal nomor sekian" pas ngedit),
	// bukan urutan yang dipakai pas ujian — soal tetap diacak sendiri untuk siswa.
	const filteredQuestions = computed(() =>
		typeFilter.value?.value === "all" ? props.questions : props.questions.filter((q) => q.questionType === typeFilter.value?.value)
	);

	const columns = [
		{
			id: "no",
			header: "No",
			cell: ({ row }: { row: { index: number } }) => row.index + 1
		},
		{
			accessorKey: "questionType",
			header: "Tipe",
			cell: ({ row }: { row: { original: QuestionSummary } }) => (row.original.questionType === "essay" ? "Esai" : "PG")
		},
		{
			accessorKey: "questionText",
			header: "Soal",
			cell: ({ row }: { row: { original: QuestionSummary } }) =>
				h("span", { class: "block max-w-sm truncate", title: row.original.questionText }, row.original.questionText)
		},
		{ accessorKey: "optionCount", header: "Jumlah Opsi" },
		{ accessorKey: "score", header: "Skor" },
		{
			id: "actions",
			header: "Aksi",
			cell: ({ row }: { row: { original: QuestionSummary } }) => {
				const UButton = resolveComponent("UButton");
				return h("div", { class: "flex gap-2" }, [
					h(UButton, { size: "xs", variant: "soft", icon: "lucide:pencil", onClick: () => emit("edit", row.original.id) }, () => "Edit"),
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
	<div class="space-y-3">
		<UFormField label="Filter Tipe Soal" class="max-w-xs">
			<USelectMenu v-model="typeFilter" :items="typeFilterItems" />
		</UFormField>

		<UTable
			:data="filteredQuestions"
			:columns="columns"
			:loading="loading"
			:column-pinning="{ right: ['actions'] }"
			sticky
			class="max-h-[32rem]"
		>
			<template #empty>
				<div class="text-center py-10 text-muted">
					Belum ada soal di kelas & mata pelajaran ini.
				</div>
			</template>
		</UTable>
	</div>
</template>
