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

	// ---------- Buat jenis ujian (Slideover) ----------

	const createOpen = ref(false);
	const newJenisName = ref("");
	const scopeOptions = [
		{ label: "Kelas & mata pelajaran ini saja", description: "Hanya berlaku di sini", value: "narrow" as const },
		{ label: "Semua pelajaran di kelas ini", description: `Berlaku untuk seluruh mapel di Kelas ${props.kelas}`, value: "class" as const },
		{ label: "Semua kelas, mapel ini saja", description: `Berlaku untuk ${props.pelajaran} di semua kelas`, value: "subject" as const },
		{ label: "Semua kelas & semua mapel", description: "Berlaku di seluruh aplikasi", value: "global" as const }
	];
	const scope = ref<"narrow" | "class" | "subject" | "global">("narrow");
	const creating = ref(false);
	const createError = ref("");

	const openCreate = () => {
		newJenisName.value = "";
		scope.value = "narrow";
		createError.value = "";
		createOpen.value = true;
	};

	const createJenis = async () => {
		createError.value = "";
		const name = newJenisName.value.trim();
		if (!name) {
			createError.value = "Nama jenis ujian wajib diisi.";
			return;
		}
		creating.value = true;
		try {
			await invoke("create_question_type", {
				name,
				description: null,
				class: (scope.value === "narrow" || scope.value === "class") ? props.kelas : null,
				subjectId: (scope.value === "narrow" || scope.value === "subject") ? subjectId.value : null
			});
			toast.add({ title: "Jenis ujian dibuat", description: `"${name}" siap diisi soal.`, color: "success", icon: "i-lucide-check-circle" });
		} catch {
			// Jenis dengan nama + scope yang sama mungkin sudah ada — tidak masalah, kita tetap
			// lanjut masuk ke halaman soal jenis tersebut untuk kelas & pelajaran ini.
		} finally {
			creating.value = false;
		}
		createOpen.value = false;
		navigateTo(`/soal/${props.kelas}/${encodeURIComponent(props.pelajaran)}/${encodeURIComponent(name)}`);
	};

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
			await invoke("delete_question_type_scoped", {
				id: deletingJenis.value.id,
				scope: deleteScope.value,
				currentClass: props.kelas,
				currentSubjectId: subjectId.value
			});
			toast.add({ title: "Jenis ujian dihapus", description: `"${deletingJenis.value.jenis}" sudah dihapus.`, color: "success", icon: "i-lucide-check-circle" });
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
			<USkeleton v-for="i in 3" :key="i" class="h-24 rounded-lg" />
		</div>

		<div v-else-if="jenisGroups.length" class="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
			<UPageCard
				v-for="g in jenisGroups"
				:key="g.id"
				:ui="{ header: 'p-2 sm:px-3', wrapper: 'items-stretch' }"


				variant="subtle"
				class="cursor-pointer hover:ring-primary/60 transition-shadow"
				@click="navigateTo(`/soal/${kelas}/${encodeURIComponent(pelajaran)}/${encodeURIComponent(g.jenis)}`)"
			>
				<template #title>

					<h1 class="text-2xl">{{ g.jenis }}</h1>


				</template>
				<template #leading>
					<div class="flex items-center justify-between w-full gap-2">
						<UBadge
							color="neutral"
							variant="subtle"
							size="xl"
							class="mt-1">
							{{ g.count }} soal
						</UBadge>
						<UDropdownMenu :items="menuItemsFor(g)" :content="{ align: 'end' }">
							<UButton
								icon="i-lucide-more-vertical"
								variant="subtle"
								color="neutral"
								size="md"
								@click.stop />
						</UDropdownMenu></div>
				</template>
			</UPageCard>
		</div>

		<div v-else class="flex flex-col items-center text-center gap-3 py-16 border border-dashed border-default rounded-lg">
			<UIcon name="i-lucide-clipboard-list" class="size-8 text-muted" />
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

		<USlideover v-model:open="createOpen" title="Buat Jenis Ujian Baru">
			<template #body>
				<form class="space-y-5" @submit.prevent="createJenis">
					<UFormField label="Nama jenis ujian">
						<UInput
							v-model="newJenisName"
							placeholder="Contoh: UTS Ganjil 2026"
							class="w-full"
							autofocus />
					</UFormField>

					<UFormField label="Berlaku untuk">
						<URadioGroup v-model="scope" :items="scopeOptions" />
					</UFormField>

					<UAlert
						v-if="createError"
						color="error"
						variant="subtle"
						:title="createError" />
				</form>
			</template>
			<template #footer>
				<UButton block :loading="creating" @click="createJenis">
					Buat Jenis Ujian
				</UButton>
			</template>
		</USlideover>

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
						color="warning"
						variant="subtle"
						icon="i-lucide-info"
						description="Soal yang sudah memakai jenis ini tidak ikut terhapus, tapi tidak akan muncul di jenis manapun lagi pada cakupan yang dipilih." />

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