<script lang="ts" setup>
	interface OnlineStudent {
		nisn: string
		name: string
		loggedInAt: number
	}

	defineProps<{
		online: OnlineStudent[]
	}>();

	const columns = [
		{ accessorKey: "name", header: "Nama" },
		{ accessorKey: "nisn", header: "NISN" },
		{
			accessorKey: "loggedInAt",
			header: "Login Pukul",
			cell: ({ row }: { row: { original: OnlineStudent } }) =>
				new Date(row.original.loggedInAt * 1000).toLocaleTimeString("id-ID")
		}
	];
</script>

<template>
	<UCard>
		<template #header>
			<span class="font-semibold">Sedang Online</span>
		</template>
		<UTable :data="online" :columns="columns">
			<template #empty>
				<div class="text-center py-6 text-muted text-sm">
					Belum ada siswa yang login.
				</div>
			</template>
		</UTable>
	</UCard>
</template>
