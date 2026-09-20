<script lang="ts" setup>
	interface SchoolInfo {
		name: string
		npsn: string
		address: string
		principal: string
	}

	const props = defineProps<{
		school: SchoolInfo | null
	}>();

	const columns = [
		{ accessorKey: "label", header: "Keterangan" },
		{ accessorKey: "value", header: "Isi" }
	];

	const rows = computed(() => props.school
		? [
			{ label: "Nama Sekolah", value: props.school.name },
			{ label: "NPSN", value: props.school.npsn },
			{ label: "Alamat", value: props.school.address },
			{ label: "Kepala Sekolah", value: props.school.principal }
		]
		: []);
</script>

<template>
	<UCard>
		<template #header>
			<span class="font-semibold">Data Sekolah</span>
		</template>

		<UTable :data="rows" :columns="columns" />
	</UCard>
</template>
