<script setup lang="ts">
	import type { TableColumn } from "@nuxt/ui";

	// Ekskul di Dapodik = rombel dengan jenis_rombel 51.
	interface Ekskul {
		nama: string
		pembina: string | null
		anggota: number
	}

	const db = useDb();

	const { rows, loading, semesterId } = useSemesterData(sem => db.query<Ekskul>(
		`SELECT r.nama, g.nama AS pembina,
			(SELECT COUNT(*) FROM anggota_rombel a WHERE a.rombongan_belajar_id = r.rombongan_belajar_id AND a.semester_id = r.semester_id) AS anggota
		FROM rombel r LEFT JOIN ptk g ON g.ptk_id = r.ptk_id
		WHERE r.semester_id = ? AND r.jenis_rombel = '51' ORDER BY r.nama COLLATE NOCASE`,
		[sem]
	), "Gagal memuat ekstrakurikuler");

	const columns: TableColumn<Ekskul>[] = [
		{ accessorKey: "nama", header: "Nama Ekskul", meta: { class: { td: "font-medium" } } },
		{ accessorKey: "pembina", header: "Pembina", cell: ({ row }) => atauStrip(row.original.pembina) },
		{ accessorKey: "anggota", header: "Jumlah Anggota", meta: { class: { th: "text-right", td: "text-right tabular-nums" } } }
	];
</script>

<template>
	<ErPage id="ref-ekskul" title="Ekstrakurikuler">
		<template #right>
			<DapodikNote />
		</template>

		<RefTable
			:data="rows"
			:columns="columns"
			:loading="loading"
			:empty="semesterId ? 'Belum ada ekskul di semester ini. Isi rombel ekskul di Dapodik, lalu Sinkron Dapodik.' : 'Belum ada semester. Tarik dulu lewat Sinkron Dapodik.'"
		/>
	</ErPage>
</template>
