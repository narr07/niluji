<script lang="ts" setup>
	import { invoke } from "@tauri-apps/api/core";
	import { fetch as tauriFetch } from "@tauri-apps/plugin-http";
	import { open as openUrl } from "@tauri-apps/plugin-shell";

	// CSV publish-to-web dari Google Sheets — auto-update kalau data sumbernya berubah,
	// tidak perlu diganti manual tiap semester selama sheet sumbernya tetap dipakai.
	const CSV_URL =
		"https://docs.google.com/spreadsheets/d/e/2PACX-1vS2dCk5Yo9KFKBqo8fT0vq8941YB6AIZvjseHzEwyOOcN6CwfCiPqc944JJwKlnCYs1e9VC5CpK4lBf/pub?output=csv";
	const CONNECTION_CHECK_URL = "https://docs.google.com/";
	// Sheet aslinya (bukan link publish-to-web CSV di atas) — dibuka di browser kalau guru mau
	// betulin nama siswa/NISN yang salah, karena CSV publish-to-web itu read-only.
	const EDIT_SHEET_URL = "https://docs.google.com/spreadsheets/d/1p4_wCArJP0p2Ap70UUx3hcIXZLgP7cZumF2AJNQN-LY/edit?gid=0#gid=0";

	interface StudentPreview {
		id: number
		nisn: string
		name: string
		class: string | null
	}

	const school = ref("");
	// Daftar nama sekolah PERSIS dari sumber CSV (bukan dari database lokal) — supaya sekolah
	// dengan nama mirip (mis. "SD NEGERI CIPINANG I" vs "SD NEGERI CIPINANG II") tetap kebeda
	// di dropdown, dan hasil pencarian tidak ketuker gabung gara-gara sama-sama mengandung kata
	// yang sama.
	const sourceSchools = ref<string[]>([]);
	const loadingSchools = ref(false);
	const schoolsError = ref("");
	const loading = ref(false);
	const errorMessage = ref("");
	const resultCount = ref<number | null>(null);
	const preview = ref<StudentPreview[]>([]);

	const checkingConnection = ref(false);
	const connectionStatus = ref<"checking" | "online" | "offline">("checking");
	const connectionMessage = ref("Memeriksa koneksi internet...");
	const connectionCheckedAt = ref<Date>();

	const canFetch = computed(() => connectionStatus.value === "online");
	const toast = useToast();

	const refreshConnection = async () => {
		checkingConnection.value = true;
		connectionStatus.value = "checking";
		connectionMessage.value = "Memeriksa koneksi internet...";
		try {
			const response = await tauriFetch(CONNECTION_CHECK_URL, { cache: "no-store" });
			if (!response.ok) throw new Error(`Server merespons status ${response.status}`);
			connectionStatus.value = "online";
			connectionMessage.value = "Internet terhubung, sumber data siswa dapat dijangkau.";
		} catch {
			connectionStatus.value = "offline";
			connectionMessage.value = "Tidak ada koneksi internet atau sumber data siswa tidak dapat dijangkau.";
		} finally {
			connectionCheckedAt.value = new Date();
			checkingConnection.value = false;
		}
		return connectionStatus.value === "online";
	};

	const loadSourceSchools = async () => {
		loadingSchools.value = true;
		schoolsError.value = "";
		try {
			sourceSchools.value = await invoke<string[]>("list_schools_from_source", { csvUrl: CSV_URL });
		} catch (error) {
			schoolsError.value = error instanceof Error ? error.message : String(error);
		} finally {
			loadingSchools.value = false;
		}
	};

	const tarik = async () => {
		const target = school.value.trim();
		if (!target) {
			errorMessage.value = "Isi nama sekolah dulu.";
			return;
		}
		if (!canFetch.value) {
			errorMessage.value = "Tidak ada koneksi internet.";
			return;
		}

		loading.value = true;
		errorMessage.value = "";
		resultCount.value = null;
		preview.value = [];
		try {
			const count = await invoke<number>("tarik_data_siswa", { csvUrl: CSV_URL, school: target });
			resultCount.value = count;
			preview.value = await invoke<StudentPreview[]>("students_by_school", { school: target });
			toast.add({ title: "Tarik data selesai", description: `${count} siswa berhasil ditarik untuk ${target}.`, icon: "lucide:check", color: "success" });
		} catch (error) {
			errorMessage.value = error instanceof Error ? error.message : String(error);
			toast.add({ title: "Tarik data gagal", description: errorMessage.value, icon: "lucide:x", color: "error" });
		} finally {
			loading.value = false;
		}
	};

	onMounted(async () => {
		const isConnected = await refreshConnection();
		if (isConnected) void loadSourceSchools();
	});
