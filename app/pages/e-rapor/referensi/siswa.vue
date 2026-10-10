<script setup lang="ts">
	import type { TableColumn } from "@nuxt/ui";

	// Data siswa dari Dapodik + koreksi lokal (tabel koreksi_siswa). Koreksi hanya berlaku di e-Rapor
	// (tampilan & cetak rapor), tidak mengubah data Dapodik dan tidak hilang saat sinkron ulang.
	// Di koreksi_siswa: NULL = ikut Dapodik, "" = sengaja dikosongkan, teks lain = nilai pengganti.

	type Field = "nama" | "nisn" | "nipd" | "jenis_kelamin" | "tempat_lahir" | "tanggal_lahir" | "agama" | "nama_ayah" | "nama_ibu";

	// Konstanta (bukan input pengguna) — aman disisipkan sebagai nama kolom SQL.
	const FIELDS: { key: Field, label: string }[] = [
		{ key: "nama", label: "Nama" },
		{ key: "nisn", label: "NISN" },
		{ key: "nipd", label: "NIS" },
		{ key: "jenis_kelamin", label: "Jenis Kelamin" },
		{ key: "tempat_lahir", label: "Tempat Lahir" },
		{ key: "tanggal_lahir", label: "Tanggal Lahir" },
		{ key: "agama", label: "Agama" },
		{ key: "nama_ayah", label: "Nama Ayah" },
		{ key: "nama_ibu", label: "Nama Ibu" }
	];

	type Nilai = Record<Field, string | null>;

	interface Siswa extends Nilai {
		peserta_didik_id: string
		rombel: string
		kelasList: string[]
		dapodik: Nilai
		beda: { label: string, dapodik: string, rapor: string }[]
		catatan: string | null
		updated_at: string | null
		updated_by: string | null
	}

	const db = useDb();
	const toast = useToast();
	const swal = useSwal();
	const { session, user, waliRombel } = useAuth();
	// Halaman ini juga dipakai wali kelas (/walas/siswa, padanan "Update Data Siswa" e-Rapor):
	// hanya siswa kelasnya. Di laptop guru hanya-baca, karena koreksi data tidak ikut terkirim ke admin.
	const { isGuruMode, load: loadMode } = useAppMode();
	const isAdmin = computed(() => user.value?.level === "admin");
	const bisaEdit = computed(() => isAdmin.value || !isGuruMode.value);
	const kelasWali = computed(() => (isAdmin.value ? null : waliRombel.value.map(r => r.rombongan_belajar_id)));

	const loading = ref(true);
	const rows = ref<Siswa[]>([]);
	const kelas = ref("semua");
	const hanyaKoreksi = ref(false);

	// Satu aturan "sama dengan Dapodik" untuk tabel maupun form: kosong dan NULL dianggap sama.
	const sama = (nilai: string, dapodik: string | null) => nilai.trim() === (dapodik ?? "").trim();
	const tampilNilai = (f: Field, v: string | null) => (f === "tanggal_lahir" ? tanggalIndo(v) : v) || "-";

	async function load() {
		const sem = session.value?.semesterId;
		if (!sem) {
			rows.value = [];
			loading.value = false;
			return;
		}
		loading.value = true;
		try {
			await loadMode();
			const ids = kelasWali.value;
			if (ids && !ids.length) {
				rows.value = [];
				return;
			}
			const cols = FIELDS.map(f => `p.${f.key} AS d_${f.key}, k.${f.key} AS k_${f.key}`).join(", ");
			// Satu siswa satu baris. Kalau di Dapodik tercatat di dua rombel reguler sekaligus (salah
			// input), kelasnya digabung berurutan "Kelas 4, Kelas 5" supaya kelihatan dan bisa dibetulkan.
			const raw = await db.query<Record<string, string | null>>(
				`SELECT p.peserta_didik_id, ${cols}, k.catatan, k.updated_at, k.updated_by,
					(SELECT group_concat(nama, ', ') FROM (
						SELECT r2.nama FROM anggota_rombel a2 JOIN rombel r2 USING (rombongan_belajar_id)
						WHERE a2.peserta_didik_id = p.peserta_didik_id AND a2.semester_id = ?1 AND r2.jenis_rombel = '1'
						ORDER BY CAST(r2.tingkat AS INTEGER), r2.nama)) AS rombel,
					MIN(CAST(r.tingkat AS INTEGER)) AS tingkat
				FROM peserta_didik p
				LEFT JOIN koreksi_siswa k ON k.peserta_didik_id = p.peserta_didik_id
				JOIN anggota_rombel a ON a.peserta_didik_id = p.peserta_didik_id AND a.semester_id = ?1
				JOIN rombel r ON r.rombongan_belajar_id = a.rombongan_belajar_id AND r.jenis_rombel = '1'
				${ids ? `WHERE a.rombongan_belajar_id IN (${ids.map(() => "?").join(",")})` : ""}
				GROUP BY p.peserta_didik_id
				ORDER BY tingkat, rombel, d_nama COLLATE NOCASE`,
				[sem, ...(ids ?? [])]
			);
			rows.value = raw.map((r) => {
				const dapodik = {} as Nilai;
				const efektif = {} as Nilai;
				const beda: Siswa["beda"] = [];
				for (const f of FIELDS) {
					const d = r[`d_${f.key}`] ?? null;
					const k = r[`k_${f.key}`] ?? null;
					dapodik[f.key] = d;
					efektif[f.key] = k ?? d;
					// Kalau Dapodik sudah dibetulkan dan sama dengan koreksi, tidak dihitung beda lagi.
					if (k !== null && !sama(k, d))
						beda.push({ label: f.label, dapodik: tampilNilai(f.key, d), rapor: k === "" ? "(dikosongkan)" : tampilNilai(f.key, k) });
				}
				const rombel = r.rombel ?? "";
				return {
					...efektif,
					peserta_didik_id: r.peserta_didik_id!,
					rombel,
					kelasList: rombel.split(", ").filter(Boolean),
					dapodik,
					beda,
					catatan: r.catatan ?? null,
					updated_at: r.updated_at ?? null,
					updated_by: r.updated_by ?? null
				};
			});
			// Kelas yang dipilih tidak ada di semester ini → kembali ke semua kelas (bukan tabel kosong).
			if (kelas.value !== "semua" && !rows.value.some(s => s.kelasList.includes(kelas.value)))
				kelas.value = "semua";
		}
		catch (e) {
			toast.add({ title: "Gagal memuat data siswa", description: pesanError(e), color: "error" });
		}
		finally {
			loading.value = false;
		}
	}

	watch(() => session.value?.semesterId, load, { immediate: true });

	const kelasItems = computed(() => [
		{ label: "Semua kelas", value: "semua" },
		...[...new Set(rows.value.flatMap(r => r.kelasList))].map(k => ({ label: k, value: k }))
	]);
	const jumlahKoreksi = computed(() => rows.value.filter(r => r.beda.length).length);
	const tampil = computed(() => rows.value.filter(r =>
		(kelas.value === "semua" || r.kelasList.includes(kelas.value)) && (!hanyaKoreksi.value || r.beda.length)));

	// ===== Edit koreksi =====
	const open = ref(false);
	const saving = ref(false);
	const edit = ref<{ siswa: Siswa, form: Record<Field, string>, catatan: string, awal: string }>();
	const namaInput = ref<{ inputRef?: HTMLInputElement }>();

	const snapshot = (form: Record<Field, string>, catatan: string) => JSON.stringify([form, catatan]);

	function openEdit(s: Siswa) {
		if (!bisaEdit.value) {
			toast.add({ title: "Koreksi data siswa dilakukan di laptop admin", description: "Di laptop guru data siswa hanya bisa dilihat.", color: "info" });
			return;
		}
		const form = {} as Record<Field, string>;
		FIELDS.forEach((f) => { form[f.key] = s[f.key] ?? ""; });
		const catatan = s.catatan ?? "";
		edit.value = { siswa: s, form, catatan, awal: snapshot(form, catatan) };
		open.value = true;
	}

	// Reka UI memfokuskan tombol tutup modal lebih dulu; arahkan ke kolom Nama.
	function fokusNama(e: Event) {
		e.preventDefault();
		nextTick(() => namaInput.value?.inputRef?.focus());
	}

	const berbeda = (f: Field) => !!edit.value && !sama(edit.value.form[f], edit.value.siswa.dapodik[f]);
	const dirty = computed(() => !!edit.value && snapshot(edit.value.form, edit.value.catatan) !== edit.value.awal);

	async function tutup() {
		if (saving.value)
			return;
		if (dirty.value && !await swal.confirm("Buang perubahan?", "Isian koreksi belum disimpan.", "Buang"))
			return;
		open.value = false;
	}

	async function simpan() {
		const e = edit.value;
		if (!e || saving.value)
			return;
		if (!e.form.nama.trim()) {
			swal.error("Nama wajib diisi");
			return;
		}
		saving.value = true;
		try {
			// Kolom sama dengan Dapodik → NULL (ikut Dapodik). Kolom berbeda → disimpan, termasuk "" (dikosongkan).
			const nilai = FIELDS.map(f => (berbeda(f.key) ? e.form[f.key].trim() : null));
			const adaKoreksi = nilai.some(v => v !== null);
			if (!adaKoreksi) {
				await db.execute("DELETE FROM koreksi_siswa WHERE peserta_didik_id = ?", [e.siswa.peserta_didik_id]);
			}
			else {
				await db.execute(
					`INSERT INTO koreksi_siswa (peserta_didik_id, ${FIELDS.map(f => f.key).join(", ")}, catatan, updated_at, updated_by)
					VALUES (?, ${FIELDS.map(() => "?").join(", ")}, ?, datetime('now','localtime'), ?)
					ON CONFLICT (peserta_didik_id) DO UPDATE SET ${FIELDS.map(f => `${f.key} = excluded.${f.key}`).join(", ")},
						catatan = excluded.catatan, updated_at = excluded.updated_at, updated_by = excluded.updated_by`,
					[e.siswa.peserta_didik_id, ...nilai, e.catatan.trim() || null, user.value?.nama ?? null]
				);
			}
			open.value = false;
			await load();
			toast.add({
				title: adaKoreksi ? "Koreksi disimpan" : "Data kembali sama dengan Dapodik",
				description: adaKoreksi ? "Hanya berlaku di e-Rapor (tampilan & cetak rapor)." : undefined,
				color: "success"
			});
		}
		catch (err) {
			swal.error("Gagal menyimpan koreksi", pesanError(err));
		}
		finally {
			saving.value = false;
		}
	}

	async function kembalikan() {
		const e = edit.value;
		if (!e || !await swal.confirm("Kembalikan ke data Dapodik?", `Semua koreksi untuk ${e.siswa.dapodik.nama} dihapus. Rapor memakai data Dapodik lagi.`, "Kembalikan"))
			return;
		try {
			await db.execute("DELETE FROM koreksi_siswa WHERE peserta_didik_id = ?", [e.siswa.peserta_didik_id]);
			open.value = false;
			await load();
			toast.add({ title: "Data dikembalikan ke Dapodik", color: "success" });
		}
		catch (err) {
			swal.error("Gagal mengembalikan", pesanError(err));
		}
	}

	const AGAMA = ["Islam", "Kristen", "Katholik", "Hindu", "Budha", "Khonghucu", "Kepercayaan kpd Tuhan YME"];
	const agamaItems = computed(() => [...new Set([...AGAMA, edit.value?.form.agama, edit.value?.siswa.dapodik.agama].filter(Boolean) as string[])]);
	const JK = [{ label: "Laki-laki (L)", value: "L" }, { label: "Perempuan (P)", value: "P" }];

	// ===== Tabel =====
	const bedaTeks = (s: Siswa) => s.beda.map(b => `${b.label}: ${b.dapodik} → ${b.rapor}`).join(" · ");
	const mono = { class: { td: "font-mono tabular-nums" } };

	const columns: TableColumn<Siswa>[] = [
		{ accessorKey: "nama", header: "Nama Siswa", size: 240, meta: { class: { td: "font-medium" } } },
		{ id: "status", header: "", size: 40 },
		{ accessorKey: "nisn", header: "NISN", cell: ({ row }) => atauStrip(row.original.nisn), meta: mono },
		{ accessorKey: "nipd", header: "NIS", cell: ({ row }) => atauStrip(row.original.nipd), meta: mono },
		{ accessorKey: "jenis_kelamin", header: "L/P", meta: { class: { th: "w-12 text-center", td: "text-center" } } },
		{ id: "ttl", header: "Tempat, Tgl Lahir", cell: ({ row }) => tempatTanggal(row.original.tempat_lahir, row.original.tanggal_lahir), meta: { class: { td: "tabular-nums" } } },
		{ accessorKey: "agama", header: "Agama", cell: ({ row }) => atauStrip(row.original.agama) },
		{ accessorKey: "rombel", header: "Kelas" },
		{ accessorKey: "nama_ayah", header: "Ayah", cell: ({ row }) => atauStrip(row.original.nama_ayah) },
		{ accessorKey: "nama_ibu", header: "Ibu", cell: ({ row }) => atauStrip(row.original.nama_ibu) },
		{ id: "opsi", header: "", size: 80 }
	];
