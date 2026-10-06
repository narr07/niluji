<script lang="ts" setup>
	import { invoke } from "@tauri-apps/api/core";
	import { fetch as tauriFetch } from "@tauri-apps/plugin-http";
	import { open as openUrl } from "@tauri-apps/plugin-shell";

	// Sumber bawaan: CSV publish-to-web dari Google Sheets — auto-update kalau data sumbernya
	// berubah, tidak perlu diganti manual tiap semester selama sheet sumbernya tetap dipakai.
	const DEFAULT_CSV_URL =
		"https://docs.google.com/spreadsheets/d/e/2PACX-1vS2dCk5Yo9KFKBqo8fT0vq8941YB6AIZvjseHzEwyOOcN6CwfCiPqc944JJwKlnCYs1e9VC5CpK4lBf/pub?output=csv";
	// Sheet aslinya (bukan link publish-to-web CSV di atas) — dibuka di browser kalau guru mau
	// betulin nama siswa/NISN yang salah, karena CSV publish-to-web itu read-only.
	const DEFAULT_EDIT_URL = "https://docs.google.com/spreadsheets/d/1p4_wCArJP0p2Ap70UUx3hcIXZLgP7cZumF2AJNQN-LY/edit?gid=0#gid=0";
	const CONNECTION_CHECK_URL = "https://docs.google.com/";
	const SOURCE_STORAGE_KEY = "niluji:tarik-siswa-sumber";

	interface StudentPreview {
		id: number
		nisn: string
		name: string
		class: string | null
	}

	type SourceMode = "default" | "custom";

	// Sekolah di kecamatan lain memakai sheet yang disiapkan operator kecamatannya sendiri — link
	// yang ditempel boleh link biasa (".../edit") atau link "Publikasikan ke web" (".../pub"),
	// keduanya diubah ke URL yang mengembalikan CSV.
	const toCsvUrl = (link: string): { csvUrl: string, editUrl: string | null } | null => {
		let url: URL;
		try {
			url = new URL(link.trim());
		} catch {
			return null;
		}
		if (url.hostname !== "docs.google.com" || !url.pathname.startsWith("/spreadsheets/")) return null;

		const gid = url.searchParams.get("gid") ?? url.hash.match(/gid=(\d+)/)?.[1];

		const published = url.pathname.match(/^\/spreadsheets\/d\/e\/([\w-]+)/);
		if (published) {
			const csv = new URL(`https://docs.google.com/spreadsheets/d/e/${published[1]}/pub`);
			if (gid) csv.searchParams.set("gid", gid);
			csv.searchParams.set("single", "true");
			csv.searchParams.set("output", "csv");
			return { csvUrl: csv.toString(), editUrl: null };
		}

		const normal = url.pathname.match(/^\/spreadsheets\/d\/([\w-]+)/);
		if (normal) {
			const csv = new URL(`https://docs.google.com/spreadsheets/d/${normal[1]}/export`);
			csv.searchParams.set("format", "csv");
			if (gid) csv.searchParams.set("gid", gid);
			return { csvUrl: csv.toString(), editUrl: `https://docs.google.com/spreadsheets/d/${normal[1]}/edit${gid ? `#gid=${gid}` : ""}` };
		}
		return null;
	};

	const sourceMode = ref<SourceMode>("default");
	const customLinkInput = ref("");
	// Link custom yang sudah dipakai (lolos validasi) — beda dengan input yang mungkin masih diketik.
	const savedCustomLink = ref("");
	const customLinkError = ref("");

	const activeSource = computed(() => {
		if (sourceMode.value === "default") return { csvUrl: DEFAULT_CSV_URL, editUrl: DEFAULT_EDIT_URL };
		return savedCustomLink.value ? toCsvUrl(savedCustomLink.value) : null;
	});

	const sourceItems = [
		{ label: "Sumber bawaan", value: "default", description: "Google Sheet yang sudah terpasang di aplikasi." },
		{ label: "Link Google Sheet lain", value: "custom", description: "Untuk kecamatan lain — pakai sheet dari operator kecamatan." }
	];

	const persistSource = () => {
		try {
			localStorage.setItem(SOURCE_STORAGE_KEY, JSON.stringify({ mode: sourceMode.value, link: savedCustomLink.value }));
		} catch {
			// Abaikan kalau localStorage tidak bisa diakses — pilihan sumber cuma tidak diingat.
		}
	};

	const restoreSource = () => {
		try {
			const saved = JSON.parse(localStorage.getItem(SOURCE_STORAGE_KEY) ?? "null") as { mode?: SourceMode, link?: string } | null;
			if (saved?.link && toCsvUrl(saved.link)) {
				savedCustomLink.value = saved.link;
				customLinkInput.value = saved.link;
			}
			if (saved?.mode === "custom") sourceMode.value = "custom";
		} catch {
			// Data lama rusak/tidak bisa dibaca — mulai dari sumber bawaan.
		}
	};

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

	const canFetch = computed(() => connectionStatus.value === "online" && !!activeSource.value);
	const toast = useToast();
	const { downloadTemplate } = useTemplateDownload();

	const refreshConnection = async () => {
		checkingConnection.value = true;
		connectionStatus.value = "checking";
		connectionMessage.value = "Memeriksa koneksi internet...";
		try {
			const response = await tauriFetch(CONNECTION_CHECK_URL, { cache: "no-store" });
			if (!response.ok) throw new Error(`Server merespons status ${response.status}`);
			connectionStatus.value = "online";
			connectionMessage.value = "Internet terhubung, Google Sheets dapat dijangkau.";
		} catch {
			connectionStatus.value = "offline";
			connectionMessage.value = "Tidak ada koneksi internet atau Google Sheets tidak dapat dijangkau.";
		} finally {
			connectionCheckedAt.value = new Date();
			checkingConnection.value = false;
		}
		return connectionStatus.value === "online";
	};

	const loadSourceSchools = async () => {
		const source = activeSource.value;
		sourceSchools.value = [];
		schoolsError.value = "";
		if (!source || connectionStatus.value !== "online") return false;

		loadingSchools.value = true;
		try {
			sourceSchools.value = await invoke<string[]>("list_schools_from_source", { csvUrl: source.csvUrl });
			if (!sourceSchools.value.length) schoolsError.value = "Sheet terbaca, tapi belum ada baris siswa yang lengkap (NISN, nama, dan nama sekolah).";
			return sourceSchools.value.length > 0;
		} catch (error) {
			schoolsError.value = error instanceof Error ? error.message : String(error);
			return false;
		} finally {
			loadingSchools.value = false;
		}
	};

	// Ganti sumber = daftar sekolah & hasil tarikan sebelumnya tidak berlaku lagi.
	const resetResult = () => {
		school.value = "";
		errorMessage.value = "";
		resultCount.value = null;
		preview.value = [];
	};

	watch(sourceMode, () => {
		persistSource();
		resetResult();
		void loadSourceSchools();
	});

	const useCustomLink = async () => {
		customLinkError.value = "";
		if (!toCsvUrl(customLinkInput.value)) {
			customLinkError.value = "Ini bukan link Google Sheets. Salin link dari tombol \"Bagikan\" di Google Sheets (diawali https://docs.google.com/spreadsheets/...).";
			return;
		}
		savedCustomLink.value = customLinkInput.value.trim();
		persistSource();
		resetResult();
		if (await loadSourceSchools()) {
			toast.add({ title: "Link bisa dipakai", description: `${sourceSchools.value.length} sekolah ditemukan di sheet ini.`, icon: "lucide:check", color: "success" });
		}
	};

	const tarik = async () => {
		const target = school.value.trim();
		const source = activeSource.value;
		if (!target) {
			errorMessage.value = "Pilih nama sekolah dulu.";
			return;
		}
		if (!source || !canFetch.value) {
			errorMessage.value = "Tidak ada koneksi internet atau sumber data belum diatur.";
			return;
		}

		loading.value = true;
		errorMessage.value = "";
		resultCount.value = null;
		preview.value = [];
		try {
			const count = await invoke<number>("tarik_data_siswa", { csvUrl: source.csvUrl, school: target });
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

	const guideSteps = [
		{
			title: "Buat Google Sheet baru",
			body: "Operator kecamatan membuat satu Google Sheet untuk seluruh sekolah di kecamatannya. Cara tercepat: unduh template di bawah, lalu di Google Sheets pilih File → Impor → Upload."
		},
		{
			title: "Baris pertama = judul kolom",
			body: "Isi baris pertama persis dengan: NISN | NAMA SISWA | KELAS | NAMA SEKOLAH. Urutan kolom bebas, huruf besar/kecil tidak masalah."
		},
		{
			title: "Isi data siswa, satu siswa per baris",
			body: "Ubah kolom NISN ke format teks (Format → Angka → Teks biasa) supaya angka 0 di depan NISN tidak hilang. KELAS boleh angka (4) atau teks (IVA)."
		},
		{
			title: "Tulis nama sekolah secara seragam",
			body: "Nama sekolah dipakai untuk menyaring siswa. \"SD NEGERI 1 CONTOH\" dan \"SDN 1 Contoh\" dianggap dua sekolah berbeda — pakai satu penulisan yang sama untuk semua siswa sekolah itu."
		},
		{
			title: "Bagikan link sheet",
			body: "Klik Bagikan → Akses umum → \"Siapa saja yang memiliki link\" sebagai Viewer, lalu Salin link. Kirim link itu ke guru/operator sekolah di kecamatan tersebut."
		},
		{
			title: "Tempel link di aplikasi",
			body: "Di bagian Sumber Data, pilih \"Link Google Sheet lain\", tempel link-nya, lalu klik \"Pakai Link\". Link disimpan, jadi cukup sekali."
		}
	];

	onMounted(async () => {
		restoreSource();
		const isConnected = await refreshConnection();
		if (isConnected) void loadSourceSchools();
	});
</script>

<template>
	<div class="space-y-6">
		<div>
			<h2 class="text-lg font-semibold">Tarik Data Siswa</h2>
			<p class="text-sm text-muted mt-1">
				Ambil data siswa dari Google Sheets, difilter berdasarkan nama sekolah, lalu disimpan ke database lokal.
			</p>
		</div>

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
				@click="refreshConnection().then((ok) => ok && loadSourceSchools())">
				Refresh koneksi
			</UButton>
		</div>

		<div class="space-y-4 rounded-lg border border-default p-4">
			<UFormField label="Sumber Data">
				<URadioGroup
					v-model="sourceMode"
					:items="sourceItems"
					variant="table"
					orientation="horizontal"
					:ui="{ fieldset: 'w-full', item: 'flex-1' }" />
			</UFormField>

			<div v-if="sourceMode === 'custom'" class="space-y-2">
				<UFormField
					label="Link Google Sheet"
					description="Tempel link dari tombol Bagikan di Google Sheets (atau link Publikasikan ke web)."
					:error="customLinkError || undefined">
					<div class="flex gap-2">
						<UInput
							v-model="customLinkInput"
							placeholder="https://docs.google.com/spreadsheets/d/..."
							icon="lucide:link"
							class="flex-1"
							@keydown.enter="useCustomLink" />
						<UButton
							icon="lucide:check"
							:loading="loadingSchools"
							:disabled="!customLinkInput.trim() || connectionStatus !== 'online'"
							@click="useCustomLink">
							Pakai Link
						</UButton>
					</div>
				</UFormField>
				<p v-if="savedCustomLink && savedCustomLink !== customLinkInput.trim()" class="text-xs text-muted">
					Masih memakai link sebelumnya — klik "Pakai Link" untuk mengganti.
				</p>
			</div>

			<div v-if="activeSource?.editUrl" class="flex flex-wrap items-center gap-x-2 text-sm text-muted">
				<UIcon name="lucide:info" class="size-4 text-info" />
				Ada nama siswa atau NISN yang salah? Perbaiki langsung di Google Sheets, lalu tarik ulang di sini.
				<UButton
					variant="link"
					color="info"
					icon="lucide:external-link"
					class="px-0"
					@click="openUrl(activeSource.editUrl)">
					Buka Sheet Sumber
				</UButton>
			</div>
		</div>

		<UCollapsible :default-open="sourceMode === 'custom' && !savedCustomLink" class="rounded-lg border border-default">
			<UButton
				variant="ghost"
				color="neutral"
				icon="lucide:book-open"
				trailing-icon="lucide:chevron-down"
				block
				class="justify-start px-4 py-3 group"
				:ui="{ trailingIcon: 'ms-auto group-data-[state=open]:rotate-180 transition-transform' }">
				Panduan untuk operator kecamatan: menyiapkan daftar siswa di Google Sheets
			</UButton>

			<template #content>
				<div class="space-y-4 border-t border-default px-4 py-4">
					<p class="text-sm text-muted">
						Dipakai di kecamatan lain? Operator kecamatan cukup menyiapkan satu Google Sheet berisi daftar siswa semua sekolah,
						lalu membagikan link-nya. Tiap sekolah menempelkan link itu dan memilih nama sekolahnya sendiri.
					</p>

					<ol class="space-y-3">
						<li v-for="(step, i) in guideSteps" :key="step.title" class="flex gap-3">
							<span class="flex size-6 shrink-0 items-center justify-center rounded-full bg-primary/10 text-xs font-semibold text-primary">
								{{ i + 1 }}
							</span>
							<div class="text-sm">
								<p class="font-medium">{{ step.title }}</p>
								<p class="text-muted">{{ step.body }}</p>
							</div>
						</li>
					</ol>

					<div class="overflow-x-auto rounded-md border border-default">
						<table class="w-full text-xs">
							<thead class="bg-elevated text-left">
								<tr>
									<th class="p-2">NISN</th>
									<th class="p-2">NAMA SISWA</th>
									<th class="p-2">KELAS</th>
									<th class="p-2">NAMA SEKOLAH</th>
								</tr>
							</thead>
							<tbody class="divide-y divide-default text-muted">
								<tr>
									<td class="p-2">0123456789</td>
									<td class="p-2">CONTOH NAMA SISWA SATU</td>
									<td class="p-2">4</td>
									<td class="p-2">SD NEGERI CONTOH 1</td>
								</tr>
								<tr>
									<td class="p-2">0123456790</td>
									<td class="p-2">CONTOH NAMA SISWA DUA</td>
									<td class="p-2">5</td>
									<td class="p-2">SD NEGERI CONTOH 1</td>
								</tr>
							</tbody>
						</table>
					</div>

					<div class="flex flex-wrap gap-2">
						<UButton
							icon="lucide:download"
							variant="soft"
							@click="downloadTemplate('/templates/tarik-siswa-template.csv', 'template-daftar-siswa-kecamatan.csv', 'Template daftar siswa')">
							Unduh Template (CSV)
						</UButton>
						<UButton
							icon="lucide:external-link"
							variant="outline"
							color="neutral"
							@click="openUrl('https://sheets.new')">
							Buat Google Sheet Baru
						</UButton>
					</div>
				</div>
			</template>
		</UCollapsible>

		<div class="grid gap-4 md:grid-cols-3 items-end">
			<UFormField label="Nama Sekolah" description="Ketik untuk mencari, lalu pilih nama sekolah yang persis dari daftar." class="md:col-span-2">
				<UInputMenu
					v-model="school"
					:items="sourceSchools"
					:loading="loadingSchools"
					:disabled="!canFetch"
					open-on-click
					:placeholder="activeSource ? 'Contoh: SD NEGERI CIPINANG I' : 'Atur link Google Sheet dulu'"
					class="w-full" />
			</UFormField>

			<UButton
				icon="lucide:cloud-download"
				:loading="loading"
				:disabled="!canFetch || loadingSchools || !school"
				@click="tarik">
				Tarik Data Siswa
			</UButton>
		</div>

		<UAlert
			v-if="schoolsError"
			color="warning"
			variant="subtle"
			title="Gagal membaca daftar sekolah dari sheet"
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
