<script lang="ts" setup>
	definePageMeta({
		layout: false,
		hideFromNav: true
	});

	interface OptionView { key: string, text: string }
	interface QuestionView { id: number, questionText: string, questionType: string, image: string | null, options: OptionView[] }
	interface JoinResponse {
		sessionId: number
		examTitle: string
		durationSeconds: number
		startedAt: number
		pgSubmittedAt: number | null
		submittedAt: number | null
		score: number | null
		questions: QuestionView[]
		answers: Record<number, string>
	}

	// "exam" covers both sessions — which one shows is decided by `pgSubmittedAt` (see
	// `currentQuestions`). "pg-checkpoint" is the deliberate stop between them: PG and Esai
	// are two separate sessions from the student's point of view, not one continuous list.
	const stage = ref<"login" | "token" | "exam" | "pg-checkpoint" | "done">("login");

	const nisn = ref("");
	const name = ref("");
	const token = ref("");
	const loading = ref(false);
	const submitting = ref(false);
	const error = ref("");

	const sessionId = ref(0);
	const examTitle = ref("");
	const questions = ref<QuestionView[]>([]);
	const answers = reactive<Record<number, string>>({});
	const remaining = ref(0);
	const score = ref<number | null>(null);
	const pgSubmittedAt = ref<number | null>(null);

	const pgQuestions = computed(() => questions.value.filter((q) => q.questionType !== "essay"));
	const essayQuestions = computed(() => questions.value.filter((q) => q.questionType === "essay"));
	const hasEssay = computed(() => essayQuestions.value.length > 0);
	// Which set of questions is on screen right now: PG until that session is submitted,
	// then Esai. An exam with no essay questions at all just stays on the PG set throughout.
	const currentQuestions = computed(() => (pgSubmittedAt.value ? essayQuestions.value : pgQuestions.value));

	let timer: ReturnType<typeof setInterval>;

	const formatTime = (secs: number) => {
		const m = Math.max(0, Math.floor(secs / 60));
		const s = Math.max(0, secs % 60);
		return `${m}:${s.toString().padStart(2, "0")}`;
	};

	const doLogin = async () => {
		loading.value = true;
		error.value = "";
		try {
			const res = await fetch("/api/login", {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify({ nisn: nisn.value, name: name.value })
			});
			if (!res.ok) throw new Error("NISN atau nama tidak cocok");
			const data = await res.json() as { name: string };
			name.value = data.name;
			stage.value = "token";
		} catch (e) {
			error.value = e instanceof Error ? e.message : "Gagal login";
		} finally {
			loading.value = false;
		}
	};

	// Final submit — only relevant once the student is on the Esai set (or there's no essay
	// at all, so PG doubles as the only/final session). Essay scoring stays manual; the score
	// shown here is still PG-only, same as before.
	const doSubmit = async () => {
		if (stage.value !== "exam") return;
		clearInterval(timer);
		submitting.value = true;
		try {
			const res = await fetch("/api/exam/submit", {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify({ sessionId: sessionId.value })
			});
			const data = await res.json() as { score: number };
			score.value = data.score;
			stage.value = "done";
		} finally {
			submitting.value = false;
		}
	};

	// Locks the PG answers in place (server-side too) and, if there's an essay portion,
	// stops at the checkpoint screen instead of silently continuing — this is the "separate
	// session" boundary from the student's point of view. The countdown keeps running.
	const doSubmitPg = async () => {
		if (stage.value !== "exam") return;
		submitting.value = true;
		try {
			await fetch("/api/exam/submit-pg", {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify({ sessionId: sessionId.value })
			});
			pgSubmittedAt.value = Math.floor(Date.now() / 1000);
			if (hasEssay.value) {
				stage.value = "pg-checkpoint";
			} else {
				await doSubmit();
			}
		} finally {
			submitting.value = false;
		}
	};

	const continueToEssay = () => {
		stage.value = "exam";
	};

	const doJoin = async () => {
		loading.value = true;
		error.value = "";
		try {
			const res = await fetch("/api/exam/join", {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify({ token: token.value, nisn: nisn.value, name: name.value })
			});
			if (!res.ok) throw new Error(await res.text());
			const data = await res.json() as JoinResponse;

			sessionId.value = data.sessionId;
			examTitle.value = data.examTitle;
			questions.value = data.questions;
			pgSubmittedAt.value = data.pgSubmittedAt;
			Object.assign(answers, data.answers);

			if (data.submittedAt) {
				score.value = data.score;
				stage.value = "done";
				return;
			}

			const elapsed = Math.floor(Date.now() / 1000) - data.startedAt;
			remaining.value = data.durationSeconds - elapsed;
			// Resuming after already finishing PG (e.g. reopened the browser) goes straight
			// back into the Esai set — no need to re-show the checkpoint screen.
			stage.value = "exam";

			timer = setInterval(() => {
				remaining.value -= 1;
				if (remaining.value <= 0) {
					if (!pgSubmittedAt.value && hasEssay.value) doSubmitPg();
					else doSubmit();
				}
			}, 1000);
		} catch (e) {
			error.value = e instanceof Error ? e.message : "Gagal masuk ujian";
		} finally {
			loading.value = false;
		}
	};

	const setAnswer = (questionId: number, optionKey: string) => {
		answers[questionId] = optionKey;
		fetch("/api/exam/answer", {
			method: "POST",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({ sessionId: sessionId.value, questionId, optionKey })
		});
	};

	const doLogout = async () => {
		await fetch("/api/logout", {
			method: "POST",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({ nisn: nisn.value })
		});
		nisn.value = "";
		name.value = "";
		token.value = "";
		questions.value = [];
		pgSubmittedAt.value = null;
		Object.keys(answers).forEach((k) => delete answers[Number(k)]);
		score.value = null;
		stage.value = "login";
	};

	onUnmounted(() => clearInterval(timer));
