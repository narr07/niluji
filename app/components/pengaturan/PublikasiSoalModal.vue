<script lang="ts" setup>
	import type { Katalog, PaketEntry } from "~/composables/useBankSoalGugus";
	import { invoke } from "@tauri-apps/api/core";
	import { BANK_SOAL_REPO, SEMUA_GUGUS } from "~/composables/useBankSoalGugus";

	// Khusus pembuat soal: menerbitkan paket soal per gugus ke repo bank soal di GitHub. Semua
	// penulisan ke GitHub dikerjakan backend memakai token yang tersimpan di penyimpanan kunci
	// sistem operasi; tanpa token yang punya hak tulis, GitHub menolak dan tombolnya tidak muncul.

	const props = defineProps<{
		open: boolean
	}>();

	const emit = defineEmits<{
		"update:open": [value: boolean]
	}>();

	const openModel = computed({
		get: () => props.open,
		set: (value) => emit("update:open", value)
	});

	interface Subject {
		id: number
		name: string
		code: string | null
	}

	interface QuestionTypeRecord {
		id: number
		name: string
	}

	interface PublishPlan {
		folder: string
		pg: number
		esai: number
		gambar: number
		menggantikan: boolean
		mapelKode: string
		mapelNama: string
	}

	interface PublishResult {
		folder: string
		file: number
		menggantikan: boolean
		commit: string
	}

	const toast = useToast();
	const errorText = (e: unknown) => (e instanceof Error ? e.message : String(e));

	// ---------- Akses pembuat soal ----------
	const tokenTersimpan = ref(false);
	const tokenInput = ref("");
	const savingToken = ref(false);
	const tokenError = ref("");

	// ---------- Wilayah & gugus ----------
	const katalog = ref<Katalog | null>(null);
	const loadingKatalog = ref(false);
	const kecamatanNama = ref("");
	const gugusNama = ref<string[]>(["Gugus 1", "Gugus 2", "Gugus 3"]);
	const savingWilayah = ref(false);
	const wilayahError = ref("");
	const kecamatan = computed(() => katalog.value?.kecamatan[0]);

	// ---------- Publikasi ----------
	const classes = ref<string[]>([]);
	const subjects = ref<Subject[]>([]);
	const jenisOptions = ref<QuestionTypeRecord[]>([]);
	const kelas = ref("");
	const subjectId = ref<number | undefined>();
	const jenis = ref("");
	const gugus = ref("");
	const judul = ref("");
	const plan = ref<PublishPlan | null>(null);
	const checking = ref(false);
	const publishing = ref(false);
	const publishError = ref("");
	const confirmOpen = ref(false);

	// ---------- Hapus paket ----------
	const hapusTarget = ref<PaketEntry | null>(null);
	const hapusOpen = ref(false);
	const deleting = ref(false);
	const hapusError = ref("");

	const subjectItems = computed(() => subjects.value.map((s) => ({ label: s.code ? `${s.code} - ${s.name}` : s.name, value: s.id })));
	const gugusItems = computed(() => [
		...(kecamatan.value?.gugus ?? []).map((g) => ({ label: g.nama, value: g.id })),
		{ label: "Semua gugus", value: SEMUA_GUGUS }
	]);
	const gugusLabel = (id: string) => (id === SEMUA_GUGUS ? "Semua gugus" : kecamatan.value?.gugus.find((g) => g.id === id)?.nama ?? id);
	const request = computed(() => ({
		kelas: kelas.value,
		subjectId: subjectId.value,
		jenis: jenis.value,
		kecamatan: kecamatan.value?.id ?? "",
		gugus: gugus.value,
		judul: judul.value.trim()
	}));
	const canCheck = computed(() => Boolean(kecamatan.value && kelas.value && subjectId.value && jenis.value && gugus.value && judul.value.trim()));
	const paketTerbit = computed<PaketEntry[]>(() => (katalog.value?.paket ?? []).filter((p) => p.kecamatan === kecamatan.value?.id));

	const loadKatalog = async () => {
		loadingKatalog.value = true;
		wilayahError.value = "";
		try {
			katalog.value = await invoke<Katalog>("bank_soal_katalog_terbaru");
			if (kecamatan.value) {
				kecamatanNama.value = kecamatan.value.nama;
				gugusNama.value = kecamatan.value.gugus.map((g) => g.nama);
			}
		} catch (e) {
			wilayahError.value = errorText(e);
		} finally {
			loadingKatalog.value = false;
		}
	};

	const loadStatus = async () => {
		const status = await invoke<{ tokenTersimpan: boolean }>("bank_soal_status");
		tokenTersimpan.value = status.tokenTersimpan;
		if (tokenTersimpan.value) await loadKatalog();
	};

	watch(
		() => props.open,
		async (isOpen) => {
			if (!isOpen) return;
			tokenInput.value = "";
			tokenError.value = "";
			publishError.value = "";
			plan.value = null;
			[classes.value, subjects.value] = await Promise.all([
				invoke<string[]>("list_classes"),
				invoke<Subject[]>("list_subjects")
			]);
			await loadStatus();
		},
		{ immediate: true }
	);

	watch([kelas, subjectId], async () => {
		jenis.value = "";
		jenisOptions.value = kelas.value && subjectId.value
			? await invoke<QuestionTypeRecord[]>("list_question_types", { class: kelas.value, subjectId: subjectId.value })
			: [];
	});
	// Nama untuk sekolah diisi dari jenis lokal, lalu bisa diubah (mis. "STS Ganjil G1" -> "STS Ganjil").
	watch(jenis, (value) => {
		judul.value = value;
	});
	watch(request, () => {
		plan.value = null;
	}, { deep: true });

	const saveToken = async () => {
		savingToken.value = true;
		tokenError.value = "";
		try {
			await invoke("bank_soal_simpan_token", { token: tokenInput.value });
			tokenInput.value = "";
			toast.add({ title: "Akses pembuat soal aktif", description: `Token punya hak tulis ke ${BANK_SOAL_REPO}.`, icon: "lucide:check", color: "success" });
			await loadStatus();
		} catch (e) {
			tokenError.value = errorText(e);
		} finally {
			savingToken.value = false;
		}
	};

	const removeToken = async () => {
		await invoke("bank_soal_hapus_token");
		tokenTersimpan.value = false;
		katalog.value = null;
		toast.add({ title: "Token dihapus dari komputer ini", icon: "lucide:check", color: "neutral" });
	};

	const saveWilayah = async () => {
		savingWilayah.value = true;
		wilayahError.value = "";
		try {
			katalog.value = await invoke<Katalog>("bank_soal_simpan_wilayah", { kecamatan: kecamatanNama.value, gugus: gugusNama.value });
			toast.add({ title: "Daftar gugus disimpan", description: "Sekolah kini bisa memilih gugus ini.", icon: "lucide:check", color: "success" });
		} catch (e) {
			wilayahError.value = errorText(e);
		} finally {
			savingWilayah.value = false;
		}
	};

	const checkPlan = async () => {
		checking.value = true;
		publishError.value = "";
		try {
			plan.value = await invoke<PublishPlan>("bank_soal_rencana", { req: request.value });
			confirmOpen.value = true;
		} catch (e) {
			publishError.value = errorText(e);
		} finally {
			checking.value = false;
		}
	};

	const askHapus = (paket: PaketEntry) => {
		hapusTarget.value = paket;
		hapusError.value = "";
		hapusOpen.value = true;
	};

	const hapusPaket = async () => {
		const target = hapusTarget.value;
		if (!target) return;
		deleting.value = true;
		hapusError.value = "";
		try {
			katalog.value = await invoke<Katalog>("bank_soal_hapus_paket", { folder: target.folder });
			hapusOpen.value = false;
			plan.value = null;
			toast.add({
				title: "Paket soal dihapus",
				description: `${target.jenis} kelas ${target.kelas} untuk ${gugusLabel(target.gugus)} tidak lagi muncul di sekolah.`,
				icon: "lucide:check",
				color: "success"
			});
		} catch (e) {
			hapusError.value = errorText(e);
		} finally {
			deleting.value = false;
		}
	};

	const publish = async () => {
		publishing.value = true;
		publishError.value = "";
		try {
			const result = await invoke<PublishResult>("bank_soal_publikasikan", { req: request.value });
			confirmOpen.value = false;
			toast.add({
				title: result.menggantikan ? "Paket soal diperbarui" : "Paket soal dipublikasikan",
				description: `${judul.value.trim()} untuk ${gugusLabel(gugus.value)} sudah bisa ditarik sekolah.`,
				icon: "lucide:check",
				color: "success"
			});
			await loadKatalog();
		} catch (e) {
			publishError.value = errorText(e);
			confirmOpen.value = false;
		} finally {
			publishing.value = false;
		}
	};
