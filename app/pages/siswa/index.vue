<script lang="ts" setup>
	import { invoke } from "@tauri-apps/api/core";

	definePageMeta({
		name: "Siswa",
		icon: "lucide:users",
		category: "cbt",
		order: 1,
		description: "Data siswa per kelas: tambah, edit, hapus, import, dan live status login"
	});

	interface StudentRecord {
		id: number
		nisn: string
		name: string
		class: string | null
	}

	interface OnlineStudent {
		nisn: string
		name: string
		loggedInAt: number
	}

	// Lets this page render with mock data when opened in a plain browser (e.g. `bun run dev`)
	// instead of inside the Tauri desktop shell, where `invoke()` has nothing to talk to.
	const isTauri = () => typeof window !== "undefined" && !!(window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;

	const toast = useToast();
	const notifyTemplateDownload = (format: string) => {
		toast.add({ title: "Template diunduh", description: `Contoh siswa format ${format} sedang diunduh.`, icon: "lucide:download", color: "success" });
	};

	const downloadTemplate = (format: "csv" | "xlsx") => {
		const link = document.createElement("a");
		link.href = `/templates/siswa-template.${format}`;
		link.download = `siswa-template.${format}`;
		document.body.appendChild(link);
		link.click();
		document.body.removeChild(link);
		notifyTemplateDownload(format === "csv" ? "CSV" : "Excel");
	};

	const templateItems = [
		[
			{
				label: "Template CSV (.csv)",
				icon: "lucide:file-text",
				onSelect: () => downloadTemplate("csv")
			},
			{
				label: "Template Excel (.xlsx)",
				icon: "lucide:file-spreadsheet",
				onSelect: () => downloadTemplate("xlsx")
			}
		]
	];

	const students = ref<StudentRecord[]>([]);
	const online = ref<OnlineStudent[]>([]);
	const classes = ref<string[]>([]);

	// Every fixed class (4/5/6) always shows up, even with zero students, so admin can
	// drill into an empty class and add students to it.
	const classGroups = computed(() => {
		const counts = new Map<string, number>();
		for (const c of classes.value) counts.set(c, 0);
		for (const s of students.value) {
			if (s.class && counts.has(s.class)) counts.set(s.class, (counts.get(s.class) ?? 0) + 1);
		}
		return [...counts.entries()].map(([cls, count]) => ({ class: cls, count }));
	});

	const loadData = async () => {
		if (!isTauri()) {
			students.value = [
				{ id: 1, nisn: "1001", name: "Ahmad Rizky", class: "9A" },
				{ id: 2, nisn: "1002", name: "Siti Nurhaliza", class: "9A" },
				{ id: 3, nisn: "1003", name: "Budi Santoso", class: "9B" }
			];
			classes.value = ["9A", "9B", "9C"];
			return;
		}
		[students.value, classes.value] = await Promise.all([
			invoke<StudentRecord[]>("list_students"),
			invoke<string[]>("list_classes")
		]);
	};

	const refreshOnline = async () => {
		if (!isTauri()) {
			online.value = [
				{ nisn: "1001", name: "Ahmad Rizky", loggedInAt: Date.now() - 1000 * 60 * 5 }
			];
			return;
		}
		online.value = await invoke<OnlineStudent[]>("list_online_students");
	};

	const importOpen = ref(false);
	const openImport = () => {
		if (!isTauri()) {
			toast.add({ title: "Mode Pratinjau", description: "Fitur import berjalan di dalam aplikasi desktop Tauri.", color: "info" });
			return;
		}
		importOpen.value = true;
	};

	let timer: ReturnType<typeof setInterval>;

	onMounted(() => {
		loadData();
		refreshOnline();
		timer = setInterval(refreshOnline, 3000);
	});

	onUnmounted(() => clearInterval(timer));

	const templateMenuOpen = ref(false);
</script>

<template>
	<UDashboardPanel id="siswa">
		<template #header>
			<UDashboardNavbar title="Siswa">
				<template #leading>
					<UDashboardSidebarCollapse />
				</template>

				<template #right>
					<UBadge color="success" variant="subtle" class="flex items-center gap-1.5">
						<span class="inline-block size-1.5 rounded-full bg-success" />
						{{ online.length }} online
					</UBadge>

					<UDropdownMenu v-model:open="templateMenuOpen" :items="templateItems">
						<UButton
							icon="lucide:file-down"
							variant="subtle"
							:trailing-icon="templateMenuOpen ? 'lucide:chevron-up' : 'lucide:chevron-down'"
						>
							Template
						</UButton>
					</UDropdownMenu>

					<UButton
						icon="lucide:upload"
						variant="soft"
						color="primary"
						@click="openImport">
						Import
					</UButton>
				</template>
			</UDashboardNavbar>
		</template>

		<template #body>
			<div class="space-y-6">
				<SiswaKelasGrid :class-groups="classGroups" />
				<SiswaOnlineTable :online="online" />
			</div>
		</template>
	</UDashboardPanel>

	<SiswaImportModal v-model:open="importOpen" @imported="loadData" />
</template>
