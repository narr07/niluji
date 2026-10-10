<script setup lang="ts">
	const { pages } = usePages();
	const { app } = useAppConfig();
	const route = useRoute();

	const isErapor = computed(() => route.path.startsWith("/e-rapor"));

	// CBT Nav
	const isUnder = (to: string) => route.path === to || (to !== "/" && route.path.startsWith(`${to}/`));
	const cbtNavItems = computed(() =>
		pages.map((item) =>
			item.children
				? { ...item, children: item.children.map((child: { to: string }) => ({ ...child, active: isUnder(child.to) })) }
				: { ...item, active: isUnder(item.to) }
		)
	);

	// e-Rapor Nav
	const { user, isWali } = useAuth();
	const { hasData, refresh: refreshAppData } = useAppData();
	const { isGuruMode, load: loadMode } = useAppMode();

	const cocok = (to?: string) => !!to && (route.path === to || (to !== "/e-rapor" && route.path.startsWith(`${to}/`)));

	const eraporNavItems = computed(() => {
		const level = user.value?.level ?? "admin";
		const items = level === "admin" ? adminMenu(hasData.value) : level === "guru" ? guruMenu(isWali.value, isGuruMode.value) : siswaMenu();
		const semua = items.flatMap(i => [i.to, ...(i.children ?? []).map(c => c.to)]).filter((t): t is string => typeof t === "string");
		const terbaik = semua.filter(cocok).sort((a, b) => b.length - a.length)[0];
		const isUnderErapor = (to?: string) => !!to && to === terbaik;
		return items.map(i => i.children
			? { ...i, defaultOpen: true, children: i.children.map(c => ({ ...c, active: isUnderErapor(c.to as string) })) }
			: { ...i, active: isUnderErapor(i.to as string) });
	});

	const navItems = computed(() => isErapor.value ? eraporNavItems.value : cbtNavItems.value);

	const open = ref(false);

	// Muat cache mapel CBT
	useSubjects();

	// Cek pembaruan diam-diam beberapa detik setelah aplikasi dibuka
	const { checkForUpdate } = useAppUpdate();

	onMounted(() => {
		if (isErapor.value) {
			Promise.all([refreshAppData(), loadMode()]);
		}
		setTimeout(() => void checkForUpdate({ silent: true }), 5000);
	});

	watch(() => route.path, (p) => {
		if (p.startsWith("/e-rapor")) {
			refreshAppData();
		}
	});
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
				<div v-if="!isErapor" class="flex items-center gap-2 px-1 overflow-hidden">
					<SvgoLogo :font-controlled="false" class="size-6 shrink-0 text-primary" />
					<span v-if="!collapsed" class="font-bold truncate">{{ app.name }}</span>
				</div>
				<div v-else class="flex flex-col gap-1 w-full px-1 overflow-hidden">
					<div class="flex items-center justify-between">
						<NuxtLink to="/e-rapor" class="flex items-center gap-2 overflow-hidden">
							<UIcon name="lucide:file-badge" class="size-6 shrink-0 text-primary" />
							<span v-if="!collapsed" class="truncate font-bold text-sm">e-Rapor SD</span>
						</NuxtLink>
						<UButton
							v-if="!collapsed"
							to="/"
							size="xs"
							color="neutral"
							variant="ghost"
							icon="lucide:arrow-left"
							title="Kembali ke Dashboard CBT"
							label="CBT"
						/>
					</div>
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
				<UserMenu v-if="isErapor" :collapsed="collapsed" />
				<SettingsMenu v-else :collapsed="collapsed" />
			</template>
		</UDashboardSidebar>

		<slot />
	</UDashboardGroup>
</template>
