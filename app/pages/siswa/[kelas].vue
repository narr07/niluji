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
	}

	const route = useRoute();
	const kelas = computed(() => route.params.kelas as string);

	const breadcrumbItems = computed(() => [
		{ label: "Siswa", icon: "lucide:users", to: "/siswa" },
		{ label: `Kelas ${kelas.value}` }
	]);

	const allStudents = ref<StudentRecord[]>([]);
	const students = computed(() => allStudents.value.filter((s) => s.class === kelas.value));

	const formOpen = ref(false);
	const editingStudent = ref<StudentRecord | null>(null);

	const loadStudents = async () => {
		allStudents.value = await invoke<StudentRecord[]>("list_students");
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
		if (!confirm(`Hapus siswa ${student.name}?`)) return;
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
			<UDashboardNavbar :title="`Siswa Kelas ${kelas}`">
				<template #leading>
					<UDashboardSidebarCollapse />
					<UButton icon="lucide:arrow-left" variant="ghost" to="/siswa" />
				</template>

				<template #right>
					<UButton icon="lucide:plus" @click="openCreate">
						Tambah Siswa
					</UButton>
				</template>
			</UDashboardNavbar>
		</template>

		<template #body>
			<UBreadcrumb :items="breadcrumbItems" class="mb-4" />
			<SiswaTable
				:students="students"
				@edit="openEdit"
				@delete="deleteStudent"
				@delete-many="deleteManyStudents" />
		</template>
	</UDashboardPanel>

	<SiswaFormModal
		v-model:open="formOpen"
		:kelas="kelas"
		:student="editingStudent"
		@saved="loadStudents" />
</template>
