<script setup lang="ts" generic="T extends Record<string, any>">
	import type { TableColumn, TableRow } from "@nuxt/ui";

	// Tabel referensi: kolom No otomatis + pencarian teks.
	// pin: kolom yang menempel saat tabel digeser (mis. nama di kiri, tombol aksi di kanan).
	// onRowClick: kalau diisi, baris bisa diklik (mis. membuka form edit).
	const props = defineProps<{
		data: T[]
		columns: TableColumn<T>[]
		empty?: string
		loading?: boolean
		pin?: { left?: string[], right?: string[] }
		onRowClick?: (row: T) => void
	}>();

	const slots = useSlots();

	const search = ref("");
	// Cari hanya di nilai teks/angka baris itu sendiri — objek bersarang (mis. data Dapodik asli di
	// halaman Siswa) dilewati, supaya mencari "object" tidak mencocokkan semua baris.
	const filtered = computed(() => {
		const q = search.value.toLowerCase().trim();
		if (!q)
			return props.data;
		return props.data.filter(r => Object.values(r).some(v =>
			v != null && typeof v !== "object" && String(v).toLowerCase().includes(q)));
	});
	const cols = computed<TableColumn<T>[]>(() => [
		{ id: "no", header: "No", cell: ({ row }) => row.index + 1, size: 48 },
		...props.columns
	]);

	// Kolom No ikut menempel kalau ada kolom yang dipin di kiri, supaya urutannya tetap rapi.
	const toPinning = (pin?: { left?: string[], right?: string[] }) => ({
		left: pin?.left?.length ? ["no", ...pin.left] : [],
		right: pin?.right ?? []
	});
	const columnPinning = ref(toPinning(props.pin));
	watch(() => props.pin, (p) => { columnPinning.value = toPinning(p); }, { deep: true });

	const rowEvents = computed(() => (props.onRowClick
		? { select: (_e: Event, row: TableRow<T>) => props.onRowClick!(row.original) }
		: {}));

	// Slot "toolbar" milik RefTable sendiri; sisanya (mis. #opsi-cell) diteruskan ke UTable.
	const tableSlots = computed(() => Object.keys(slots).filter(name => name !== "toolbar"));
</script>

<template>
	<div class="flex flex-col gap-3">
		<div class="flex items-center justify-between gap-2">
			<slot name="toolbar" />
			<UInput
				v-model="search"
				icon="lucide:search"
				placeholder="Cari…"
				class="ms-auto w-64"
				aria-label="Cari di tabel" />
		</div>
		<UTable
			v-model:column-pinning="columnPinning"
			:data="filtered"
			:columns="cols"
			:loading="loading"
			:empty="empty ?? 'Belum ada data. Tarik dulu lewat Sinkron Dapodik.'"
			sticky
			class="max-h-[65vh] rounded border border-default"
			:ui="onRowClick ? { tr: 'cursor-pointer' } : undefined"
			v-on="rowEvents"
		>
			<template v-for="name in tableSlots" #[name]="slotProps">
				<slot :name="name" v-bind="slotProps ?? {}" />
			</template>
		</UTable>
		<p class="text-xs text-muted">
			{{ filtered.length }} dari {{ data.length }} data<template v-if="onRowClick"> · klik baris untuk membuka</template>
		</p>
	</div>
</template>
