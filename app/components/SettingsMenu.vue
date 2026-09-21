<script lang="ts" setup>
	import type { DropdownMenuItem } from "@nuxt/ui";

	defineProps<{
		collapsed?: boolean
	}>();

	// Nuxt UI cuma generate CSS variable (--ui-color-{nama}-500) untuk warna yang lagi aktif
	// dipakai sebagai primary/neutral — jadi swatch chip di sini butuh nilai warna sendiri
	// (bukan var CSS) supaya semua pilihan tetap kelihatan warnanya, bukan cuma yang aktif.
	const PRIMARY_COLORS: { name: string, hex: string }[] = [
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

	const NEUTRAL_COLORS: { name: string, hex: string }[] = [
		{ name: "slate", hex: "#64748b" },
		{ name: "gray", hex: "#6b7280" },
		{ name: "zinc", hex: "#71717a" },
		{ name: "neutral", hex: "#737373" },
		{ name: "stone", hex: "#78716c" },
		{ name: "taupe", hex: "oklch(54.7% 0.021 43.1)" },
		{ name: "mauve", hex: "oklch(54.2% 0.034 322.5)" },
		{ name: "mist", hex: "oklch(56% 0.021 213.5)" },
		{ name: "olive", hex: "oklch(58% 0.031 107.3)" }
	];

	const PRIMARY_STORAGE_KEY = "niluji-theme-primary";
	const NEUTRAL_STORAGE_KEY = "niluji-theme-neutral";

	const appConfig = useAppConfig();
	const colorMode = useColorMode();

	const setColor = (kind: "primary" | "neutral", color: string, storageKey: string) => {
		appConfig.ui.colors[kind] = color;
		try {
			localStorage.setItem(storageKey, color);
		} catch {
			// Abaikan kalau localStorage tidak bisa diakses — pilihan warna cuma tidak
			// tersimpan untuk sesi berikutnya, bukan hal fatal.
		}
	};

	onMounted(() => {
		try {
			const savedPrimary = localStorage.getItem(PRIMARY_STORAGE_KEY);
			if (savedPrimary && PRIMARY_COLORS.some((c) => c.name === savedPrimary)) {
				appConfig.ui.colors.primary = savedPrimary;
			}
			const savedNeutral = localStorage.getItem(NEUTRAL_STORAGE_KEY);
			if (savedNeutral && NEUTRAL_COLORS.some((c) => c.name === savedNeutral)) {
				appConfig.ui.colors.neutral = savedNeutral;
			}
		} catch {
			// Sama seperti di atas — biarkan pakai warna default kalau gagal dibaca.
		}
	});

	const hexOf = (list: { name: string, hex: string }[], name: string) =>
		list.find((c) => c.name === name)?.hex ?? "#737373";

	const items = computed<DropdownMenuItem[][]>(() => [
		[
			{
				label: "Setting Data",
				icon: "lucide:settings",
				children: [
					{ label: "Data Sekolah", icon: "lucide:school", to: "/pengaturan", exact: true },
					{ label: "Data Kelas", icon: "lucide:layers", to: "/pengaturan/kelas" },
					{ label: "Data Pelajaran", icon: "lucide:book-open", to: "/pengaturan/pelajaran" },
					{ label: "Tarik Data Siswa", icon: "lucide:users", to: "/pengaturan/tarik-data-siswa" },
					{ label: "Tarik Soal Online", icon: "lucide:cloud-download", to: "/pengaturan/tarik-soal-online" }
				]
			},
			{ label: "Panduan", icon: "lucide:book-open-check", to: "/panduan" },
			{
				label: "Tentang",
				icon: "lucide:info",
				children: [
					{ label: "About", icon: "lucide:info", to: "/tentang", exact: true },
					{ label: "Fitur", icon: "lucide:layout-grid", to: "/tentang/fitur" },
					{ label: "Changelog", icon: "lucide:history", to: "/tentang/changelog" }
				]
			}
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
						children: PRIMARY_COLORS.map((color) => ({
							label: color.name,
							hex: color.hex,
							slot: "chip",
							type: "checkbox",
							checked: appConfig.ui.colors.primary === color.name,
							onSelect: (e: Event) => {
								e.preventDefault();
								setColor("primary", color.name, PRIMARY_STORAGE_KEY);
							}
						}))
					},
					{
						label: "Netral",
						slot: "chip",
						hex: hexOf(NEUTRAL_COLORS, appConfig.ui.colors.neutral),
						content: { align: "end", collisionPadding: 16 },
						children: NEUTRAL_COLORS.map((color) => ({
							label: color.name,
							hex: color.hex,
							slot: "chip",
							type: "checkbox",
							checked: appConfig.ui.colors.neutral === color.name,
							onSelect: (e: Event) => {
								e.preventDefault();
								setColor("neutral", color.name, NEUTRAL_STORAGE_KEY);
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
			}
		]
	]);
</script>

<template>
	<UDropdownMenu
		:items="items"
		:content="{ align: 'center', collisionPadding: 12 }"
		:ui="{ content: collapsed ? 'w-48' : 'w-(--reka-dropdown-menu-trigger-width)' }">
		<UButton
			icon="lucide:settings"
			color="neutral"
			variant="ghost"
			block
			:square="collapsed"
			:label="collapsed ? undefined : 'Pengaturan'"
			:trailing-icon="collapsed ? undefined : 'lucide:chevrons-up-down'"
			class="data-[state=open]:bg-elevated"
			:ui="{ trailingIcon: 'text-dimmed' }" />

		<template #chip-leading="{ item }">
			<div class="inline-flex items-center justify-center shrink-0 size-5">
				<span
					class="rounded-full ring ring-bg size-2"
					:style="{ backgroundColor: (item as any).hex }" />
			</div>
		</template>
	</UDropdownMenu>
</template>
