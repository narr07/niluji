<script lang="ts" setup>
	import { invoke } from "@tauri-apps/api/core";

	definePageMeta({
		hideFromNav: true
	});

	interface StudentRecord {
		id: number
		nisn: string
		name: string
		class: string | null
		rombel?: string | null
		pin?: string
	}

	const ROMBEL_SEMUA = "__semua";
	const ROMBEL_TANPA = "__tanpa";

	const route = useRoute();
	const toast = useToast();
	const kelas = computed(() => route.params.kelas as string);
	// Halaman khusus untuk siswa yang kelasnya kosong atau tidak ada di Data Kelas.
	const isLainnya = computed(() => kelas.value === KELAS_LAINNYA);
	const pageTitle = computed(() => (isLainnya.value ? "Siswa dengan kelas tidak terdaftar" : `Siswa Kelas ${kelas.value}`));

	const breadcrumbItems = computed(() => [
		{ label: "Siswa", icon: "lucide:users", to: "/siswa" },
		{ label: isLainnya.value ? "Kelas tidak terdaftar" : `Kelas ${kelas.value}` }
	]);

	const allStudents = ref<StudentRecord[]>([]);
	const registeredClasses = ref<string[]>([]);
	const rombelFilter = ref(ROMBEL_SEMUA);

	const classStudents = computed(() =>
		isLainnya.value
			? allStudents.value.filter((s) => !s.class || !registeredClasses.value.includes(s.class))
			: allStudents.value.filter((s) => s.class === kelas.value)
	);

	const rombelItems = computed(() => {
		const rombels = [...new Set(classStudents.value.map((s) => s.rombel?.trim()).filter((r): r is string => !!r))].sort();
		if (!rombels.length) return [];
		const hasWithout = classStudents.value.some((s) => !s.rombel?.trim());
		return [
			{ label: "Semua rombel", value: ROMBEL_SEMUA },
			...rombels.map((r) => ({ label: `Rombel ${r}`, value: r })),
			...(hasWithout ? [{ label: "Tanpa rombel", value: ROMBEL_TANPA }] : [])
		];
	});

	const students = computed(() => {
		if (rombelFilter.value === ROMBEL_SEMUA) return classStudents.value;
		if (rombelFilter.value === ROMBEL_TANPA) return classStudents.value.filter((s) => !s.rombel?.trim());
		return classStudents.value.filter((s) => s.rombel?.trim() === rombelFilter.value);
	});

	watch(kelas, () => {
		rombelFilter.value = ROMBEL_SEMUA;
	});

	const formOpen = ref(false);
	const kartuOpen = ref(false);
	const editingStudent = ref<StudentRecord | null>(null);
	const fixing = ref(false);

	const loadStudents = async () => {
		[allStudents.value, registeredClasses.value] = await Promise.all([
			invoke<StudentRecord[]>("list_students"),
			invoke<string[]>("list_classes")
		]);
		if (!rombelItems.value.some((item) => item.value === rombelFilter.value)) rombelFilter.value = ROMBEL_SEMUA;
	};

	// Untuk siswa lama seperti "IVA": cocokkan ke kelas terdaftar, setelah database dicadangkan.
	const fixClasses = async () => {
		fixing.value = true;
		try {
			const fixed = await invoke<number>("fix_student_classes");
			toast.add(
				fixed
					? { title: "Kelas dicocokkan", description: `${fixed} siswa dipindahkan ke kelas terdaftar.`, icon: "lucide:check", color: "success" }
					: { title: "Tidak ada yang bisa dicocokkan", description: "Tambahkan kelasnya di Pengaturan › Data Kelas, atau edit siswa satu per satu.", icon: "lucide:info", color: "neutral" }
			);
			await loadStudents();
		} catch (e) {
			toast.add({ title: "Gagal mencocokkan kelas", description: e instanceof Error ? e.message : String(e), icon: "lucide:x", color: "error" });
		} finally {
			fixing.value = false;
		}
	};

	const openCreate = () => {
		editingStudent.value = null;
		formOpen.value = true;
	};

	const openEdit = (student: StudentRecord) => {
		editingStudent.value = student;
		formOpen.value = true;
	};

	const deleteStudent = async (student: StudentRecord) => {
		const ok = await confirmDelete({ title: `Hapus siswa ${student.name}?` });
		if (!ok) return;
		await invoke("delete_student", { id: student.id });
		await loadStudents();
	};

	const deleteManyStudents = async (ids: number[]) => {
		await Promise.all(ids.map((id) => invoke("delete_student", { id })));
		await loadStudents();
	};

	onMounted(loadStudents);
</script>

<template>
	<UDashboardPanel id="siswa-kelas">
		<template #header>
			<UDashboardNavbar :title="pageTitle">
				<template #leading>
					<UDashboardSidebarCollapse />
					<UButton icon="lucide:arrow-left" variant="ghost" to="/siswa" />
				</template>

				<template #right>
					<UButton
						v-if="!isLainnya"
						icon="lucide:id-card"
						variant="subtle"
						color="neutral"
						@click="kartuOpen = true">
						Kartu Peserta
					</UButton>
					<UButton icon="lucide:plus" @click="openCreate">
						Tambah Siswa
					</UButton>
				</template>
			</UDashboardNavbar>
		</template>

		<template #body>
			<UBreadcrumb :items="breadcrumbItems" class="mb-4" />

			<UAlert
				v-if="isLainnya"
				color="warning"
				variant="subtle"
				icon="lucide:triangle-alert"
				class="mb-4"
				title="Kelas siswa ini tidak ada di Data Kelas"
				description="Siswa di sini tidak bisa masuk ujian kelas mana pun. Cocokkan otomatis (misalnya IVA menjadi Kelas 4, rombel IVA), atau edit kelasnya satu per satu."
				:actions="[{ label: 'Cocokkan kelas otomatis', icon: 'lucide:wand-sparkles', loading: fixing, onClick: fixClasses }]" />

			<div v-if="rombelItems.length" class="mb-4 flex flex-wrap items-center gap-3">
				<USelect v-model="rombelFilter" :items="rombelItems" class="w-56" />
				<span class="text-sm text-muted">{{ students.length }} siswa</span>
			</div>

			<SiswaTable
				:students="students"
				:show-class="isLainnya"
				@edit="openEdit"
				@delete="deleteStudent"
				@delete-many="deleteManyStudents" />
		</template>
	</UDashboardPanel>

	<SiswaFormModal
		v-model:open="formOpen"
		:kelas="isLainnya ? '' : kelas"
		:student="editingStudent"
		@saved="loadStudents" />

	<SiswaKartuPesertaModal
		v-model:open="kartuOpen"
		:students="students"
		:default-class="kelas"
		@regenerated="loadStudents" />
</template>
