<script lang="ts" setup>
	import { invoke } from "@tauri-apps/api/core";

	definePageMeta({
		name: "Dashboard",
		icon: "lucide:layout-dashboard",
		top: true,
		description: "Ringkasan status server, jumlah data, dan info sekolah"
	});

	interface ServerInfo {
		port: number | null
		lanIp: string | null
		dbOk: boolean
	}

	interface DashboardStats {
		students: number
		subjects: number
		questions: number
	}

	interface Exam {
		id: number
	}

	interface SchoolInfo {
		name: string
		npsn: string
		address: string
		principal: string
	}

	const school = ref<SchoolInfo | null>(null);
	const info = ref<ServerInfo | null>(null);
	const dashboardStats = ref<DashboardStats | null>(null);
	const examCount = ref<number | null>(null);
	const loading = ref(false);

	// Port 80 is the common case (see server.rs PREFERRED_PORT) so the URL only shows
	// ":port" when it actually fell back to something else.
	const joinUrl = computed(() => {
		if (!info.value?.lanIp || !info.value?.port) return "";
		const portSuffix = info.value.port === 80 ? "" : `:${info.value.port}`;
		return `http://${info.value.lanIp}${portSuffix}/cbt`;
	});

	const stats = computed(() => [
		{ title: "Siswa", icon: "lucide:users", value: dashboardStats.value?.students ?? "-", to: "/siswa" },
		{ title: "Mata Pelajaran", icon: "lucide:book-open", value: dashboardStats.value?.subjects ?? "-", to: "/pengaturan/pelajaran" },
		{ title: "Soal", icon: "lucide:list-checks", value: dashboardStats.value?.questions ?? "-", to: "/soal" },
		{ title: "Ujian", icon: "lucide:calendar-clock", value: examCount.value ?? "-", to: "/ujian" }
	]);

	const refresh = async () => {
		loading.value = true;
		try {
			const [serverInfo, dashStats, schoolInfo, exams] = await Promise.all([
				invoke<ServerInfo>("get_server_info"),
				invoke<DashboardStats>("get_dashboard_stats"),
				invoke<SchoolInfo>("get_school"),
				invoke<Exam[]>("list_exams")
			]);
			info.value = serverInfo;
			dashboardStats.value = dashStats;
			school.value = schoolInfo;
			examCount.value = exams.length;
		} finally {
			loading.value = false;
		}
	};

	onMounted(refresh);
</script>

<template>
	<UDashboardPanel id="home">
		<template #header>
			<UDashboardNavbar :ui="{ right: 'gap-3' }">
				<template #leading>
					<UDashboardSidebarCollapse />
				</template>

				<template #title>
					<div class="flex items-center gap-2">
						<SvgoLogo :font-controlled="false" class="size-6 shrink-0 text-primary-900 dark:text-primary-200 " />
						<span class="font-bold">{{ school?.name ?? "..." }}</span>
					</div>
				</template>

				<template #right>
					<span class="text-sm text-muted font-mono">
						{{ info?.lanIp && info?.port ? `${info.lanIp}:${info.port}` : "-" }}
					</span>
					<UButton
						icon="lucide:refresh-cw"
						color="neutral"
						variant="ghost"
						:loading="loading"
						@click="refresh"
					/>
				</template>
			</UDashboardNavbar>
		</template>

		<template #body>
			<div class="space-y-6">
				<DashboardStatsGrid :stats="stats" />
				<DashboardAksesUjian :join-url="joinUrl" />
				<DashboardSekolahInfo :school="school" />
			</div>
		</template>
	</UDashboardPanel>
</template>
