<script setup lang="ts">
	const { pages } = usePages();
	const { app } = useAppConfig();
	const route = useRoute();

	// Halaman turunan (mis. /hasil/4/PAI) adalah route terpisah di Nuxt, bukan child dari /hasil,
	// jadi bawaan UNavigationMenu tidak menganggap menu "Hasil Ujian" aktif di sana. Aktif ditandai
	// manual berdasarkan awalan path supaya menu induknya tetap menyala di seluruh turunannya.
	const isUnder = (to: string) => route.path === to || (to !== "/" && route.path.startsWith(`${to}/`));
	const navItems = computed(() =>
		pages.map((item) =>
			item.children
				? { ...item, children: item.children.map((child: { to: string }) => ({ ...child, active: isUnder(child.to) })) }
				: { ...item, active: isUnder(item.to) }
		)
	);

	const open = ref(false);

	// Muat cache mapel sejak awal, supaya judul halaman per mapel langsung tampil dengan kodenya.
	useSubjects();

	// Cek pembaruan diam-diam beberapa detik setelah aplikasi dibuka (tidak menghambat start);
	// kalau offline/gagal, tidak ada pesan apa pun — guru bisa cek manual di Pengaturan → Tentang.
	const { checkForUpdate } = useAppUpdate();
	onMounted(() => setTimeout(() => void checkForUpdate({ silent: true }), 5000));
</script>

<template>
	<UDashboardGroup unit="rem">
		<UDashboardSidebar
			id="default"
			v-model:open="open"
			collapsible
			resizable
			class="bg-elevated/25"
		>
			<template #header="{ collapsed }">
				<div class="flex items-center gap-2 px-1 overflow-hidden">
					<SvgoLogo :font-controlled="false" class="size-6 shrink-0 text-primary" />
					<span v-if="!collapsed" class="font-bold truncate">{{ app.name }}</span>
				</div>
			</template>

			<template #default="{ collapsed }">
				<UNavigationMenu
					:collapsed="collapsed"
					:items="navItems"
					orientation="vertical"
					tooltip
					popover
				/>
			</template>

			<template #footer="{ collapsed }">
				<SettingsMenu :collapsed="collapsed" />
			</template>
		</UDashboardSidebar>

		<slot />
	</UDashboardGroup>
</template>
