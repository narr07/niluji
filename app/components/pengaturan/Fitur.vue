<script lang="ts" setup>
	import { invoke } from "@tauri-apps/api/core";

	interface FiturItem {
		judul: string;
		menu: string;
		icon: string;
		ringkasan: string;
		isi: string[];
	}

	const fitur: FiturItem[] = [
		{
			judul: "Import dari Naskah Soal Word",
			menu: "Bank Soal > (dalam Jenis Ujian) > Import dari Word",
			icon: "i-lucide-file-text",
			ringkasan: "Naskah soal Word (.docx) + kunci jawaban terpisah, langsung jadi soal tanpa diketik ulang.",
			isi: [
				"Pilih file naskah soal (.docx) dan file kunci jawaban (.docx) — sistem otomatis mencocokkan soal dengan jawabannya berdasarkan nomor.",
				"Jumlah soal PG dan esai biasanya terdeteksi otomatis dari file kunci jawaban. Kalau deteksinya salah, angkanya bisa diedit manual sebelum diproses.",
				"Ada tabel preview sesudah diproses — soal yang polanya tidak terbaca jelas ditandai \"Cek manual\". Klik Edit di baris manapun untuk membetulkan, atau Tambah Baris Manual untuk soal di luar yang terdeteksi.",
			],
		},
		{
			judul: "Import CSV / Excel",
			menu: "Bank Soal > (dalam Jenis Ujian) > Import CSV / Excel",
			icon: "i-lucide-table",
			ringkasan: "Soal dalam bentuk tabel (kolom soal, pilihan, kunci, skor) diunggah langsung ke jenis ujian yang dibuka.",
			isi: [],
		},
		{
			judul: "Tarik Soal Online",
			menu: "Pengaturan > Tarik Soal Online",
			icon: "i-lucide-cloud-download",
			ringkasan: "Tarik bank soal sekolah lain dari repo GitHub NILUJI tanpa impor manual.",
			isi: [
				"Kalau sekolah sudah punya bank soal yang diunggah ke repo GitHub NILUJI (format Markdown hasil Export), soal itu bisa ditarik langsung.",
				"Pilih Kelas, Mata Pelajaran, lalu Jenis Ujian — daftarnya diambil otomatis dari file di repo. Klik Ambil & Preview, cek soalnya, lalu Impor ke Bank Soal.",
			],
		},
		{
			judul: "Tarik Data Siswa",
			menu: "Pengaturan > Tarik Data Siswa",
			icon: "i-lucide-users",
			ringkasan: "Tarik data siswa dari spreadsheet online sekolah, tanpa input manual atau CSV.",
			isi: [
				"Ketik nama sekolah (bisa cari sebagian nama, lalu pilih yang persis dari daftar), klik Tarik Data Siswa.",
				"Ada kesalahan nama atau NISN di data sumbernya? Klik \"Buka & Edit Data Sumber\" untuk membetulkan langsung di spreadsheet, lalu tarik ulang.",
			],
		},
	];

	const exportOpen = ref(false);

	interface BackupInfo {
		fileName: string;
		path: string;
		sizeBytes: number;
		createdAt: number;
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
				color: "success",
			});
			await loadBackups();
		} catch (error) {
			toast.add({
				title: "Backup gagal",
				description: error instanceof Error ? error.message : String(error),
				icon: "lucide:x",
				color: "error",
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
				:icon="f.icon"
			>
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
				description="Kebalikan dari Tarik Soal Online — keluarkan bank soal jadi file (Markdown + folder gambar) untuk diunggah ke repo GitHub."
				icon="i-lucide-upload"
			>
				<p class="text-xs text-muted mt-2">
					Dibatasi PIN (default 1234, ganti lewat Pengaturan > Data Sekolah).
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
				title="Backup Database"
				description="Database (soal, siswa, hasil ujian) otomatis dibackup tiap kali aplikasi dibuka. 14 backup terakhir disimpan."
				icon="i-lucide-database-backup"
			>
				<p v-if="latestBackup" class="text-xs text-muted mt-2">
					Backup terakhir: {{ formatDate(latestBackup.createdAt) }} ({{ formatSize(latestBackup.sizeBytes) }}) — {{ backups.length }} backup tersimpan
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
			</UPageCard>
		</div>
	</div>

	<PengaturanExportSoalModal v-model:open="exportOpen" />
</template>