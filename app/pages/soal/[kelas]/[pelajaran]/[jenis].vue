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
	const importNaskahOpen = ref(false);
	const importNaskahFormat = ref<"docx" | "pdf">("docx");
	const openImportNaskah = (format: "docx" | "pdf") => {
		importNaskahFormat.value = format;
		importNaskahOpen.value = true;
	};
	const importMarkdownOpen = ref(false);
	const editingId = ref<number | null>(null);
	const previewOpen = ref(false);
	const previewId = ref<number | null>(null);
	const importMenuOpen = ref(false);

	// Modal di halaman ini membawa pustaka besar (editor TipTap, pembaca Word/PDF, dll.). Kodenya
	// baru dimuat saat modal pertama kali dibuka, lalu modal tetap terpasang supaya pembukaan
	// berikutnya instan. Tanpa ini, membuka daftar soal pertama kali harus memuat ±2 MB kode.
	const mountedOnFirstOpen = (open: Ref<boolean>) => {
		const mounted = ref(false);
		watch(open, (isOpen) => {
			if (isOpen) mounted.value = true;
		}, { immediate: true });
		return mounted;
	};
	const formMounted = mountedOnFirstOpen(formOpen);
	const importMounted = mountedOnFirstOpen(importOpen);
	const importNaskahMounted = mountedOnFirstOpen(importNaskahOpen);
	const importMarkdownMounted = mountedOnFirstOpen(importMarkdownOpen);
	const previewMounted = mountedOnFirstOpen(previewOpen);

	// Satu tombol Import Soal dengan pilihan format, bukan beberapa tombol terpisah —
	// alur importnya sendiri (modal Word vs CSV/Excel vs Markdown) tidak berubah, cuma pintu masuknya digabung.
	const importItems = [[
		{ label: "Dari Naskah Word (.docx)", icon: "lucide:file-text", onSelect: () => openImportNaskah("docx") },
		{ label: "Dari PDF Soal (.pdf)", icon: "lucide:file-scan", onSelect: () => openImportNaskah("pdf") },
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
		await deleteManyQuestions([question.id]);
	};

	const deleteManyQuestions = async (ids: number[]) => {
		const results = await Promise.allSettled(ids.map((id) => invoke("delete_question", { id })));
		const failed = results.filter((r): r is PromiseRejectedResult => r.status === "rejected");
		if (failed.length) {
			useToast().add({
				title: `${failed.length} soal tidak dihapus`,
				description: String(failed[0]!.reason),
				icon: "lucide:triangle-alert",
				color: "warning"
			});
		}
		await loadData();
	};

	onMounted(loadData);
</script>

<template>
	<UDashboardPanel id="soal-detail">
		<template #header>
			<UDashboardNavbar :title="`Kelas ${kelas} · ${subjectLabel} · ${jenis}`">
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

	<LazySoalFormModal
		v-if="formMounted"
		v-model:open="formOpen"
		:kelas="kelas"
		:jenis="jenis"
		:subject-id="subjectId"
		:editing-id="editingId"
		@saved="loadData" />

	<LazySoalImportModal
		v-if="importMounted"
		v-model:open="importOpen"
		:kelas="kelas"
		:jenis="jenis"
		:subject-id="subjectId"
		@saved="loadData" />

	<LazySoalImportNaskahModal
		v-if="importNaskahMounted"
		v-model:open="importNaskahOpen"
		:format="importNaskahFormat"
		:kelas="kelas"
		:jenis="jenis"
		:subject-id="subjectId"
		@saved="loadData" />

	<LazySoalImportMarkdownModal
		v-if="importMarkdownMounted"
		v-model:open="importMarkdownOpen"
		:kelas="kelas"
		:jenis="jenis"
		:subject-id="subjectId"
		@saved="loadData" />

	<LazySoalPreviewModal v-if="previewMounted" v-model:open="previewOpen" :question-id="previewId" />
</template>
