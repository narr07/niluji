<script lang="ts" setup>
	import { invoke } from "@tauri-apps/api/core";

	interface FiturItem {
		judul: string
		menu: string
		isi: string[]
	}

	const fitur: FiturItem[] = [
		{
			judul: "Import dari Naskah Soal Word",
			menu: "Bank Soal > (dalam Jenis Ujian) > Import dari Word",
			isi: [
				"Punya naskah soal yang sudah diketik di Word (format siap cetak) plus file kunci jawaban terpisah? Nggak perlu diketik ulang. Pilih file naskah soal (.docx) dan file kunci jawaban (.docx) — sistem otomatis mencocokkan soal dengan jawabannya berdasarkan nomor.",
				"Jumlah soal PG dan esai biasanya terdeteksi otomatis dari file kunci jawaban. Kalau deteksinya salah atau soalnya kurang dari yang seharusnya, angkanya bisa diedit manual sebelum diproses.",
				"Sesudah diproses, ada tabel preview — soal yang polanya nggak terbaca jelas ditandai \"Cek manual\" dan nggak ikut kecentang otomatis. Klik Edit di baris manapun (termasuk yang \"Cek manual\") buat membetulkan isinya langsung, atau klik Tambah Baris Manual kalau mau menambah soal di luar yang terdeteksi dari file (misalnya file cuma 20 soal tapi maunya 30)."
			]
		},
		{
			judul: "Import CSV / Excel",
			menu: "Bank Soal > (dalam Jenis Ujian) > Import CSV / Excel",
			isi: [
				"Cocok kalau soal sudah disiapkan dalam bentuk tabel (kolom soal, pilihan A-D, kunci jawaban, skor). Unggah filenya, soal langsung masuk ke jenis ujian yang sedang dibuka."
			]
		},
		{
			judul: "Tarik Soal Online",
			menu: "Pengaturan > Tarik Soal Online",
			isi: [
				"Kalau sekolah sudah punya bank soal yang diunggah ke repo GitHub NILUJI (format Markdown hasil Export, lihat poin Export Bank Soal di bawah), soal itu bisa ditarik langsung tanpa impor manual.",
				"Pilih Kelas, Mata Pelajaran, lalu Jenis Ujian (daftarnya diambil otomatis dari file yang ada di repo). Klik Ambil & Preview, cek soalnya, lalu Impor ke Bank Soal."
			]
		},
		{
			judul: "Tarik Data Siswa",
			menu: "Pengaturan > Tarik Data Siswa",
			isi: [
				"Alternatif dari input manual/impor CSV, kalau data siswa sekolah sudah ada di spreadsheet online bersama. Ketik nama sekolah (bisa cari sebagian nama, lalu pilih yang persis dari daftar), klik Tarik Data Siswa.",
				"Ada kesalahan nama atau NISN di data sumbernya? Klik \"Buka & Edit Data Sumber\" di halaman itu buat langsung membetulkannya di spreadsheet, lalu tarik ulang di sini."
			]
		}
	];

	const exportOpen = ref(false);

	interface BackupInfo {
		fileName: string
		path: string
		sizeBytes: number
		createdAt: number
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
			<UCard v-for="f in fitur" :key="f.judul">
				<template #header>
					<p class="font-semibold">
						{{ f.judul }}
					</p>
					<UBadge
						color="neutral"
						variant="subtle"
						size="sm"
						class="mt-1">
						{{ f.menu }}
					</UBadge>
				</template>
				<div class="space-y-2 text-sm">
					<p v-for="(p, i) in f.isi" :key="i">
						{{ p }}
					</p>
				</div>
			</UCard>

			<UCard>
				<template #header>
					<span class="font-semibold">Export Bank Soal</span>
				</template>
				<p class="text-sm text-muted mb-3">
					Kebalikan dari Tarik Soal Online — fitur ini buat mengeluarkan bank soal jadi file (Markdown + folder gambar)
					yang nanti diunggah ke repo GitHub, supaya bisa ditarik sekolah lain lewat Tarik Soal Online. Dibatasi PIN
					supaya tidak sembarang orang bisa export (PIN default 1234, ganti sendiri lewat Pengaturan > Data Sekolah).
				</p>
				<p class="text-xs text-muted mb-3">
					Mau lihat/edit format Markdown-nya langsung?
					<a
						href="/templates/contoh-soal-markdown.md"
						download
						class="text-primary underline"
						@click="notifyTemplateDownload('format Markdown')">Unduh contoh</a>.
				</p>
				<UButton icon="lucide:download" variant="soft" @click="exportOpen = true">
					Export Bank Soal
				</UButton>
			</UCard>

			<UCard>
				<template #header>
					<span class="font-semibold">Backup Database</span>
				</template>
				<p class="text-sm text-muted mb-3">
					Database (soal, siswa, hasil ujian) otomatis dibackup tiap kali aplikasi dibuka — tersimpan di folder
					data aplikasi, 14 backup terakhir disimpan. Bisa juga backup manual kapan saja lewat tombol di bawah.
				</p>
				<p v-if="latestBackup" class="text-xs text-muted mb-3">
					Backup terakhir: {{ formatDate(latestBackup.createdAt) }} ({{ formatSize(latestBackup.sizeBytes) }}) — {{ backups.length }} backup tersimpan
				</p>
				<UButton
					icon="lucide:database-backup"
					variant="soft"
					:loading="backingUp"
					@click="backupNow">
					Backup Sekarang
				</UButton>
			</UCard>
		</div>
	</div>

	<PengaturanExportSoalModal v-model:open="exportOpen" />
</template>
