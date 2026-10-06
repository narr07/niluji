<script lang="ts" setup>
	import { invoke } from "@tauri-apps/api/core";

	const props = defineProps<{
		kelas: string
		pelajaran: string
	}>();

	interface QuestionSummary {
		subject: string
		class: string | null
		jenis: string | null
	}

	interface QuestionTypeRecord {
		id: number
		name: string
	}

	interface Subject {
		id: number
		name: string
	}

	const toast = useToast();

	const questions = ref<QuestionSummary[]>([]);
	const questionTypes = ref<QuestionTypeRecord[]>([]);
	const subjects = ref<Subject[]>([]);
	const loading = ref(true);

	const subjectId = computed(() => subjects.value.find((s) => s.name === props.pelajaran)?.id);

	const load = async () => {
		loading.value = true;
		try {
			subjects.value = await invoke<Subject[]>("list_subjects");
			[questionTypes.value, questions.value] = await Promise.all([
				invoke<QuestionTypeRecord[]>("list_question_types", { class: props.kelas, subjectId: subjectId.value }),
				invoke<QuestionSummary[]>("list_questions")
			]);
		} finally {
			loading.value = false;
		}
	};

	const jenisGroups = computed(() =>
		questionTypes.value.map((t) => ({
			id: t.id,
			jenis: t.name,
			count: questions.value.filter((q) => q.class === props.kelas && q.subject === props.pelajaran && q.jenis === t.name).length
		}))
	);

	// ---------- Buat jenis ujian (Slideover bersama, lihat SoalBuatJenisSlideover) ----------

	const createOpen = ref(false);
	const openCreate = () => (createOpen.value = true);

	// ---------- Edit jenis ujian (Modal) ----------

	const editOpen = ref(false);
	const editingId = ref<number | null>(null);
	const editName = ref("");
	const editError = ref("");
	const savingEdit = ref(false);

	const openEdit = (g: { id: number, jenis: string }) => {
		editingId.value = g.id;
		editName.value = g.jenis;
		editError.value = "";
		editOpen.value = true;
	};

	const saveEdit = async () => {
		editError.value = "";
		const name = editName.value.trim();
		if (!name) {
			editError.value = "Nama jenis ujian wajib diisi.";
			return;
		}
		savingEdit.value = true;
		try {
			await invoke("update_question_type", { id: editingId.value, name, description: null });
			toast.add({ title: "Perubahan disimpan", color: "success", icon: "i-lucide-check-circle" });
			editOpen.value = false;
			await load();
		} catch (e) {
			editError.value = e instanceof Error ? e.message : String(e);
		} finally {
			savingEdit.value = false;
		}
	};

	// ---------- Hapus jenis ujian (Modal) ----------

	const deleteOpen = ref(false);
	const deletingJenis = ref<{ id: number, jenis: string } | null>(null);
	const deleteScope = ref<"subject" | "class" | "all">("subject");
	const deleting = ref(false);
	const deleteError = ref("");
	const deleteScopeOptions = computed(() => [
		{ label: `Hanya di mata pelajaran "${props.pelajaran}"`, description: "Semua kelas untuk mapel ini", value: "subject" as const },
		{ label: `Hanya di Kelas ${props.kelas}`, description: "Semua mapel untuk kelas ini", value: "class" as const },
		{ label: "Di semua kelas & semua mata pelajaran", description: "Hapus total dari aplikasi", value: "all" as const }
	]);

	const removeJenis = (g: { id: number, jenis: string }) => {
		deletingJenis.value = g;
		deleteScope.value = "subject";
		deleteError.value = "";
		deleteOpen.value = true;
	};

	const confirmDelete = async () => {
		if (!deletingJenis.value || !subjectId.value) return;
		deleting.value = true;
		deleteError.value = "";
		try {
			const deletedQuestions = await invoke<number>("delete_question_type_scoped", {
				id: deletingJenis.value.id,
				scope: deleteScope.value,
				currentClass: props.kelas,
				currentSubjectId: subjectId.value
			});
			toast.add({
				title: "Jenis ujian dihapus",
				description: `"${deletingJenis.value.jenis}" sudah dihapus${deletedQuestions ? `, beserta ${deletedQuestions} soal di dalamnya` : ""}.`,
				color: "success",
				icon: "i-lucide-check-circle"
			});
			deleteOpen.value = false;
			await load();
		} catch (e) {
			deleteError.value = e instanceof Error ? e.message : String(e);
		} finally {
			deleting.value = false;
		}
	};

	const menuItemsFor = (g: { id: number, jenis: string }) => [
		[{ label: "Edit nama", icon: "i-lucide-pencil", onSelect: () => openEdit(g) }],
		[{ label: "Hapus", icon: "i-lucide-trash-2", color: "error" as const, onSelect: () => removeJenis(g) }]
	];

	onMounted(load);
