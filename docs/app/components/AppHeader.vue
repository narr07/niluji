<script setup lang="ts">
	import type { ContentNavigationItem } from "@nuxt/content";

	const navigation = inject<Ref<ContentNavigationItem[]>>("navigation");
	const { header } = useAppConfig();
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

			<template v-if="header?.links">
				<UButton
					v-for="(link, index) of header.links"
					:key="index"
					v-bind="{ color: 'neutral', variant: 'ghost', ...link }"
				/>
			</template>

			<UColorModeButton v-if="header?.colorMode" />
		</template>

		<template #body>
			<UContentNavigation
				highlight
				:navigation="navigation"
			/>
		</template>
	</UHeader>
</template>
