<script setup lang="ts">
	// Padanan e-Rapor: Data Logo dan TTD. Gambar dikecilkan otomatis lalu disimpan di database
	// (ikut backup). Dipakai di sampul pelengkap dan tanda tangan kepala sekolah di rapor.

	type Jenis = keyof typeof KUNCI_GAMBAR;

	const db = useDb();
	const toast = useToast();
	const swal = useSwal();

	const SLOT: { jenis: Jenis, judul: string, ket: string, ukuran: [number, number], icon: string }[] = [
		{ jenis: "logo", judul: "Logo Sampul", ket: "Tampil di sampul pelengkap rapor (mis. logo Tut Wuri Handayani atau logo sekolah).", ukuran: [600, 600], icon: "lucide:image" },
		{ jenis: "ttdKepsek", judul: "Tanda Tangan Kepala Sekolah", ket: "Pakai gambar berlatar putih/transparan. Tampil di atas nama kepala sekolah.", ukuran: [600, 300], icon: "lucide:signature" },
		{ jenis: "stempel", judul: "Stempel Sekolah", ket: "PNG transparan paling bagus. Tampil menimpa sebagian tanda tangan kepala sekolah.", ukuran: [400, 400], icon: "lucide:stamp" }
	];

	const gambar = ref<GambarRapor>({});
	const loading = ref(true);
	const proses = ref<Jenis | null>(null);
	const input = ref<Partial<Record<Jenis, HTMLInputElement>>>({});

	async function load() {
		loading.value = true;
		try {
			gambar.value = await muatGambarRapor(db);
		}
		catch (e) {
			toast.add({ title: "Gagal memuat gambar", description: pesanError(e), color: "error" });
		}
		finally {
			loading.value = false;
		}
	}
	onMounted(() => Promise.all([load(), loadKop(), loadWali()]));

	// ===== Kop surat =====
	const kop = ref<Kop>({ baris: [] });
	const kopInput = ref<Partial<Record<"kiri" | "kanan", HTMLInputElement>>>({});
	const KUNCI_KOP = { kiri: "gambar_kop_kiri", kanan: "gambar_kop_kanan" } as const;
	async function loadKop() {
		try {
			kop.value = await muatKop(db);
			while (kop.value.baris.length < KOP_BARIS)
				kop.value.baris.push("");
		}
		catch (e) {
			toast.add({ title: "Gagal memuat kop", description: pesanError(e), color: "error" });
		}
	}
	let kopTimer: ReturnType<typeof setTimeout> | undefined;
	function ubahBaris(i: number, v: string) {
		kop.value.baris[i] = v;
		clearTimeout(kopTimer);
		kopTimer = setTimeout(() => db.setSetting("kop_baris", JSON.stringify(kop.value.baris)).catch(e => toast.add({ title: "Gagal menyimpan kop", description: pesanError(e), color: "error" })), 500);
	}
	async function kembalikanKop() {
		await db.execute("DELETE FROM settings WHERE key = 'kop_baris'");
		await loadKop();
	}
	async function pilihLogoKop(sisi: "kiri" | "kanan", e: Event) {
		const el = e.target as HTMLInputElement;
		const file = el.files?.[0];
		el.value = "";
		if (!file)
			return;
		try {
			const data = await kecilkanGambar(file, 400, 400);
			await db.setSetting(KUNCI_KOP[sisi], data);
			kop.value = { ...kop.value, [sisi]: data };
		}
		catch (err) {
			toast.add({ title: "Gagal menyimpan logo", description: pesanError(err), color: "error" });
		}
	}
	async function hapusLogoKop(sisi: "kiri" | "kanan") {
		await db.execute("DELETE FROM settings WHERE key = ?", [KUNCI_KOP[sisi]]);
		kop.value = { ...kop.value, [sisi]: undefined };
	}

	// ===== Tanda tangan wali kelas (per guru) =====
	interface Wali { ptkId: string, nama: string, kelas: string, ttd?: string }
	const wali = ref<Wali[]>([]);
	const waliInput = ref<HTMLInputElement>();
	const waliTarget = ref<Wali>();
	async function loadWali() {
		try {
			const rows = await db.query<{ ptkId: string, nama: string, gelar_depan: string | null, gelar_belakang: string | null, kelas: string }>(
				`SELECT g.ptk_id AS ptkId, g.nama, g.gelar_depan, g.gelar_belakang, group_concat(r.nama, ', ') AS kelas
				FROM rombel r JOIN ptk g ON g.ptk_id = r.ptk_id WHERE r.jenis_rombel = '1' AND r.semester_id = ?
				GROUP BY g.ptk_id ORDER BY MIN(CAST(r.tingkat AS INTEGER))`,
				[useAuth().session.value?.semesterId]
			);
			wali.value = await Promise.all(rows.map(async w => ({ ptkId: w.ptkId, nama: namaLengkap(w), kelas: w.kelas, ttd: (await db.getSetting(kunciTtdGuru(w.ptkId))) || undefined })));
		}
		catch (e) {
			toast.add({ title: "Gagal memuat wali kelas", description: pesanError(e), color: "error" });
		}
	}
	function pilihTtdWali(w: Wali) {
		waliTarget.value = w;
		waliInput.value?.click();
	}
	async function unggahTtdWali(e: Event) {
		const el = e.target as HTMLInputElement;
		const file = el.files?.[0];
		el.value = "";
		const w = waliTarget.value;
		if (!file || !w)
			return;
		try {
			const data = await kecilkanGambar(file, 600, 300);
			await db.setSetting(kunciTtdGuru(w.ptkId), data);
			w.ttd = data;
		}
		catch (err) {
			toast.add({ title: "Gagal menyimpan tanda tangan", description: pesanError(err), color: "error" });
		}
	}
	async function hapusTtdWali(w: Wali) {
		await db.execute("DELETE FROM settings WHERE key = ?", [kunciTtdGuru(w.ptkId)]);
		w.ttd = undefined;
	}

	async function pilih(jenis: Jenis, e: Event) {
		const el = e.target as HTMLInputElement;
		const file = el.files?.[0];
		el.value = "";
		if (!file)
			return;
		const slot = SLOT.find(s => s.jenis === jenis)!;
		proses.value = jenis;
		try {
			const data = await kecilkanGambar(file, ...slot.ukuran, jenis === "logo" && file.type === "image/jpeg" ? "image/jpeg" : "image/png");
			await db.setSetting(KUNCI_GAMBAR[jenis], data);
			gambar.value = { ...gambar.value, [jenis]: data };
			toast.add({ title: `${slot.judul} tersimpan`, color: "success" });
		}
		catch (err) {
			toast.add({ title: "Gagal menyimpan gambar", description: pesanError(err), color: "error" });
		}
		finally {
			proses.value = null;
		}
	}

	async function hapus(jenis: Jenis) {
		const slot = SLOT.find(s => s.jenis === jenis)!;
		if (!await swal.confirm(`Hapus ${slot.judul}?`, "Rapor dicetak tanpa gambar ini.", "Hapus"))
			return;
		try {
			await db.execute("DELETE FROM settings WHERE key = ?", [KUNCI_GAMBAR[jenis]]);
			gambar.value = { ...gambar.value, [jenis]: undefined };
		}
		catch (e) {
			toast.add({ title: "Gagal menghapus", description: pesanError(e), color: "error" });
		}
	}
