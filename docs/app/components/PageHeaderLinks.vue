<script setup lang="ts">
	const route = useRoute();
	const toast = useToast();
	const copied = ref(false);

	const copyPageLink = async () => {
		if (typeof window === "undefined") return;
		const url = window.location.href;
		try {
			await navigator.clipboard.writeText(url);
		} catch {
			const el = document.createElement("textarea");
			el.value = url;
			el.style.position = "fixed";
			el.style.opacity = "0";
			document.body.appendChild(el);
			el.select();
			document.execCommand("copy");
			el.remove();
		}
		copied.value = true;
		toast.add({
			title: "Tautan halaman disalin",
			description: route.path,
			icon: "i-lucide-check-circle"
		});
		setTimeout(() => {
			copied.value = false;
		}, 1500);
	};
</script>

<template>
	<UButton
		:icon="copied ? 'i-lucide-check' : 'i-lucide-link'"
		color="neutral"
		variant="outline"
		size="sm"
		class="shrink-0 whitespace-nowrap"
		@click="copyPageLink"
	>
		<span>{{ copied ? "Tersalin" : "Salin Tautan" }}</span>
	</UButton>
</template>
