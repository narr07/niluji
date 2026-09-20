<script lang="ts" setup>
	const props = defineProps<{
		joinUrl: string
	}>();

	const qrCanvas = ref<HTMLCanvasElement | null>(null);

	watch(
		() => props.joinUrl,
		async (url) => {
			await nextTick();
			if (!url || !qrCanvas.value) return;
			const QRCode = await import("qrcode");
			await QRCode.toCanvas(qrCanvas.value, url, { width: 160, margin: 1 });
		},
		{ immediate: true }
	);
</script>

<template>
	<UCard>
		<template #header>
			<span class="font-semibold">Akses Ujian Siswa</span>
		</template>

		<div class="flex flex-col sm:flex-row items-center gap-6">
			<canvas ref="qrCanvas" class="rounded-md border border-default shrink-0" />
			<div class="space-y-2 text-center sm:text-left">
				<p class="text-sm text-muted">
					Siswa scan QR ini pakai kamera HP, atau buka alamat berikut di browser (harus 1 WiFi/hotspot yang sama):
				</p>
				<p class="text-xl font-mono font-semibold">
					{{ joinUrl || "-" }}
				</p>
			</div>
		</div>
	</UCard>
</template>
