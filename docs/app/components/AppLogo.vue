<script setup lang="ts">
	const { version } = useAppConfig();
</script>

<template>
	<div class="inline-flex items-center gap-2.5">
		<svg
			xmlns="http://www.w3.org/2000/svg"
			viewBox="0 0 256 256"
			class="size-7 shrink-0 text-highlighted"
			role="img"
			aria-labelledby="niluji-docs-logo-title">
			<title id="niluji-docs-logo-title">NILUJI</title>
			<path
				fill="currentColor"
				d="M 36 56 L 36 200 A 20 20 0 0 0 76 200 L 76 56 A 20 20 0 0 0 36 56 Z" />
			<path
				fill="currentColor"
				d="M 38.68 66 L 118.68 204.56 A 20 20 0 0 0 153.32 184.56 L 73.32 46 A 20 20 0 0 0 38.68 66 Z" />
			<path
				fill="currentColor"
				d="M 172 64 A 20 20 0 1 0 212 64 A 20 20 0 1 0 172 64 Z" />
			<path
				fill="#10B981"
				d="M 150.14 208.7 L 218.14 140.7 A 20 20 0 0 0 189.86 112.42 L 121.86 180.42 A 20 20 0 0 0 150.14 208.7 Z" />
		</svg>
		<span class="font-bold text-lg tracking-tight text-highlighted">NILUJI</span>
		<UBadge
			v-if="version"
			color="primary"
			variant="subtle"
			size="xs"
			class="font-mono">
			v{{ version }}
		</UBadge>
	</div>
</template>
