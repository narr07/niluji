<script lang="ts" setup>
	definePageMeta({
		hideFromNav: true
	});

	const links = [[
		{ label: "Data Sekolah", icon: "lucide:school", to: "/pengaturan", exact: true },
		{ label: "Data Kelas", icon: "lucide:layers", to: "/pengaturan/kelas" },
		{ label: "Data Pelajaran", icon: "lucide:book-open", to: "/pengaturan/pelajaran" },
		{ label: "Tarik Soal Online", icon: "lucide:cloud-download", to: "/pengaturan/tarik-soal-online" }
	]];
</script>

<template>
	<UDashboardPanel id="pengaturan">
		<template #header>
			<UDashboardNavbar title="Pengaturan">
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
