<script setup lang="ts">
	import { invoke } from "@tauri-apps/api/core";

	// Admin membuka akses sementara supaya guru bisa tarik data / kirim nilai dari rumah.
	// Server mati lagi saat sesi ditutup (atau aplikasi ditutup) — link otomatis tidak berlaku.

	interface SesiInfo {
		aktif: boolean
		pin: string | null
		lanUrl: string | null
		publicUrl: string | null
		tunnelError: string | null
		dibuka: string | null
		log: { waktu: string, nama: string, aksi: string, ok: boolean, detail: string }[]
	}

	const swal = useSwal();
	const toast = useToast();
	const { session } = useAuth();
	const info = ref<SesiInfo>();
	const opening = ref<"online" | "lan" | null>(null);
	let timer: ReturnType<typeof setInterval> | undefined;

	async function load() {
		info.value = await invoke<SesiInfo>("sesi_status");
	}

	onMounted(() => {
		load();
		timer = setInterval(load, 3000);
	});
	onBeforeUnmount(() => clearInterval(timer));

	async function buka(online: boolean) {
		opening.value = online ? "online" : "lan";
		try {
			info.value = await invoke<SesiInfo>("sesi_buka", { online });
			if (online && info.value.tunnelError)
				swal.error("Akses online gagal", `${info.value.tunnelError}\n\nGuru yang ada di Wi-Fi sekolah tetap bisa memakai link jaringan lokal.`);
		}
		catch (e) {
			swal.error("Gagal membuka sesi", pesanError(e));
		}
		finally {
			opening.value = null;
		}
	}

	async function tutup() {
		if (!await swal.confirm("Tutup sesi?", "Link dan PIN langsung tidak berlaku. Guru yang sedang mengirim akan gagal.", "Tutup"))
			return;
		await invoke("sesi_tutup");
		await load();
	}

	const link = computed(() => info.value?.publicUrl ?? info.value?.lanUrl ?? "");
	const penyedia = computed(() => {
		const u = info.value?.publicUrl ?? "";
		return u.includes("trycloudflare") ? "Cloudflare" : u.includes(".lhr.") ? "localhost.run (cadangan)" : "";
	});

	const pesan = computed(() => [
		`*Sesi e-Rapor ${session.value?.sekolahNama ?? ""}*`,
		"Buka aplikasi e-Rapor di laptop → Sinkron ke Admin, lalu isi:",
		`Link: ${link.value}`,
		`PIN: ${info.value?.pin ?? ""}`,
		"Login pakai username & password guru masing-masing."
	].join("\n"));

	async function salin(text: string, label: string) {
		await navigator.clipboard.writeText(text);
		toast.add({ title: `${label} disalin`, color: "success", icon: "lucide:copy-check" });
	}
</script>

