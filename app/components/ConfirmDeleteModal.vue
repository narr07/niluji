<script lang="ts" setup>
	const { open, title, description, confirmLabel } = useConfirmDeleteState();
</script>

<template>
	<UModal
		v-model:open="open"
		:title="title"
		:ui="{ content: 'max-w-sm' }"
		@update:open="(v) => !v && resolveConfirmDelete(false)">
		<template #body>
			<div class="space-y-4">
				<p v-if="description" class="text-sm text-muted">
					{{ description }}
				</p>
				<div class="flex gap-2 justify-end">
					<UButton variant="soft" color="neutral" @click="resolveConfirmDelete(false)">
						Batal
					</UButton>
					<UButton icon="lucide:trash-2" color="error" @click="resolveConfirmDelete(true)">
						{{ confirmLabel }}
					</UButton>
				</div>
			</div>
		</template>
	</UModal>
</template>
