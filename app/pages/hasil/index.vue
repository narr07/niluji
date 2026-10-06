<script lang="ts" setup>
	import { invoke } from "@tauri-apps/api/core";

	definePageMeta({
		name: "Hasil Ujian",
		icon: "lucide:bar-chart-3",
		category: "cbt",
		order: 4,
		description: "Hasil ujian per kelas dan mata pelajaran"
	});

	interface HasilKelasStats {
		ujian: number
		sesi: number
		selesai: number
		target: number
		rataRata: number | null
		totalSiswa: number
	}

	const allClasses = ref<string[]>([]);
	const students = ref<{ nisn: string, class: string | null }[]>([]);
	const { loading, classes: classesTerlaksana, ujianIn, sessions } = useHasilTerlaksana();

	// Hanya kelas yang punya ujian TERLAKSANA (ada di Kelola Ujian, ada soalnya di Bank Soal,
	// dan sudah dimulai — lihat useHasilTerlaksana), tetap dalam urutan dari Pengaturan.
	const classes = computed(() => allClasses.value.filter((c) => classesTerlaksana.value.has(c)));

	// Ringkasan per kelas untuk kartu. "Ujian" = ujian terlaksana saja, jadi ujian yang
	// dijadwalkan untuk nanti tidak ikut memperbesar target pengerjaan.
	// Progres = pengerjaan selesai ÷ (jumlah ujian × jumlah siswa), bukan "siswa yang sudah ikut
	// minimal satu ujian" — yang terakhir langsung 100% begitu semua siswa mengerjakan 1 dari 5
	// ujian. Pengerjaan selesai hanya dihitung untuk NISN yang ADA di daftar siswa kelas ini,
	// supaya siswa yang sudah pindah kelas/dihapus/NISN-nya dibetulkan tidak membuat angkanya
	// melebihi target ("32 dari 30").
	// Rata-rata memakai nilai otomatis saat submit (PG saja — esai dinilai guru belakangan dan
	// tidak masuk ke nilai ini), dari semua sesi yang sudah dikumpulkan.
	const stats = computed(() => {
		const result: Record<string, HasilKelasStats> = {};
		for (const kelas of classes.value) {
			const roster = new Set(students.value.filter((s) => s.class === kelas).map((s) => s.nisn));
			const ujian = ujianIn(kelas).length;
			const kelasSessions = sessions.value.filter((s) => s.class === kelas);
			const submitted = kelasSessions.filter((s) => s.submittedAt !== null);
			const scored = submitted.filter((s) => s.score !== null);
			const target = ujian * roster.size;
			result[kelas] = {
				ujian,
				sesi: kelasSessions.length,
				selesai: Math.min(target, submitted.filter((s) => roster.has(s.studentNisn)).length),
				target,
				rataRata: scored.length ? scored.reduce((sum, s) => sum + s.score!, 0) / scored.length : null,
				totalSiswa: roster.size
			};
		}
		return result;
	});

	onMounted(async () => {
		[allClasses.value, students.value] = await Promise.all([
			invoke<string[]>("list_classes"),
			invoke<{ nisn: string, class: string | null }[]>("list_students")
		]);
	});
</script>

<template>
	<UDashboardPanel id="hasil">
		<template #header>
			<UDashboardNavbar title="Hasil Ujian">
				<template #leading>
					<UDashboardSidebarCollapse />
				</template>
			</UDashboardNavbar>
		</template>

		<template #body>
			<HasilKelasGrid :classes="classes" :stats="stats" :loading="loading" />
		</template>
	</UDashboardPanel>
</template>