</script>

<template>
	<ErPage id="logo-ttd" title="Logo & Tanda Tangan">
		<div class="space-y-4">
			<UAlert
				color="neutral"
				variant="subtle"
				icon="lucide:info"
				description="Gambar dikecilkan otomatis dan disimpan di database (ikut backup). Tanda tangan & stempel tampil di rapor kalau pilihan &quot;Tampilkan nama & NIP&quot; di halaman Cetak Rapor aktif."
			/>
			<div class="grid gap-4 lg:grid-cols-3">
				<PanelCard
					v-for="s in SLOT"
					:key="s.jenis"
					:title="s.judul"
					:icon="s.icon"
					:description="s.ket">
					<div class="flex flex-col gap-3">
						<USkeleton v-if="loading" class="h-40 w-full" />
						<div
							v-else
							class="flex h-40 items-center justify-center rounded-md border border-dashed border-default bg-white p-2"
						>
							<img
								v-if="gambar[s.jenis]"
								:src="gambar[s.jenis]"
								:alt="s.judul"
								class="max-h-full max-w-full object-contain">
							<span v-else class="text-sm text-muted">Belum ada gambar</span>
						</div>
						<input
							:ref="el => { if (el) input[s.jenis] = el as HTMLInputElement }"
							type="file"
							accept="image/png,image/jpeg"
							class="hidden"
							:aria-label="`Pilih file ${s.judul}`"
							@change="pilih(s.jenis, $event)"
						>
						<div class="flex gap-2">
							<UButton
								class="flex-1 justify-center"
								icon="lucide:upload"
								variant="soft"
								:loading="proses === s.jenis"
								@click="input[s.jenis]?.click()">
								{{ gambar[s.jenis] ? "Ganti" : "Unggah" }}
							</UButton>
							<UButton
								v-if="gambar[s.jenis]"
								icon="lucide:trash-2"
								variant="ghost"
								color="error"
								:aria-label="`Hapus ${s.judul}`"
								@click="hapus(s.jenis)"
							/>
						</div>
					</div>
				</PanelCard>
			</div>

			<PanelCard title="Kop Surat" icon="lucide:panel-top" description="Dipakai di pelengkap rapor dan transkrip ijazah (kalau pilihan &quot;Pakai kop&quot; aktif saat cetak). Logo diunggah, teks diketik sendiri.">
				<div class="grid gap-4 lg:grid-cols-[1fr_1fr]">
					<div class="space-y-3">
						<div class="flex gap-3">
							<div v-for="sisi in (['kiri', 'kanan'] as const)" :key="sisi" class="flex flex-1 flex-col gap-2">
								<p class="text-xs text-muted">
									Logo {{ sisi }} {{ sisi === "kiri" ? "(mis. pemda)" : "(opsional, mis. sekolah)" }}
								</p>
								<div class="flex h-24 items-center justify-center rounded-md border border-dashed border-default bg-white p-1">
									<img
										v-if="kop[sisi]"
										:src="kop[sisi]"
										:alt="`Logo ${sisi}`"
										class="max-h-full max-w-full object-contain">
									<span v-else class="text-xs text-muted">Kosong</span>
								</div>
								<input
									:ref="el => { if (el) kopInput[sisi] = el as HTMLInputElement }"
									type="file"
									accept="image/png,image/jpeg"
									class="hidden"
									:aria-label="`Pilih logo ${sisi}`"
									@change="pilihLogoKop(sisi, $event)">
								<div class="flex gap-1">
									<UButton
										size="xs"
										variant="soft"
										icon="lucide:upload"
										class="flex-1 justify-center"
										@click="kopInput[sisi]?.click()">
										{{ kop[sisi] ? "Ganti" : "Unggah" }}
									</UButton>
									<UButton
										v-if="kop[sisi]"
										size="xs"
										variant="ghost"
										color="error"
										icon="lucide:trash-2"
										:aria-label="`Hapus logo ${sisi}`"
										@click="hapusLogoKop(sisi)" />
								</div>
							</div>
						</div>
						<UFormField v-for="(b, i) in kop.baris" :key="i" :label="['Baris 1 (instansi)', 'Baris 2', 'Baris 3 (nama sekolah, huruf besar)', 'Baris 4 (alamat, huruf kecil)'][i]">
							<UInput :model-value="b" class="w-full" @update:model-value="ubahBaris(i, String($event))" />
						</UFormField>
						<UButton
							size="xs"
							variant="link"
							color="neutral"
							icon="lucide:rotate-ccw"
							@click="kembalikanKop">
							Kembalikan teks bawaan dari data sekolah
						</UButton>
					</div>
					<div>
						<p class="mb-2 text-xs text-muted">
							Pratinjau
						</p>
						<div class="rounded-md bg-white p-4">
							<KopSurat :kop="kop" />
						</div>
					</div>
				</div>
			</PanelCard>

			<PanelCard title="Tanda Tangan Wali Kelas" icon="lucide:signature" description="Tampil di atas nama wali kelas pada rapor (kalau pilihan nama & NIP aktif).">
				<input
					ref="waliInput"
					type="file"
					accept="image/png,image/jpeg"
					class="hidden"
					aria-label="Pilih tanda tangan wali kelas"
					@change="unggahTtdWali">
				<ul class="divide-y divide-default">
					<li v-for="w in wali" :key="w.ptkId" class="flex items-center gap-3 py-2">
						<div class="min-w-0 flex-1">
							<p class="truncate text-sm font-medium">
								{{ w.nama }}
							</p>
							<p class="text-xs text-muted">
								{{ w.kelas }}
							</p>
						</div>
						<div class="flex h-12 w-32 items-center justify-center rounded border border-dashed border-default bg-white">
							<img
								v-if="w.ttd"
								:src="w.ttd"
								:alt="`Tanda tangan ${w.nama}`"
								class="max-h-full max-w-full object-contain">
							<span v-else class="text-xs text-muted">–</span>
						</div>
						<UButton
							size="xs"
							variant="soft"
							icon="lucide:upload"
							@click="pilihTtdWali(w)">
							{{ w.ttd ? "Ganti" : "Unggah" }}
						</UButton>
						<UButton
							v-if="w.ttd"
							size="xs"
							variant="ghost"
							color="error"
							icon="lucide:trash-2"
							:aria-label="`Hapus tanda tangan ${w.nama}`"
							@click="hapusTtdWali(w)" />
					</li>
				</ul>
				<p v-if="!wali.length" class="text-sm text-muted">
					Belum ada wali kelas di semester ini.
				</p>
			</PanelCard>
		</div>
	</ErPage>
</template>
