<script setup lang="ts">
	import type { RaporKelas, RaporSiswa } from "~/composables/useRapor";

	// Padanan e-Rapor (siswa): Download Rapor. Hanya tampil kalau rapor kelasnya sudah dibagikan
	// (tombol di halaman Cetak Rapor). Simpan sebagai PDF lewat dialog cetak.
	const db = useDb();
	const toast = useToast();
	const { user, session } = useAuth();
	const { muat } = useRapor();

	const data = ref<RaporKelas>();
	const saya = ref<RaporSiswa>();
	const status = ref<"memuat" | "belum" | "tanpa-kelas" | "siap">("memuat");
	const tempat = ref("");
	const tanggal = ref("");

	onMounted(async () => {
		try {
			const sem = session.value?.semesterId;
			const r = await db.first<{ id: string }>(
				`SELECT a.rombongan_belajar_id AS id FROM anggota_rombel a JOIN rombel r USING (rombongan_belajar_id)
				WHERE a.peserta_didik_id = ? AND r.semester_id = ? AND r.jenis_rombel = '1' LIMIT 1`,
				[user.value?.peserta_didik_id, sem]
			);
			if (!r || !sem) {
				status.value = "tanpa-kelas";
				return;
			}
			if ((await db.getSetting(`rapor_dibagikan:${sem}:${r.id}`)) !== "1") {
				status.value = "belum";
				return;
			}
			const d = await muat(r.id, sem);
			data.value = d;
			saya.value = d.siswa.find(s => s.id === user.value?.peserta_didik_id);
			tempat.value = (await db.getSetting("rapor_tempat")) ?? "";
			tanggal.value = (await db.getSetting(`rapor_tanggal:${sem}`)) ?? "";
			status.value = saya.value ? "siap" : "tanpa-kelas";
		}
		catch (e) {
			toast.add({ title: "Gagal memuat rapor", description: pesanError(e), color: "error" });
			status.value = "belum";
		}
	});

	const cetak = () => window.print();

	useHead({ style: [{ key: "rapor-page", textContent: "@page { size: 210mm 297mm; margin: 0; }" }] });
</script>

<template>
	<ErPage id="siswa-rapor" title="Download Rapor">
		<template #right>
			<UButton v-if="status === 'siap'" icon="lucide:download" @click="cetak">
				Simpan PDF / Cetak
			</UButton>
		</template>

		<USkeleton v-if="status === 'memuat'" class="h-[60vh] w-full" />
		<div v-else-if="status !== 'siap'" class="flex flex-col items-center gap-3 py-24 text-center text-muted">
			<UIcon name="lucide:file-lock" class="size-12" />
			<p v-if="status === 'belum'">
				Rapor semester ini belum dibagikan oleh wali kelas.
			</p>
			<p v-else>
				Akun ini belum terdaftar di kelas mana pun pada semester ini.
			</p>
		</div>
		<div v-else-if="data && saya" class="overflow-x-auto rounded-md bg-elevated p-4">
			<RaporLembar
				:data="data"
				:siswa="saya"
				:tempat="tempat"
				:tanggal="tanggal"
				:tanda-tangan="true" />
		</div>

		<Teleport to="body">
			<div v-if="data && saya" class="cetak-root">
				<RaporLembar
					:data="data"
					:siswa="saya"
					:tempat="tempat"
					:tanggal="tanggal"
					:tanda-tangan="true" />
			</div>
		</Teleport>
	</ErPage>
</template>
