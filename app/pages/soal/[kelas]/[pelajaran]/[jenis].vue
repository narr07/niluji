<script lang="ts" setup>
	import { invoke } from "@tauri-apps/api/core";

	definePageMeta({
		hideFromNav: true
	});

	interface QuestionSummary {
		id: number
		subject: string
		class: string | null
		jenis: string | null
		questionText: string
		questionType: string
		image: string | null
		score: number
		optionCount: number
	}

	interface Subject {
		id: number
		name: string
		code: string | null
	}

	const route = useRoute();
	const kelas = computed(() => route.params.kelas as string);
	const pelajaran = computed(() => decodeURIComponent(route.params.pelajaran as string));
	const jenis = computed(() => decodeURIComponent(route.params.jenis as string));

	const allQuestions = ref<QuestionSummary[]>([]);
	const questions = computed(() =>
		allQuestions.value.filter((q) => q.class === kelas.value && q.subject === pelajaran.value && q.jenis === jenis.value)
	);
	const subjects = ref<Subject[]>([]);
	const subjectId = computed(() => subjects.value.find((s) => s.name === pelajaran.value)?.id);
	const subjectLabel = computed(() => subjects.value.find((s) => s.name === pelajaran.value)?.code || pelajaran.value);

	const breadcrumbItems = computed(() => [
		{ label: "Bank Soal", icon: "lucide:list-checks", to: "/soal" },
		{ label: `Kelas ${kelas.value}`, to: `/soal/${kelas.value}` },
		{ label: subjectLabel.value, to: `/soal/${kelas.value}/${encodeURIComponent(pelajaran.value)}` },
		{ label: jenis.value }
	]);

	const loading = ref(false);
	const formOpen = ref(false);
	const importOpen = ref(false);
	const importDocxOpen = ref(false);
	const importMarkdownOpen = ref(false);
	const editingId = ref<number | null>(null);
	const previewOpen = ref(false);
	const previewId = ref<number | null>(null);
	const importMenuOpen = ref(false);

	// Satu tombol Import Soal dengan pilihan format, bukan beberapa tombol terpisah —
	// alur importnya sendiri (modal Word vs CSV/Excel vs Markdown) tidak berubah, cuma pintu masuknya digabung.
	const importItems = [[
		{ label: "Dari Naskah Word (.docx)", icon: "lucide:file-text", onSelect: () => (importDocxOpen.value = true) },
		{ label: "Dari CSV / Excel", icon: "lucide:file-spreadsheet", onSelect: () => (importOpen.value = true) },
		{ label: "Dari file Markdown (.md)", icon: "lucide:file-code", onSelect: () => (importMarkdownOpen.value = true) }
	]];

	const loadData = async () => {
		loading.value = true;
		try {
			[allQuestions.value, subjects.value] = await Promise.all([
				invoke<QuestionSummary[]>("list_questions"),
				invoke<Subject[]>("list_subjects")
			]);
		} finally {
			loading.value = false;
		}
	};

	const openCreate = () => {
		editingId.value = null;
		formOpen.value = true;
	};

	const openEdit = (id: number) => {
		editingId.value = id;
		formOpen.value = true;
	};

	const openPreview = (id: number) => {
		previewId.value = id;
		previewOpen.value = true;
	};

	const deleteQuestion = async (question: QuestionSummary) => {
		const ok = await confirmDelete({ title: "Hapus soal ini?", description: stripSoalMarkdown(question.questionText) });
		if (!ok) return;
		await invoke("delete_question", { id: question.id });
		await loadData();
	};

	const deleteManyQuestions = async (ids: number[]) => {
		await Promise.all(ids.map((id) => invoke("delete_question", { id })));
		await loadData();
	};

	onMounted(loadData);
</script>

<template>
	<UDashboardPanel id="soal-detail">
		<template #header>
			<UDashboardNavbar :title="`Kelas ${kelas} — ${subjectLabel} — ${jenis}`">
				<template #leading>
					<UDashboardSidebarCollapse />
					<UButton icon="lucide:arrow-left" variant="ghost" :to="`/soal/${kelas}/${encodeURIComponent(pelajaran)}`" />
				</template>

				<template #right>
					<UDropdownMenu v-model:open="importMenuOpen" :items="importItems">
						<UButton
							icon="lucide:upload"
							variant="soft"
							:trailing-icon="importMenuOpen ? 'lucide:chevron-up' : 'lucide:chevron-down'">
							Import Soal
						</UButton>
					</UDropdownMenu>
					<UButton icon="lucide:plus" @click="openCreate">
						Tambah Soal
					</UButton>
				</template>
			</UDashboardNavbar>
		</template>

		<template #body>
			<UBreadcrumb :items="breadcrumbItems" class="mb-4" />
			<SoalTable
				:questions="questions"
				:loading="loading"
				@edit="openEdit"
				@delete="deleteQuestion"
				@delete-many="deleteManyQuestions"
				@preview="openPreview" />
		</template>
	</UDashboardPanel>

	<SoalFormModal
		v-model:open="formOpen"
		:kelas="kelas"
		:jenis="jenis"
		:subject-id="subjectId"
		:editing-id="editingId"
		@saved="loadData" />

	<SoalImportModal
		v-model:open="importOpen"
		:kelas="kelas"
		:jenis="jenis"
		:subject-id="subjectId"
		@saved="loadData" />

	<SoalImportDocxModal
		v-model:open="importDocxOpen"
		:kelas="kelas"
		:jenis="jenis"
		:subject-id="subjectId"
		@saved="loadData" />

	<SoalImportMarkdownModal
		v-model:open="importMarkdownOpen"
		:kelas="kelas"
		:jenis="jenis"
		:subject-id="subjectId"
		@saved="loadData" />

	<SoalPreviewModal v-model:open="previewOpen" :question-id="previewId" />
</template>
