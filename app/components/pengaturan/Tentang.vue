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

	const stacks = [
		{ label: "Nuxt 4", icon: "i-simple-icons-nuxtdotjs" },
		{ label: "Nuxt UI 4", icon: "i-simple-icons-nuxtdotjs" },
		{ label: "Tailwind CSS 4", icon: "i-simple-icons-tailwindcss" },
		{ label: "Tauri 2 (Rust)", icon: "i-simple-icons-tauri" },
		{ label: "SQLite", icon: "i-simple-icons-sqlite" },
		{ label: "axum (Rust)", icon: "i-lucide-server" },
	];
</script>

<template>
	<div class="space-y-6 w-full">
		<UCard variant="subtle">
			<div class="flex items-center gap-4">
				<img src="/logo.png" alt="Logo Niluji" class="size-16 shrink-0">
				<div class="min-w-0">
					<div class="flex items-center gap-2 flex-wrap">
						<h2 class="text-xl font-bold">
							{{ appName }}
						</h2>
						<UBadge color="primary" variant="subtle" size="sm">
							v{{ appVersion || "..." }}
						</UBadge>
					</div>
					<p class="text-sm text-muted mt-0.5">
						Platform CBT & e-Rapor offline untuk sekolah dasar
					</p>
				</div>
			</div>
			<p class="text-sm text-toned leading-relaxed mt-4">
				Guru mengelola bank soal, ujian, dan hasil dari satu aplikasi desktop; siswa
				mengerjakan ujian lewat browser di jaringan lokal yang sama — tanpa internet.
			</p>
		</UCard>

		<div class="grid gap-4 sm:grid-cols-2">
			<UCard>
				<template #header>
					<span class="font-semibold">Teknologi</span>
				</template>
				<div class="flex flex-wrap gap-2">
					<UBadge
						v-for="s in stacks"
						:key="s.label"
						:icon="s.icon"
						color="neutral"
						variant="subtle"
						size="md">
						{{ s.label }}
					</UBadge>
				</div>
				<p class="text-xs text-muted mt-3">
					Server HTTP tertanam untuk ujian siswa di jaringan lokal, tanpa internet.
				</p>
			</UCard>

			<UCard>
				<template #header>
					<span class="font-semibold">Tentang Pembuat</span>
				</template>
				<UUser
					name="narr07"
					description="Pembuat & pengelola Niluji"
					:avatar="{ src: 'https://github.com/narr07.png' }"
					to="https://github.com/narr07"
					target="_blank"
				/>
				<UButton
					to="https://github.com/narr07/niluji"
					target="_blank"
					icon="i-simple-icons-github"
					variant="soft"
					color="neutral"
					size="sm"
					class="mt-3">
					Kode Sumber
				</UButton>
			</UCard>
		</div>
	</div>
</template>