<script setup lang="ts">
	import type { ContentNavigationItem } from "@nuxt/content";

	const navigation = inject<Ref<ContentNavigationItem[]>>("navigation");
</script>

<template>
	<UContainer>
		<UPage>
			<template #left>
				<UPageAside>
					<UContentNavigation
						highlight
						:navigation="navigation"
						:ui="{
							link: 'items-start gap-2 py-1.5',
							linkLeadingIcon: 'shrink-0 mt-0.5 size-4',
							linkTitle: 'whitespace-normal break-words leading-snug text-left'
						}"
					/>
				</UPageAside>
			</template>

			<slot />
		</UPage>
	</UContainer>
</template>
