<script setup lang="ts">
	const { state, close } = useSwal();

	const ICON = {
		success: { name: "lucide:circle-check", cls: "text-success" },
		error: { name: "lucide:circle-x", cls: "text-error" },
		warning: { name: "lucide:circle-alert", cls: "text-warning" },
		info: { name: "lucide:info", cls: "text-info" }
	};

	const open = computed({
		get: () => state.value.open,
		set: (v) => { if (!v) close(false); }
	});
</script>

<template>
	<UModal v-model:open="open" :ui="{ content: 'max-w-sm max-h-[85vh]' }">
		<template #content>
			<div class="flex max-h-[85vh] flex-col items-center gap-3 p-6 text-center">
				<UIcon :name="ICON[state.type].name" class="size-16 shrink-0" :class="ICON[state.type].cls" />
				<p class="shrink-0 text-xl font-semibold">
					{{ state.title }}
				</p>
				<p v-if="state.text" class="w-full flex-1 overflow-y-auto text-start text-sm whitespace-pre-line text-muted">
					{{ state.text }}
				</p>
				<div class="mt-2 flex shrink-0 gap-2">
					<UButton
						v-if="state.cancelText"
						color="neutral"
						variant="outline"
						@click="close(false)">
						{{ state.cancelText }}
					</UButton>
					<UButton variant="solid" class="min-w-20 justify-center" @click="close(true)">
						{{ state.confirmText || "OK" }}
					</UButton>
				</div>
			</div>
		</template>
	</UModal>
</template>
