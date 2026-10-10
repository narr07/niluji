<script setup lang="ts">
	// Satu kartu akun (dipakai di tampilan & salinan cetak). Gaya global supaya ikut berlaku saat diteleport.
	defineProps<{ sekolah: string, sub: string, nama: string, username: string, password: string }>();
</script>

<template>
	<div class="kartu">
		<p class="kartu-sekolah">
			{{ sekolah }}
		</p>
		<p class="kartu-peran">
			Kartu Akun e-Rapor · {{ sub }}
		</p>
		<p class="kartu-nama">
			{{ nama }}
		</p>
		<div class="kartu-kredensial">
			<span class="kartu-label">Username</span>
			<span class="kartu-nilai">{{ username }}</span>
			<span class="kartu-label">Password</span>
			<span class="kartu-nilai">{{ password }}</span>
		</div>
		<p class="kartu-catatan">
			Rahasiakan akun Anda · ganti password setelah masuk
		</p>
	</div>
</template>

<style>
	.kartu {
		display: flex;
		flex-direction: column;
		height: 44mm;
		border: 1px dashed var(--ui-border, #d4d4d8);
		border-radius: 8px;
		padding: 3mm 4mm;
		background: var(--ui-bg, #fff);
		break-inside: avoid;
		overflow: hidden;
	}
	.kartu .kartu-sekolah { font-weight: 700; font-size: 10pt; line-height: 1.2; }
	.kartu .kartu-peran { font-size: 7pt; color: var(--ui-text-muted, #71717a); margin-bottom: 1.5mm; }
	.kartu .kartu-nama { font-weight: 600; font-size: 10pt; line-height: 1.2; margin-bottom: 1mm; }
	.kartu .kartu-kredensial { display: grid; grid-template-columns: auto 1fr; gap: 0 2mm; align-items: baseline; }
	.kartu .kartu-label { font-size: 8pt; color: var(--ui-text-muted, #71717a); }
	.kartu .kartu-label::after { content: ":"; }
	.kartu .kartu-nilai { font-family: ui-monospace, "Consolas", monospace; font-weight: 700; font-size: 9.5pt; word-break: break-all; }
	.kartu .kartu-catatan { margin-top: auto; font-size: 6.5pt; color: var(--ui-text-muted, #a1a1aa); }
	@media print {
		.kartu { border: 1px dashed #999; color: #000; background: #fff; }
		.kartu .kartu-peran, .kartu .kartu-label { color: #555; }
		.kartu .kartu-catatan { color: #777; }
	}
</style>
