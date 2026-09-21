<script lang="ts" setup>
	import { invoke } from "@tauri-apps/api/core";

	definePageMeta({
		name: "Kelola Ujian",
		icon: "lucide:calendar-clock",
		category: "cbt",
		order: 3,
		description: "Kelola sesi ujian: kelas, mata pelajaran, jadwal, dan token"
	});

	interface Subject {
		id: number
		name: string
		code: string | null
	}

	interface QuestionSummary {
		subject: string
		class: string | null
		jenis: string | null
	}

	interface Exam {
		id: number
		subject: string
		class: string | null
		jenis: string | null
		title: string
		duration: number
		scheduledAt: number | null
		windowEnd: number | null
		token: string
	}

	const subjects = ref<Subject[]>([]);
	const exams = ref<Exam[]>([]);
	const questions = ref<QuestionSummary[]>([]);
	const editingExam = ref<Exam | null>(null);
	const formOpen = ref(false);

	const loadData = async () => {
		[subjects.value, exams.value, questions.value] = await Promise.all([
			invoke<Subject[]>("list_subjects"),
			invoke<Exam[]>("list_exams"),
			invoke<QuestionSummary[]>("list_questions")
		]);
	};

	const openCreate = () => {
		editingExam.value = null;
		formOpen.value = true;
	};

	const openEdit = (exam: Exam) => {
		editingExam.value = exam;
		formOpen.value = true;
	};

	const onSaved = async () => {
		formOpen.value = false;
		editingExam.value = null;
		await loadData();
	};

	const deleteExam = async (exam: Exam) => {
		if (!confirm(`Hapus ujian "${exam.title}"? Semua sesi & hasil siswa untuk ujian ini juga akan terhapus.`)) return;
		await invoke("delete_exam", { id: exam.id });
		if (editingExam.value?.id === exam.id) editingExam.value = null;
		await loadData();
	};

	onMounted(loadData);
</script>

<template>
	<UDashboardPanel id="ujian">
		<template #header>
			<UDashboardNavbar title="Kelola Ujian">
				<template #leading>
					<UDashboardSidebarCollapse />
				</template>
				<template #right>
					<UButton icon="lucide:plus" @click="openCreate">
						Buat Ujian Baru
					</UButton>
				</template>
			</UDashboardNavbar>
		</template>

		<template #body>
			<UjianTable
				:exams="exams"
				:subjects="subjects"
				@edit="openEdit"
				@delete="deleteExam" />
		</template>
	</UDashboardPanel>

	<UModal v-model:open="formOpen" :title="editingExam ? 'Edit Ujian' : 'Buat Ujian Baru'" :ui="{ content: 'max-w-3xl' }">
		<template #body>
			<UjianForm
				:subjects="subjects"
				:questions="questions"
				:editing-exam="editingExam"
				@cancel="formOpen = false"
				@saved="onSaved" />
		</template>
	</UModal>
</template>
