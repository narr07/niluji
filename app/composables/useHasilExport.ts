import { invoke } from "@tauri-apps/api/core";
import { save as saveDialog } from "@tauri-apps/plugin-dialog";

export interface HasilSessionRow {
	studentName: string
	studentNisn: string
	examTitle: string
	class: string | null
	subject: string
	jenis: string | null
	totalQuestions: number
	answeredCount: number
	pgTotal: number
	pgAnswered: number
	submittedAt: number | null
	score: number | null
	essayScore?: number | null
	finalScore?: number | null
	essayPending?: number
	autoSubmitted?: boolean
}

const finalScoreText = (s: HasilSessionRow) => {
	if (s.finalScore !== null && s.finalScore !== undefined) return s.finalScore.toFixed(0);
	return s.submittedAt && s.essayPending ? "Menunggu nilai esai" : "-";
};

const statusText = (s: HasilSessionRow) => {
	if (s.submittedAt) return s.autoSubmitted ? "Selesai otomatis" : "Selesai";
	if (s.pgTotal > 0 && s.pgAnswered >= s.pgTotal && s.totalQuestions > s.pgTotal) return "PG selesai, lanjut esai";
	if (s.pgTotal > 0 && s.pgAnswered < s.pgTotal) return `Mengerjakan PG (${s.pgAnswered}/${s.pgTotal})`;
	return "Sedang mengerjakan";
};

const headerRow = ["Nama", "NISN", "Mata Pelajaran", "Jenis Ujian", "Ujian", "Progress", "Status", "Nilai PG", "Nilai Esai", "Nilai Akhir"];

const toRow = (s: HasilSessionRow) => [
	s.studentName,
	s.studentNisn,
	s.subject,
	s.jenis ?? "-",
	s.examTitle,
	`${s.answeredCount}/${s.totalQuestions}`,
	statusText(s),
	s.score === null || s.score === undefined || s.pgTotal === 0 ? "-" : s.score.toFixed(0),
	s.essayScore === null || s.essayScore === undefined ? "-" : s.essayScore.toFixed(0),
	finalScoreText(s)
];

// SheetJS dan JSZip dimuat saat guru benar-benar mengekspor, supaya halaman Hasil Ujian cepat terbuka.
const sheetBytes = async (rows: (string | number)[][]): Promise<Uint8Array> => {
	const XLSX = await import("xlsx");
	const sheet = XLSX.utils.aoa_to_sheet(rows);
	const workbook = XLSX.utils.book_new();
	XLSX.utils.book_append_sheet(workbook, sheet, "Hasil");
	return new Uint8Array(XLSX.write(workbook, { type: "array", bookType: "xlsx" }) as ArrayBuffer);
};

