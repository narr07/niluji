<script setup lang="ts">
const backend = useSiswaBackend();
const toast = useToast();

const STORAGE_KEY = "niluji-siswa-server";
const readSaved = () => {
	try {
		return window.localStorage.getItem(STORAGE_KEY) || "";
	} catch {
		return "";
	}
};
const saveAddress = (value: string) => {
	try {
		window.localStorage.setItem(STORAGE_KEY, value);
	} catch {
		// Penyimpanan lokal hanya kemudahan, aplikasi tetap jalan tanpanya.
	}
};

const address = ref("");
const savedAddress = ref("");
const connecting = ref(false);
const scanning = ref(false);
const errorMessage = ref("");
const cameraDenied = ref(false);
const info = ref<AppInfo | null>(null);

onMounted(async () => {
	savedAddress.value = readSaved();
	address.value = savedAddress.value;
	info.value = await backend.appInfo().catch(() => null);
});

const toMessage = (error: unknown) => (typeof error === "string" ? error : error instanceof Error ? error.message : "Terjadi kesalahan. Coba lagi.");

const connect = async (value = address.value) => {
	errorMessage.value = "";
	cameraDenied.value = false;
	connecting.value = true;
	try {
		const server = await backend.checkServer(value);
		// Yang disimpan hanya alamat server, bukan token ujian dari QR.
		savedAddress.value = server.baseUrl.replace(/^http:\/\//, "");
		saveAddress(savedAddress.value);
		await backend.openExam(value);
	} catch (error) {
		errorMessage.value = toMessage(error);
	} finally {
		connecting.value = false;
	}
};

const scan = async () => {
	errorMessage.value = "";
	cameraDenied.value = false;
	scanning.value = true;
	try {
		const result = await backend.scanQr();
		if (result.status === "denied") {
			cameraDenied.value = true;
			return;
		}
		if (result.status === "cancelled") return;
		address.value = result.content;
		toast.add({ title: "QR terbaca", description: "Menghubungkan ke server ujian...", icon: "i-lucide-scan-qr-code", color: "success" });
		await connect(result.content);
	} catch (error) {
		errorMessage.value = toMessage(error);
	} finally {
		scanning.value = false;
	}
};

const busy = computed(() => connecting.value || scanning.value);
</script>

<template>
	<main class="safe-area min-h-dvh bg-default flex flex-col">
		<div class="flex-1 w-full max-w-md mx-auto px-4 flex flex-col gap-5">
			<header class="flex items-center gap-3 pt-2">
				<div class="size-11 rounded-xl bg-primary text-inverted flex items-center justify-center shrink-0">
					<UIcon
						name="i-lucide-graduation-cap"
						class="size-6"
					/>
				</div>
				<div class="min-w-0">
					<h1 class="text-lg font-bold text-highlighted leading-tight">
						NILUJI Siswa
					</h1>
					<p class="text-sm text-muted">
						Masuk ke ujian di ruang kelas
					</p>
				</div>
			</header>

			<UCard>
				<div class="space-y-4">
					<div>
						<h2 class="font-semibold text-highlighted">
							Sambungkan ke server ujian
						</h2>
						<p class="text-sm text-muted mt-1">
							Pindai QR pada kartu peserta, atau ketik alamat yang ditampilkan pengawas.
						</p>
					</div>

					<UButton
						v-if="info?.mobile"
						block
						size="xl"
						icon="i-lucide-scan-qr-code"
						label="Pindai QR Kartu Peserta"
						:loading="scanning"
						:disabled="busy && !scanning"
						@click="scan"
					/>

					<USeparator
						v-if="info?.mobile"
						label="atau"
					/>

					<form
						class="space-y-3"
						@submit.prevent="connect()"
					>
						<UFormField
							label="Alamat server"
							hint="Contoh: 192.168.1.10"
						>
							<UInput
								v-model="address"
								size="xl"
								class="w-full"
								icon="i-lucide-server"
								placeholder="192.168.1.10"
								inputmode="url"
								autocapitalize="off"
								autocomplete="off"
								:spellcheck="false"
								:disabled="busy"
							/>
						</UFormField>
						<UButton
							type="submit"
							block
							size="xl"
							:color="info?.mobile ? 'neutral' : 'primary'"
							:variant="info?.mobile ? 'outline' : 'solid'"
							icon="i-lucide-log-in"
							label="Masuk ke Ujian"
							:loading="connecting"
							:disabled="busy && !connecting"
						/>
					</form>

					<UAlert
						v-if="errorMessage"
						color="error"
						variant="subtle"
						icon="i-lucide-wifi-off"
						title="Belum tersambung"
						:description="errorMessage"
					/>

					<UAlert
						v-if="cameraDenied"
						color="warning"
						variant="subtle"
						icon="i-lucide-camera-off"
						title="Izin kamera ditolak"
						description="Izinkan kamera untuk memindai QR, atau ketik alamat server secara manual."
						:actions="[{ label: 'Buka Pengaturan', color: 'warning', variant: 'outline', onClick: () => backend.openCameraSettings() }]"
					/>
				</div>
			</UCard>

			<UButton
				v-if="savedAddress && savedAddress !== address"
				color="neutral"
				variant="soft"
				icon="i-lucide-history"
				:label="`Pakai alamat terakhir: ${savedAddress}`"
				class="justify-center"
				:disabled="busy"
				@click="address = savedAddress"
			/>

			<UCard variant="subtle">
				<div class="space-y-3 text-sm">
					<p class="font-semibold text-highlighted">
						Selama ujian
					</p>
					<ul class="space-y-2 text-muted">
						<li class="flex gap-2">
							<UIcon
								name="i-lucide-pin"
								class="size-4 mt-0.5 shrink-0 text-primary"
							/>
							<span>Android akan meminta izin menyematkan aplikasi. Setujui supaya aplikasi lain tidak bisa dibuka.</span>
						</li>
						<li class="flex gap-2">
							<UIcon
								name="i-lucide-shield"
								class="size-4 mt-0.5 shrink-0 text-primary"
							/>
							<span>Tangkapan layar dan rekam layar diblokir, dan layar tetap menyala.</span>
						</li>
						<li class="flex gap-2">
							<UIcon
								name="i-lucide-arrow-left"
								class="size-4 mt-0.5 shrink-0 text-primary"
							/>
							<span>Tombol kembali meminta konfirmasi sebelum keluar. Jawaban yang sudah tersimpan tetap aman di server.</span>
						</li>
					</ul>
				</div>
			</UCard>

			<p class="text-xs text-dimmed text-center mt-auto pb-2">
				NILUJI Siswa{{ info ? ` v${info.version}` : "" }} • Hanya membuka server ujian yang dipilih
			</p>
		</div>
	</main>
</template>
