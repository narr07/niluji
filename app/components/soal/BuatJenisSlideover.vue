<script lang="ts" setup>
	import { invoke } from "@tauri-apps/api/core";

	// Form "Buat Jenis Ujian" yang dipakai di semua level Bank Soal. Kelas lalu mata pelajaran
	// WAJIB dipilih dulu (urut seperti menu Bank Soal); kalau dibuka dari dalam halaman kelas /
	// mapel, pilihannya sudah terisi sesuai halaman itu tapi tetap bisa diganti.
	const props = defineProps<{
		kelas?: string
		pelajaran?: string
	}>();

	const open = defineModel<boolean>("open", { default: false });

	const toast = useToast();
	const { subjects } = useSubjects();
	const classes = ref<string[]>([]);

	const selectedKelas = ref<string>();
	const selectedPelajaran = ref<string>();
	const name = ref("");
	const scope = ref<"narrow" | "class" | "subject" | "global">("narrow");
	const creating = ref(false);
	const error = ref("");

	const classItems = computed(() => classes.value.map((c) => ({ label: `Kelas ${c}`, value: c })));
	const subjectItems = computed(() =>
		(subjects.value ?? []).map((s) => ({ label: s.code ? `${s.code} — ${s.name}` : s.name, value: s.name }))
	);
	const subjectId = computed(() => subjects.value?.find((s) => s.name === selectedPelajaran.value)?.id);
	const subjectLabel = computed(() => subjects.value?.find((s) => s.name === selectedPelajaran.value)?.code || selectedPelajaran.value);

	const scopeOptions = computed(() => {
		const k = selectedKelas.value ? `Kelas ${selectedKelas.value}` : "kelas terpilih";
		const p = subjectLabel.value ?? "mapel terpilih";
		return [
			{ label: `${k} · ${p} saja`, description: "Hanya berlaku di sini", value: "narrow" as const },
			{ label: `Semua mapel di ${k}`, description: `Berlaku untuk seluruh mapel di ${k}`, value: "class" as const },
			{ label: `${p} di semua kelas`, description: `Berlaku untuk ${p} di semua kelas`, value: "subject" as const },
			{ label: "Semua kelas & semua mapel", description: "Berlaku di seluruh aplikasi", value: "global" as const }
		];
	});

	const step = computed(() => (!selectedKelas.value ? 1 : !selectedPelajaran.value ? 2 : 3));

	watch(open, async (isOpen) => {
		if (!isOpen) return;
		selectedKelas.value = props.kelas;
		selectedPelajaran.value = props.pelajaran;
		name.value = "";
		scope.value = "narrow";
		error.value = "";
		classes.value = await invoke<string[]>("list_classes");
	});

	const submit = async () => {
		error.value = "";
		const trimmed = name.value.trim();
		if (!selectedKelas.value) return void (error.value = "Pilih kelas dulu.");
		if (!selectedPelajaran.value || !subjectId.value) return void (error.value = "Pilih mata pelajaran dulu.");
		if (!trimmed) return void (error.value = "Nama jenis ujian wajib diisi.");

		creating.value = true;
		try {
			await invoke("create_question_type", {
				name: trimmed,
				description: null,
				class: (scope.value === "narrow" || scope.value === "class") ? selectedKelas.value : null,
				subjectId: (scope.value === "narrow" || scope.value === "subject") ? subjectId.value : null
			});
			toast.add({ title: "Jenis ujian dibuat", description: `"${trimmed}" siap diisi soal.`, color: "success", icon: "i-lucide-check-circle" });
		} catch {
			// Jenis dengan nama + scope yang sama mungkin sudah ada — tidak masalah, tetap lanjut
			// masuk ke halaman soal jenis tersebut untuk kelas & pelajaran ini.
		} finally {
			creating.value = false;
		}
		open.value = false;
		await navigateTo(`/soal/${selectedKelas.value}/${encodeURIComponent(selectedPelajaran.value)}/${encodeURIComponent(trimmed)}`);
	};
</script>

<template>
	<USlideover v-model:open="open" title="Buat Jenis Ujian Baru" description="Pilih kelas dan mata pelajaran dulu, lalu beri nama jenis ujiannya.">
		<template #body>
			<form class="space-y-5" @submit.prevent="submit">
				<UFormField label="1. Kelas" required>
					<USelectMenu
						v-model="selectedKelas"
						:items="classItems"
						value-key="value"
						placeholder="Pilih kelas"
						class="w-full" />
				</UFormField>

				<UFormField label="2. Mata Pelajaran" required :hint="step < 2 ? 'Pilih kelas dulu' : undefined">
					<USelectMenu
						v-model="selectedPelajaran"
						:items="subjectItems"
						value-key="value"
						:disabled="step < 2"
						placeholder="Pilih mata pelajaran"
						class="w-full" />
				</UFormField>

				<UFormField label="3. Nama jenis ujian" required :hint="step < 3 ? 'Pilih mata pelajaran dulu' : undefined">
					<UInput
						v-model="name"
						:disabled="step < 3"
						placeholder="Contoh: UTS Ganjil 2026"
						class="w-full" />
				</UFormField>

				<UFormField v-if="step === 3" label="Berlaku untuk">
					<URadioGroup v-model="scope" :items="scopeOptions" />
				</UFormField>

				<UAlert
					v-if="error"
					color="error"
					variant="subtle"
					:title="error" />
			</form>
		</template>
		<template #footer>
			<UButton
				block
				:loading="creating"
				:disabled="step < 3 || !name.trim()"
				@click="submit">
				Buat Jenis Ujian
			</UButton>
		</template>
	</USlideover>
</template>
