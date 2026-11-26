<script lang="ts" setup>
	import { invoke } from "@tauri-apps/api/core";

	interface FiturItem {
		judul: string
		menu: string
		icon: string
		ringkasan: string
		isi: string[]
	}

	const fitur: FiturItem[] = [
		{
			judul: "Import dari Naskah Soal Word",
			menu: "Bank Soal > (dalam Jenis Ujian) > Import dari Word",
			icon: "i-lucide-file-text",
			ringkasan: "Naskah soal Word (.docx) + kunci jawaban terpisah, langsung jadi soal tanpa diketik ulang.",
			isi: [
				"Pilih file naskah soal (.docx) dan file kunci jawaban (.docx), sistem otomatis mencocokkan soal dengan jawabannya berdasarkan nomor.",
				"Jumlah soal PG dan esai biasanya terdeteksi otomatis dari file kunci jawaban. Kalau deteksinya salah, angkanya bisa diedit manual sebelum diproses.",
				"Ada tabel preview sesudah diproses: soal yang polanya tidak terbaca jelas ditandai \"Cek manual\". Klik Edit di baris manapun untuk membetulkan, atau Tambah Baris Manual untuk soal di luar yang terdeteksi."
			]
		},
		{
			judul: "Import CSV / Excel",
			menu: "Bank Soal > (dalam Jenis Ujian) > Import CSV / Excel",
			icon: "i-lucide-table",
			ringkasan: "Soal dalam bentuk tabel (kolom soal, pilihan, kunci, skor) diunggah langsung ke jenis ujian yang dibuka.",
			isi: []
		},
		{
			judul: "Tarik Soal Online",
			menu: "Pengaturan > Tarik Soal Online",
			icon: "i-lucide-cloud-download",
			ringkasan: "Tarik bank soal gugus yang dipublikasikan pembuat soal, tanpa impor manual.",
			isi: [
				"Pilih gugus sekolah (diingat untuk pemakaian berikutnya), lalu Kelas, Mata Pelajaran, dan Jenis Ujian. Soal untuk \"semua gugus\" ikut tampil.",
				"Klik Ambil & Preview, cek soalnya, pilih mata pelajaran sekolah tempat soal disimpan, lalu Impor ke Bank Soal. Butuh internet hanya saat menarik."
			]
		},
		{
			judul: "Tarik Data Siswa",
			menu: "Pengaturan > Tarik Data Siswa",
			icon: "i-lucide-users",
			ringkasan: "Tarik data siswa dari spreadsheet online sekolah, tanpa input manual atau CSV.",
			isi: [
				"Ketik nama sekolah (bisa cari sebagian nama, lalu pilih yang persis dari daftar), klik Lihat Pratinjau. Belum ada data yang disimpan.",
				"Periksa status tiap siswa (Baru, Berubah, Sama, Tidak valid), hapus centang yang tidak ingin disimpan, lalu klik Simpan. Backup dibuat otomatis sebelum menyimpan.",
				"Ada kesalahan nama atau NISN di data sumbernya? Klik \"Buka & Edit Data Sumber\" untuk membetulkan langsung di spreadsheet, lalu tarik ulang."
			]
		}
	];

	const exportOpen = ref(false);
	const publikasiOpen = ref(false);
	// Jendela pembuat soal baru dimuat saat pertama dibuka, lalu tetap terpasang.
	const publikasiMounted = ref(false);
	watch(publikasiOpen, (open) => {
		if (open) publikasiMounted.value = true;
	});

	interface BackupInfo {
		fileName: string
		path: string
		sizeBytes: number
		createdAt: number
		kind: "harian" | "manual" | "sebelum-perubahan"
	}

	const toast = useToast();
	const notifyTemplateDownload = (name: string) => {
		toast.add({ title: "Template diunduh", description: `Contoh ${name} sedang diunduh.`, icon: "lucide:download", color: "success" });
	};

	const backingUp = ref(false);
	const backups = ref<BackupInfo[]>([]);
	const latestBackup = computed(() => backups.value[0]);

	const formatSize = (bytes: number) => `${(bytes / 1024).toFixed(0)} KB`;
	const formatDate = (unixSeconds: number) => new Date(unixSeconds * 1000).toLocaleString("id-ID");

	const kindLabel: Record<BackupInfo["kind"], string> = {
		"harian": "Harian",
		"manual": "Manual",
		"sebelum-perubahan": "Sebelum perubahan"
	};
	const selectedBackup = ref<string>();
	const restoring = ref(false);
	const backupItems = computed(() => backups.value.map((b) => ({
		label: `${kindLabel[b.kind]} · ${formatDate(b.createdAt)} · ${formatSize(b.sizeBytes)}`,
		value: b.fileName
	})));

	const restoreSelected = async () => {
		const fileName = selectedBackup.value;
		if (!fileName) return;
		const ok = await confirmDelete({
			title: "Pulihkan database dari backup ini?",
			description: "Semua data saat ini diganti dengan isi backup. Salinan kondisi sekarang dibuat otomatis lebih dulu, jadi langkah ini masih bisa dibatalkan.",
			confirmLabel: "Pulihkan"
		});
		if (!ok) return;
		restoring.value = true;
		try {
			await invoke("restore_backup", { fileName });
			toast.add({ title: "Database dipulihkan", icon: "lucide:check", color: "success" });
			reloadNuxtApp({ force: true });
		} catch (error) {
			toast.add({
				title: "Pemulihan gagal",
				description: error instanceof Error ? error.message : String(error),
				icon: "lucide:x",
				color: "error"
			});
		} finally {
			restoring.value = false;
		}
	};

	const loadBackups = async () => {
		backups.value = await invoke<BackupInfo[]>("list_backups");
	};

	const backupNow = async () => {
		backingUp.value = true;
		try {
			const info = await invoke<BackupInfo>("backup_now");
			toast.add({
				title: "Backup selesai",
				description: `Disimpan sebagai ${info.fileName}`,
				icon: "lucide:check",
				color: "success"
			});
			await loadBackups();
		} catch (error) {
			toast.add({
				title: "Backup gagal",
				description: error instanceof Error ? error.message : String(error),
				icon: "lucide:x",
				color: "error"
			});
		} finally {
			backingUp.value = false;
		}
	};

	onMounted(loadBackups);
