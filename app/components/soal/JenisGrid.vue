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

	const questions = ref<QuestionSummary[]>([]);
	const questionTypes = ref<QuestionTypeRecord[]>([]);
	const newJenisName = ref("");
	const creating = ref(false);
	const errorMessage = ref("");

	const load = async () => {
		[questionTypes.value, questions.value] = await Promise.all([
			invoke<QuestionTypeRecord[]>("list_question_types"),
			invoke<QuestionSummary[]>("list_questions")
		]);
	};

	// Semua jenis yang pernah dibuat ditampilkan di sini (bukan cuma yang udah ada soalnya),
	// biar jenis yang baru dibuat langsung kelihatan meski belum diisi soal sama sekali —
	// sama kayak Kelas/Pelajaran yang tetap muncul walau masih kosong.
	const jenisGroups = computed(() =>
		questionTypes.value.map((t) => ({
			id: t.id,
			jenis: t.name,
			count: questions.value.filter((q) => q.class === props.kelas && q.subject === props.pelajaran && q.jenis === t.name).length
		}))
	);

	const createJenis = async () => {
		errorMessage.value = "";
		const name = newJenisName.value.trim();
		if (!name) {
			errorMessage.value = "Nama jenis ujian wajib diisi.";
			return;
		}
		creating.value = true;
		try {
			await invoke("create_question_type", { name, description: null });
		} catch {
			// Jenis dengan nama sama mungkin sudah ada di daftar global — itu tidak masalah,
			// kita tetap lanjut masuk ke halaman soal jenis tersebut untuk kelas & pelajaran ini.
		} finally {
			creating.value = false;
		}
		navigateTo(`/soal/${props.kelas}/${encodeURIComponent(props.pelajaran)}/${encodeURIComponent(name)}`);
	};

	const editOpen = ref(false);
	const editingId = ref<number | null>(null);
	const editName = ref("");
	const savingEdit = ref(false);

	const openEdit = (g: { id: number, jenis: string }) => {
		editingId.value = g.id;
		editName.value = g.jenis;
		errorMessage.value = "";
		editOpen.value = true;
	};

	const saveEdit = async () => {
		errorMessage.value = "";
		if (!editName.value.trim()) {
			errorMessage.value = "Nama jenis ujian wajib diisi.";
			return;
		}
		savingEdit.value = true;
		try {
			await invoke("update_question_type", { id: editingId.value, name: editName.value, description: null });
			editOpen.value = false;
			await load();
		} catch (e) {
			errorMessage.value = e instanceof Error ? e.message : String(e);
		} finally {
			savingEdit.value = false;
		}
	};

	const removeJenis = async (g: { id: number, jenis: string }) => {
		if (!confirm(`Hapus jenis ujian "${g.jenis}"? Soal yang sudah memakai jenis ini tidak ikut terhapus, tapi tidak akan muncul di jenis manapun lagi.`)) return;
		await invoke("delete_question_type", { id: g.id });
		await load();
	};

	onMounted(load);
</script>

<template>
	<div class="space-y-6">
		<UCard>
			<template #header>
				<span class="font-semibold">Buat Jenis Ujian Baru</span>
			</template>
			<form class="flex flex-col sm:flex-row gap-3" @submit.prevent="createJenis">
				<UInput v-model="newJenisName" placeholder="Contoh: UTS Ganjil 2026" class="flex-1" />
				<UButton type="submit" icon="lucide:plus" :loading="creating">
					Jenis Ujian
				</UButton>
			</form>
			<UAlert
				v-if="errorMessage && !editOpen"
				color="error"
				variant="subtle"
				class="mt-4"
				:title="errorMessage" />
			<p class="text-sm text-muted mt-3">
				Setiap jenis ujian punya soal sendiri-sendiri, jadi bikin ujian baru tidak perlu ganti/hapus soal yang sudah ada.
			</p>
		</UCard>

		<div v-if="jenisGroups.length" class="grid gap-4 sm:grid-cols-3 lg:grid-cols-4">
			<UCard
				v-for="g in jenisGroups"
				:key="g.jenis"
				class="relative cursor-pointer hover:ring-2 hover:ring-primary transition-shadow"
				@click="navigateTo(`/soal/${kelas}/${encodeURIComponent(pelajaran)}/${encodeURIComponent(g.jenis)}`)"
			>
				<div class="absolute top-2 right-2 flex gap-1">
					<UButton
						size="xs"
						variant="ghost"
						icon="lucide:pencil"
						@click.stop="openEdit(g)" />
					<UButton
						size="xs"
						variant="ghost"
						color="error"
						icon="lucide:trash-2"
						@click.stop="removeJenis(g)" />
				</div>

				<div class="text-center py-4">
					<p class="text-lg font-semibold">
						{{ g.jenis }}
					</p>
					<p class="text-muted text-sm mt-1">
						{{ g.count }} soal
					</p>
				</div>
			</UCard>
		</div>
		<p v-else class="text-center text-muted py-6">
			Belum ada jenis ujian sama sekali. Buat dulu di atas.
		</p>

		<UModal v-model:open="editOpen" title="Edit Jenis Ujian">
			<template #body>
				<form class="space-y-4" @submit.prevent="saveEdit">
					<UFormField label="Nama">
						<UInput v-model="editName" placeholder="Contoh: UTS Ganjil 2026" />
					</UFormField>

					<UAlert
						v-if="errorMessage"
						color="error"
						variant="subtle"
						:title="errorMessage" />

					<UButton type="submit" block :loading="savingEdit">
						Simpan Perubahan
					</UButton>
				</form>
			</template>
		</UModal>
	</div>
</template>
