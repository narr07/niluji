<script setup lang="ts">
	import type { ContentNavigationItem } from "@nuxt/content";

	const navigation = inject<Ref<ContentNavigationItem[]>>("navigation");
	const { header } = useAppConfig();

	// Tautan berlabel hanya muat di layar lebar. Di HP tautan itu pindah ke menu,
	// dan header hanya menyisakan tombol ikon supaya halaman tidak melebar.
	const labeledLinks = computed(() => (header?.links || []).filter(link => "label" in link && link.label));
	const iconLinks = computed(() => (header?.links || []).filter(link => !("label" in link) || !link.label));
</script>

<template>
	<UHeader
		:ui="{ center: 'flex-1' }"
		:to="header?.to || '/'"
	>
		<UContentSearchButton
			v-if="header?.search"
			:collapsed="false"
			label="Cari panduan atau fitur..."
			class="w-full max-w-md"
		/>

		<template #left>
			<NuxtLink
				:to="header?.to || '/'"
				class="focus-visible:outline-3 outline-primary/25 rounded-md p-1 -ms-1 flex items-center gap-2"
			>
				<AppLogo class="w-auto h-7 shrink-0" />
			</NuxtLink>
		</template>

		<template #right>
			<UContentSearchButton
				v-if="header?.search"
				class="lg:hidden"
			/>

			<UButton
				v-for="(link, index) of labeledLinks"
				:key="`label-${index}`"
				class="hidden lg:inline-flex"
				v-bind="link"
			/>

			<UButton
				v-for="(link, index) of iconLinks"
				:key="`icon-${index}`"
				v-bind="link"
			/>

			<UColorModeButton v-if="header?.colorMode" />
		</template>

		<template #body>
			<div class="flex flex-col gap-1 mb-4 pb-4 border-b border-default">
				<UButton
					v-for="(link, index) of labeledLinks"
					:key="`menu-${index}`"
					class="justify-start"
					v-bind="link"
				/>
			</div>

			<UContentNavigation
				highlight
				:navigation="navigation"
			/>
		</template>
	</UHeader>
</template>
