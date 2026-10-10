<script setup lang="ts">
	// Kop surat sekolah untuk hasil cetak (pelengkap rapor, transkrip). Logo kiri/kanan opsional.
	defineProps<{ kop: Kop }>();
</script>

<template>
	<div class="kop-surat">
		<img
			v-if="kop.kiri"
			:src="kop.kiri"
			alt=""
			class="kop-logo">
		<div v-else class="kop-logo" />
		<div class="kop-teks">
			<p v-for="(b, i) in kop.baris.filter(Boolean)" :key="i" :class="{ utama: i === kop.baris.filter(Boolean).length - 2, kecil: i === kop.baris.filter(Boolean).length - 1 }">
				{{ b }}
			</p>
		</div>
		<img
			v-if="kop.kanan"
			:src="kop.kanan"
			alt=""
			class="kop-logo">
		<div v-else class="kop-logo" />
	</div>
</template>

<style>
	.kop-surat { display: flex; align-items: center; gap: 12px; border-bottom: 3px double #000; padding-bottom: 6px; margin-bottom: 14px; color: #000; font-family: Arial, Helvetica, sans-serif; }
	.kop-surat .kop-logo { width: 2.2cm; height: 2.2cm; object-fit: contain; flex-shrink: 0; }
	.kop-surat .kop-teks { flex: 1; text-align: center; line-height: 1.25; }
	.kop-surat .kop-teks p { margin: 0; font-size: 12pt; font-weight: bold; }
	.kop-surat .kop-teks p.utama { font-size: 15pt; }
	.kop-surat .kop-teks p.kecil { font-size: 9.5pt; font-weight: normal; }
</style>
