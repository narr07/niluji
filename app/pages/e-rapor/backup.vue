<script setup lang="ts">
	import { invoke } from "@tauri-apps/api/core";

	// Padanan e-Rapor: Backup & Restore. Restore aman: file diperiksa dulu dan kondisi sekarang
	// otomatis dibackup ("sebelum-restore-…"), jadi restore yang keliru bisa dibatalkan.

	interface Berkas { nama: string, ukuran: number, waktu: number }

	const swal = useSwal();
	const toast = useToast();
	const { logout } = useAuth();
	const running = ref(false);
	const daftar = ref<Berkas[]>([]);
	const memulihkan = ref<string | null>(null);
	const fileInput = ref<HTMLInputElement>();

	async function loadDaftar() {
		try {
			daftar.value = await invoke<Berkas[]>("db_backup_list");
		}
		catch (e) {
			toast.add({ title: "Gagal membaca folder backup", description: pesanError(e), color: "error" });
		}
	}
	onMounted(loadDaftar);

	async function backup() {
		running.value = true;
		try {
			const path = await invoke<string>("db_backup");
			await loadDaftar();
			swal.success("Backup selesai", `File tersimpan di:\n${path}`);
		}
		catch (e) {
			swal.error("Backup gagal", pesanError(e));
		}
		finally {
			running.value = false;
		}
	}

	function openFolder() {
		invoke("open_data_dir", { sub: "backups" }).catch(e => swal.error("Gagal membuka folder", pesanError(e)));
	}

	const waktu = (detik: number) => new Date(detik * 1000).toLocaleString("id-ID", { dateStyle: "medium", timeStyle: "short" });
	const ukuran = (b: number) => (b > 1048576 ? `${(b / 1048576).toFixed(1)} MB` : `${Math.max(1, Math.round(b / 1024))} KB`);

	async function restore(sumber: { nama?: string, bytes?: number[] }, label: string) {
		const ok = await swal.confirm(
			"Pulihkan data dari backup?",
			`Semua data e-Rapor sekarang diganti dengan isi "${label}". Kondisi sekarang otomatis dibackup dulu, jadi masih bisa dikembalikan. Setelah selesai Anda perlu login ulang.`,
			"Pulihkan"
		);
		if (!ok)
			return;
		memulihkan.value = label;
		try {
			const aman = await invoke<string>("db_restore", { nama: sumber.nama ?? null, bytes: sumber.bytes ?? null });
			await swal.success("Data berhasil dipulihkan", `Data sebelum restore tersimpan sebagai "${aman}". Silakan login ulang.`);
			await logout();
		}
		catch (e) {
			swal.error("Restore gagal", `${pesanError(e)}\n\nData sekarang tidak berubah.`);
		}
		finally {
			memulihkan.value = null;
		}
	}

	async function pilihFile(e: Event) {
		const el = e.target as HTMLInputElement;
		const file = el.files?.[0];
		el.value = "";
		if (file)
			await restore({ bytes: Array.from(new Uint8Array(await file.arrayBuffer())) }, file.name);
	}
</script>

<template>
	<ErPage id="backup" title="Backup & Restore">
		<div class="grid gap-4 lg:grid-cols-2">
			<PanelCard title="Backup Data e-Rapor" icon="lucide:hard-drive-download">
				<div class="flex flex-col gap-3 text-sm">
					<p>Simpan salinan seluruh database e-Rapor (pengguna, data Dapodik, nilai, foto, dan pengaturan) ke satu file <code>.sqlite</code>. Lakukan sebelum ambil ulang data Dapodik, sebelum pindah komputer, dan di akhir setiap semester.</p>
					<div class="flex flex-wrap gap-2">
						<UButton
							variant="solid"
							icon="lucide:hard-drive-download"
							:loading="running"
							@click="backup">
							Backup Sekarang
						</UButton>
						<UButton
							color="neutral"
							variant="outline"
							icon="lucide:folder-open"
							@click="openFolder">
							Buka Folder Backup
						</UButton>
					</div>
				</div>
			</PanelCard>

			<PanelCard title="Restore Data e-Rapor" icon="lucide:history" description="Kondisi sekarang otomatis dibackup sebelum dipulihkan.">
				<div class="flex flex-col gap-3">
					<ul v-if="daftar.length" class="max-h-72 divide-y divide-default overflow-y-auto rounded-md border border-default">
						<li v-for="b in daftar" :key="b.nama" class="flex items-center gap-3 px-3 py-2 text-sm">
							<UIcon :name="b.nama.startsWith('sebelum-restore') ? 'lucide:undo-2' : 'lucide:database'" class="size-4 shrink-0 text-muted" />
							<div class="min-w-0 flex-1">
								<p class="truncate font-medium" :title="b.nama">
									{{ b.nama }}
								</p>
								<p class="text-xs text-muted">
									{{ waktu(b.waktu) }} · {{ ukuran(b.ukuran) }}
								</p>
							</div>
							<UButton
								size="xs"
								variant="soft"
								icon="lucide:history"
								:loading="memulihkan === b.nama"
								:disabled="!!memulihkan"
								:aria-label="`Pulihkan ${b.nama}`"
								@click="restore({ nama: b.nama }, b.nama)"
							>
								Pulihkan
							</UButton>
						</li>
					</ul>
					<p v-else class="text-sm text-muted">
						Belum ada file backup di folder backup.
					</p>
					<input
						ref="fileInput"
						type="file"
						accept=".sqlite,.db"
						class="hidden"
						aria-label="Pilih file backup"
						@change="pilihFile">
					<UButton
						color="neutral"
						variant="outline"
						icon="lucide:file-up"
						:disabled="!!memulihkan"
						class="self-start"
						@click="fileInput?.click()">
						Pulihkan dari file lain…
					</UButton>
				</div>
			</PanelCard>
		</div>
	</ErPage>
</template>
