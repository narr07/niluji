<script lang="ts" setup>
	import { invoke } from "@tauri-apps/api/core";

	interface StudentRecord {
		id: number
		nisn: string
		name: string
		class: string | null
		pin?: string
	}

	interface SchoolInfo {
		name: string
		npsn: string
		address: string
		principal: string
	}

	interface ServerInfo {
		port: number | null
		lanIp: string | null
		lanIps?: string[]
		dbOk: boolean
	}

	const props = defineProps<{
		open: boolean
		students: StudentRecord[]
		defaultClass?: string
	}>();

	const emit = defineEmits<{
		"update:open": [value: boolean]
		regenerated: []
	}>();

	const openModel = computed({
		get: () => props.open,
		set: (v) => emit("update:open", v)
	});

	const isTauri = () => typeof window !== "undefined" && !!(window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;
	const toast = useToast();

	const school = ref<SchoolInfo>({ name: "Sekolah", npsn: "", address: "", principal: "" });
	const joinUrl = ref("");
	const qrDataUrl = ref("");
	const filterClass = ref(props.defaultClass || "ALL");
	const regenerating = ref(false);

	const classOptions = computed(() => {
		const unique = [...new Set(props.students.map((s) => s.class).filter((c): c is string => !!c))].sort();
		return [{ label: "Semua Kelas", value: "ALL" }, ...unique.map((c) => ({ label: `Kelas ${c}`, value: c }))];
	});

	const filteredStudents = computed(() => {
		if (filterClass.value === "ALL") return props.students;
		return props.students.filter((s) => s.class === filterClass.value);
	});

	const loadMeta = async () => {
		if (!isTauri()) {
			school.value = { name: "SDN Percobaan", npsn: "12345678", address: "", principal: "" };
			joinUrl.value = "http://192.168.1.10/cbt";
		} else {
			const [sInfo, srv] = await Promise.all([
				invoke<SchoolInfo>("get_school"),
				invoke<ServerInfo>("get_server_info")
			]);
			school.value = sInfo;
			if (srv.lanIp && srv.port) {
				const portSuffix = srv.port === 80 ? "" : `:${srv.port}`;
				joinUrl.value = `http://${srv.lanIp}${portSuffix}/cbt`;
			}
		}

		if (joinUrl.value) {
			const QRCode = await import("qrcode");
			qrDataUrl.value = await QRCode.toDataURL(joinUrl.value, { width: 120, margin: 1 });
		}
	};

	watch(
		() => props.open,
		(open) => {
			if (open) {
				filterClass.value = props.defaultClass || "ALL";
				void loadMeta();
			}
		},
		{ immediate: true }
	);

	const regeneratePins = async () => {
		if (!isTauri()) return;
		const targetLabel = filterClass.value === "ALL" ? "semua kelas" : `Kelas ${filterClass.value}`;
		const ok = await confirmDelete({
			title: `Acak ulang PIN siswa (${targetLabel})?`,
			description: "PIN lama tidak akan berlaku lagi. Pastikan kartu peserta dicetak ulang setelah ini.",
			confirmLabel: "Acak Ulang PIN"
		});
		if (!ok) return;

		regenerating.value = true;
		try {
			const count = await invoke<number>("regenerate_student_pins", {
				class: filterClass.value === "ALL" ? null : filterClass.value
			});
			toast.add({
				title: "PIN diperbarui",
				description: `${count} PIN siswa berhasil diacak ulang.`,
				icon: "lucide:check",
				color: "success"
			});
			emit("regenerated");
		} finally {
			regenerating.value = false;
		}
	};

	const printCards = () => {
		if (typeof window !== "undefined") {
			window.print();
		}
	};
</script>

<template>
	<UModal
		v-model:open="openModel"
		title="Kartu Peserta Ujian & PIN Siswa"
		description="Cetak kartu peserta dilengkapi NISN, PIN 6 digit, dan QR akses ujian lokal"
		:ui="{ content: 'max-w-4xl' }">
		<template #body>
			<div class="space-y-4">
				<div class="flex flex-wrap items-center justify-between gap-3 print:hidden">
					<div class="flex items-center gap-2">
						<USelectMenu
							v-if="!defaultClass"
							v-model="filterClass"
							:items="classOptions"
							value-key="value"
							class="w-44" />
						<span class="text-sm text-muted">
							{{ filteredStudents.length }} kartu siap cetak
						</span>
					</div>
					<div class="flex items-center gap-2">
						<UButton
							variant="soft"
							color="warning"
							icon="lucide:key-round"
							:loading="regenerating"
							@click="regeneratePins">
							Acak Ulang PIN
						</UButton>
						<UButton
							color="primary"
							icon="lucide:printer"
							:disabled="!filteredStudents.length"
							@click="printCards">
							Cetak Kartu
						</UButton>
					</div>
				</div>

				<div
					v-if="filteredStudents.length"
					class="grid grid-cols-1 sm:grid-cols-2 gap-3 max-h-[65vh] overflow-y-auto p-1 print:max-h-none print:overflow-visible print:grid-cols-2">
					<div
						v-for="student in filteredStudents"
						:key="student.id"
						class="rounded-lg border border-default p-3.5 bg-default flex items-start justify-between gap-3 break-inside-avoid">
						<div class="space-y-1.5 min-w-0 flex-1">
							<div class="border-b border-default pb-1.5">
								<p class="text-[11px] font-semibold uppercase tracking-wider text-muted truncate">
									Kartu Peserta CBT · {{ school.name || "NILUJI" }}
								</p>
							</div>
							<div class="pt-0.5 space-y-1 text-xs">
								<div class="flex gap-2">
									<span class="text-muted w-14 shrink-0">Nama</span>
									<span class="font-semibold text-highlighted truncate">: {{ student.name }}</span>
								</div>
								<div class="flex gap-2">
									<span class="text-muted w-14 shrink-0">NISN</span>
									<span class="font-mono text-default">: {{ student.nisn }}</span>
								</div>
								<div class="flex gap-2">
									<span class="text-muted w-14 shrink-0">Kelas</span>
									<span class="text-default">: {{ student.class ?? "-" }}</span>
								</div>
								<div class="flex items-center gap-2 pt-1">
									<span class="text-muted w-14 shrink-0">PIN</span>
									<span class="font-mono font-bold text-sm px-2 py-0.5 rounded bg-elevated border border-default tracking-widest text-highlighted">
										{{ student.pin || "-" }}
									</span>
								</div>
							</div>
							<p v-if="joinUrl" class="text-[10px] font-mono text-muted pt-1 truncate">
								Akses: {{ joinUrl }}
							</p>
						</div>

						<div v-if="qrDataUrl" class="shrink-0 flex flex-col items-center">
							<img
								:src="qrDataUrl"
								alt="QR Ujian"
								class="size-20 rounded border border-default p-0.5 bg-white">
							<span class="text-[9px] text-muted mt-0.5">Scan Ujian</span>
						</div>
					</div>
				</div>

				<div v-else class="text-center py-10 text-muted">
					Belum ada data siswa untuk dicetak.
				</div>
			</div>
		</template>
	</UModal>
</template>

