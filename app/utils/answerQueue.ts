export type AnswerSaveStatus = "saving" | "saved" | "error" | "rejected";

export interface AnswerDraft {
	value: string
	doubtful: boolean
}

export type AnswerSendResult
	= | { status: "saved" }
		| { status: "retry" }
		| { status: "rejected", message: string };

type SendAnswer = (questionId: number, draft: AnswerDraft) => Promise<AnswerSendResult>;
type StatusListener = (questionId: number, status: AnswerSaveStatus, message?: string) => void;

interface QueueEntry {
	draft: AnswerDraft
	version: number
	inFlight: boolean
	timer?: ReturnType<typeof setTimeout>
}

// Menyimpan jawaban terbaru per soal. Tiap soal hanya punya satu permintaan yang berjalan, jadi
// teks lama tidak bisa tiba belakangan dan menimpa teks yang lebih baru.
export class AnswerSaveQueue {
	private entries = new Map<number, QueueEntry>();
	private waiters = new Set<() => void>();
	private generation = 0;

	constructor(
		private readonly send: SendAnswer,
		private readonly onStatus: StatusListener,
		private readonly retryMs = 3000
	) {}

	get pendingCount() {
		return this.entries.size;
	}

	set(questionId: number, draft: AnswerDraft, delayMs = 0) {
		const entry = this.entries.get(questionId) ?? { draft, version: 0, inFlight: false };
		entry.draft = draft;
		entry.version += 1;
		this.clearTimer(entry);
		this.entries.set(questionId, entry);
		this.onStatus(questionId, "saving");

		if (delayMs > 0) {
			entry.timer = setTimeout(() => {
				entry.timer = undefined;
				void this.pump(questionId);
			}, delayMs);
		} else {
			void this.pump(questionId);
		}
	}

	// Mengirim semua yang tertunda sekarang, lalu menunggu sampai habis atau batas waktu tercapai.
	flush(timeoutMs: number): Promise<boolean> {
		for (const [questionId, entry] of this.entries) {
			this.clearTimer(entry);
			void this.pump(questionId);
		}
		if (!this.entries.size) return Promise.resolve(true);

		return new Promise((resolve) => {
			const finish = () => {
				clearTimeout(timeout);
				this.waiters.delete(check);
				resolve(this.entries.size === 0);
			};
			const check = () => {
				if (!this.entries.size) finish();
			};
			const timeout = setTimeout(finish, timeoutMs);
			this.waiters.add(check);
		});
	}

	reset() {
		this.generation += 1;
		for (const entry of this.entries.values()) this.clearTimer(entry);
		this.entries.clear();
		this.notifyWaiters();
	}

	private clearTimer(entry: QueueEntry) {
		if (entry.timer !== undefined) clearTimeout(entry.timer);
		entry.timer = undefined;
	}

	private notifyWaiters() {
		for (const waiter of [...this.waiters]) waiter();
	}

	private async pump(questionId: number) {
		const entry = this.entries.get(questionId);
		if (!entry || entry.inFlight) return;

		const generation = this.generation;
		const version = entry.version;
		entry.inFlight = true;

		let result: AnswerSendResult;
		try {
			result = await this.send(questionId, entry.draft);
		} catch {
			result = { status: "retry" };
		}
		entry.inFlight = false;
		if (generation !== this.generation) return;

		if (entry.version !== version) {
			if (entry.timer === undefined) void this.pump(questionId);
			return;
		}

		if (result.status === "retry") {
			this.onStatus(questionId, "error");
			entry.timer = setTimeout(() => {
				entry.timer = undefined;
				void this.pump(questionId);
			}, this.retryMs);
			return;
		}

		this.entries.delete(questionId);
		if (result.status === "saved") this.onStatus(questionId, "saved");
		else this.onStatus(questionId, "rejected", result.message);
		this.notifyWaiters();
	}
}
