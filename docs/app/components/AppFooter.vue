<script setup lang="ts">
	const { footer } = useAppConfig();
</script>

<template>
	<UFooter>
		<template #left>
			<div class="flex flex-wrap items-center gap-x-2 gap-y-1 text-sm text-muted">
				<span>{{ footer.credits }}</span>
				<span>•</span>
				<span>
					Dikembangkan oleh
					<a
						href="https://permadi.dev"
						target="_blank"
						rel="noopener"
						class="font-medium text-highlighted underline decoration-emerald-500/50 underline-offset-4 hover:text-primary"
					>
						Dinar Permadi (permadi.dev)
					</a>
				</span>
			</div>
		</template>

		<template #right>
			<UColorModeButton v-if="footer?.colorMode" />

			<div
				v-if="footer?.links"
				class="flex flex-wrap items-center justify-center lg:justify-end gap-1 max-w-full min-w-0"
			>
				<UButton
					v-for="(link, index) of footer?.links"
					:key="index"
					v-bind="{ color: 'neutral', variant: 'ghost', ...link }"
				/>
			</div>
		</template>
	</UFooter>
</template>
