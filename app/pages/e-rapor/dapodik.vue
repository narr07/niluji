<script setup lang="ts">
	import type { WebService } from "~/composables/useDapodikSync";

	// Gabungan menu "Web Service Dapodik" + "Ambil Data Dapodik" di e-Rapor. Tiga proses yang di
	// e-Rapor dijalankan manual satu per satu (Tes Koneksi → Ambil Semester → Ambil Seluruh Data)
	// di sini jalan berurutan dengan satu tombol.

	const db = useDb();
	const swal = useSwal();
	const toast = useToast();
	const { session, updateSession } = useAuth();
	const { refresh } = useAppData();
	const { testKoneksi, ambilSemester, ambilSemua } = useDapodikSync();
	const { susun } = usePembelajaranRapor();

	const empty = (): WebService => ({ id: 0, nama_aplikasi: "e-Rapor", ip_erapor: "localhost", ip_dapodik: "localhost", port: 5774, token: "", npsn: "" });
	const ws = ref<WebService>(empty());
	const showToken = ref(false);
	const testing = ref(false);
	const koneksi = ref<{ ok: boolean, text: string }>();

	const semesterId = ref(session.value?.semesterId ?? currentSemesterId());
	const semesters = semesterOptions();

	type StepState = "idle" | "run" | "ok" | "error";
	const steps = ref<{ label: string, state: StepState, detail?: string }[]>([]);
	const running = ref(false);
	const counts = ref<Record<string, { before: number, after: number }>>();
	const lastSync = ref<{ semesterId: string, at: string }>();

	const pesan = pesanError;

	async function load() {
		try {
			const row = await db.first<WebService>("SELECT * FROM webservice ORDER BY id LIMIT 1");
			if (row)
				ws.value = row;
			const raw = await db.getSetting("last_sync");
			try {
				lastSync.value = raw ? JSON.parse(raw) : undefined;
			}
			catch {
				lastSync.value = undefined; // isi setting rusak: abaikan, akan tertulis ulang saat sinkron
			}
		}
		catch (e) {
			toast.add({ title: "Gagal memuat koneksi", description: pesan(e), color: "error" });
		}
	}
	onMounted(load);

	async function save(silent = false) {
		const w = ws.value;
		if (!w.ip_dapodik.trim() || !w.token.trim() || !w.npsn.trim()) {
			swal.error("Data belum lengkap", "IP Dapodik, Key Web Service, dan NPSN wajib diisi.");
			return false;
		}
		try {
			const params = [w.nama_aplikasi.trim() || "e-Rapor", w.ip_erapor.trim() || "localhost", w.ip_dapodik.trim(), Number(w.port) || 5774, w.token.trim(), w.npsn.trim()];
			if (w.id)
				await db.execute("UPDATE webservice SET nama_aplikasi = ?, ip_erapor = ?, ip_dapodik = ?, port = ?, token = ?, npsn = ? WHERE id = ?", [...params, w.id]);
			else
				await db.execute("INSERT INTO webservice (nama_aplikasi, ip_erapor, ip_dapodik, port, token, npsn) VALUES (?, ?, ?, ?, ?, ?)", params);
			await load();
			if (!silent)
				toast.add({ title: "Koneksi disimpan", color: "success" });
			return true;
		}
		catch (e) {
			toast.add({ title: "Gagal menyimpan koneksi", description: pesan(e), color: "error" });
			return false;
		}
	}

	async function tes() {
		if (!await save(true))
			return;
		testing.value = true;
		try {
			koneksi.value = { ok: true, text: await testKoneksi(ws.value) };
		}
		catch (e) {
			koneksi.value = { ok: false, text: pesan(e) };
		}
		finally {
			testing.value = false;
		}
	}

	async function snapshot(sem: string) {
		const [guru, siswa, rombel, pembelajaran] = await Promise.all([
			db.scalar("SELECT COUNT(*) FROM ptk"),
			db.scalar("SELECT COUNT(DISTINCT peserta_didik_id) FROM anggota_rombel WHERE semester_id = ?", [sem]),
			db.scalar("SELECT COUNT(*) FROM rombel WHERE semester_id = ?", [sem]),
			db.scalar("SELECT COUNT(*) FROM pembelajaran_rapor WHERE semester_id = ?", [sem])
		]);
		return { guru, siswa, rombel, pembelajaran } as Record<string, number>;
	}

	// Admin yang login sebelum ada data belum punya sekolah/semester di sesinya — isi otomatis.
	async function fillSession() {
		if (!session.value)
			return;
		const sekolah = await db.first<{ sekolah_id: string, nama: string }>("SELECT sekolah_id, nama FROM sekolah LIMIT 1");
		updateSession({
			...(!session.value.sekolahId && sekolah ? { sekolahId: sekolah.sekolah_id, sekolahNama: sekolah.nama } : {}),
			...(!session.value.semesterId ? { semesterId: semesterId.value } : {})
		});
	}

	async function sinkron() {
		if (!await save(true))
			return;
		const sem = semesterId.value;
		steps.value = [
			{ label: "Tes koneksi Dapodik", state: "idle" },
			{ label: `Ambil semester ${semesterLabel(sem)}`, state: "idle" },
			{ label: "Ambil seluruh data", state: "idle" },
			{ label: "Susun pembelajaran rapor", state: "idle" }
		];
		counts.value = undefined;
		running.value = true;
		const run = async (i: number, fn: () => Promise<string>) => {
			const step = steps.value[i]!;
			step.state = "run";
			try {
				step.detail = await fn();
				step.state = "ok";
			}
			catch (e) {
				step.state = "error";
				step.detail = pesan(e);
				throw e;
			}
		};
		try {
			// Di dalam try: kalau gagal, tombol tidak berputar terus (running dikembalikan di finally).
			const before = await snapshot(sem);
			await run(0, async () => {
				const t = await testKoneksi(ws.value);
				koneksi.value = { ok: true, text: t };
				return t;
			});
			await run(1, () => ambilSemester(sem));
			await run(2, async () => {
				const r = await ambilSemua(sem, (m) => { steps.value[2]!.detail = m; });
				return `${r.guru} guru, ${r.siswa} siswa, ${r.rombel} rombel, ${r.pembelajaran} pembelajaran`;
			});
			// Guru kelas SD dipecah per mapel; aman diulang (pengampu manual & nilai tidak disentuh).
			await run(3, async () => {
				const r = await susun(sem);
				return `${r.pembelajaran} pembelajaran untuk ${r.kelas} kelas${r.tanpaGuruKelas.length ? ` (tanpa Guru Kelas di Dapodik: ${r.tanpaGuruKelas.join(", ")})` : ""}`;
			});
			const after = await snapshot(sem);
			counts.value = Object.fromEntries(Object.keys(after).map(k => [k, { before: before[k] ?? 0, after: after[k] ?? 0 }]));
			await fillSession();
			await refresh();
			await load();
			toast.add({ title: "Sinkron selesai", description: `Data ${semesterLabel(sem)} sudah terbaru`, color: "success" });
		}
		catch (e) {
			toast.add({ title: "Sinkron gagal", description: pesan(e), color: "error" });
		}
		finally {
			running.value = false;
		}
	}

	const selesai = computed(() => steps.value.filter(s => s.state === "ok").length);

	const STEP_ICON: Record<StepState, string> = {
		idle: "lucide:circle",
		run: "lucide:loader-circle",
		ok: "lucide:circle-check",
		error: "lucide:circle-x"
	};
	const STEP_CLASS: Record<StepState, string> = {
		idle: "text-dimmed",
		run: "text-primary animate-spin",
		ok: "text-success",
		error: "text-error"
	};

	const delta = (c: { before: number, after: number }) => {
		const d = c.after - c.before;
		return d === 0 ? "tetap" : d > 0 ? `+${d}` : String(d);
	};
	const LABEL: Record<string, string> = { guru: "Guru", siswa: "Siswa", rombel: "Rombel", pembelajaran: "Mapel Rapor" };
