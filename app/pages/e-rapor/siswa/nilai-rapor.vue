<script setup lang="ts">
	// Padanan e-Rapor (siswa): Rekap Capaian → Capaian Rapor + Deskripsi Capaian.
	const { user } = useAuth();
</script>

<template>
	<ErPage id="siswa-nilai" title="Nilai Rapor">
		<PerkembanganNilai v-if="user?.peserta_didik_id" :peserta-didik-id="user.peserta_didik_id" />
		<p v-else class="py-24 text-center text-muted">
			Akun ini belum terhubung dengan data siswa. Hubungi admin sekolah.
		</p>
	</ErPage>
</template>
