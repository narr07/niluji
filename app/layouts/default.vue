<script setup lang="ts">
	const { pages } = usePages();
	const { app } = useAppConfig();

	const open = ref(false);
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
					:items="pages"
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