export const useHasilExport = () => {
	const toast = useToast();

	const reportError = (context: string, error: unknown) => {
		const message = error instanceof Error ? error.message : String(error);
		console.error(`[export] ${context}:`, error);
		toast.add({ title: "Export gagal", description: message, icon: "lucide:x", color: "error" });
	};

	const exportOneXlsx = async (defaultFilename: string, sessions: HasilSessionRow[]) => {
		if (!sessions.length) {
			toast.add({ title: "Tidak ada data untuk diekspor", color: "warning", icon: "lucide:alert-triangle" });
			return;
		}
		try {
			const path = await saveDialog({ defaultPath: defaultFilename, filters: [{ name: "Excel", extensions: ["xlsx"] }] });
			if (!path) return;

			const bytes = await sheetBytes([headerRow, ...sessions.map(toRow)]);
			await invoke("write_file_bytes", { path, bytes: Array.from(bytes) });

			toast.add({ title: "Export selesai", description: `File disimpan di ${path}`, icon: "lucide:check", color: "success" });
		} catch (error) {
			reportError("exportOneXlsx", error);
		}
	};

	const exportZipOfXlsx = async (defaultFilename: string, files: { name: string, sessions: HasilSessionRow[] }[]) => {
		if (!files.length) {
			toast.add({ title: "Tidak ada data untuk diekspor", color: "warning", icon: "lucide:alert-triangle" });
			return;
		}
		try {
			const path = await saveDialog({ defaultPath: defaultFilename, filters: [{ name: "Zip", extensions: ["zip"] }] });
			if (!path) return;

			const { default: JSZip } = await import("jszip");
			const zip = new JSZip();
			for (const file of files) {
				zip.file(`${file.name}.xlsx`, await sheetBytes([headerRow, ...file.sessions.map(toRow)]));
			}
			const bytes = await zip.generateAsync({ type: "uint8array" });
			await invoke("write_file_bytes", { path, bytes: Array.from(bytes) });

			toast.add({
				title: "Export selesai",
				description: `${files.length} file disimpan dalam ${path}`,
				icon: "lucide:check",
				color: "success"
			});
		} catch (error) {
			reportError("exportZipOfXlsx", error);
		}
	};

	const safeName = (s: string) => s.trim().replace(/[\\/:*?"<>|]+/g, "-").replace(/\s+/g, "-");

	interface AnswerDetail {
		questionId: number
		questionText: string
		questionType: string
		studentAnswer: string | null
		correctAnswer: string | null
		isCorrect: boolean
		essayScore: number | null
		maxScore: number
	}

	interface SessionDetail {
		studentName: string
		examTitle: string
		subject: string
		items: AnswerDetail[]
	}

	const detailHeaderRow = ["Mata Pelajaran", "Jenis Ujian", "Ujian", "No", "Soal", "Tipe", "Jawaban Siswa", "Kunci Jawaban", "Hasil"];

	const answerResult = (item: AnswerDetail) => {
		if (item.questionType === "essay") {
			if (item.essayScore !== null) return `${item.essayScore}/${item.maxScore}`;
			return item.studentAnswer?.trim() ? "Belum dinilai" : "Tidak dijawab";
		}
		return item.isCorrect ? "Benar" : "Salah";
	};

	const fetchSessionDetailRows = async (sessionId: number, jenis: string | null): Promise<(string | number)[][]> => {
		const detail = await invoke<SessionDetail>("get_exam_session_detail", { sessionId });
		return detail.items.map((item, i) => [
			detail.subject,
			jenis ?? "-",
			detail.examTitle,
			i + 1,
			item.questionText,
			item.questionType === "essay" ? "Esai" : "PG",
			item.studentAnswer ?? "-",
			item.questionType === "essay" ? "-" : (item.correctAnswer ?? "-"),
			answerResult(item)
		]);
	};

	const exportStudentDetailXlsx = async (
		defaultFilename: string,
		sessions: { sessionId: number, jenis: string | null }[]
	) => {
		if (!sessions.length) {
			toast.add({ title: "Tidak ada data untuk diekspor", color: "warning", icon: "lucide:alert-triangle" });
			return;
		}
		try {
			const path = await saveDialog({ defaultPath: defaultFilename, filters: [{ name: "Excel", extensions: ["xlsx"] }] });
			if (!path) return;

			const rows = (await Promise.all(sessions.map((s) => fetchSessionDetailRows(s.sessionId, s.jenis)))).flat();
			const bytes = await sheetBytes([detailHeaderRow, ...rows]);
			await invoke("write_file_bytes", { path, bytes: Array.from(bytes) });

			toast.add({ title: "Export selesai", description: `File disimpan di ${path}`, icon: "lucide:check", color: "success" });
		} catch (error) {
			reportError("exportStudentDetailXlsx", error);
		}
	};

	const exportStudentDetailZip = async (
		defaultFilename: string,
		students: { name: string, sessions: { sessionId: number, jenis: string | null }[] }[]
	) => {
		if (!students.length) {
			toast.add({ title: "Tidak ada data untuk diekspor", color: "warning", icon: "lucide:alert-triangle" });
			return;
		}
		try {
			const path = await saveDialog({ defaultPath: defaultFilename, filters: [{ name: "Zip", extensions: ["zip"] }] });
			if (!path) return;

			const { default: JSZip } = await import("jszip");
			const zip = new JSZip();
			for (const student of students) {
				const rows = (await Promise.all(student.sessions.map((s) => fetchSessionDetailRows(s.sessionId, s.jenis)))).flat();
				zip.file(`${safeName(student.name)}.xlsx`, await sheetBytes([detailHeaderRow, ...rows]));
			}
			const bytes = await zip.generateAsync({ type: "uint8array" });
			await invoke("write_file_bytes", { path, bytes: Array.from(bytes) });

			toast.add({
				title: "Export selesai",
				description: `${students.length} file disimpan dalam ${path}`,
				icon: "lucide:check",
				color: "success"
			});
		} catch (error) {
			reportError("exportStudentDetailZip", error);
		}
	};

	return { exportOneXlsx, exportZipOfXlsx, exportStudentDetailXlsx, exportStudentDetailZip, safeName };
};