</script>

<template>
	<ErPage id="dapodik" title="Sinkron Dapodik">
		<template #right>
			<UBadge
				v-if="koneksi"
				:color="koneksi.ok ? 'success' : 'error'"
				variant="subtle"
				:icon="koneksi.ok ? 'lucide:circle-check' : 'lucide:circle-x'"
			>
				{{ koneksi.ok ? "Terhubung" : "Gagal terhubung" }}
			</UBadge>
		</template>

		<div class="grid gap-4 xl:grid-cols-2">
			<PanelCard title="Koneksi Web Service" icon="lucide:plug" description="Isi sesuai menu Pengaturan → Web Service di Dapodik">
				<form class="flex flex-col gap-3" @submit.prevent="save()">
					<div class="grid grid-cols-[1fr_100px] gap-2">
						<UFormField label="IP Server Dapodik" help="Satu komputer dengan Dapodik: localhost">
							<UInput v-model="ws.ip_dapodik" icon="lucide:server" :disabled="running" />
						</UFormField>
						<UFormField label="Port">
							<UInput
								v-model.number="ws.port"
								type="number"
								:min="1"
								:max="65535"
								:disabled="running" />
						</UFormField>
					</div>
					<UFormField label="Key Web Service" help="Salin dari Dapodik lewat tombol Salin Token">
						<UInput
							v-model="ws.token"
							:type="showToken ? 'text' : 'password'"
							icon="lucide:key-round"
							autocomplete="off"
							class="font-mono"
							:disabled="running"
						>
							<template #trailing>
								<UButton
									color="neutral"
									variant="link"
									size="sm"
									:icon="showToken ? 'lucide:eye-off' : 'lucide:eye'"
									:aria-label="showToken ? 'Sembunyikan key' : 'Tampilkan key'"
									@click="showToken = !showToken"
								/>
							</template>
						</UInput>
					</UFormField>
					<UFormField label="NPSN Sekolah">
						<UInput v-model="ws.npsn" icon="lucide:school" :disabled="running" />
					</UFormField>

					<UAlert
						v-if="koneksi"
						:color="koneksi.ok ? 'success' : 'error'"
						variant="subtle"
						:icon="koneksi.ok ? 'lucide:circle-check' : 'lucide:circle-x'"
						:description="koneksi.text"
					/>

					<div class="flex justify-end gap-2">
						<UButton
							type="submit"
							color="neutral"
							icon="lucide:save"
							:disabled="running">
							Simpan
						</UButton>
						<UButton
							icon="lucide:plug-zap"
							:loading="testing"
							:disabled="running"
							@click="tes">
							Tes Koneksi
						</UButton>
					</div>
				</form>
			</PanelCard>

			<PanelCard title="Tarik Data" icon="lucide:download-cloud" description="Sekolah, guru, rombel, anggota, pembelajaran, dan siswa">
				<div class="flex flex-col gap-4">
					<UFormField label="Semester">
						<USelect
							v-model="semesterId"
							:items="semesters"
							class="w-full"
							:disabled="running" />
					</UFormField>

					<UButton
						block
						size="lg"
						variant="solid"
						icon="lucide:refresh-cw"
						:loading="running"
						:disabled="!ws.token.trim() || running"
						@click="sinkron"
					>
						Sinkronkan Sekarang
					</UButton>

					<UProgress
						v-if="steps.length"
						:model-value="selesai"
						:max="steps.length"
						size="sm" />

					<ol v-if="steps.length" class="flex flex-col gap-2">
						<li v-for="s in steps" :key="s.label" class="flex items-start gap-2 text-sm">
							<UIcon :name="STEP_ICON[s.state]" class="mt-0.5 size-4 shrink-0" :class="STEP_CLASS[s.state]" />
							<div class="min-w-0">
								<p :class="s.state === 'idle' ? 'text-muted' : 'text-highlighted'">
									{{ s.label }}
								</p>
								<p v-if="s.detail" class="text-xs wrap-break-word" :class="s.state === 'error' ? 'text-error' : 'text-muted'">
									{{ s.detail }}
								</p>
							</div>
						</li>
					</ol>

					<div v-if="counts" class="grid grid-cols-2 gap-2 sm:grid-cols-4">
						<div v-for="(c, k) in counts" :key="k" class="rounded-md border border-default p-3">
							<p class="text-xs text-muted uppercase">
								{{ LABEL[k] ?? k }}
							</p>
							<p class="text-xl font-semibold">
								{{ c.after }}
							</p>
							<p class="text-xs" :class="c.after !== c.before ? 'text-primary' : 'text-dimmed'">
								{{ delta(c) }}
							</p>
						</div>
					</div>

					<p v-if="lastSync" class="text-xs text-muted">
						Terakhir sinkron: {{ semesterLabel(lastSync.semesterId) }}, {{ lastSync.at }}
					</p>
				</div>
			</PanelCard>
		</div>
	</ErPage>
</template>
