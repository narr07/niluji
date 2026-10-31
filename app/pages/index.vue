<script lang="ts" setup>
	import { invoke } from "@tauri-apps/api/core";

	definePageMeta({
		name: "Dashboard",
		icon: "lucide:layout-dashboard",
		top: true,
		order: 0,
		description: "Ringkasan status server, pantauan sesi ujian aktif, dan info sekolah"
	});

	interface ServerInfo {
		port: number | null
		lanIp: string | null
		lanIps?: string[]
		dbOk: boolean
	}

	interface DashboardStats {
		students: number
		subjects: number
		jenisUjian: number
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
		requirePin?: boolean
	}

	interface SessionProgress {
		sessionId: number
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

	interface SchoolInfo {
		name: string
		npsn: string
		address: string
		principal: string
	}

	const isTauri = () => typeof window !== "undefined" && !!(window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;

	const school = ref<SchoolInfo | null>(null);
	const info = ref<ServerInfo | null>(null);
	const selectedIp = ref<string>("");
	const dashboardStats = ref<DashboardStats | null>(null);
	const exams = ref<Exam[]>([]);
	const sessions = ref<SessionProgress[]>([]);
	const loading = ref(false);
	const tokenModalExam = ref<Exam | null>(null);

	const { copy, copied } = useClipboard();

	const availableIps = computed(() => {
		const list = info.value?.lanIps?.length ? info.value.lanIps : (info.value?.lanIp ? [info.value.lanIp] : []);
		return list;
	});

	const activeIp = computed(() => selectedIp.value || info.value?.lanIp || "");

	const activeExams = computed(() => {
		const now = Math.floor(Date.now() / 1000);
		return exams.value.filter((e) => {
			if (e.scheduledAt !== null && now < e.scheduledAt) return false;
			if (e.windowEnd !== null && now > e.windowEnd) return false;
			return true;
		});
	});

	const joinUrl = computed(() => {
		if (!activeIp.value || !info.value?.port) return "";
		const portSuffix = info.value.port === 80 ? "" : `:${info.value.port}`;
		return `http://${activeIp.value}${portSuffix}/cbt`;
	});

	const serverDisplayLabel = computed(() => {
		if (!activeIp.value || !info.value?.port) return "Server Offline";
		return info.value.port === 80 ? activeIp.value : `${activeIp.value}:${info.value.port}`;
	});

	const stats = computed(() => [
		{ title: "Siswa", icon: "lucide:users", value: dashboardStats.value?.students ?? "-", to: "/siswa" },
		{ title: "Mata Pelajaran", icon: "lucide:book-open", value: dashboardStats.value?.subjects ?? "-", to: "/pengaturan/pelajaran" },
		{ title: "Jenis Ujian", icon: "lucide:list-checks", value: dashboardStats.value?.jenisUjian ?? "-", to: "/soal" },
		{ title: "Ujian Aktif", icon: "lucide:calendar-clock", value: activeExams.value.length, to: "/ujian" }
	]);

	const sessionCounters = computed(() => {
		let workingPg = 0;
		let workingEssay = 0;
		let finished = 0;
		for (const s of sessions.value) {
			if (s.submittedAt) finished++;
			else if (s.pgTotal > 0 && s.pgAnswered >= s.pgTotal && s.totalQuestions > s.pgTotal) workingEssay++;
			else workingPg++;
		}
		return {
			total: sessions.value.length,
			workingPg,
			workingEssay,
			finished
		};
	});

	const preparationChecklist = computed(() => [
		{
			label: "1. Isi Identitas Sekolah",
			done: Boolean(school.value?.name && school.value.name !== "Nama Sekolah Belum Diatur"),
			to: "/pengaturan/sekolah"
		},
		{
			label: "2. Tambah / Impor Data Siswa",
			done: (dashboardStats.value?.students ?? 0) > 0,
			to: "/siswa"
		},
		{
			label: "3. Siapkan Bank Soal",
			done: (dashboardStats.value?.jenisUjian ?? 0) > 0,
			to: "/soal"
		},
		{
			label: "4. Buat Jadwal & Token Ujian",
			done: exams.value.length > 0,
			to: "/ujian"
		}
	]);

	const refresh = async () => {
		loading.value = true;
		try {
			if (!isTauri()) {
				info.value = { port: 80, lanIp: "192.168.1.10", lanIps: ["192.168.1.10", "10.0.0.5"], dbOk: true };
				dashboardStats.value = { students: 32, subjects: 9, jenisUjian: 4 };
				school.value = { name: "SDN Percobaan 01", npsn: "20100001", address: "Jl. Pendidikan No. 1", principal: "Drs. Hendra" };
				exams.value = [];
				sessions.value = [];
				if (!selectedIp.value && info.value.lanIp) selectedIp.value = info.value.lanIp;
				return;
			}
			const [serverInfo, dashStats, schoolInfo, examList, sessionList] = await Promise.all([
				invoke<ServerInfo>("get_server_info"),
				invoke<DashboardStats>("get_dashboard_stats"),
				invoke<SchoolInfo>("get_school"),
				invoke<Exam[]>("list_exams"),
				invoke<SessionProgress[]>("list_exam_sessions")
			]);
			info.value = serverInfo;
			if (!selectedIp.value && serverInfo.lanIp) {
				selectedIp.value = serverInfo.lanIp;
			}
			dashboardStats.value = dashStats;
			school.value = schoolInfo;
			exams.value = examList;
			sessions.value = sessionList;
		} finally {
			loading.value = false;
		}
	};

	const hasilLinkForExam = (exam: Exam) => {
		if (!exam.class) return "/hasil";
		if (exam.jenis) {
			return `/hasil/${exam.class}/${encodeURIComponent(exam.subject)}/${encodeURIComponent(exam.jenis)}`;
		}
		return `/hasil/${exam.class}/${encodeURIComponent(exam.subject)}`;
	};

	let timer: ReturnType<typeof setInterval>;

	onMounted(() => {
		void refresh();
		timer = setInterval(refresh, 5000);
	});

	onUnmounted(() => clearInterval(timer));
</script>

<template>
	<UDashboardPanel id="home">
		<template #header>
			<UDashboardNavbar>
				<template #leading>
					<UDashboardSidebarCollapse />
				</template>

				<template #title>
					<span class="font-bold text-highlighted">{{ school?.name ?? "Dashboard" }}</span>
				</template>

				<template #right>
					<div class="flex items-center gap-2">
						<USelectMenu
							v-if="availableIps.length > 1"
							v-model="selectedIp"
							:items="availableIps"
							size="sm"
							class="w-36 font-mono shrink-0"
							aria-label="Pilih IP Jaringan" />

						<DashboardAksesUjian :join-url="joinUrl" :label="serverDisplayLabel" />

						<UButton
							size="sm"
							icon="lucide:refresh-cw"
							variant="soft"
							color="neutral"
							class="shrink-0"
							:loading="loading"
							aria-label="Muat ulang data dashboard"
							@click="refresh" />
					</div>
				</template>
			</UDashboardNavbar>
		</template>

		<template #body>
			<div class="space-y-6">
				<DashboardStatsGrid :stats="stats" />

				<!-- Live Active Exams & Session Monitor -->
				<div class="grid gap-6 lg:grid-cols-3">
					<UCard class="lg:col-span-2">
						<template #header>
							<div class="flex items-center justify-between gap-2">
								<div class="flex items-center gap-2">
									<UIcon name="lucide:radio" class="size-4 text-success" />
									<span class="font-semibold text-highlighted">Pantauan Ujian Aktif &amp; Token</span>
								</div>
								<UButton
									size="xs"
									variant="soft"
									icon="lucide:plus"
									to="/ujian">
									Kelola Ujian
								</UButton>
							</div>
						</template>

						<div v-if="activeExams.length" class="divide-y divide-default">
							<div
								v-for="exam in activeExams"
								:key="exam.id"
								class="py-3 first:pt-0 last:pb-0 flex items-center justify-between flex-wrap gap-3">
								<div>
									<div class="flex items-center gap-2">
										<span class="font-semibold text-highlighted">{{ exam.title }}</span>
										<UBadge
											v-if="exam.class"
											color="neutral"
											variant="subtle"
											size="xs">
											Kelas {{ exam.class }}
										</UBadge>
										<UBadge color="info" variant="subtle" size="xs">
											{{ exam.subject }}
										</UBadge>
										<UBadge
											v-if="exam.requirePin"
											color="warning"
											variant="subtle"
											size="xs">
											Wajib PIN
										</UBadge>
									</div>
									<p class="text-xs text-muted mt-0.5">
										Durasi {{ exam.duration }} menit
									</p>
								</div>

								<div class="flex items-center gap-2">
									<UButton
										size="xs"
										variant="outline"
										color="primary"
										icon="lucide:maximize-2"
										class="font-mono font-bold"
										@click="tokenModalExam = exam">
										Token: {{ exam.token }}
									</UButton>
									<UButton
										size="xs"
										variant="soft"
										color="neutral"
										icon="lucide:bar-chart-3"
										:to="hasilLinkForExam(exam)">
										Pantau Hasil
									</UButton>
								</div>
							</div>
						</div>
						<div v-else class="text-center py-6 text-muted text-sm space-y-2">
							<p>Belum ada ujian yang sedang berlangsung pada jam ini.</p>
							<UButton
								size="xs"
								variant="soft"
								icon="lucide:calendar-plus"
								to="/ujian">
								Jadwalkan Ujian
							</UButton>
						</div>
					</UCard>

					<!-- Live Student Progress Counter / Quick-Start Checklist -->
					<UCard>
						<template #header>
							<div class="flex items-center gap-2">
								<UIcon name="lucide:activity" class="size-4 text-primary" />
								<span class="font-semibold text-highlighted">Status Pengerjaan Siswa</span>
							</div>
						</template>

						<div v-if="sessionCounters.total > 0" class="space-y-3">
							<div class="grid grid-cols-3 gap-2 text-center">
								<div class="rounded-lg border border-default p-2.5 bg-elevated/40">
									<div class="text-xl font-bold text-default">
										{{ sessionCounters.workingPg }}
									</div>
									<div class="text-[11px] text-muted">
										Sesi PG
									</div>
								</div>
								<div class="rounded-lg border border-default p-2.5 bg-elevated/40">
									<div class="text-xl font-bold text-warning">
										{{ sessionCounters.workingEssay }}
									</div>
									<div class="text-[11px] text-muted">
										Sesi Esai
									</div>
								</div>
								<div class="rounded-lg border border-default p-2.5 bg-elevated/40">
									<div class="text-xl font-bold text-success">
										{{ sessionCounters.finished }}
									</div>
									<div class="text-[11px] text-muted">
										Selesai
									</div>
								</div>
							</div>
							<UButton
								block
								size="sm"
								variant="soft"
								icon="lucide:bar-chart-3"
								to="/hasil">
								Buka Semua Hasil Ujian
							</UButton>
						</div>

						<div v-else class="space-y-2">
							<p class="text-xs text-muted mb-2">
								Daftar periksa persiapan ujian:
							</p>
							<NuxtLink
								v-for="step in preparationChecklist"
								:key="step.label"
								:to="step.to"
								class="flex items-center justify-between gap-2 p-2 rounded-md border border-default hover:bg-elevated transition-colors text-xs">
								<span class="flex items-center gap-2">
									<UIcon
										:name="step.done ? 'lucide:check-circle-2' : 'lucide:circle'"
										:class="step.done ? 'text-success' : 'text-muted'"
										class="size-4 shrink-0" />
									<span :class="step.done ? 'text-default' : 'font-medium text-highlighted'">{{ step.label }}</span>
								</span>
								<UIcon name="lucide:chevron-right" class="size-3.5 text-muted" />
							</NuxtLink>
						</div>
					</UCard>
				</div>

				<DashboardSekolahInfo :school="school" />
			</div>
		</template>
	</UDashboardPanel>

	<!-- Projector Token Display Modal -->
	<UModal
		:open="Boolean(tokenModalExam)"
		:title="tokenModalExam ? `Token Ujian: ${tokenModalExam.title}` : 'Token Ujian'"
		@update:open="(v) => { if (!v) tokenModalExam = null }">
		<template #body>
			<div v-if="tokenModalExam" class="text-center py-6 space-y-4">
				<p class="text-sm text-muted">
					Kelas {{ tokenModalExam.class ?? "-" }} · {{ tokenModalExam.subject }}
				</p>
				<div class="py-6 px-4 rounded-xl bg-elevated border-2 border-primary/30">
					<p class="text-5xl sm:text-6xl font-mono font-bold tracking-widest text-highlighted">
						{{ tokenModalExam.token }}
					</p>
				</div>
				<p v-if="joinUrl" class="text-sm font-mono text-muted">
					Alamat Akses: <strong class="text-default">{{ joinUrl }}</strong>
				</p>
				<div class="flex justify-center gap-2 pt-2">
					<UButton
						variant="soft"
						:icon="copied ? 'lucide:check' : 'lucide:copy'"
						@click="copy(tokenModalExam.token)">
						{{ copied ? "Token Disalin" : "Salin Token" }}
					</UButton>
				</div>
			</div>
		</template>
	</UModal>
</template>
