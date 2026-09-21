<script lang="ts" setup>
	import { invoke } from "@tauri-apps/api/core";

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

	const props = defineProps<{
		subjects: Subject[]
		questions: QuestionSummary[]
		editingExam: Exam | null
	}>();

	const emit = defineEmits<{
		cancel: []
		saved: []
	}>();

	const saving = ref(false);
	const formError = ref("");

	const pad = (n: number) => n.toString().padStart(2, "0");
	const dateParts = (timestamp: number) => {
		const d = new Date(timestamp * 1000);
		return {
			date: `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`,
			time: `${pad(d.getHours())}:${pad(d.getMinutes())}`
		};
	};
	const toTimestamp = (date: string, time: string) => Math.floor(new Date(`${date}T${time}`).getTime() / 1000);

	const today = () => {
		const d = new Date();
		return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
	};

	const emptyForm = () => ({
		subject: undefined as { label: string, value: number } | undefined,
		class: undefined as string | undefined,
		jenis: undefined as string | undefined,
		title: "",
		examDate: today(),
		startTime: "",
		endTime: "",
		duration: 60,
		token: ""
	});

	const form = reactive(emptyForm());

	// Kelas & mata pelajaran yang bisa dipilih dibatasi ke yang sudah punya soal di Bank
	// Soal — gak masuk akal bikin ujian buat kombinasi yang soalnya masih kosong.
	const classesWithQuestions = computed(() => [...new Set(props.questions.map((q) => q.class).filter((c): c is string => !!c))].sort());

	const subjectItemsForClass = computed(() => {
		if (!form.class) return [];
		const names = new Set(props.questions.filter((q) => q.class === form.class).map((q) => q.subject));
		return props.subjects.filter((s) => names.has(s.name)).map((s) => ({ label: s.code || s.name, value: s.id }));
	});

	// Jenis soal (paket soal) yang sudah ada buat kombinasi kelas + mata pelajaran ini —
	// dipilih, bukan diketik manual, biar ujian baru gak perlu ganti soal yang sudah ada.
	const jenisItemsForClassAndSubject = computed(() => {
		if (!form.class || !form.subject) return [];
		const subjectName = props.subjects.find((s) => s.id === form.subject!.value)?.name;
		return [
			...new Set(
				props.questions
					.filter((q) => q.class === form.class && q.subject === subjectName && q.jenis)
					.map((q) => q.jenis as string)
			)
		];
	});

	const onJenisSelected = (value: string | undefined) => {
		form.jenis = value;
		if (value) form.title = value;
	};

	watch(
		() => props.editingExam,
		(exam) => {
			formError.value = "";
			if (!exam) {
				Object.assign(form, emptyForm());
				return;
			}
			const subject = props.subjects.find((s) => s.name === exam.subject);
			const { date, time } = exam.scheduledAt ? dateParts(exam.scheduledAt) : { date: "", time: "" };
			const endTime = exam.windowEnd ? dateParts(exam.windowEnd).time : "";
			Object.assign(form, {
				subject: subject ? { label: subject.code || subject.name, value: subject.id } : undefined,
				class: exam.class ?? undefined,
				jenis: exam.jenis ?? undefined,
				title: exam.title,
				examDate: date,
				startTime: time,
				endTime,
				duration: exam.duration,
				token: exam.token
			});
		},
		{ immediate: true }
	);

	const submit = async () => {
		formError.value = "";
		if (
			!form.subject
			|| !form.class
			|| !form.title.trim()
			|| !form.token.trim()
			|| !form.examDate
			|| !form.startTime
			|| !form.endTime
		) {
			formError.value = "Kelas, mata pelajaran, jenis soal, tanggal, jam mulai, jam selesai, dan token wajib diisi.";
			return;
		}

		const scheduledAt = toTimestamp(form.examDate, form.startTime);
		const windowEnd = toTimestamp(form.examDate, form.endTime);
		if (Number.isNaN(scheduledAt) || Number.isNaN(windowEnd)) {
			formError.value = "Tanggal/jam tidak valid.";
			return;
		}
		if (windowEnd <= scheduledAt) {
			formError.value = "Jam selesai harus setelah jam mulai.";
			return;
		}

		saving.value = true;
		try {
			const payload = {
				subjectId: form.subject.value,
				class: form.class,
				jenis: form.jenis ?? null,
				title: form.title,
				duration: form.duration,
				scheduledAt,
				windowEnd,
				token: form.token
			};
			if (props.editingExam) {
				await invoke("update_exam", { id: props.editingExam.id, ...payload });
			} else {
				await invoke("create_exam", payload);
			}
			emit("saved");
		} catch (e) {
			formError.value = e instanceof Error ? e.message : String(e);
		} finally {
			saving.value = false;
		}
	};
</script>

<template>
	<div>
		<form class="grid gap-4 sm:grid-cols-2" @submit.prevent="submit">
			<UFormField label="Kelas" :description="classesWithQuestions.length ? undefined : 'Belum ada soal di Bank Soal.'">
				<USelectMenu
					v-model="form.class"
					:items="classesWithQuestions"
					placeholder="Pilih kelas"
					@update:model-value="form.subject = undefined; form.jenis = undefined" />
			</UFormField>

			<UFormField
				label="Mata Pelajaran"
				:description="form.class && subjectItemsForClass.length === 0 ? 'Belum ada soal untuk kelas ini.' : undefined"
			>
				<USelectMenu
					v-model="form.subject"
					:items="subjectItemsForClass"
					:disabled="!form.class"
					placeholder="Pilih mata pelajaran"
					@update:model-value="form.jenis = undefined"
				/>
			</UFormField>

			<UFormField
				label="Jenis Soal"
				class="sm:col-span-2"
				:description="form.subject && jenisItemsForClassAndSubject.length === 0 ? 'Belum ada jenis soal untuk kelas & mata pelajaran ini — buat dulu di Bank Soal.' : 'Menentukan soal mana yang dipakai untuk ujian ini.'"
			>
				<USelectMenu
					:model-value="form.jenis"
					:items="jenisItemsForClassAndSubject"
					:disabled="!form.subject"
					placeholder="Pilih jenis soal"
					@update:model-value="onJenisSelected"
				/>
			</UFormField>

			<UFormField label="Tanggal Ujian">
				<UInput v-model="form.examDate" type="date" class="w-full" />
			</UFormField>

			<UFormField label="Durasi Pengerjaan (menit)" description="Waktu yang didapat tiap siswa sejak mereka mulai">
				<UInputNumber v-model="form.duration" :min="1" class="w-full" />
			</UFormField>

			<UFormField label="Jam Mulai" description="Kapan siswa boleh mulai membuka ujian">
				<UInput v-model="form.startTime" type="time" class="w-full" />
			</UFormField>

			<UFormField label="Jam Selesai" description="Batas terakhir siswa boleh mulai membuka ujian">
				<UInput v-model="form.endTime" type="time" class="w-full" />
			</UFormField>

			<UFormField label="Token Masuk">
				<UInput v-model="form.token" placeholder="Contoh: UTS2026" />
			</UFormField>

			<UAlert
				v-if="formError"
				color="error"
				variant="subtle"
				class="sm:col-span-2"
				:title="formError" />

			<div class="sm:col-span-2 flex gap-2">
				<UButton type="submit" :loading="saving">
					{{ editingExam ? "Simpan Perubahan" : "Simpan Ujian" }}
				</UButton>
				<UButton variant="soft" color="neutral" @click="emit('cancel')">
					Batal
				</UButton>
			</div>
		</form>
	</div>
</template>