</script>

<template>
	<ErPage id="ref-siswa" :title="isAdmin ? 'Siswa' : 'Data Siswa Kelas'">
		<template #right>
			<UBadge
				v-if="jumlahKoreksi"
				color="warning"
				variant="subtle"
				icon="lucide:pencil-line">
				{{ jumlahKoreksi }} dikoreksi
			</UBadge>
			<DapodikNote />
		</template>

		<RefTable
			:data="tampil"
			:columns="columns"
			:loading="loading"
			:pin="{ left: ['nama'], right: ['opsi'] }"
			:on-row-click="openEdit"
			:empty="session?.semesterId ? 'Belum ada siswa di sini.' : 'Belum ada semester. Tarik dulu lewat Sinkron Dapodik.'"
		>
			<template #toolbar>
				<div class="flex items-center gap-3">
					<USelect
						v-model="kelas"
						:items="kelasItems"
						icon="lucide:layers"
						class="w-44"
						:disabled="loading" />
					<UCheckbox v-model="hanyaKoreksi" label="Hanya yang dikoreksi" :disabled="!jumlahKoreksi" />
				</div>
			</template>

			<template #status-cell="{ row }">
				<UTooltip v-if="row.original.beda.length" :text="bedaTeks(row.original)">
					<UIcon
						name="lucide:pencil-line"
						class="size-4 text-warning"
						role="img"
						aria-label="Beda dengan Dapodik" />
				</UTooltip>
			</template>

			<template #opsi-cell="{ row }">
				<UButton
					v-if="bisaEdit"
					size="xs"
					color="neutral"
					variant="outline"
					icon="lucide:pencil"
					@click.stop="openEdit(row.original)">
					Edit
				</UButton>
			</template>
		</RefTable>
	</ErPage>

	<UModal
		v-model:open="open"
		title="Koreksi Data Siswa"
		:description="edit?.siswa.dapodik.nama ?? undefined"
		:dismissible="!dirty && !saving"
		:content="{ onOpenAutoFocus: fokusNama }"
		:ui="{ content: 'max-w-2xl' }"
		@close:prevent="tutup"
	>
		<template #body>
			<form
				v-if="edit"
				id="form-siswa"
				class="flex flex-col gap-4"
				@submit.prevent="simpan">
				<UAlert
					color="info"
					variant="subtle"
					icon="lucide:info"
					description="Koreksi hanya berlaku di e-Rapor (tampilan & cetak rapor). Data di Dapodik tidak berubah. Sebaiknya perbaiki juga di Dapodik. Kolom yang dikosongkan akan tampil kosong di rapor."
				/>

				<div class="grid gap-3 sm:grid-cols-2">
					<UFormField
						v-for="f in FIELDS"
						:key="f.key"
						:label="f.label"
						:class="f.key === 'nama' ? 'sm:col-span-2' : ''"
						:help="`Dapodik: ${tampilNilai(f.key, edit.siswa.dapodik[f.key])}`"
						:ui="{ help: berbeda(f.key) ? 'text-warning' : 'text-dimmed' }"
					>
						<USelect
							v-if="f.key === 'jenis_kelamin'"
							v-model="edit.form.jenis_kelamin"
							:items="JK"
							placeholder="Pilih jenis kelamin"
							class="w-full"
							:disabled="saving"
						/>
						<USelect
							v-else-if="f.key === 'agama'"
							v-model="edit.form.agama"
							:items="agamaItems"
							placeholder="Pilih agama"
							class="w-full"
							:disabled="saving"
						/>
						<UInput
							v-else-if="f.key === 'tanggal_lahir'"
							v-model="edit.form.tanggal_lahir"
							type="date"
							class="w-full"
							:disabled="saving"
						/>
						<UInput
							v-else-if="f.key === 'nama'"
							ref="namaInput"
							v-model="edit.form.nama"
							:disabled="saving"
						/>
						<UInput
							v-else
							v-model="edit.form[f.key]"
							:disabled="saving"
							:class="['nisn', 'nipd'].includes(f.key) ? 'font-mono' : ''"
						/>
					</UFormField>
				</div>

				<UFormField label="Alasan koreksi" help="Opsional, mis. &quot;Nama salah ketik di Dapodik, sesuai akta kelahiran&quot;">
					<UTextarea
						v-model="edit.catatan"
						:rows="2"
						autoresize
						class="w-full"
						:disabled="saving" />
				</UFormField>

				<p v-if="edit.siswa.updated_at" class="text-xs text-muted">
					Terakhir dikoreksi {{ waktuIndo(edit.siswa.updated_at) }}{{ edit.siswa.updated_by ? ` oleh ${edit.siswa.updated_by}` : "" }}
				</p>
			</form>
		</template>
		<template #footer>
			<div class="flex w-full items-center justify-between gap-2">
				<UButton
					v-if="edit?.siswa.updated_at"
					color="warning"
					variant="ghost"
					icon="lucide:rotate-ccw"
					:disabled="saving"
					@click="kembalikan"
				>
					Kembalikan ke Dapodik
				</UButton>
				<span v-else />
				<div class="flex gap-2">
					<UButton
						color="neutral"
						variant="outline"
						:disabled="saving"
						@click="tutup">
						Batal
					</UButton>
					<UButton
						type="submit"
						form="form-siswa"
						variant="solid"
						icon="lucide:save"
						:loading="saving">
						Simpan
					</UButton>
				</div>
			</div>
		</template>
	</UModal>
</template>
