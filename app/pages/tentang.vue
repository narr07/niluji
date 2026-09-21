<script lang="ts" setup>
	definePageMeta({
		hideFromNav: true
	});

	const links = [[
		{ label: "About", icon: "lucide:info", to: "/tentang", exact: true },
		{ label: "Fitur", icon: "lucide:layout-grid", to: "/tentang/fitur" },
		{ label: "Changelog", icon: "lucide:history", to: "/tentang/changelog" }
	]];
</script>

<template>
	<UDashboardPanel id="tentang">
		<template #header>
			<UDashboardNavbar title="Tentang">
				<template #leading>
					<UDashboardSidebarCollapse />
				</template>
			</UDashboardNavbar>

			<UDashboardToolbar>
				<UNavigationMenu :items="links" highlight class="-mx-1 flex-1" />
			</UDashboardToolbar>
		</template>

		<template #body>
			<NuxtPage />
		</template>
	</UDashboardPanel>
</template>
