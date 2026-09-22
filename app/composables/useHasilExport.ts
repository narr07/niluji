import { invoke } from "@tauri-apps/api/core";
import { save as saveDialog } from "@tauri-apps/plugin-dialog";
import JSZip from "jszip";
import * as XLSX from "xlsx";

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
}

const statusText = (s: HasilSessionRow) => {
	if (s.submittedAt) return "Selesai";
	if (s.pgTotal > 0 && s.pgAnswered >= s.pgTotal && s.totalQuestions > s.pgTotal) return "PG selesai, lanjut esai";
	if (s.pgTotal > 0 && s.pgAnswered < s.pgTotal) return `Mengerjakan PG (${s.pgAnswered}/${s.pgTotal})`;
	return "Sedang mengerjakan";
};

const headerRow = ["Nama", "NISN", "Mata Pelajaran", "Jenis Ujian", "Ujian", "Progress", "Status", "Nilai"];

const toRow = (s: HasilSessionRow) => [
	s.studentName,
	s.studentNisn,
	s.subject,
	s.jenis ?? "-",
	s.examTitle,
	`${s.answeredCount}/${s.totalQuestions}`,
	statusText(s),
	s.score === null ? "-" : s.score.toFixed(0)
];

const sheetBytes = (rows: (string | number)[][]): Uint8Array => {
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

	// Satu file .xlsx — dipakai untuk "Export Semua" (satu jenis ujian) dan "Export per Siswa"
	// (satu siswa terpilih).
	const exportOneXlsx = async (defaultFilename: string, sessions: HasilSessionRow[]) => {
		if (!sessions.length) {
			toast.add({ title: "Tidak ada data untuk diekspor", color: "warning", icon: "lucide:alert-triangle" });
			return;
		}
		try {
			const path = await saveDialog({ defaultPath: defaultFilename, filters: [{ name: "Excel", extensions: ["xlsx"] }] });
			if (!path) return;

			const bytes = sheetBytes([headerRow, ...sessions.map(toRow)]);
			await invoke("write_file_bytes", { path, bytes: Array.from(bytes) });

			toast.add({ title: "Export selesai", description: `File disimpan di ${path}`, icon: "lucide:check", color: "success" });
		} catch (error) {
			reportError("exportOneXlsx", error);
		}
	};

	// Banyak file .xlsx dibundel jadi satu .zip — dipakai untuk export per siswa SEMUA siswa
	// sekaligus (satu file per siswa), dan untuk export tingkat kelas (satu file per
	// mapel+jenis, atau per siswa lintas mapel).
	const exportZipOfXlsx = async (defaultFilename: string, files: { name: string, sessions: HasilSessionRow[] }[]) => {
		if (!files.length) {
			toast.add({ title: "Tidak ada data untuk diekspor", color: "warning", icon: "lucide:alert-triangle" });
			return;
		}
		try {
			const path = await saveDialog({ defaultPath: defaultFilename, filters: [{ name: "Zip", extensions: ["zip"] }] });
			if (!path) return;

			const zip = new JSZip();
			for (const file of files) {
				zip.file(`${file.name}.xlsx`, sheetBytes([headerRow, ...file.sessions.map(toRow)]));
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

	// Nama file yang aman dipakai di sistem file Windows — spasi/simbol aneh pada nama siswa
	// atau mapel diganti strip supaya tidak bikin path error.
	const safeName = (s: string) => s.trim().replace(/[\\/:*?"<>|]+/g, "-").replace(/\s+/g, "-");

	// ---------- Export DETAIL per soal (bukan cuma ringkasan 1 baris per ujian) ----------
	// Dipakai khusus untuk "Export per Siswa" — nilainya baru beda dari "Export Semua" kalau
	// isinya sampai ke tingkat jawaban tiap soal, bukan cuma skor akhir.

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
			return item.essayScore === null ? "Belum dinilai" : `${item.essayScore}/${item.maxScore}`;
		}
		return item.isCorrect ? "Benar" : "Salah";
	};

	// Satu sesi ujian jadi beberapa baris (satu baris per soal), diambil dari backend yang
	// sudah mengurutkan soal sesuai urutan input guru di Bank Soal (bukan urutan acak yang
	// dilihat siswa saat mengerjakan).
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

	// Satu file .xlsx berisi rincian per soal — untuk satu siswa (bisa lebih dari satu sesi,
	// mis. beberapa mata pelajaran/jenis ujian digabung dalam satu file).
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
			const bytes = sheetBytes([detailHeaderRow, ...rows]);
			await invoke("write_file_bytes", { path, bytes: Array.from(bytes) });

			toast.add({ title: "Export selesai", description: `File disimpan di ${path}`, icon: "lucide:check", color: "success" });
		} catch (error) {
			reportError("exportStudentDetailXlsx", error);
		}
	};

	// Banyak file .xlsx (rincian per soal, satu file per siswa) dibundel jadi satu .zip.
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

			const zip = new JSZip();
			for (const student of students) {
				const rows = (await Promise.all(student.sessions.map((s) => fetchSessionDetailRows(s.sessionId, s.jenis)))).flat();
				zip.file(`${safeName(student.name)}.xlsx`, sheetBytes([detailHeaderRow, ...rows]));
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
