import { describe, expect, it } from "bun:test";
import { AnswerSaveQueue, type AnswerDraft, type AnswerSendResult } from "../app/utils/answerQueue";

const draft = (value: string): AnswerDraft => ({ value, doubtful: false });
const tick = (ms = 0) => new Promise((resolve) => setTimeout(resolve, ms));

const manualSender = () => {
	const calls: { questionId: number, value: string, resolve: (result: AnswerSendResult) => void }[] = [];
	const send = (questionId: number, item: AnswerDraft) => new Promise<AnswerSendResult>((resolve) => {
		calls.push({ questionId, value: item.value, resolve });
	});
	return { calls, send };
};

describe("AnswerSaveQueue", () => {
	it("mengirim satu permintaan per soal dan mengakhiri dengan nilai terbaru", async () => {
		const { calls, send } = manualSender();
		const queue = new AnswerSaveQueue(send, () => {});

		queue.set(1, draft("a"));
		queue.set(1, draft("ab"));
		queue.set(1, draft("abc"));
		expect(calls.map((c) => c.value)).toEqual(["a"]);

		calls[0]!.resolve({ status: "saved" });
		await tick();
		expect(calls.map((c) => c.value)).toEqual(["a", "abc"]);
		expect(queue.pendingCount).toBe(1);

		calls[1]!.resolve({ status: "saved" });
		await tick();
		expect(queue.pendingCount).toBe(0);
	});

	it("menggabungkan ketikan cepat dengan jeda", async () => {
		const { calls, send } = manualSender();
		const queue = new AnswerSaveQueue(send, () => {});
		queue.set(1, draft("a"), 15);
		queue.set(1, draft("ab"), 15);
		await tick(40);
		expect(calls.map((c) => c.value)).toEqual(["ab"]);
	});

	it("mencoba ulang saat jaringan gagal", async () => {
		const results: AnswerSendResult[] = [{ status: "retry" }, { status: "saved" }];
		const statuses: string[] = [];
		const queue = new AnswerSaveQueue(async () => results.shift()!, (_, status) => statuses.push(status), 5);
		queue.set(1, draft("a"));
		expect(await queue.flush(200)).toBe(true);
		expect(statuses).toEqual(["saving", "error", "saved"]);
	});

	it("menganggap exception pengiriman sebagai gangguan sementara", async () => {
		let attempts = 0;
		const queue = new AnswerSaveQueue(async () => {
			attempts += 1;
			if (attempts === 1) throw new Error("offline");
			return { status: "saved" };
		}, () => {}, 5);
		queue.set(1, draft("a"));
		expect(await queue.flush(200)).toBe(true);
		expect(attempts).toBe(2);
	});

	it("tidak mengulang jawaban yang ditolak server", async () => {
		let attempts = 0;
		let rejection = "";
		const queue = new AnswerSaveQueue(async () => {
			attempts += 1;
			return { status: "rejected", message: "Waktu habis" };
		}, (_, status, message) => {
			if (status === "rejected") rejection = message ?? "";
		}, 5);
		queue.set(1, draft("a"));
		expect(await queue.flush(100)).toBe(true);
		await tick(20);
		expect(attempts).toBe(1);
		expect(rejection).toBe("Waktu habis");
	});

	it("flush mengirim jawaban yang masih menunggu jeda", async () => {
		const sent: string[] = [];
		const queue = new AnswerSaveQueue(async (_, item) => {
			sent.push(item.value);
			return { status: "saved" };
		}, () => {});
		queue.set(1, draft("esai"), 60_000);
		expect(await queue.flush(200)).toBe(true);
		expect(sent).toEqual(["esai"]);
	});

	it("flush berhenti pada batas waktu bila server tidak merespons", async () => {
		const { send } = manualSender();
		const queue = new AnswerSaveQueue(send, () => {});
		queue.set(1, draft("a"));
		expect(await queue.flush(15)).toBe(false);
		queue.reset();
		expect(queue.pendingCount).toBe(0);
	});
});
