<script lang="ts" setup>
const props = defineProps<{
	joinUrl: string;
}>();

const isOpen = ref(false);
const qrCanvasModal = ref<HTMLCanvasElement | null>(null);
const qrWrapper = ref<HTMLDivElement | null>(null);
let resizeObserver: ResizeObserver | null = null;

async function renderQr(size: number) {
	await nextTick();
	if (!props.joinUrl || !qrCanvasModal.value || size <= 0) return;
	const QRCode = await import("qrcode");
	await QRCode.toCanvas(qrCanvasModal.value, props.joinUrl, {
		width: Math.floor(size),
		margin: 2,
	});
}

function observeWrapper() {
	if (!qrWrapper.value) return;
	resizeObserver = new ResizeObserver((entries) => {
		for (const entry of entries) {
			const { width, height } = entry.contentRect;
			const size = Math.max(Math.min(width, height) - 8, 120);
			renderQr(size);
		}
	});
	resizeObserver.observe(qrWrapper.value);
}

watch(isOpen, async (open) => {
	if (open) {
		await nextTick();
		observeWrapper();
	} else {
		resizeObserver?.disconnect();
		resizeObserver = null;
	}
});

watch(
	() => props.joinUrl,
	() => {
		if (isOpen.value && qrWrapper.value) {
			const { width, height } = qrWrapper.value.getBoundingClientRect();
			renderQr(Math.max(Math.min(width, height) - 8, 120));
		}
	}
);

onUnmounted(() => resizeObserver?.disconnect());

const { copy, copied } = useClipboard();

function handleCopyJoinUrl() {
	if (!props.joinUrl) return;
	copy(props.joinUrl);
}
</script>

<template>


	<UModal
		v-model:open="isOpen"
		title="Scan QR untuk Akses Ujian"
		description="Siswa scan QR ini pakai kamera HP (harus 1 WiFi/hotspot yang sama)"
		:ui="{
			content: 'max-w-lg h-[min(90vh,600px)]',
			body: 'flex-1 min-h-0 overflow-hidden flex',
		}"
	>
		<UButton icon="i-lucide-qr-code"    />

		<template #footer>
			<UButton
				:color="copied ? 'success' : 'neutral'"
				variant="ghost"
				:icon="copied ? 'i-lucide-check' : 'i-lucide-copy'"
				class="w-full justify-center text-lg font-mono font-semibold break-all"
				@click="handleCopyJoinUrl"
			>
				{{ joinUrl || "-" }}
			</UButton>
		</template>

		<template #body>
			<div ref="qrWrapper" class="flex-1 min-h-0 w-full flex items-center justify-center">
				<canvas ref="qrCanvasModal" class="rounded-md border border-default" />
			</div>
		</template>
	</UModal>


</template>