</script>

<template>
	<div class="space-y-6 w-full">
		<div class="grid gap-4 sm:grid-cols-2">
			<UPageCard
				v-for="f in fitur"
				:key="f.judul"
				:title="f.judul"
				:description="f.ringkasan"
				:icon="f.icon">
				<template #header>
					<UBadge
						color="neutral"
						variant="subtle"
						size="sm"
						class="mb-1">
						{{ f.menu }}
					</UBadge>
				</template>

				<UCollapsible v-if="f.isi.length" class="mt-1">
					<UButton
						label="Lihat detail"
						color="neutral"
						variant="link"
						trailing-icon="i-lucide-chevron-down"
						size="xs"
						class="px-0" />

					<template #content>
						<div class="space-y-2 text-sm text-toned mt-2">
							<p v-for="(p, i) in f.isi" :key="i">
								{{ p }}
							</p>
						</div>
					</template>
				</UCollapsible>
			</UPageCard>

			<UPageCard
				title="Export Bank Soal"
				description="Keluarkan bank soal menjadi file (Markdown + folder gambar), untuk cadangan atau dibagikan sebagai file."
				icon="i-lucide-upload">
				<p class="text-xs text-muted mt-2">
					Atur PIN Export dulu di Pengaturan &gt; Data Sekolah. File hasil export memuat kunci jawaban,
					jadi jangan unggah soal yang masih akan diujikan ke repo publik.
					<a
						href="/templates/contoh-soal-markdown.md"
						download
						class="text-primary underline"
						@click="notifyTemplateDownload('format Markdown')">Unduh contoh format</a>.
				</p>
				<UButton
					icon="lucide:download"
					variant="soft"
					size="sm"
					class="mt-3"
					@click="exportOpen = true">
					Export Bank Soal
				</UButton>
			</UPageCard>

			<UPageCard
				title="Publikasikan Bank Soal Gugus"
				description="Khusus pembuat soal: terbitkan paket soal per gugus supaya sekolah bisa menariknya lewat Tarik Soal Online."
				icon="i-lucide-upload-cloud">
				<p class="text-xs text-muted mt-2">
					Hanya bisa dipakai dengan token GitHub milik pembuat soal yang punya hak tulis ke repo bank soal.
				</p>
				<UButton
					icon="lucide:upload-cloud"
					variant="soft"
					size="sm"
					class="mt-3"
					@click="publikasiOpen = true">
					Buka
				</UButton>
			</UPageCard>

			<UPageCard
				title="Backup Database"
				description="Backup harian dibuat saat aplikasi dibuka dan diperbarui tiap jam, disimpan untuk 14 hari pemakaian. Gambar soal ikut dicadangkan."
				icon="i-lucide-database-backup">
				<p v-if="latestBackup" class="text-xs text-muted mt-2">
					Backup terakhir: {{ formatDate(latestBackup.createdAt) }} ({{ formatSize(latestBackup.sizeBytes) }}), {{ backups.length }} backup tersimpan
				</p>
				<UButton
					icon="lucide:database-backup"
					variant="soft"
					size="sm"
					class="mt-3"
					:loading="backingUp"
					@click="backupNow">
					Backup Sekarang
				</UButton>

				<div v-if="backupItems.length" class="mt-4 space-y-2 border-t border-default pt-3">
					<UFormField label="Pulihkan dari backup">
						<USelectMenu
							v-model="selectedBackup"
							:items="backupItems"
							value-key="value"
							placeholder="Pilih backup"
							class="w-full" />
					</UFormField>
					<UButton
						icon="lucide:history"
						variant="soft"
						color="warning"
						size="sm"
						:loading="restoring"
						:disabled="!selectedBackup"
						@click="restoreSelected">
						Pulihkan
					</UButton>
				</div>
			</UPageCard>
		</div>
	</div>

	<PengaturanExportSoalModal v-model:open="exportOpen" />
	<LazyPengaturanPublikasiSoalModal v-if="publikasiMounted" v-model:open="publikasiOpen" />
</template>