</script>

<template>
	<div class="space-y-6">
		<div>
			<h2 class="text-lg font-semibold">Tarik Data Siswa</h2>
			<p class="text-sm text-muted mt-1">
				Ambil data siswa dari sumber online, difilter berdasarkan nama sekolah, lalu disimpan ke database lokal.
			</p>
		</div>

		<UAlert
			color="info"
			variant="subtle"
			title="Ada nama siswa atau NISN yang salah?"
			icon="lucide:info">
			<template #description>
				<p>Data ini bersumber dari Google Sheets — kalau ada kesalahan, bisa dibuka & diedit langsung di sana, lalu tarik ulang datanya di sini.</p>
				<UButton
					variant="link"
					color="info"
					icon="lucide:external-link"
					class="px-0 mt-1"
					@click="openUrl(EDIT_SHEET_URL)">
					Buka & Edit Data Sumber
				</UButton>
			</template>
		</UAlert>

		<div class="flex flex-wrap items-center gap-3 rounded-lg border border-default bg-elevated/40 px-4 py-3">
			<UIcon
				:name="connectionStatus === 'online' ? 'lucide:wifi' : connectionStatus === 'offline' ? 'lucide:wifi-off' : 'lucide:loader-circle'"
				:class="['size-5', connectionStatus === 'online' ? 'text-success' : connectionStatus === 'offline' ? 'text-error' : 'animate-spin text-muted']" />
			<div class="flex-1 text-sm">
				<p :class="connectionStatus === 'online' ? 'text-success' : connectionStatus === 'offline' ? 'text-error' : 'text-muted'">
					{{ connectionMessage }}
				</p>
				<p v-if="connectionCheckedAt" class="mt-0.5 text-xs text-muted">
					Diperiksa pukul {{ connectionCheckedAt.toLocaleTimeString('id-ID') }}
				</p>
			</div>
			<UButton
				variant="soft"
				icon="lucide:refresh-cw"
				:loading="checkingConnection"
				@click="refreshConnection().then(() => canFetch && loadSourceSchools())">
				Refresh koneksi
			</UButton>
		</div>

		<div class="grid gap-4 md:grid-cols-3 items-end">
			<UFormField label="Nama Sekolah" description="Ketik untuk mencari, lalu pilih nama sekolah yang persis dari daftar." class="md:col-span-2">
				<UInputMenu
					v-model="school"
					:items="sourceSchools"
					:loading="loadingSchools"
					:disabled="!canFetch"
					open-on-click
					placeholder="Contoh: SD NEGERI CIPINANG I"
					class="w-full" />
			</UFormField>

			<UButton
				icon="lucide:cloud-download"
				:loading="loading"
				:disabled="!canFetch || loadingSchools"
				@click="tarik">
				Tarik Data Siswa
			</UButton>
		</div>

		<UAlert
			v-if="schoolsError"
			color="warning"
			variant="subtle"
			title="Gagal mengambil daftar sekolah dari sumber"
			:description="schoolsError" />

		<UAlert
			v-if="errorMessage"
			color="error"
			variant="subtle"
			title="Gagal menarik data"
			:description="errorMessage" />
		<UAlert
			v-if="resultCount !== null"
			color="success"
			variant="subtle"
			:title="`Berhasil menarik ${resultCount} siswa untuk sekolah ini.`" />

		<div v-if="preview.length" class="overflow-x-auto rounded-lg border border-default max-h-96">
			<table class="w-full text-sm">
				<thead class="bg-elevated text-left sticky top-0">
					<tr>
						<th class="p-2">NISN</th>
						<th class="p-2">Nama</th>
						<th class="p-2">Kelas</th>
					</tr>
				</thead>
				<tbody class="divide-y divide-default">
					<tr v-for="s in preview" :key="s.id">
						<td class="p-2">{{ s.nisn }}</td>
						<td class="p-2">{{ s.name }}</td>
						<td class="p-2">{{ s.class }}</td>
					</tr>
				</tbody>
			</table>
		</div>
	</div>
</template>
