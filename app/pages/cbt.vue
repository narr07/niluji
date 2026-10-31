<script lang="ts" setup>
	definePageMeta({
		layout: false,
		hideFromNav: true,
		colorMode: "light"
	});

	interface OptionView { key: string, text: string, image: string | null }
	interface QuestionView { id: number, questionText: string, questionType: string, image: string | null, options: OptionView[] }
	interface JoinResponse {
		sessionId: number
		examTitle: string
		durationSeconds: number
		startedAt: number
		remainingSeconds: number
		deviceToken: string
		pgSubmittedAt: number | null
		submittedAt: number | null
		score: number | null
		questions: QuestionView[]
		answers: Record<number, string>
		doubtful: Record<number, boolean>
	}
	interface AnswerResponse {
		remainingSeconds: number
	}

	const route = useRoute();
	const stage = ref<"login" | "token" | "exam" | "pg-checkpoint" | "done">("login");

	const nisn = ref("");
	const name = ref("");
	const token = ref("");
	const pin = ref("");
	const loading = ref(false);
	const submitting = ref(false);
	const error = ref("");

	const sessionId = ref(0);
	const examTitle = ref("");
	const questions = ref<QuestionView[]>([]);
	const answers = ref<Record<number, string>>({});
	const doubtful = ref<Record<number, boolean>>({});
	const remaining = ref(0);
	const score = ref<number | null>(null);
	const pgSubmittedAt = ref<number | null>(null);

	const fontScale = ref<"sm" | "base" | "lg">("base");
	const fontScaleClass = computed(() => {
		if (fontScale.value === "sm") return "text-sm";
		if (fontScale.value === "lg") return "text-lg leading-relaxed";
		return "text-base";
	});

	const saveStatus = ref<Record<number, "saved" | "saving" | "error">>({});
	const hasPendingSaves = computed(() => Object.values(saveStatus.value).some((s) => s !== "saved"));
	const hasErrorSaves = computed(() => Object.values(saveStatus.value).some((s) => s === "error"));

	const pgQuestions = computed(() => questions.value.filter((q) => q.questionType !== "essay"));
	const essayQuestions = computed(() => questions.value.filter((q) => q.questionType === "essay"));
	const hasEssay = computed(() => essayQuestions.value.length > 0);
	const currentQuestions = computed(() => (pgSubmittedAt.value ? essayQuestions.value : pgQuestions.value));

	const viewMode = ref<"satu" | "scroll">("satu");
	const mobileNavOpen = ref(false);
	const currentIndex = ref(0);
	const currentQuestion = computed(() => currentQuestions.value[currentIndex.value]);
	const isAnswered = (q: QuestionView) => {
		const a = answers.value[q.id];
		return typeof a === "string" && a.trim().length > 0;
	};
	const isDoubtful = (q: QuestionView) => Boolean(doubtful.value[q.id]);

	const goPrev = () => {
		if (currentIndex.value > 0) currentIndex.value--;
	};
	const goNext = () => {
		if (currentIndex.value < currentQuestions.value.length - 1) currentIndex.value++;
	};
	watch(currentQuestions, () => {
		currentIndex.value = 0;
	});

	const confirmOpen = ref(false);
	const confirmFinish = () => {
		confirmOpen.value = false;
		if (!pgSubmittedAt.value && hasEssay.value) doSubmitPg();
		else doSubmit();
	};

	let timer: ReturnType<typeof setInterval>;

	const formatTime = (secs: number) => {
		const m = Math.max(0, Math.floor(secs / 60));
		const s = Math.max(0, secs % 60);
		return `${m}:${s.toString().padStart(2, "0")}`;
	};

	const getOrCreateDeviceToken = () => {
		if (typeof window === "undefined") return "";
		const key = "niluji-device-token";
		let existing = window.localStorage.getItem(key);
		if (!existing) {
			existing = `dev-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 10)}`;
			window.localStorage.setItem(key, existing);
		}
		return existing;
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
			if (!res.ok) throw new Error("NISN atau nama tidak cocok dengan data peserta");
			const data = await res.json() as { name: string };
			name.value = data.name;
			stage.value = "token";
		} catch (e) {
			error.value = e instanceof Error ? e.message : "Gagal masuk";
		} finally {
			loading.value = false;
		}
	};

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
			const deviceToken = getOrCreateDeviceToken();
			const res = await fetch("/api/exam/join", {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify({
					token: token.value,
					nisn: nisn.value,
					name: name.value,
					pin: pin.value.trim() || undefined,
					deviceToken
				})
			});
			if (!res.ok) throw new Error(await res.text());
			const data = await res.json() as JoinResponse;

			if (data.deviceToken && typeof window !== "undefined") {
				window.localStorage.setItem("niluji-device-token", data.deviceToken);
			}

			sessionId.value = data.sessionId;
			examTitle.value = data.examTitle;
			questions.value = data.questions;
			pgSubmittedAt.value = data.pgSubmittedAt;
			answers.value = { ...data.answers };
			doubtful.value = { ...(data.doubtful ?? {}) };

			if (data.submittedAt) {
				score.value = data.score;
				stage.value = "done";
				return;
			}

			remaining.value = typeof data.remainingSeconds === "number"
				? data.remainingSeconds
				: Math.max(0, data.durationSeconds - (Math.floor(Date.now() / 1000) - data.startedAt));
			stage.value = "exam";

			clearInterval(timer);
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

	const saveAnswer = async (questionId: number, optionKey: string, isDoubtfulVal?: boolean) => {
		saveStatus.value[questionId] = "saving";
		try {
			const res = await fetch("/api/exam/answer", {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify({
					sessionId: sessionId.value,
					questionId,
					optionKey,
					isDoubtful: isDoubtfulVal ?? Boolean(doubtful.value[questionId])
				})
			});
			if (!res.ok) throw new Error(`status ${res.status}`);
			const data = (await res.json().catch(() => null)) as AnswerResponse | null;
			if (data && typeof data.remainingSeconds === "number") {
				remaining.value = data.remainingSeconds;
			}
			if ((answers.value[questionId] ?? "") === optionKey) {
				saveStatus.value[questionId] = "saved";
			}
		} catch {
			if ((answers.value[questionId] ?? "") !== optionKey) return;
			saveStatus.value[questionId] = "error";
			setTimeout(() => {
				if ((answers.value[questionId] ?? "") === optionKey) {
					saveAnswer(questionId, optionKey, doubtful.value[questionId]);
				}
			}, 4000);
		}
	};

	const setAnswer = (questionId: number, optionKey: string) => {
		answers.value[questionId] = optionKey;
		saveAnswer(questionId, optionKey, doubtful.value[questionId]);
	};

	const toggleDoubtful = (questionId: number) => {
		const next = !doubtful.value[questionId];
		doubtful.value[questionId] = next;
		const currentOpt = answers.value[questionId] ?? "";
		saveAnswer(questionId, currentOpt, next);
	};

	const optionDisplayLetter = (idx: number) => String.fromCharCode(65 + idx);

	const doLogout = async () => {
		await fetch("/api/logout", {
			method: "POST",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({ nisn: nisn.value })
		});
		nisn.value = "";
		name.value = "";
		token.value = "";
		pin.value = "";
		questions.value = [];
		pgSubmittedAt.value = null;
		answers.value = {};
		doubtful.value = {};
		saveStatus.value = {};
		score.value = null;
		stage.value = "login";
	};

	const onKeydown = (ev: KeyboardEvent) => {
		if (stage.value !== "exam" || viewMode.value !== "satu" || !currentQuestion.value) return;
		const target = ev.target as HTMLElement | null;
		if (target && (target.tagName === "INPUT" || target.tagName === "TEXTAREA" || target.isContentEditable)) {
			return;
		}
		if (ev.key === "ArrowLeft") {
			ev.preventDefault();
			goPrev();
		} else if (ev.key === "ArrowRight") {
			ev.preventDefault();
			goNext();
		} else if (ev.key.toLowerCase() === "r") {
			ev.preventDefault();
			toggleDoubtful(currentQuestion.value.id);
		} else if (currentQuestion.value.questionType !== "essay") {
			const idx = ev.key.toUpperCase().charCodeAt(0) - 65;
			if (ev.key.length === 1 && idx >= 0 && idx < currentQuestion.value.options.length) {
				ev.preventDefault();
				const opt = currentQuestion.value.options[idx];
				if (opt) setAnswer(currentQuestion.value.id, opt.key);
			}
		}
	};

	onMounted(() => {
		const qToken = route.query.token;
		if (typeof qToken === "string" && qToken.trim()) {
			token.value = qToken.trim().toUpperCase();
		}
		window.addEventListener("keydown", onKeydown);
	});

	onUnmounted(() => {
		clearInterval(timer);
		if (typeof window !== "undefined") {
			window.removeEventListener("keydown", onKeydown);
		}
	});
