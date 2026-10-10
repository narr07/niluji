<script setup lang="ts">
	import type { TableColumn } from "@nuxt/ui";

	interface Kelas {
		rombongan_belajar_id: string
		nama: string
		tingkat: string | null
		kurikulum: string | null
		wali: string | null
		jumlah: number
		mapel: number
	}

	interface Anggota {
		peserta_didik_id: string
		nama: string
		nisn: string | null
		jenis_kelamin: string | null
		jenis_pendaftaran: string | null
		dikoreksi: number
	}

	const db = useDb();
	const toast = useToast();
	const { session } = useAuth();

	// Jumlah siswa dihitung per semester (sama dengan Dashboard); "Mapel Rapor" = hasil susun pembelajaran.
	const { rows, loading } = useSemesterData(sem => db.query<Kelas>(
		`SELECT r.rombongan_belajar_id, r.nama, r.tingkat, r.kurikulum, g.nama AS wali,
			(SELECT COUNT(*) FROM anggota_rombel a WHERE a.rombongan_belajar_id = r.rombongan_belajar_id AND a.semester_id = r.semester_id) AS jumlah,
			(SELECT COUNT(*) FROM pembelajaran_rapor pr WHERE pr.rombongan_belajar_id = r.rombongan_belajar_id AND pr.semester_id = r.semester_id) AS mapel
		FROM rombel r LEFT JOIN ptk g ON g.ptk_id = r.ptk_id
		WHERE r.semester_id = ? AND r.jenis_rombel = '1' ORDER BY CAST(r.tingkat AS INTEGER), r.nama`,
		[sem]
	), "Gagal memuat data kelas");

	// ===== Modal anggota =====
	// Status buka dipisah dari data: modal langsung terbuka dengan loading, isinya tidak hilang saat animasi tutup.
	const open = ref(false);
	const loadingAnggota = ref(false);
	const judul = ref("");
	const anggota = ref<Anggota[]>([]);

	async function showAnggota(r: Kelas) {
		judul.value = r.nama;
		anggota.value = [];
		cariAnggota.value = "";
		open.value = true;
		loadingAnggota.value = true;
		try {
			// Nama dari siswa_rapor = sudah termasuk koreksi admin (sama dengan yang tercetak di rapor).
			anggota.value = await db.query<Anggota>(
				`SELECT s.peserta_didik_id, s.nama, s.nisn, s.jenis_kelamin, a.jenis_pendaftaran,
					EXISTS (SELECT 1 FROM koreksi_siswa k WHERE k.peserta_didik_id = s.peserta_didik_id) AS dikoreksi
				FROM anggota_rombel a JOIN siswa_rapor s USING (peserta_didik_id)
				WHERE a.rombongan_belajar_id = ? ORDER BY s.nama COLLATE NOCASE`,
				[r.rombongan_belajar_id]
			);
		}
		catch (e) {
			toast.add({ title: "Gagal memuat anggota kelas", description: pesanError(e), color: "error" });
		}
		finally {
			loadingAnggota.value = false;
		}
	}

	const cariAnggota = ref("");
	const anggotaTampil = computed(() => {
		const q = cariAnggota.value.toLowerCase().trim();
		return q ? anggota.value.filter(a => a.nama.toLowerCase().includes(q) || a.nisn?.includes(q)) : anggota.value;
	});
	// "Siswa baru" hampir semua siswa — badge hanya untuk yang lain (mis. pindahan) supaya daftar tetap ringkas.
	const pendaftaranKhusus = (a: Anggota) => (a.jenis_pendaftaran && !/^siswa baru$/i.test(a.jenis_pendaftaran) ? a.jenis_pendaftaran : "");

	const jumlahLP = computed(() => {
		const l = anggota.value.filter(a => a.jenis_kelamin === "L").length;
		return { l, p: anggota.value.length - l };
	});

	const angka = { class: { th: "text-right", td: "text-right tabular-nums" } };

	const columns: TableColumn<Kelas>[] = [
		{ accessorKey: "nama", header: "Nama Kelas", size: 140, meta: { class: { td: "font-medium" } } },
		{ accessorKey: "tingkat", header: "Tingkat", cell: ({ row }) => atauStrip(row.original.tingkat) },
		{ accessorKey: "wali", header: "Wali Kelas", cell: ({ row }) => atauStrip(row.original.wali) },
		{ accessorKey: "kurikulum", header: "Kurikulum", cell: ({ row }) => atauStrip(row.original.kurikulum) },
		{ accessorKey: "jumlah", header: "Jumlah Siswa", meta: angka },
		{ accessorKey: "mapel", header: "Mapel Rapor", meta: angka },
		{ id: "opsi", header: "", size: 110 }
	];
