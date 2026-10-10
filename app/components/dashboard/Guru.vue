<script setup lang="ts">
	import type { TableColumn } from "@nuxt/ui";

	interface Pembelajaran {
		rombel: string
		mapel: string
		siswa: number
		terisi: number
	}

	const { session, user, waliRombel } = useAuth();
	const db = useDb();
	const toast = useToast();

	const sem = computed(() => session.value?.semesterId ?? "");
	const ptk = computed(() => user.value?.ptk_id ?? "");

	const loading = ref(true);
	const counts = ref({ rombel: 0, siswa: 0, mapel: 0, ekskul: 0 });
	const pembelajaran = ref<Pembelajaran[]>([]);
	const inputDibuka = ref(true);

	const stats = computed(() => [
		{ title: "Rombel Diampu", icon: "lucide:layers", value: counts.value.rombel },
		{ title: "Siswa", icon: "lucide:users", value: counts.value.siswa },
		{ title: "Mata Pelajaran", icon: "lucide:book-open", value: counts.value.mapel },
		{ title: "Ekskul Dibina", icon: "lucide:trophy", value: counts.value.ekskul }
	]);

	async function load() {
		if (!ptk.value || !sem.value) {
			loading.value = false;
			return;
		}
		loading.value = true;
		try {
			const p = [ptk.value, sem.value];
			const [rombel, siswa, mapel, ekskul, list, flag] = await Promise.all([
				db.scalar("SELECT COUNT(DISTINCT rombongan_belajar_id) FROM pembelajaran_rapor WHERE ptk_id = ? AND semester_id = ?", p),
				db.scalar(`SELECT COUNT(DISTINCT a.peserta_didik_id) FROM anggota_rombel a
					WHERE a.rombongan_belajar_id IN (SELECT rombongan_belajar_id FROM pembelajaran_rapor WHERE ptk_id = ? AND semester_id = ?)`, p),
				db.scalar("SELECT COUNT(DISTINCT kode_mapel) FROM pembelajaran_rapor WHERE ptk_id = ? AND semester_id = ?", p),
				db.scalar("SELECT COUNT(*) FROM rombel WHERE ptk_id = ? AND semester_id = ? AND jenis_rombel = '51'", p),
				db.query<Pembelajaran>(`SELECT r.nama AS rombel, m.singkat AS mapel,
						(SELECT COUNT(*) FROM anggota_rombel a WHERE a.rombongan_belajar_id = r.rombongan_belajar_id) AS siswa,
						(SELECT COUNT(*) FROM nilai_rapor n WHERE n.pembelajaran_rapor_id = pr.id AND n.nilai IS NOT NULL) AS terisi
					FROM pembelajaran_rapor pr JOIN rombel r USING (rombongan_belajar_id) JOIN mapel_rapor m ON m.kode = pr.kode_mapel
					WHERE pr.ptk_id = ? AND pr.semester_id = ? ORDER BY CAST(r.tingkat AS INTEGER), r.nama, m.urutan`, p),
				db.getSetting(`input_nilai_dibuka:${sem.value}`)
			]);
			counts.value = { rombel, siswa, mapel, ekskul };
			pembelajaran.value = list;
			inputDibuka.value = flag !== "0";
		}
		catch (e) {
			toast.add({ title: "Gagal memuat dashboard", description: pesanError(e), color: "error" });
		}
		finally {
			loading.value = false;
		}
	}

	watch([sem, ptk], load, { immediate: true });

	const columns: TableColumn<Pembelajaran>[] = [
		{ accessorKey: "rombel", header: "Kelas" },
		{ accessorKey: "mapel", header: "Mata Pelajaran" },
		{ id: "nilai", header: "Nilai Terisi", cell: ({ row }) => `${row.original.terisi}/${row.original.siswa}` }
	];
</script>

<template>
	<ErPage id="home" title="Dashboard">
		<template #title>
			<div class="flex items-center gap-2">
				<UIcon name="lucide:file-badge" class="size-6 shrink-0 text-primary" />
				<span class="font-bold">{{ session?.sekolahNama }}</span>
			</div>
		</template>
		<template #right>
			<UBadge
				v-for="r in waliRombel"
				:key="r.rombongan_belajar_id"
				variant="subtle"
				icon="lucide:users">
				Wali {{ r.nama }}
			</UBadge>
			<UBadge
				v-if="!loading"
				:color="inputDibuka ? 'success' : 'warning'"
				variant="subtle"
				:icon="inputDibuka ? 'lucide:lock-open' : 'lucide:lock'"
			>
				Input nilai {{ inputDibuka ? "dibuka" : "ditutup" }}
			</UBadge>
		</template>

		<div v-if="loading" class="space-y-6">
			<USkeleton class="h-24 w-full" />
			<USkeleton class="h-64 w-full" />
		</div>

		<div v-else class="space-y-6">
			<UAlert
				v-if="!user?.ptk_id"
				color="warning"
				variant="subtle"
				icon="lucide:triangle-alert"
				title="Akun ini belum terhubung dengan data guru Dapodik"
				description="Minta administrator membuat ulang akun Anda lewat Generate Semua Pengguna."
			/>

			<StatsGrid :stats="stats" />

			<PanelCard title="Pembelajaran yang Anda Ampu" icon="lucide:book-open">
				<template #actions>
					<UButton
						to="/e-rapor/guru/nilai"
						size="sm"
						icon="lucide:table"
						trailing-icon="lucide:arrow-right">
						Isi Nilai
					</UButton>
				</template>
				<UTable :data="pembelajaran" :columns="columns" empty="Belum ada pembelajaran di semester ini" />
			</PanelCard>
		</div>
	</ErPage>
</template>
