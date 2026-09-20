<script lang="ts" setup>
	import { getName, getVersion } from "@tauri-apps/api/app";

	const appName = ref("Niluji");
	const appVersion = ref("");

	onMounted(async () => {
		try {
			[appName.value, appVersion.value] = await Promise.all([getName(), getVersion()]);
		} catch {
			// Tidak fatal — tampilkan nilai default kalau API Tauri tidak tersedia (mis. preview browser).
		}
	});
</script>

<template>
	<div class="space-y-6 max-w-2xl">
		<UCard>
			<div class="flex items-center gap-4">
				<img src="/logo.png" alt="Logo Niluji" class="size-14">
				<div>
					<h2 class="text-xl font-bold">
						{{ appName }}
					</h2>
					<p class="text-sm text-muted">
						Versi {{ appVersion || "..." }}
					</p>
				</div>
			</div>
			<p class="text-sm mt-4">
				Platform CBT (Computer-Based Test) dan e-Rapor offline untuk sekolah dasar.
				Guru mengelola bank soal, ujian, dan hasil dari satu aplikasi desktop; siswa
				mengerjakan ujian lewat browser di jaringan lokal yang sama — tanpa internet.
			</p>
		</UCard>

		<UCard>
			<template #header>
				<span class="font-semibold">Teknologi</span>
			</template>
			<ul class="text-sm space-y-1 text-muted">
				<li>Nuxt 4 + Nuxt UI 4 + Tailwind CSS 4</li>
				<li>Tauri 2 (Rust) + SQLite</li>
				<li>Server HTTP tertanam (axum) untuk ujian siswa di jaringan lokal</li>
			</ul>
		</UCard>

		<UCard>
			<template #header>
				<span class="font-semibold">Tentang Pembuat</span>
			</template>
			<p class="text-sm text-muted">
				Dibuat oleh <a
					href="https://github.com/narr07"
					target="_blank"
					rel="noopener"
					class="text-primary hover:underline">narr07</a>.
				Kode sumber tersedia di
				<a
					href="https://github.com/narr07/niluji"
					target="_blank"
					rel="noopener"
					class="text-primary hover:underline">github.com/narr07/niluji</a>.
			</p>
		</UCard>
	</div>
</template>
