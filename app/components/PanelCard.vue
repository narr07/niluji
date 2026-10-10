<script setup lang="ts">
	// Kartu bergaya niluji (lihat nuxtor-main/app/components/dashboard/SekolahInfo.vue).
	// ui: kelas tambahan untuk slot UCard (root/header/body/footer), digabung dengan bawaan.
	const props = defineProps<{ title?: string, icon?: string, description?: string, ui?: { root?: string, header?: string, body?: string, footer?: string } }>();
	const ui = computed(() => ({ ...props.ui, body: ["sm:p-4", props.ui?.body].filter(Boolean).join(" ") }));
</script>

<template>
	<UCard :ui="ui">
		<template v-if="title || $slots.actions" #header>
			<div class="flex flex-wrap items-center gap-2">
				<UIcon v-if="icon" :name="icon" class="size-4 text-muted" />
				<div class="min-w-0">
					<p class="font-semibold">
						{{ title }}
					</p>
					<p v-if="description" class="text-xs text-muted">
						{{ description }}
					</p>
				</div>
				<div class="ms-auto flex flex-wrap items-center gap-2">
					<slot name="actions" />
				</div>
			</div>
		</template>
		<slot />
	</UCard>
</template>