</script>

<template>
	<div
		class="min-h-screen p-4 md:p-6 pb-24 md:pb-8 bg-elevated/40 text-default"
		:class="stage === 'exam' ? 'flex items-start justify-center' : 'flex items-center justify-center'">
		<!-- Login -->
		<UCard v-if="stage === 'login'" class="w-full max-w-md shadow-lg border border-default">
			<template #header>
				<div class="text-center space-y-1">
					<div class="inline-flex items-center justify-center size-11 rounded-xl bg-primary/10 text-primary mb-1">
						<UIcon name="lucide:graduation-cap" class="size-6" />
					</div>
					<h1 class="text-xl font-bold text-highlighted">
						Portal Ujian Siswa
					</h1>
					<p class="text-xs text-muted">
						Masukkan NISN dan nama sesuai daftar peserta ujian
					</p>
				</div>
			</template>

			<form class="space-y-4" @submit.prevent="doLogin">
				<UFormField label="NISN" required>
					<UInput
						v-model="nisn"
						size="xl"
						color="primary"
						placeholder="Contoh: 0123456789"
						autocomplete="off"
						class="w-full" />
				</UFormField>

				<UFormField label="Nama Lengkap" required>
					<UInput
						v-model="name"
						size="xl"
						color="primary"
						placeholder="Ketik nama lengkapmu"
						autocomplete="off"
						class="w-full" />
				</UFormField>

				<UButton
					type="submit"
					block
					size="xl"
					color="primary"
					variant="solid"
					class="font-semibold min-h-11"
					:loading="loading">
					Lanjutkan
				</UButton>
			</form>

			<UAlert
				v-if="error"
				color="error"
				variant="subtle"
				icon="lucide:alert-circle"
				class="mt-4"
				:title="error" />
		</UCard>

		<!-- Token & PIN entry -->
		<UCard v-else-if="stage === 'token'" class="w-full max-w-md shadow-lg border border-default">
			<template #header>
				<div class="flex items-center justify-between gap-2">
					<div>
						<p class="text-xs text-muted uppercase tracking-wider">
							Peserta Terverifikasi
						</p>
						<h1 class="text-lg font-bold text-highlighted">
							{{ name }} ({{ nisn }})
						</h1>
					</div>
					<UButton
						size="xs"
						variant="ghost"
						color="neutral"
						icon="lucide:arrow-left"
						@click="stage = 'login'">
						Ganti
					</UButton>
				</div>
			</template>

			<form class="space-y-4" @submit.prevent="doJoin">
				<UFormField label="Token Ujian" required description="Diumumkan oleh pengawas di ruang ujian.">
					<UInput
						v-model="token"
						size="xl"
						color="primary"
						placeholder="Contoh: MTK6A"
						autocomplete="off"
						class="w-full font-mono uppercase" />
				</UFormField>

				<UFormField label="PIN Kartu Peserta" description="Isi 6 digit PIN pada kartu peserta (jika diwajibkan pengawas).">
					<UInput
						v-model="pin"
						size="xl"
						color="primary"
						placeholder="6 digit PIN (opsional)"
						maxlength="6"
						inputmode="numeric"
						autocomplete="off"
						class="w-full font-mono tracking-widest" />
				</UFormField>

				<UButton
					type="submit"
					block
					size="xl"
					color="primary"
					variant="solid"
					icon="lucide:play"
					class="font-semibold min-h-11"
					:loading="loading">
					Mulai Kerjakan Ujian
				</UButton>
			</form>

			<UAlert
				v-if="error"
				color="error"
				variant="subtle"
				icon="lucide:alert-circle"
				class="mt-4"
				:title="error" />
		</UCard>

		<!-- PG -> Esai checkpoint -->
		<UCard v-else-if="stage === 'pg-checkpoint'" class="w-full max-w-md text-center shadow-lg border border-default">
			<template #header>
				<div class="space-y-1">
					<UIcon name="lucide:check-circle-2" class="size-8 text-success mx-auto" />
					<h1 class="text-xl font-bold text-highlighted">
						Sesi Pilihan Ganda Selesai
					</h1>
				</div>
			</template>

			<p class="text-muted mb-6 text-sm leading-relaxed">
				Jawaban Pilihan Ganda kamu sudah dikunci dan tersimpan di server. Waktu ujian tetap berjalan, silakan lanjutkan ke sesi Esai sekarang.
			</p>

			<UButton
				block
				size="xl"
				icon="lucide:arrow-right"
				class="min-h-11"
				@click="continueToEssay">
				Lanjut ke Sesi Esai
			</UButton>
		</UCard>

		<!-- Exam -->
		<div v-else-if="stage === 'exam'" class="w-full max-w-5xl space-y-4">
			<UCard class="sticky top-2 z-20 shadow-sm border border-default">
				<div class="flex items-center justify-between flex-wrap gap-3">
					<div class="min-w-0">
						<h1 class="font-bold text-highlighted truncate">
							{{ examTitle }}
						</h1>
						<div class="flex items-center flex-wrap gap-2 mt-0.5 text-xs text-muted">
							<span>{{ name }}</span>
							<span>·</span>
							<span class="uppercase font-medium text-default">
								Sesi {{ pgSubmittedAt ? "Esai" : "Pilihan Ganda" }}
							</span>
							<span>·</span>
							<span
								v-if="hasErrorSaves"
								class="inline-flex items-center gap-1 text-error font-medium">
								<UIcon name="lucide:wifi-off" class="size-3.5" />
								Mencoba simpan ulang...
							</span>
							<span
								v-else-if="hasPendingSaves"
								class="inline-flex items-center gap-1 text-warning font-medium">
								<UIcon name="lucide:loader-circle" class="size-3.5 animate-spin" />
								Menyimpan...
							</span>
							<span
								v-else
								class="inline-flex items-center gap-1 text-success font-medium">
								<UIcon name="lucide:check-circle-2" class="size-3.5" />
								Tersimpan
							</span>
						</div>
					</div>

					<div class="flex items-center flex-wrap gap-2">
						<!-- Font size switcher -->
						<div class="inline-flex items-center rounded-md border border-default p-0.5 bg-elevated/50" role="group" aria-label="Ukuran teks soal">
							<button
								type="button"
								class="px-2 py-1 text-xs rounded transition-colors min-h-8"
								:class="fontScale === 'sm' ? 'bg-default text-highlighted font-semibold shadow-xs' : 'text-muted hover:text-default'"
								title="Teks kecil"
								@click="fontScale = 'sm'">
								A-
							</button>
							<button
								type="button"
								class="px-2 py-1 text-xs rounded transition-colors min-h-8"
								:class="fontScale === 'base' ? 'bg-default text-highlighted font-semibold shadow-xs' : 'text-muted hover:text-default'"
								title="Teks normal"
								@click="fontScale = 'base'">
								A
							</button>
							<button
								type="button"
								class="px-2 py-1 text-xs rounded transition-colors min-h-8"
								:class="fontScale === 'lg' ? 'bg-default text-highlighted font-semibold shadow-xs' : 'text-muted hover:text-default'"
								title="Teks besar"
								@click="fontScale = 'lg'">
								A+
							</button>
						</div>

						<UButton
							size="sm"
							:variant="viewMode === 'satu' ? 'solid' : 'outline'"
							color="neutral"
							@click="viewMode = 'satu'">
							1 Soal
						</UButton>
						<UButton
							size="sm"
							:variant="viewMode === 'scroll' ? 'solid' : 'outline'"
							color="neutral"
							@click="viewMode = 'scroll'">
							Semua
						</UButton>

						<UBadge
							:color="remaining <= 300 ? 'error' : 'primary'"
							size="lg"
							variant="subtle"
							class="font-mono text-sm px-3 py-1.5">
							<UIcon name="lucide:timer" class="size-4 mr-1" />
							{{ formatTime(remaining) }}
						</UBadge>
					</div>
				</div>
			</UCard>

			<!-- Mode: satu soal + navigator -->
			<div v-if="viewMode === 'satu' && currentQuestion" class="flex flex-col lg:flex-row gap-4 items-start">
				<UCard class="flex-1 w-full border border-default">
					<template #header>
						<div class="flex items-center justify-between gap-2">
							<div class="flex items-center gap-2">
								<UBadge color="primary" variant="subtle" size="md">
									Soal {{ currentIndex + 1 }} / {{ currentQuestions.length }}
								</UBadge>
								<UBadge
									v-if="isDoubtful(currentQuestion)"
									color="warning"
									variant="subtle"
									size="md">
									<UIcon name="lucide:flag" class="size-3.5 mr-1" />
									Ragu-ragu
								</UBadge>
							</div>
							<span class="text-xs text-muted hidden sm:inline">
								Pintasan: A-D pilih, R ragu-ragu, Panah kiri/kanan pindah
							</span>
						</div>
					</template>

					<div :class="fontScaleClass" class="space-y-4">
						<div
							class="font-medium text-highlighted leading-relaxed"
							v-html="renderSoalMarkdown(currentQuestion.questionText, currentQuestion.image ? `/${currentQuestion.image}` : undefined)" />

						<img
							v-if="currentQuestion.image && !hasSoalImagePlaceholder(currentQuestion.questionText)"
							:src="`/${currentQuestion.image}`"
							class="max-w-full max-h-80 rounded-lg border border-default object-contain"
							alt="Gambar soal">

						<div v-if="currentQuestion.questionType === 'essay'">
							<UTextarea
								:model-value="answers[currentQuestion.id] ?? ''"
								:rows="6"
								placeholder="Tulis jawaban uraianmu di sini..."
								class="w-full"
								@update:model-value="(v: string) => setAnswer(currentQuestion!.id, v)" />
						</div>
						<div v-else class="space-y-2.5">
							<button
								v-for="(opt, idx) in currentQuestion.options"
								:key="opt.key"
								type="button"
								class="w-full text-left flex items-start gap-3.5 p-3.5 rounded-lg border transition-colors min-h-12 cursor-pointer"
								:class="answers[currentQuestion.id] === opt.key
									? 'border-primary bg-primary/10 text-highlighted ring-1 ring-primary'
									: 'border-default hover:bg-elevated/70 text-default'"
								@click="setAnswer(currentQuestion!.id, opt.key)">
								<span
									class="inline-flex items-center justify-center size-7 rounded-md text-sm font-bold shrink-0 mt-0.5 transition-colors"
									:class="answers[currentQuestion.id] === opt.key
										? 'bg-primary text-white'
										: 'bg-elevated text-default border border-default'">
									{{ optionDisplayLetter(idx) }}
								</span>
								<div class="flex-1 min-w-0 space-y-2">
									<div v-if="opt.text" class="leading-relaxed break-words">
										{{ opt.text }}
									</div>
									<img
										v-if="opt.image"
										:src="`/${opt.image}`"
										class="max-h-36 rounded-md border border-default object-contain"
										alt="Gambar pilihan">
								</div>
							</button>
						</div>
					</div>

					<!-- Desktop bottom navigation inside card -->
					<div class="hidden md:flex items-center justify-between gap-3 mt-6 pt-4 border-t border-default">
						<UButton
							variant="soft"
							color="neutral"
							icon="lucide:chevron-left"
							class="min-h-11"
							:disabled="currentIndex === 0"
							@click="goPrev">
							Sebelumnya
						</UButton>

						<UButton
							:variant="isDoubtful(currentQuestion) ? 'solid' : 'soft'"
							color="warning"
							icon="lucide:flag"
							class="min-h-11"
							@click="toggleDoubtful(currentQuestion.id)">
							{{ isDoubtful(currentQuestion) ? "Ditandai Ragu-ragu" : "Tandai Ragu-ragu" }}
						</UButton>

						<UButton
							v-if="currentIndex < currentQuestions.length - 1"
							trailing-icon="lucide:chevron-right"
							class="min-h-11"
							@click="goNext">
							Selanjutnya
						</UButton>
						<UButton
							v-else
							color="success"
							icon="lucide:check-circle"
							class="min-h-11"
							:disabled="hasPendingSaves"
							@click="confirmOpen = true">
							{{ !pgSubmittedAt && hasEssay ? "Selesai Sesi PG" : "Selesai & Kumpulkan" }}
						</UButton>
					</div>
				</UCard>

				<!-- Desktop Question Navigator -->
				<UCard class="w-full lg:w-64 shrink-0 border border-default">
					<template #header>
						<div class="flex items-center justify-between">
							<span class="text-sm font-semibold text-highlighted">Daftar Nomor Soal</span>
							<span class="text-xs text-muted">
								{{ currentQuestions.filter(isAnswered).length }}/{{ currentQuestions.length }}
							</span>
						</div>
					</template>
					<div class="grid grid-cols-6 lg:grid-cols-5 gap-2 max-h-64 lg:max-h-[55vh] overflow-y-auto pr-1">
						<button
							v-for="(q, i) in currentQuestions"
							:key="q.id"
							type="button"
							class="min-h-11 rounded-md border text-sm font-semibold flex flex-col items-center justify-center transition-colors relative cursor-pointer"
							:class="[
								i === currentIndex ? 'ring-2 ring-primary ring-offset-1' : '',
								isDoubtful(q)
									? 'bg-warning/20 text-warning border-warning'
									: isAnswered(q)
										? 'bg-primary text-white border-primary'
										: 'border-default hover:bg-elevated text-default'
							]"
							:aria-label="`Soal nomor ${i + 1}${isDoubtful(q) ? ' (ragu-ragu)' : isAnswered(q) ? ' (terjawab)' : ' (belum dijawab)'}`"
							@click="currentIndex = i">
							<span>{{ i + 1 }}</span>
						</button>
					</div>
					<div class="grid grid-cols-3 gap-2 mt-4 pt-3 border-t border-default text-xs text-muted">
						<span class="inline-flex items-center gap-1.5">
							<span class="size-3 rounded-xs bg-primary inline-block shrink-0" />
							Terjawab
						</span>
						<span class="inline-flex items-center gap-1.5">
							<span class="size-3 rounded-xs bg-warning/30 border border-warning inline-block shrink-0" />
							Ragu
						</span>
						<span class="inline-flex items-center gap-1.5">
							<span class="size-3 rounded-xs border border-default inline-block shrink-0" />
							Kosong
						</span>
					</div>
				</UCard>
			</div>

			<!-- Mode: semua soal, discroll ke bawah -->
			<template v-else-if="viewMode === 'scroll'">
				<UCard v-for="(q, i) in currentQuestions" :key="q.id" class="border border-default">
					<template #header>
						<div class="flex items-center justify-between gap-2">
							<span class="font-semibold text-highlighted">Soal {{ i + 1 }}</span>
							<UButton
								size="xs"
								:variant="isDoubtful(q) ? 'solid' : 'soft'"
								color="warning"
								icon="lucide:flag"
								@click="toggleDoubtful(q.id)">
								{{ isDoubtful(q) ? "Ragu-ragu" : "Tandai Ragu" }}
							</UButton>
						</div>
					</template>

					<div :class="fontScaleClass" class="space-y-4">
						<div
							class="font-medium text-highlighted leading-relaxed"
							v-html="renderSoalMarkdown(q.questionText, q.image ? `/${q.image}` : undefined)" />

						<img
							v-if="q.image && !hasSoalImagePlaceholder(q.questionText)"
							:src="`/${q.image}`"
							class="max-w-full max-h-80 rounded-lg border border-default object-contain"
							alt="Gambar soal">

						<div v-if="q.questionType === 'essay'">
							<UTextarea
								:model-value="answers[q.id] ?? ''"
								:rows="4"
								placeholder="Tulis jawaban uraianmu di sini..."
								class="w-full"
								@update:model-value="(v: string) => setAnswer(q.id, v)" />
						</div>
						<div v-else class="space-y-2.5">
							<button
								v-for="(opt, idx) in q.options"
								:key="opt.key"
								type="button"
								class="w-full text-left flex items-start gap-3.5 p-3.5 rounded-lg border transition-colors min-h-12 cursor-pointer"
								:class="answers[q.id] === opt.key
									? 'border-primary bg-primary/10 text-highlighted ring-1 ring-primary'
									: 'border-default hover:bg-elevated/70 text-default'"
								@click="setAnswer(q.id, opt.key)">
								<span
									class="inline-flex items-center justify-center size-7 rounded-md text-sm font-bold shrink-0 mt-0.5 transition-colors"
									:class="answers[q.id] === opt.key
										? 'bg-primary text-white'
										: 'bg-elevated text-default border border-default'">
									{{ optionDisplayLetter(idx) }}
								</span>
								<div class="flex-1 min-w-0 space-y-2">
									<div v-if="opt.text" class="leading-relaxed break-words">
										{{ opt.text }}
									</div>
									<img
										v-if="opt.image"
										:src="`/${opt.image}`"
										class="max-h-36 rounded-md border border-default object-contain"
										alt="Gambar pilihan">
								</div>
							</button>
						</div>
					</div>
				</UCard>
			</template>

			<UAlert
				v-if="hasPendingSaves"
				color="warning"
				variant="subtle"
				icon="lucide:refresh-cw"
				title="Menyimpan jawaban ke server..."
				description="Koneksi sempat melambat atau terputus. Sistem sedang mencoba lagi secara otomatis. Tunggu sampai status menjadi Tersimpan sebelum menekan Selesai."
				class="mb-3" />

			<UButton
				v-if="!pgSubmittedAt && hasEssay"
				block
				size="xl"
				class="min-h-12 font-semibold"
				:loading="submitting"
				:disabled="hasPendingSaves"
				@click="confirmOpen = true">
				Selesai Sesi Pilihan Ganda
			</UButton>
			<UButton
				v-else
				block
				size="xl"
				class="min-h-12 font-semibold"
				:loading="submitting"
				:disabled="hasPendingSaves"
				@click="confirmOpen = true">
				Selesai &amp; Kumpulkan Ujian
			</UButton>

			<!-- Mobile Sticky Bottom Bar (for single question mode) -->
			<div
				v-if="viewMode === 'satu' && currentQuestion"
				class="fixed bottom-0 inset-x-0 z-30 md:hidden bg-default/95 backdrop-blur border-t border-default px-3 py-2.5 flex items-center justify-between gap-2 shadow-lg">
				<UButton
					variant="soft"
					color="neutral"
					icon="lucide:chevron-left"
					class="min-h-11 px-3"
					:disabled="currentIndex === 0"
					aria-label="Soal sebelumnya"
					@click="goPrev" />

				<UButton
					:variant="isDoubtful(currentQuestion) ? 'solid' : 'soft'"
					color="warning"
					icon="lucide:flag"
					class="min-h-11 flex-1 justify-center text-xs"
					@click="toggleDoubtful(currentQuestion.id)">
					Ragu
				</UButton>

				<UButton
					variant="outline"
					color="neutral"
					icon="lucide:layout-grid"
					class="min-h-11 flex-1 justify-center text-xs"
					@click="mobileNavOpen = true">
					{{ currentIndex + 1 }}/{{ currentQuestions.length }}
				</UButton>

				<UButton
					v-if="currentIndex < currentQuestions.length - 1"
					trailing-icon="lucide:chevron-right"
					class="min-h-11 px-3"
					aria-label="Soal selanjutnya"
					@click="goNext" />
				<UButton
					v-else
					color="success"
					icon="lucide:check"
					class="min-h-11 px-3"
					:disabled="hasPendingSaves"
					aria-label="Kumpulkan ujian"
					@click="confirmOpen = true" />
			</div>

			<!-- Mobile Navigator Modal -->
			<UModal v-model:open="mobileNavOpen" title="Pilih Nomor Soal">
				<template #body>
					<div class="grid grid-cols-5 gap-2.5 max-h-[60vh] overflow-y-auto p-1">
						<button
							v-for="(q, i) in currentQuestions"
							:key="q.id"
							type="button"
							class="min-h-11 rounded-md border text-sm font-semibold flex items-center justify-center transition-colors"
							:class="[
								i === currentIndex ? 'ring-2 ring-primary' : '',
								isDoubtful(q)
									? 'bg-warning/20 text-warning border-warning'
									: isAnswered(q)
										? 'bg-primary text-white border-primary'
										: 'border-default text-default'
							]"
							@click="currentIndex = i; mobileNavOpen = false">
							{{ i + 1 }}
						</button>
					</div>
				</template>
			</UModal>

			<!-- Confirmation Modal -->
			<UModal v-model:open="confirmOpen" title="Konfirmasi Pengumpulan Jawaban">
				<template #body>
					<div class="space-y-3 text-sm">
						<p class="text-default">
							Periksa kembali ringkasan jawabanmu sebelum mengumpulkan. Jawaban yang sudah dikumpulkan tidak dapat diubah kembali.
						</p>
						<div class="grid grid-cols-3 gap-2 rounded-lg bg-elevated/60 p-3 border border-default text-center">
							<div>
								<div class="text-lg font-bold text-primary">
									{{ currentQuestions.filter(isAnswered).length }}
								</div>
								<div class="text-xs text-muted">
									Terjawab
								</div>
							</div>
							<div>
								<div class="text-lg font-bold text-warning">
									{{ currentQuestions.filter(isDoubtful).length }}
								</div>
								<div class="text-xs text-muted">
									Ragu-ragu
								</div>
							</div>
							<div>
								<div class="text-lg font-bold text-default">
									{{ currentQuestions.length - currentQuestions.filter(isAnswered).length }}
								</div>
								<div class="text-xs text-muted">
									Belum Dijawab
								</div>
							</div>
						</div>
						<UAlert
							v-if="currentQuestions.some(isDoubtful)"
							color="warning"
							variant="subtle"
							icon="lucide:flag"
							title="Masih ada soal yang ditandai Ragu-ragu"
							description="Kamu masih bisa mengecek kembali soal yang ragu-ragu sebelum mengumpulkan." />
					</div>
				</template>
				<template #footer>
					<div class="flex justify-end gap-2 w-full">
						<UButton
							variant="soft"
							color="neutral"
							class="min-h-11"
							@click="confirmOpen = false">
							Periksa Kembali
						</UButton>
						<UButton
							color="primary"
							class="min-h-11"
							:loading="submitting"
							@click="confirmFinish">
							Ya, Kumpulkan Sekarang
						</UButton>
					</div>
				</template>
			</UModal>
		</div>

		<!-- Done -->
		<UCard v-else-if="stage === 'done'" class="w-full max-w-md text-center shadow-lg border border-default">
			<template #header>
				<div class="space-y-1">
					<UIcon name="lucide:party-popper" class="size-8 text-success mx-auto" />
					<h1 class="text-xl font-bold text-highlighted">
						Ujian Selesai
					</h1>
				</div>
			</template>

			<p class="text-muted mb-6 text-sm leading-relaxed">
				Seluruh jawabanmu sudah diterima dan disimpan di server ujian. Hasil akhir akan diumumkan oleh guru mata pelajaran.
			</p>

			<UButton
				block
				size="lg"
				variant="soft"
				icon="lucide:log-out"
				class="min-h-11"
				@click="doLogout">
				Keluar dari Sesi
			</UButton>
		</UCard>
	</div>
</template>
