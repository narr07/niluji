<script lang="ts" setup>
	import type { DropdownMenuItem } from "@nuxt/ui";

	defineProps<{ collapsed?: boolean }>();

	const { user, session, headerSemester, logout } = useAuth();
	const appConfig = useAppConfig();
	const colorMode = useColorMode();
	const showInfo = ref(false);

	// Daftar warna & penyimpanan pilihan tema sama dengan SettingsMenu niluji.
	const PRIMARY_COLORS = [
		{ name: "red", hex: "#ef4444" },
		{ name: "orange", hex: "#f97316" },
		{ name: "amber", hex: "#f59e0b" },
		{ name: "yellow", hex: "#eab308" },
		{ name: "lime", hex: "#84cc16" },
		{ name: "green", hex: "#22c55e" },
		{ name: "emerald", hex: "#10b981" },
		{ name: "teal", hex: "#14b8a6" },
		{ name: "cyan", hex: "#06b6d4" },
		{ name: "sky", hex: "#0ea5e9" },
		{ name: "blue", hex: "#3b82f6" },
		{ name: "indigo", hex: "#6366f1" },
		{ name: "violet", hex: "#8b5cf6" },
		{ name: "purple", hex: "#a855f7" },
		{ name: "fuchsia", hex: "#d946ef" },
		{ name: "pink", hex: "#ec4899" },
		{ name: "rose", hex: "#f43f5e" }
	];
	const NEUTRAL_COLORS = [
		{ name: "slate", hex: "#64748b" },
		{ name: "gray", hex: "#6b7280" },
		{ name: "zinc", hex: "#71717a" },
		{ name: "neutral", hex: "#737373" },
		{ name: "stone", hex: "#78716c" }
	];
	const PRIMARY_KEY = "erapor-theme-primary";
	const NEUTRAL_KEY = "erapor-theme-neutral";

	function setColor(kind: "primary" | "neutral", color: string, key: string) {
		appConfig.ui.colors[kind] = color;
		try {
			localStorage.setItem(key, color);
		}
		catch {}
	}

	onMounted(() => {
		try {
			const p = localStorage.getItem(PRIMARY_KEY);
			if (p && PRIMARY_COLORS.some(c => c.name === p))
				appConfig.ui.colors.primary = p;
			const n = localStorage.getItem(NEUTRAL_KEY);
			if (n && NEUTRAL_COLORS.some(c => c.name === n))
				appConfig.ui.colors.neutral = n;
		}
		catch {}
	});

	const hexOf = (list: { name: string, hex: string }[], name: string) => list.find(c => c.name === name)?.hex ?? "#737373";

	const items = computed<DropdownMenuItem[][]>(() => [
		[{
			type: "label",
			label: user.value?.nama,
			description: `${session.value?.sekolahNama ?? "Belum ada sekolah"} · ${headerSemester.value}`
		}],
		[
			{ label: "Profil", icon: "lucide:user", to: "/e-rapor/profil" },
			...(settingsMenu(user.value?.level) as DropdownMenuItem[])
		],
		[
			{
				label: "Tema",
				icon: "lucide:palette",
				children: [
					{
						label: "Utama",
						slot: "chip",
						hex: hexOf(PRIMARY_COLORS, appConfig.ui.colors.primary),
						content: { align: "center", collisionPadding: 16 },
						children: PRIMARY_COLORS.map(color => ({
							label: color.name,
							hex: color.hex,
							slot: "chip",
							type: "checkbox",
							checked: appConfig.ui.colors.primary === color.name,
							onSelect: (e: Event) => {
								e.preventDefault();
								setColor("primary", color.name, PRIMARY_KEY);
							}
						}))
					},
					{
						label: "Netral",
						slot: "chip",
						hex: hexOf(NEUTRAL_COLORS, appConfig.ui.colors.neutral),
						content: { align: "end", collisionPadding: 16 },
						children: NEUTRAL_COLORS.map(color => ({
							label: color.name,
							hex: color.hex,
							slot: "chip",
							type: "checkbox",
							checked: appConfig.ui.colors.neutral === color.name,
							onSelect: (e: Event) => {
								e.preventDefault();
								setColor("neutral", color.name, NEUTRAL_KEY);
							}
						}))
					}
				]
			},
			{
				label: "Tampilan",
				icon: "lucide:sun-moon",
				children: [
					{
						label: "Terang",
						icon: "lucide:sun",
						type: "checkbox",
						checked: colorMode.value === "light",
						onSelect: (e: Event) => {
							e.preventDefault();
							colorMode.preference = "light";
						}
					},
					{
						label: "Gelap",
						icon: "lucide:moon",
						type: "checkbox",
						checked: colorMode.value === "dark",
						onSelect: (e: Event) => {
							e.preventDefault();
							colorMode.preference = "dark";
						}
					}
				]
			},
			{ label: "Info Aplikasi", icon: "lucide:info", onSelect: () => { showInfo.value = true; } }
		],
		[{ label: "Keluar", icon: "lucide:log-out", color: "error", onSelect: () => logout() }]
	]);
</script>

<template>
	<UDropdownMenu
		:items="items"
		:content="{ align: 'center', collisionPadding: 12 }"
		:ui="{ content: collapsed ? 'w-56' : 'w-(--reka-dropdown-menu-trigger-width)' }"
	>
		<UButton
			color="neutral"
			variant="ghost"
			block
			:square="collapsed"
			:label="collapsed ? undefined : user?.nama"
			:avatar="{ alt: user?.nama }"
			:trailing-icon="collapsed ? undefined : 'lucide:chevrons-up-down'"
			class="data-[state=open]:bg-elevated"
			:ui="{ trailingIcon: 'text-dimmed', label: 'truncate' }"
		/>

		<template #chip-leading="{ item }">
			<div class="inline-flex size-5 shrink-0 items-center justify-center">
				<span class="size-2 rounded-full ring ring-bg" :style="{ backgroundColor: (item as any).hex }" />
			</div>
		</template>
	</UDropdownMenu>

	<UModal v-model:open="showInfo" title="Info Aplikasi">
		<template #body>
			<div class="space-y-2 text-sm">
				<p><b>e-Rapor SD (nuxt-erapor)</b> versi 0.1.0</p>
				<p>Aplikasi rapor desktop berbasis Nuxt + Tauri. Tampilannya mengikuti niluji, dan alur datanya mengikuti e-Rapor SD 2025.1.</p>
				<p class="text-muted">
					Data master (sekolah, guru, siswa, rombel, pembelajaran) diambil dari Web Service Dapodik dan hanya bisa diperbaiki lewat Dapodik.
				</p>
			</div>
		</template>
	</UModal>
</template>