</script>

<template>
	<div class="min-h-screen p-6" :class="stage === 'exam' ? 'flex items-start justify-center' : 'flex items-center justify-center'">
		<!-- Login -->
		<UCard v-if="stage === 'login'" class="w-full max-w-sm">
			<template #header>
				<h1 class="text-xl font-bold">
					Login Ujian
				</h1>
			</template>

			<form class="space-y-4" @submit.prevent="doLogin">
				<UFormField label="NISN">
					<UInput v-model="nisn" size="lg" />
				</UFormField>

				<UFormField label="Nama">
					<UInput v-model="name" size="lg" />
				</UFormField>

				<UButton
					type="submit"
					block
					size="lg"
					:loading="loading">
					Masuk
				</UButton>
			</form>

			<UAlert
				v-if="error"
				color="error"
				variant="subtle"
				class="mt-4"
				:title="error" />
		</UCard>

		<!-- Token entry -->
		<UCard v-else-if="stage === 'token'" class="w-full max-w-sm">
			<template #header>
				<h1 class="text-xl font-bold">
					Selamat datang, {{ name }}!
				</h1>
			</template>

			<form class="space-y-4" @submit.prevent="doJoin">
				<UFormField label="Token Ujian">
					<UInput v-model="token" size="lg" placeholder="Contoh: UTS2026" />
				</UFormField>

				<UButton
					type="submit"
					block
					size="lg"
					:loading="loading">
					Mulai Ujian
				</UButton>
			</form>

			<UAlert
				v-if="error"
				color="error"
				variant="subtle"
				class="mt-4"
				:title="error" />
		</UCard>

		<!-- PG -> Esai checkpoint -->
		<UCard v-else-if="stage === 'pg-checkpoint'" class="w-full max-w-sm text-center">
			<template #header>
				<h1 class="text-xl font-bold">
					Sesi Pilihan Ganda Selesai
				</h1>
			</template>

			<p class="text-muted mb-6">
				Jawaban Pilihan Ganda kamu sudah tersimpan dan tidak bisa diubah lagi. Waktu ujian tetap berjalan — lanjutkan ke sesi Esai sekarang.
			</p>

			<UButton block size="lg" @click="continueToEssay">
				Lanjut ke Sesi Esai
			</UButton>
		</UCard>

		<!-- Exam -->
		<div v-else-if="stage === 'exam'" class="w-full max-w-2xl space-y-4">
			<UCard class="sticky top-4 z-10">
				<div class="flex items-center justify-between">
					<div>
						<h1 class="font-bold">
							{{ examTitle }}
						</h1>
						<p class="text-muted text-xs uppercase tracking-wide">
							Sesi {{ pgSubmittedAt ? "Esai" : "Pilihan Ganda" }}
						</p>
					</div>
					<UBadge :color="remaining <= 60 ? 'error' : 'primary'" size="lg" variant="subtle">
						{{ formatTime(remaining) }}
					</UBadge>
				</div>
			</UCard>

			<UCard v-for="(q, i) in currentQuestions" :key="q.id">
				<template #header>
					<span class="font-medium">{{ i + 1 }}. {{ q.questionText }}</span>
				</template>

				<img
					v-if="q.image"
					:src="`/${q.image}`"
					class="max-w-full rounded-md mb-3"
					alt="">

				<div v-if="q.questionType === 'essay'">
					<UTextarea
						:model-value="answers[q.id] ?? ''"
						:rows="4"
						placeholder="Tulis jawabanmu di sini..."
						@update:model-value="(v: string) => setAnswer(q.id, v)"
					/>
				</div>
				<div v-else class="space-y-2">
					<label
						v-for="opt in q.options"
						:key="opt.key"
						class="flex items-center gap-3 p-2 rounded-md hover:bg-elevated cursor-pointer"
					>
						<input
							type="radio"
							:name="`q-${q.id}`"
							:value="opt.key"
							:checked="answers[q.id] === opt.key"
							@change="setAnswer(q.id, opt.key)"
						>
						<span>{{ opt.key }}. {{ opt.text }}</span>
					</label>
				</div>
			</UCard>

			<UButton
				v-if="!pgSubmittedAt && hasEssay"
				block
				size="lg"
				:loading="submitting"
				@click="doSubmitPg">
				Selesai Sesi Pilihan Ganda
			</UButton>
			<UButton
				v-else
				block
				size="lg"
				:loading="submitting"
				@click="doSubmit">
				Selesai & Kumpulkan
			</UButton>
		</div>

		<!-- Done -->
		<UCard v-else-if="stage === 'done'" class="w-full max-w-sm text-center">
			<template #header>
				<h1 class="text-xl font-bold">
					Ujian Selesai
				</h1>
			</template>

			<p class="text-muted mb-6">
				Jawabanmu sudah tersimpan. Nilai akan diumumkan guru setelah soal esai dikoreksi.
			</p>

			<UButton
				block
				variant="soft"
				icon="lucide:log-out"
				@click="doLogout">
				Keluar
			</UButton>
		</UCard>
	</div>
</template>