<template>
	<ErPage id="sesi-online" title="Sesi Online">
		<template #right>
			<UBadge
				v-if="info?.aktif"
				color="success"
				variant="subtle"
				class="flex items-center gap-1.5">
				<span class="inline-block size-1.5 animate-pulse rounded-full bg-success" />
				Aktif sejak {{ info.dibuka?.slice(11, 16) }}
			</UBadge>
			<UButton
				v-if="info?.aktif"
				color="error"
				variant="soft"
				icon="lucide:power"
				@click="tutup">
				Tutup Sesi
			</UButton>
		</template>

		<!-- Belum aktif -->
		<div v-if="!info?.aktif" class="mx-auto flex max-w-2xl flex-col gap-6 py-8">
			<div class="text-center">
				<UIcon name="lucide:globe" class="size-12 text-primary" />
				<h2 class="mt-2 text-xl font-semibold">
					Buka akses untuk laptop guru
				</h2>
				<p class="mt-1 text-muted">
					Guru menarik data kelas lalu mengisi nilai tanpa internet. Buka sesi hanya saat ada guru yang mau tarik data atau kirim nilai.
				</p>
			</div>
			<div class="grid gap-4 sm:grid-cols-2">
				<UPageCard
					title="Online (dari rumah)"
					icon="lucide:globe"
					description="Tanpa akun. Otomatis lewat Cloudflare, atau localhost.run kalau Cloudflare bermasalah. Link berganti setiap sesi."
					variant="subtle"
					:ui="{ leading: 'p-2.5 rounded-full bg-primary/10 ring ring-inset ring-primary/25' }"
				>
					<UButton
						variant="solid"
						icon="lucide:globe"
						:loading="opening === 'online'"
						:disabled="!!opening"
						@click="buka(true)">
						Buka Sesi Online
					</UButton>
				</UPageCard>
				<UPageCard
					title="Wi-Fi sekolah saja"
					icon="lucide:wifi"
					description="Tanpa internet. Hanya laptop yang tersambung ke jaringan yang sama dengan laptop admin."
					variant="subtle"
					:ui="{ leading: 'p-2.5 rounded-full bg-primary/10 ring ring-inset ring-primary/25' }"
				>
					<UButton
						variant="soft"
						icon="lucide:wifi"
						:loading="opening === 'lan'"
						:disabled="!!opening"
						@click="buka(false)">
						Buka Sesi Lokal
					</UButton>
				</UPageCard>
			</div>
			<p v-if="opening === 'online'" class="text-center text-sm text-muted">
				Menyiapkan link online… Dicoba otomatis: Cloudflare dulu, kalau gagal localhost.run. Pertama kali perlu mengunduh cloudflared (±55 MB).
			</p>
		</div>

		<!-- Aktif -->
		<div v-else class="space-y-6">
			<div class="grid gap-4 lg:grid-cols-[1fr_280px]">
				<PanelCard title="Kirim ke grup guru" icon="lucide:send">
					<div class="space-y-3">
						<div v-if="info.publicUrl">
							<p class="text-xs text-muted uppercase">
								Link online <span class="normal-case">· lewat {{ penyedia }}</span>
							</p>
							<div class="flex items-center gap-2">
								<code class="flex-1 truncate rounded bg-elevated px-2 py-1.5 text-sm">{{ info.publicUrl }}</code>
								<UButton icon="lucide:copy" variant="ghost" @click="salin(info.publicUrl, 'Link')" />
							</div>
							<p class="mt-1 text-xs text-muted">
								Kadang perlu ±30 detik sampai link bisa dibuka setelah sesi dibuka.
							</p>
						</div>
						<UAlert
							v-else-if="info.tunnelError"
							color="warning"
							variant="subtle"
							icon="lucide:globe-lock"
							title="Akses online tidak tersedia"
							:description="info.tunnelError"
						/>
						<div v-if="info.lanUrl">
							<p class="text-xs text-muted uppercase">
								Link Wi-Fi sekolah
							</p>
							<div class="flex items-center gap-2">
								<code class="flex-1 truncate rounded bg-elevated px-2 py-1.5 text-sm">{{ info.lanUrl }}</code>
								<UButton icon="lucide:copy" variant="ghost" @click="salin(info.lanUrl, 'Link')" />
							</div>
						</div>
						<UButton
							block
							variant="solid"
							icon="lucide:message-circle"
							@click="salin(pesan, 'Pesan WA')">
							Salin Pesan untuk WhatsApp
						</UButton>
					</div>
				</PanelCard>

				<PanelCard title="PIN Sesi" icon="lucide:hash">
					<div class="flex flex-col items-center gap-2 py-2">
						<p class="font-mono text-4xl font-bold tracking-[0.3em] text-highlighted">
							{{ info.pin }}
						</p>
						<UButton
							size="sm"
							variant="ghost"
							icon="lucide:copy"
							@click="salin(info.pin ?? '', 'PIN')">
							Salin
						</UButton>
						<p class="text-center text-xs text-muted">
							PIN baru setiap sesi. 30× salah PIN/password, sesi tertutup otomatis.
						</p>
					</div>
				</PanelCard>
			</div>

			<PanelCard title="Aktivitas" icon="lucide:activity" :description="`${info.log.length} kejadian di sesi ini`">
				<ul v-if="info.log.length" class="divide-y divide-default">
					<li v-for="(l, i) in info.log" :key="i" class="flex items-center gap-3 py-2 text-sm">
						<UIcon :name="l.ok ? 'lucide:circle-check' : 'lucide:circle-x'" class="size-4 shrink-0" :class="l.ok ? 'text-success' : 'text-error'" />
						<span class="w-14 shrink-0 text-xs text-muted">{{ l.waktu.slice(11, 16) }}</span>
						<span class="w-48 shrink-0 truncate font-medium">{{ l.nama }}</span>
						<span class="w-28 shrink-0">{{ l.aksi }}</span>
						<span class="flex-1 truncate text-muted">{{ l.detail }}</span>
					</li>
				</ul>
				<p v-else class="py-6 text-center text-sm text-muted">
					Belum ada guru yang terhubung.
				</p>
			</PanelCard>
		</div>
	</ErPage>
</template>
