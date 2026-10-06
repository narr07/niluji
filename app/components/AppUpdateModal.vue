<script lang="ts" setup>
	import { getVersion } from "@tauri-apps/api/app";

	const { status, update, errorMessage, progress, modalOpen, busyReason, installUpdate, refreshBusy } = useAppUpdate();
	const currentVersion = ref("");

	const working = computed(() => status.value === "downloading" || status.value === "installing");
	const releaseDate = computed(() => (update.value?.date ? new Date(update.value.date.replace(" ", "T")).toLocaleDateString("id-ID", { dateStyle: "long" }) : ""));

	watch(modalOpen, async (isOpen) => {
		if (!isOpen) return;
		try {
			currentVersion.value = await getVersion();
		} catch {
			currentVersion.value = "";
		}
	});
</script>

<template>
	<UModal
		v-model:open="modalOpen"
		:dismissible="!working"
		:close="!working"
		title="Pembaruan Aplikasi"
		:ui="{ content: 'max-w-lg' }">
		<template #body>
			<div v-if="update" class="space-y-4">
				<div class="flex items-center gap-3">
					<UBadge color="neutral" variant="subtle" size="lg">
						v{{ currentVersion || "?" }}
					</UBadge>
					<UIcon name="i-lucide-arrow-right" class="size-4 text-muted" />
					<UBadge color="primary" variant="solid" size="lg">
						v{{ update.version }}
					</UBadge>
					<span v-if="releaseDate" class="ms-auto text-xs text-muted">{{ releaseDate }}</span>
				</div>

				<div v-if="update.body" class="max-h-64 overflow-y-auto rounded-md border border-default bg-elevated/40 p-3 text-sm whitespace-pre-line">
					{{ update.body }}
				</div>

				<UAlert
					v-if="busyReason"
					color="warning"
					variant="subtle"
					icon="i-lucide-shield-alert"
					title="Tunggu ujian selesai dulu"
					:description="`${busyReason} Update akan me-restart aplikasi dan memutus ujian siswa.`">
					<template #actions>
						<UButton
							size="xs"
							variant="outline"
							color="warning"
							icon="i-lucide-refresh-cw"
							@click="refreshBusy">
							Periksa lagi
						</UButton>
					</template>
				</UAlert>

				<UAlert
					v-else
					color="info"
					variant="subtle"
					icon="i-lucide-database-backup"
					description="Data soal, siswa, dan hasil ujian tetap aman. Aplikasi membuat backup dulu, lalu menutup dan membuka kembali dirinya setelah update terpasang." />

				<div v-if="working" class="space-y-1.5">
					<UProgress :model-value="progress" size="sm" />
					<p class="text-xs text-muted">
						{{ status === "installing" ? "Memasang pembaruan..." : progress === null ? "Mengunduh..." : `Mengunduh... ${progress}%` }}
					</p>
				</div>

				<UAlert
					v-if="status === 'error'"
					color="error"
					variant="subtle"
					title="Pembaruan gagal"
					:description="errorMessage" />
			</div>
		</template>

		<template #footer>
			<div class="flex w-full justify-end gap-2">
				<UButton
					variant="ghost"
					color="neutral"
					:disabled="working"
					@click="modalOpen = false">
					Nanti saja
				</UButton>
				<UButton
					icon="i-lucide-download"
					:loading="working"
					:disabled="!!busyReason || !update"
					@click="installUpdate">
					Update & Restart
				</UButton>
			</div>
		</template>
	</UModal>
</template>
