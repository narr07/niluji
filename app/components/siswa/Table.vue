<script lang="ts" setup>
	import { h, resolveComponent } from "vue";
	import type { TableColumn } from "@nuxt/ui";

	interface StudentRecord {
		id: number
		nisn: string
		name: string
		class: string | null
	}

	const props = defineProps<{
		students: StudentRecord[]
	}>();

	const emit = defineEmits<{
		edit: [student: StudentRecord]
		delete: [student: StudentRecord]
		deleteMany: [ids: number[]]
	}>();

	const UButton = resolveComponent("UButton");

	const selected = ref<number[]>([]);

	const allSelected = computed(() => props.students.length > 0 && selected.value.length === props.students.length);
	const someSelected = computed(() => selected.value.length > 0 && !allSelected.value);

	const toggleAll = (value: boolean) => {
		selected.value = value ? props.students.map((s) => s.id) : [];
	};

	const toggleOne = (id: number, value: boolean) => {
		selected.value = value ? [...selected.value, id] : selected.value.filter((v) => v !== id);
	};

	watch(() => props.students, () => {
		selected.value = selected.value.filter((id) => props.students.some((s) => s.id === id));
	});

	const confirmDeleteMany = async () => {
		const ok = await confirmDelete({ title: `Hapus ${selected.value.length} siswa terpilih?` });
		if (!ok) return;
		emit("deleteMany", selected.value);
		selected.value = [];
	};

	// state sorting, default sort by "name" ascending
	const sorting = ref([{ id: "name", desc: false }]);

	const columns: TableColumn<StudentRecord>[] = [
		{ id: "select", header: "", enableSorting: false },
		{
			accessorKey: "name",
			header: ({ column }) => {
				const isSorted = column.getIsSorted();

				return h(UButton, {
					color: "neutral",
					variant: "ghost",
					label: "Nama",
					icon: isSorted
						? (isSorted === "asc" ? "i-lucide-arrow-up-narrow-wide" : "i-lucide-arrow-down-wide-narrow")
						: "i-lucide-arrow-up-down",
					class: "-mx-2.5",
					onClick: () => column.toggleSorting(column.getIsSorted() === "asc")
				});
			}
		},
		{ accessorKey: "nisn", header: "NISN" },
		{
			id: "actions",
			header: "Aksi",
			cell: ({ row }: { row: { original: StudentRecord } }) => {
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
	<div class="space-y-3">
		<Transition
			enter-active-class="transition duration-200 ease-out"
			enter-from-class="opacity-0 translate-y-3"
			enter-to-class="opacity-100 translate-y-0"
			leave-active-class="transition duration-150 ease-in"
			leave-to-class="opacity-0 translate-y-3"
		>
			<UCard
				v-if="selected.length"
				 
				class="fixed bottom-6 left-1/2 -translate-x-1/2 z-50 shadow-lg"
				:ui="{ body: 'flex items-center gap-3 py-2 px-4 sm:p-2' }"
			>
				<UBadge>{{ selected.length }}</UBadge>
				<span>Siswa dipilih</span>
				<USeparator orientation="vertical" class="h-4" />
				<div class="flex items-center gap-2">
					<UButton variant="subtle" color="neutral" @click="selected = []">
						Batal
					</UButton>
					<UButton
						variant="subtle"
						color="error"
						icon="lucide:trash-2"
						@click="confirmDeleteMany"
					>
						Hapus
					</UButton>
				</div>
			</UCard>
		</Transition>

		<UTable v-model:sorting="sorting" :data="students" :columns="columns">
			<template #select-header>
				<UCheckbox :model-value="allSelected" :indeterminate="someSelected" @update:model-value="toggleAll" />
			</template>
			<template #select-cell="{ row }">
				<UCheckbox
					:model-value="selected.includes(row.original.id)"
					@update:model-value="(v: boolean) => toggleOne(row.original.id, v)" />
			</template>
			<template #empty>
				<div class="text-center py-10 text-muted">
					Belum ada siswa di kelas ini.
				</div>
			</template>
		</UTable>
	</div>
</template>