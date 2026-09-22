<script lang="ts" setup>
interface SchoolInfo {
	name: string;
	npsn: string;
	address: string;
	principal: string;
}

const props = defineProps<{
	school: SchoolInfo | null;
}>();

const rows = computed(() =>
	props.school
		? [
				{ label: "Nama Sekolah", value: props.school.name, icon: "i-lucide-school" },
				{ label: "NPSN", value: props.school.npsn, icon: "i-lucide-hash" },
				{ label: "Alamat", value: props.school.address, icon: "i-lucide-map-pin" },
				{ label: "Kepala Sekolah", value: props.school.principal, icon: "i-lucide-user" },
			]
		: []
);
</script>

<template>
	<UCard>
		<template #header>
			<div class="flex items-center gap-2">
				<UIcon name="i-lucide-building-2" class="size-4 text-muted" />
				<span class="font-semibold">Data Sekolah</span>
			</div>
		</template>

		<dl v-if="rows.length" class="divide-y divide-default">
			<div
				v-for="row in rows"
				:key="row.label"
				class="flex items-start gap-3 py-3 first:pt-0 last:pb-0"
			>
				<UIcon :name="row.icon" class="size-4 mt-0.5 text-muted shrink-0" />
				<div class="min-w-0 flex-1 grid sm:grid-cols-3 gap-1 sm:gap-3">
					<dt class="text-sm text-muted sm:col-span-1">
						{{ row.label }}
					</dt>
					<dd class="text-sm font-medium text-highlighted sm:col-span-2 break-words">
						{{ row.value || "-" }}
					</dd>
				</div>
			</div>
		</dl>

		<div v-else class="flex flex-col items-center gap-2 py-10 text-muted">
			<UIcon name="i-lucide-building-2" class="size-6" />
			<span class="text-sm">Data sekolah belum diisi.</span>
		</div>
	</UCard>
</template>