</script>

<template>
	<UModal
		v-model:open="openModel"
		title="Publikasikan Bank Soal Gugus"
		description="Khusus pembuat soal. Sekolah menarik soal ini lewat Pengaturan › Tarik Soal Online."
		:ui="{ content: 'max-w-3xl' }">
		<template #body>
			<div class="space-y-6">
				<!-- Akses pembuat soal -->
				<section class="space-y-3">
					<h3 class="font-semibold text-highlighted">
						1. Akses pembuat soal
					</h3>
					<div v-if="tokenTersimpan" class="flex flex-wrap items-center justify-between gap-3 rounded-lg border border-default p-3">
						<div class="flex items-center gap-2 text-sm">
							<UIcon name="lucide:shield-check" class="size-5 text-success" />
							<span>Aktif di komputer ini untuk repo <code>{{ BANK_SOAL_REPO }}</code>.</span>
						</div>
						<UButton
							size="sm"
							variant="soft"
							color="neutral"
							icon="lucide:log-out"
							@click="removeToken">
							Hapus token
						</UButton>
					</div>
					<template v-else>
						<p class="text-sm text-muted">
							Tempel token GitHub milik pembuat soal. Buat di github.com › Settings › Developer settings › Personal access tokens ›
							<b>Fine-grained tokens</b>: Repository access <b>Only select repositories</b> → <code>{{ BANK_SOAL_REPO }}</code>,
							Permissions › Repository › <b>Contents: Read and write</b>. Token disimpan di penyimpanan kunci Windows, bukan di database.
						</p>
						<div class="flex gap-2">
							<UInput
								v-model="tokenInput"
								type="password"
								placeholder="github_pat_..."
								class="flex-1"
								autocomplete="off" />
							<UButton
								icon="lucide:key-round"
								:loading="savingToken"
								:disabled="!tokenInput.trim()"
								@click="saveToken">
								Simpan
							</UButton>
						</div>
						<UAlert
							v-if="tokenError"
							color="error"
							variant="subtle"
							:title="tokenError" />
					</template>
				</section>

				<template v-if="tokenTersimpan">
					<USeparator />

					<!-- Wilayah & gugus -->
					<section class="space-y-3">
						<h3 class="font-semibold text-highlighted">
							2. Kecamatan dan gugus
						</h3>
						<div class="grid gap-4 sm:grid-cols-2">
							<UFormField label="Kecamatan">
								<UInput v-model="kecamatanNama" placeholder="Kecamatan Rajagaluh" class="w-full" />
							</UFormField>
							<UFormField label="Daftar gugus" help="Ketik nama gugus lalu tekan Enter.">
								<UInputTags v-model="gugusNama" placeholder="Gugus 1" class="w-full" />
							</UFormField>
						</div>
						<div class="flex items-center justify-between gap-3">
							<p class="text-xs text-muted">
								Gugus yang masih punya paket terbit tidak bisa dihapus atau diganti namanya. Hapus paketnya dulu di daftar Sudah terbit.
							</p>
							<UButton
								size="sm"
								variant="soft"
								icon="lucide:save"
								:loading="savingWilayah || loadingKatalog"
								@click="saveWilayah">
								Simpan daftar gugus
							</UButton>
						</div>
						<UAlert
							v-if="wilayahError"
							color="error"
							variant="subtle"
							:title="wilayahError" />
					</section>

					<USeparator />

					<!-- Publikasi -->
					<section class="space-y-3">
						<h3 class="font-semibold text-highlighted">
							3. Publikasikan paket soal
						</h3>
						<p v-if="!kecamatan" class="text-sm text-muted">
							Simpan daftar gugus dulu sebelum mempublikasikan.
						</p>
						<template v-else>
							<div class="grid gap-4 sm:grid-cols-3">
								<UFormField label="Kelas">
									<USelect
										v-model="kelas"
										:items="classes"
										placeholder="Pilih kelas"
										class="w-full" />
								</UFormField>
								<UFormField label="Mata pelajaran">
									<USelectMenu
										v-model="subjectId"
										:items="subjectItems"
										value-key="value"
										placeholder="Pilih mapel"
										class="w-full" />
								</UFormField>
								<UFormField label="Jenis ujian di bank soal Anda">
									<USelectMenu
										v-model="jenis"
										:items="jenisOptions.map((j) => j.name)"
										:disabled="!kelas || !subjectId"
										placeholder="Pilih jenis"
										class="w-full" />
								</UFormField>
							</div>
							<div class="grid gap-4 sm:grid-cols-2">
								<UFormField label="Gugus tujuan">
									<USelect
										v-model="gugus"
										:items="gugusItems"
										placeholder="Pilih gugus"
										class="w-full" />
								</UFormField>
								<UFormField label="Nama yang dilihat sekolah" help="Contoh: STS Ganjil 2026/2027">
									<UInput v-model="judul" class="w-full" />
								</UFormField>
							</div>
							<UAlert
								color="warning"
								variant="subtle"
								icon="lucide:triangle-alert"
								description="Repo bank soal bersifat publik dan file soal memuat kunci jawaban. Publikasikan setelah soal siap dibagikan ke sekolah." />
							<div class="flex justify-end">
								<UButton
									icon="lucide:upload-cloud"
									:loading="checking"
									:disabled="!canCheck"
									@click="checkPlan">
									Periksa &amp; Publikasikan
								</UButton>
							</div>
							<UAlert
								v-if="publishError"
								color="error"
								variant="subtle"
								title="Gagal mempublikasikan"
								:description="publishError" />
						</template>
					</section>

					<template v-if="paketTerbit.length">
						<USeparator />
						<section class="space-y-2">
							<h3 class="font-semibold text-highlighted">
								Sudah terbit ({{ paketTerbit.length }})
							</h3>
							<ul class="max-h-56 overflow-y-auto divide-y divide-default rounded-lg border border-default text-sm">
								<li v-for="p in paketTerbit" :key="p.folder" class="flex flex-wrap items-center justify-between gap-2 px-3 py-2">
									<span>
										<UBadge color="neutral" variant="subtle" class="mr-2">{{ gugusLabel(p.gugus) }}</UBadge>
										Kelas {{ p.kelas }} · {{ p.mapelKode }} · <b>{{ p.jenis }}</b>
									</span>
									<span class="flex items-center gap-2">
										<span class="text-xs text-muted">{{ p.pg }} PG, {{ p.esai }} esai</span>
										<UButton
											size="xs"
											color="error"
											variant="ghost"
											icon="lucide:trash-2"
											:aria-label="`Hapus ${p.jenis} kelas ${p.kelas}`"
											@click="askHapus(p)" />
									</span>
								</li>
							</ul>
						</section>
					</template>
				</template>

				<UModal
					v-model:open="confirmOpen"
					title="Publikasikan paket soal ini?"
					:dismissible="!publishing"
					:ui="{ content: 'max-w-md', footer: 'justify-end' }">
					<template #body>
						<div v-if="plan" class="space-y-3 text-sm">
							<p>
								<b>{{ judul.trim() }}</b> · Kelas {{ kelas }} · {{ plan.mapelKode }} - {{ plan.mapelNama }}
							</p>
							<p>
								Untuk <b>{{ gugusLabel(gugus) }}</b>: {{ plan.pg }} soal PG, {{ plan.esai }} soal esai, {{ plan.gambar }} gambar.
							</p>
							<p class="font-mono text-xs text-muted break-all">
								{{ plan.folder }}/soal.md
							</p>
							<UAlert
								v-if="plan.menggantikan"
								color="warning"
								variant="subtle"
								icon="lucide:refresh-cw"
								title="Menggantikan paket yang sudah terbit"
								description="Sekolah yang menarik setelah ini mendapat versi baru. Soal yang sudah ditarik sekolah tidak berubah." />
						</div>
					</template>
					<template #footer>
						<UButton
							label="Tidak"
							color="neutral"
							variant="outline"
							:disabled="publishing"
							@click="confirmOpen = false" />
						<UButton
							label="Ya, publikasikan"
							icon="lucide:upload-cloud"
							:loading="publishing"
							@click="publish" />
					</template>
				</UModal>

				<UModal
					v-model:open="hapusOpen"
					title="Hapus paket soal ini?"
					:dismissible="!deleting"
					:ui="{ content: 'max-w-md', footer: 'justify-end' }">
					<template #body>
						<div v-if="hapusTarget" class="space-y-3 text-sm">
							<p>
								<b>{{ hapusTarget.jenis }}</b> · Kelas {{ hapusTarget.kelas }} · {{ hapusTarget.mapelKode }} - {{ hapusTarget.mapelNama }}
							</p>
							<p>
								Untuk <b>{{ gugusLabel(hapusTarget.gugus) }}</b>: {{ hapusTarget.pg }} soal PG, {{ hapusTarget.esai }} soal esai.
							</p>
							<p class="text-muted">
								File soal dan gambarnya dihapus dari repo bank soal, dan paket ini hilang dari daftar Tarik Soal Online.
								Soal yang sudah ditarik sekolah tetap ada di bank soal sekolah itu. Riwayat GitHub masih menyimpan isi lamanya.
							</p>
							<UAlert
								v-if="hapusError"
								color="error"
								variant="subtle"
								title="Gagal menghapus"
								:description="hapusError" />
						</div>
					</template>
					<template #footer>
						<UButton
							label="Tidak"
							color="neutral"
							variant="outline"
							:disabled="deleting"
							@click="hapusOpen = false" />
						<UButton
							label="Ya, hapus"
							color="error"
							icon="lucide:trash-2"
							:loading="deleting"
							@click="hapusPaket" />
					</template>
				</UModal>
			</div>
		</template>
	</UModal>
</template>