</script>

<template>
	<div class="space-y-6">
		<div class="flex items-start justify-between gap-4">
			<div>
				<h2 class="text-lg font-semibold text-highlighted">
					Jenis Ujian
				</h2>
				<p class="text-sm text-muted mt-0.5">
					Kelas {{ kelas }} · {{ pelajaran }}
				</p>
			</div>
			<UButton icon="i-lucide-plus" @click="openCreate">
				Buat Jenis Ujian
			</UButton>
		</div>

		<div v-if="loading" class="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
			<USkeleton v-for="i in 3" :key="i" class="h-28 rounded-lg" />
		</div>

		<div v-else-if="jenisGroups.length" class="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
			<div
				v-for="g in jenisGroups"
				:key="g.id"
				class="flex flex-col gap-3 rounded-lg bg-elevated/50 ring ring-default p-4 sm:p-6 cursor-pointer hover:ring-primary/60 transition-shadow"
				@click="navigateTo(`/soal/${kelas}/${encodeURIComponent(pelajaran)}/${encodeURIComponent(g.jenis)}`)"
			>
				<div class="flex items-start justify-between gap-2">
					<h3 class="text-base font-semibold text-highlighted leading-snug break-words min-w-0 flex-1">
						{{ g.jenis }}
					</h3>
					<UDropdownMenu :items="menuItemsFor(g)" :content="{ align: 'end' }">
						<UButton
							icon="i-lucide-more-vertical"
							variant="ghost"
							color="neutral"
							size="sm"
							class="shrink-0 -mr-1.5 -mt-1"
							@click.stop />
					</UDropdownMenu>
				</div>

				<UBadge
					color="neutral"
					variant="subtle"
					size="lg"
					class="font-medium self-start">
					{{ g.count }} soal
				</UBadge>
			</div>
		</div>

		<div v-else class="flex flex-col items-center text-center gap-3 py-16 border border-dashed border-default rounded-lg">
			<div class="flex items-center justify-center size-12 rounded-full bg-elevated">
				<UIcon name="i-lucide-clipboard-list" class="size-6 text-muted" />
			</div>
			<div>
				<p class="font-medium text-highlighted">
					Belum ada jenis ujian
				</p>
				<p class="text-sm text-muted mt-1">
					Buat jenis ujian dulu, misalnya "UTS Ganjil 2026", sebelum menambahkan soal.
				</p>
			</div>
			<UButton icon="i-lucide-plus" variant="subtle" @click="openCreate">
				Buat Jenis Ujian
			</UButton>
		</div>

		<SoalBuatJenisSlideover v-model:open="createOpen" :kelas="kelas" :pelajaran="pelajaran" />

		<UModal v-model:open="editOpen" title="Edit Jenis Ujian">
			<template #body>
				<form class="space-y-4" @submit.prevent="saveEdit">
					<UFormField label="Nama">
						<UInput
							v-model="editName"
							placeholder="Contoh: UTS Ganjil 2026"
							class="w-full"
							autofocus />
					</UFormField>

					<UAlert
						v-if="editError"
						color="error"
						variant="subtle"
						:title="editError" />

					<UButton type="submit" block :loading="savingEdit">
						Simpan Perubahan
					</UButton>
				</form>
			</template>
		</UModal>

		<UModal v-model:open="deleteOpen" :title="`Hapus &quot;${deletingJenis?.jenis}&quot;?`">
			<template #body>
				<div class="space-y-4">
					<p class="text-sm text-muted">
						Jenis ujian ini mungkin dipakai di kelas/pelajaran lain juga. Pilih cakupan penghapusannya
						supaya tidak ikut hilang dari kelas/pelajaran yang tidak kamu maksud.
					</p>

					<URadioGroup v-model="deleteScope" :items="deleteScopeOptions" />

					<UAlert
						color="error"
						variant="subtle"
						icon="i-lucide-triangle-alert"
						title="Soal di dalamnya ikut terhapus permanen"
						description="Semua soal jenis ini pada cakupan yang dipilih akan dihapus dan tidak bisa dikembalikan. Soal di kelas/mapel lain yang masih memakai jenis ini tidak tersentuh." />

					<UAlert
						v-if="deleteError"
						color="error"
						variant="subtle"
						:title="deleteError" />

					<div class="flex gap-2">
						<UButton color="error" :loading="deleting" @click="confirmDelete">
							Hapus
						</UButton>
						<UButton variant="soft" color="neutral" @click="deleteOpen = false">
							Batal
						</UButton>
					</div>
				</div>
			</template>
		</UModal>
	</div>
</template>