import { invoke } from "@tauri-apps/api/core";

interface ExamRow {
	id: number
	subject: string
	class: string | null
	jenis: string | null
	scheduledAt: number | null
	windowEnd: number | null
}

interface QuestionRow {
	subject: string
	class: string | null
	jenis: string | null
}

interface SessionRow {
	studentNisn: string
	class: string | null
	subject: string
	jenis: string | null
	submittedAt: number | null
	score: number | null
}

export interface UjianTerlaksana {
	examId: number
	kelas: string
	subject: string
	jenis: string
	/** "berlangsung" = jendela ujian masih terbuka; "selesai" = sudah lewat. */
	status: "berlangsung" | "selesai"
}

// Hasil Ujian hanya menampilkan ujian yang benar-benar TERLAKSANA, disaring berlapis:
//   1. Ujiannya ada di Kelola Ujian (kelas + mapel + jenis).
//   2. Bank Soal punya soal untuk kelas + mapel + jenis itu.
//   3. Waktu mulainya sudah lewat (sedang berlangsung atau sudah selesai) — ujian yang
//      dijadwalkan untuk nanti belum muncul.
// Pengecualian: ujian yang SUDAH punya sesi siswa selalu ditampilkan, walau soalnya kemudian
// dihapus dari Bank Soal atau jadwalnya diundur — hasil yang sudah ada tidak boleh hilang dari
// tampilan. Kelas → mapel → jenis di Hasil Ujian semuanya diturunkan dari daftar ini.
export const useHasilTerlaksana = () => {
	const exams = ref<ExamRow[]>([]);
	const questions = ref<QuestionRow[]>([]);
	const sessions = ref<SessionRow[]>([]);
	const loading = ref(true);
	const now = ref(Date.now() / 1000);

	const load = async () => {
		loading.value = true;
		try {
			[exams.value, questions.value, sessions.value] = await Promise.all([
				invoke<ExamRow[]>("list_exams"),
				invoke<QuestionRow[]>("list_questions"),
				invoke<SessionRow[]>("list_exam_sessions")
			]);
			now.value = Date.now() / 1000;
		} finally {
			loading.value = false;
		}
	};

	const key = (kelas: string | null, subject: string, jenis: string | null) => `${kelas}\u0000${subject}\u0000${jenis ?? ""}`;

	const terlaksana = computed<UjianTerlaksana[]>(() => {
		const withQuestions = new Set(questions.value.map((q) => key(q.class, q.subject, q.jenis)));
		const withSessions = new Set(sessions.value.map((s) => key(s.class, s.subject, s.jenis)));

		return exams.value.flatMap((e) => {
			if (!e.class) return [];
			const k = key(e.class, e.subject, e.jenis);
			const started = e.scheduledAt !== null && e.scheduledAt <= now.value;
			if (!withSessions.has(k) && !(withQuestions.has(k) && started)) return [];
			return [{
				examId: e.id,
				kelas: e.class,
				subject: e.subject,
				jenis: e.jenis ?? "",
				status: e.windowEnd !== null && e.windowEnd > now.value ? "berlangsung" as const : "selesai" as const
			}];
		});
	});

	const classes = computed(() => new Set(terlaksana.value.map((u) => u.kelas)));
	const subjectsIn = (kelas: string) => new Set(terlaksana.value.filter((u) => u.kelas === kelas).map((u) => u.subject));
	const ujianIn = (kelas: string, subject?: string) =>
		terlaksana.value.filter((u) => u.kelas === kelas && (subject === undefined || u.subject === subject));

	onMounted(load);

	return { loading, terlaksana, classes, subjectsIn, ujianIn, sessions, reload: load };
};