</script>

<template>
	<ErPage id="ref-kelas" title="Kelas">
		<template #right>
			<DapodikNote />
		</template>

		<RefTable
			:data="rows"
			:columns="columns"
			:loading="loading"
			:pin="{ left: ['nama'], right: ['opsi'] }"
			:on-row-click="showAnggota"
			:empty="session?.semesterId ? 'Belum ada kelas di semester ini. Tarik dulu lewat Sinkron Dapodik.' : 'Belum ada semester. Tarik dulu lewat Sinkron Dapodik.'"
		>
			<template #opsi-cell="{ row }">
				<UButton
					size="xs"
					color="neutral"
					variant="outline"
					icon="lucide:users"
					@click.stop="showAnggota(row.original)"
				>
					Anggota
				</UButton>
			</template>
		</RefTable>
	</ErPage>

	<UModal
		v-model:open="open"
		:title="`Anggota ${judul}`"
		:description="loadingAnggota ? 'Memuat…' : `${anggota.length} siswa · ${jumlahLP.l} laki-laki, ${jumlahLP.p} perempuan`"
		:ui="{ content: 'max-w-xl' }"
	>
		<template #body>
			<div class="flex flex-col gap-3">
				<UInput
					v-model="cariAnggota"
					icon="lucide:search"
					placeholder="Cari nama atau NISN…"
					class="w-full"
					:disabled="loadingAnggota"
				/>

				<div v-if="loadingAnggota" class="flex flex-col gap-2">
					<USkeleton v-for="i in 6" :key="i" class="h-11 w-full" />
				</div>

				<ol v-else-if="anggotaTampil.length" class="divide-y divide-default rounded-md border border-default">
					<li v-for="(a, i) in anggotaTampil" :key="a.peserta_didik_id" class="flex items-center gap-3 px-3 py-2">
						<span class="w-6 shrink-0 text-right text-xs text-dimmed tabular-nums">{{ i + 1 }}</span>
						<div class="flex min-w-0 flex-1 items-center gap-1.5">
							<span class="truncate text-sm font-medium" :title="a.nama">{{ a.nama }}</span>
							<UTooltip v-if="a.dikoreksi" text="Data dikoreksi di e-Rapor (beda dengan Dapodik)">
								<UIcon
									name="lucide:pencil-line"
									class="size-3.5 shrink-0 text-warning"
									role="img"
									aria-label="Dikoreksi" />
							</UTooltip>
							<UBadge
								v-if="pendaftaranKhusus(a)"
								size="sm"
								color="neutral"
								variant="subtle"
								class="shrink-0">
								{{ pendaftaranKhusus(a) }}
							</UBadge>
						</div>
						<span class="shrink-0 font-mono text-xs text-muted tabular-nums">{{ a.nisn || "-" }}</span>
						<UBadge
							size="sm"
							variant="subtle"
							class="w-6 shrink-0 justify-center"
							:color="a.jenis_kelamin === 'L' ? 'info' : 'secondary'"
						>
							{{ a.jenis_kelamin || "-" }}
						</UBadge>
					</li>
				</ol>

				<p v-else class="py-8 text-center text-sm text-muted">
					{{ cariAnggota ? "Tidak ada siswa yang cocok." : "Belum ada anggota di kelas ini." }}
				</p>
			</div>
		</template>
	</UModal>
</template>
