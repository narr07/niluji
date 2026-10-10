<script setup lang="ts">
	import type { TableColumn } from "@nuxt/ui";

	interface Guru {
		ptk_id: string
		nama: string
		gelar_depan: string | null
		gelar_belakang: string | null
		nip: string | null
		nuptk: string | null
		jenis_kelamin: string | null
		tempat_lahir: string | null
		tanggal_lahir: string | null
		jenis_ptk: string | null
		jabatan_ptk: string | null
		status_kepegawaian: string | null
	}

	interface EditGelar { ptk_id: string, nama: string, gelar_depan: string, gelar_belakang: string }

	const db = useDb();
	const swal = useSwal();
	const toast = useToast();

	const loading = ref(true);
	const rows = ref<Guru[]>([]);
	const open = ref(false);
	const saving = ref(false);
	// Isi form dipisah dari status buka modal, supaya isinya tidak hilang saat animasi menutup.
	const edit = ref<EditGelar>();

	async function load() {
		loading.value = true;
		try {
			rows.value = await db.query<Guru>(`SELECT ptk_id, nama, gelar_depan, gelar_belakang, nip, nuptk, jenis_kelamin,
				tempat_lahir, tanggal_lahir, jenis_ptk, jabatan_ptk, status_kepegawaian FROM ptk ORDER BY nama COLLATE NOCASE`);
		}
		catch (e) {
			toast.add({ title: "Gagal memuat data guru", description: pesanError(e), color: "error" });
		}
		finally {
			loading.value = false;
		}
	}
	onMounted(load);

	function openEdit(r: Guru) {
		edit.value = { ptk_id: r.ptk_id, nama: r.nama, gelar_depan: r.gelar_depan ?? "", gelar_belakang: r.gelar_belakang ?? "" };
		open.value = true;
	}

	async function save() {
		if (!edit.value || saving.value)
			return;
		saving.value = true;
		try {
			const e = edit.value;
			await db.execute("UPDATE ptk SET gelar_depan = ?, gelar_belakang = ? WHERE ptk_id = ?", [e.gelar_depan.trim() || null, e.gelar_belakang.trim() || null, e.ptk_id]);
			// Perbarui baris di tabel langsung, tanpa memuat ulang seluruh daftar.
			const row = rows.value.find(r => r.ptk_id === e.ptk_id);
			if (row) {
				row.gelar_depan = e.gelar_depan.trim() || null;
				row.gelar_belakang = e.gelar_belakang.trim() || null;
			}
			open.value = false;
			toast.add({ title: "Gelar disimpan", description: row ? namaLengkap(row) : undefined, color: "success" });
		}
		catch (err) {
			// Modal tetap terbuka supaya isian tidak hilang.
			swal.error("Gagal menyimpan gelar", pesanError(err));
		}
		finally {
			saving.value = false;
		}
	}

	const columns: TableColumn<Guru>[] = [
		{ id: "nama", header: "Nama (dengan gelar)", cell: ({ row }) => namaLengkap(row.original), size: 260, meta: { class: { td: "font-medium" } } },
		{ accessorKey: "nip", header: "NIP", cell: ({ row }) => row.original.nip || "-", meta: { class: { td: "font-mono tabular-nums" } } },
		{ accessorKey: "nuptk", header: "NUPTK", cell: ({ row }) => row.original.nuptk || "-", meta: { class: { td: "font-mono tabular-nums" } } },
		{ accessorKey: "jenis_kelamin", header: "L/P", meta: { class: { th: "w-12 text-center", td: "text-center" } } },
		{ id: "ttl", header: "Tempat, Tgl Lahir", cell: ({ row }) => tempatTanggal(row.original.tempat_lahir, row.original.tanggal_lahir) },
		{ accessorKey: "jenis_ptk", header: "Jenis PTK" },
		{ accessorKey: "status_kepegawaian", header: "Status" },
		{ id: "opsi", header: "", size: 90 }
	];
</script>

<template>
	<ErPage id="ref-guru" title="Guru">
		<template #right>
			<DapodikNote />
		</template>

		<RefTable
			:data="rows"
			:columns="columns"
			:loading="loading"
			:pin="{ left: ['nama'], right: ['opsi'] }"
			:on-row-click="openEdit"
		>
			<template #opsi-cell="{ row }">
				<UButton
					size="xs"
					color="neutral"
					variant="outline"
					icon="lucide:pencil"
					@click.stop="openEdit(row.original)"
				>
					Gelar
				</UButton>
			</template>
		</RefTable>
	</ErPage>

	<UModal v-model:open="open" title="Edit Gelar Guru" :description="edit?.nama">
		<template #body>
			<form
				v-if="edit"
				id="form-gelar"
				class="flex flex-col gap-3"
				@submit.prevent="save">
				<div class="grid grid-cols-2 gap-2">
					<UFormField label="Gelar Depan">
						<UInput v-model="edit.gelar_depan" placeholder="mis. Drs." autofocus />
					</UFormField>
					<UFormField label="Gelar Belakang">
						<UInput v-model="edit.gelar_belakang" placeholder="mis. S.Pd." />
					</UFormField>
				</div>
				<p class="text-sm">
					Di rapor: <b>{{ namaLengkap(edit) }}</b>
				</p>
				<p class="text-xs text-muted">
					Gelar hanya tersimpan di e-Rapor (dipakai saat cetak rapor), tidak dikirim ke Dapodik.
				</p>
			</form>
		</template>
		<template #footer>
			<div class="flex w-full justify-end gap-2">
				<UButton
					color="neutral"
					variant="outline"
					:disabled="saving"
					@click="open = false">
					Batal
				</UButton>
				<UButton
					type="submit"
					form="form-gelar"
					variant="solid"
					icon="lucide:save"
					:loading="saving">
					Simpan
				</UButton>
			</div>
		</template>
	</UModal>
</template>
