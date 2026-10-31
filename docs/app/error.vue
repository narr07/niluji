<script setup lang="ts">
	import type { NuxtError } from "#app";

	defineProps<{
		error: NuxtError;
	}>();

	useHead({
		htmlAttrs: {
			lang: "id"
		}
	});

	useSeoMeta({
		title: "Halaman Tidak Ditemukan",
		description: "Halaman dokumentasi yang Anda cari tidak ditemukan."
	});

	const { data: navigation } = await useAsyncData("navigation", () => queryCollectionNavigation("docs"));
	const { data: files } = useLazyAsyncData("search", () => queryCollectionSearchSections("docs"), {
		server: false
	});

	provide("navigation", navigation);
</script>

<template>
	<UApp>
		<AppHeader />

		<UError :error="error" />

		<AppFooter />

		<ClientOnly>
			<LazyUContentSearch
				:files="files"
				:navigation="navigation"
			/>
		</ClientOnly>
	</UApp>
</template>
