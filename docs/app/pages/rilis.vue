<script setup lang="ts">
	interface ReleaseAsset {
		name: string;
		browser_download_url: string;
		size: number;
	}

	interface GithubRelease {
		id: number;
		tag_name: string;
		name: string;
		published_at: string;
		html_url: string;
		body: string;
		assets: ReleaseAsset[];
	}

	interface LocalChangelogItem {
		version: string;
		title: string;
		date: string;
		badgeColor: "success" | "primary" | "neutral";
		highlights: string[];
		htmlUrl: string;
		assets: { label: string; icon: string; url: string }[];
	}

	useSeoMeta({
		title: "Rilis & Unduhan Installer",
		description: "Unduh installer aplikasi desktop NILUJI terbaru beserta catatan perubahan versi (changelog)."
	});

	const runtimeConfig = useRuntimeConfig();
	const baseURL = runtimeConfig.app.baseURL || "/";
	const templateUrl = (file: string) => `${baseURL}templates/${file}`.replace("//", "/");

	const detectedOs = ref<"windows" | "linux" | "macos">("windows");

	const fallbackReleases: LocalChangelogItem[] = [
		{
			version: "v1.3.0",
			title: "Keamanan Ujian, Ketepatan Nilai & Perlindungan Hasil",
			date: "2026-11-19",
			badgeColor: "success",
			htmlUrl: "https://github.com/narr07/niluji/releases/tag/v1.3.0",
			assets: [
				{ label: "Niluji_1.3.0_x64-setup.exe", icon: "i-lucide-monitor", url: "https://github.com/narr07/niluji/releases/download/v1.3.0/Niluji_1.3.0_x64-setup.exe" },
				{ label: "Niluji_1.3.0_x64_en-US.msi", icon: "i-lucide-monitor", url: "https://github.com/narr07/niluji/releases/download/v1.3.0/Niluji_1.3.0_x64_en-US.msi" },
				{ label: "Niluji_1.3.0_amd64.deb", icon: "i-lucide-terminal", url: "https://github.com/narr07/niluji/releases/download/v1.3.0/Niluji_1.3.0_amd64.deb" },
				{ label: "Niluji_1.3.0_amd64.AppImage", icon: "i-lucide-terminal", url: "https://github.com/narr07/niluji/releases/download/v1.3.0/Niluji_1.3.0_amd64.AppImage" },
				{ label: "Niluji_1.3.0_aarch64.dmg", icon: "i-lucide-laptop", url: "https://github.com/narr07/niluji/releases/download/v1.3.0/Niluji_1.3.0_aarch64.dmg" },
				{ label: "Niluji_1.3.0_x64.dmg", icon: "i-lucide-laptop", url: "https://github.com/narr07/niluji/releases/download/v1.3.0/Niluji_1.3.0_x64.dmg" }
			],
			highlights: [
				"Setiap sesi siswa punya token rahasia, sehingga siswa tidak bisa mengubah atau mengumpulkan ujian siswa lain.",
				"Ujian baru mewajibkan PIN, siswa hanya bisa masuk ke ujian kelasnya, dan percobaan login yang salah dibatasi.",
				"Bobot nilai dipakai persis seperti yang diisi guru, dan Nilai Akhir menunggu sampai semua jawaban esai dinilai.",
				"Hasil ujian tidak berubah saat Bank Soal diedit. Tombol Nilai Ulang tersedia bila kunci jawaban perlu dibetulkan.",
				"Backup harian 14 hari, backup otomatis sebelum perubahan penting, dan pemulihan database dari menu Fitur.",
				"Sesi yang ditinggalkan siswa dikumpulkan otomatis, dan pengawas bisa membukanya lagi dengan tambahan waktu.",
				"PIN Export ditentukan sendiri oleh sekolah, dan impor Word mendukung opsi E."
			]
		},
		{
			version: "v1.0.1",
			title: "Perbaikan Stabilitas, Impor Markdown & Backup Otomatis",
			date: "2026-09-22",
			badgeColor: "primary",
			htmlUrl: "https://github.com/narr07/niluji/releases/tag/v1.0.1",
			assets: [
				{ label: "Niluji_1.0.1_x64-setup.exe", icon: "i-lucide-monitor", url: "https://github.com/narr07/niluji/releases/download/v1.0.1/Niluji_1.0.1_x64-setup.exe" },
				{ label: "Niluji_1.0.1_x64_en-US.msi", icon: "i-lucide-monitor", url: "https://github.com/narr07/niluji/releases/download/v1.0.1/Niluji_1.0.1_x64_en-US.msi" }
			],
			highlights: [
				"Perbaikan ketahanan sesi ujian ketika terdapat butir soal yang dihapus dari Bank Soal.",
				"Penyimpanan ulang jawaban otomatis (auto-retry) ketika jaringan WiFi lokal sempat terputus sementara.",
				"Dukungan penyisipan gambar di tengah teks pertanyaan maupun pada masing-masing opsi jawaban A-E.",
				"Pemisahan pengaturan acak soal Pilihan Ganda dan acak soal Esai.",
				"Ekspor Hasil Ujian per siswa berisi rincian jawaban per butir soal dan mode arsip .zip satu kelas penuh.",
				"Impor soal mendukung 3 format: Naskah Word (.docx), CSV/Excel, dan paket Markdown (.md / .zip).",
				"Backup database SQLite otomatis setiap aplikasi dibuka (menyimpan 14 cadangan terakhir)."
			]
		},
		{
			version: "v1.0.0",
			title: "Rilis Awal NILUJI Desktop CBT",
			date: "2026-09-22",
			badgeColor: "neutral",
			htmlUrl: "https://github.com/narr07/niluji/releases/tag/v1.0.0",
			assets: [],
			highlights: [
				"Platform ujian sekolah 100% offline berbasis jaringan lokal (LAN/WiFi) dengan Nuxt 4, Tauri 2, Axum, dan SQLite.",
				"Impor naskah soal dari Microsoft Word (.docx) beserta tabel kunci jawaban terpisah.",
				"Fitur Tarik Data Siswa dari Google Sheets dan Tarik Soal Online dari repositori Markdown ber-PIN.",
				"Manajemen hasil ujian berjenjang (Kelas > Mata Pelajaran > Jenis Ujian) dan ekspor nilai ke Excel (.xlsx)."
			]
		}
	];

	const OS_LABEL = { windows: "Windows", linux: "Linux", macos: "macOS" } as const;
	type OsKey = keyof typeof OS_LABEL;
	const OS_KEYS = Object.keys(OS_LABEL) as OsKey[];
	const assetOs = (name: string): OsKey | null => {
		if (/\.(?:exe|msi)$/i.test(name)) return "windows";
		if (/\.(?:deb|rpm|appimage)$/i.test(name)) return "linux";
		if (/\.dmg$/i.test(name)) return "macos";
		return null;
	};

	const liveReleases = ref<LocalChangelogItem[]>(fallbackReleases);
	const latestRelease = computed(() => liveReleases.value[0]);
	const latestVersion = computed(() => latestRelease.value?.version || "");

	// Sistem operasi hanya ditampilkan bila rilis terbaru benar-benar memuat installernya.
	const availableOs = computed(() => {
		const found = new Set<OsKey>();
		for (const asset of latestRelease.value?.assets || []) {
			const os = assetOs(asset.label);
			if (os) found.add(os);
		}
		return OS_KEYS.filter(os => found.has(os));
	});
	const availableOsLabel = computed(() => availableOs.value.map(os => OS_LABEL[os]).join(" • "));
	const missingOsLabel = computed(() => OS_KEYS.filter(os => !availableOs.value.includes(os)).map(os => OS_LABEL[os]).join(" dan "));

	const targetOs = computed<OsKey>(() => availableOs.value.includes(detectedOs.value) ? detectedOs.value : (availableOs.value[0] || "windows"));
	const primaryDownloadLabel = computed(() => `Unduh untuk ${OS_LABEL[targetOs.value]} (${latestVersion.value})`);
	const primaryDownloadUrl = computed(() => {
		const assets = latestRelease.value?.assets || [];
		const match = assets.find(a => assetOs(a.label) === targetOs.value && /\.(?:exe|deb|dmg)$/i.test(a.label))
			|| assets.find(a => assetOs(a.label) === targetOs.value);
		return match?.url || "https://github.com/narr07/niluji/releases/latest";
	});

	onMounted(async () => {
		const ua = navigator.userAgent.toLowerCase();
		if (ua.includes("linux")) detectedOs.value = "linux";
		else if (ua.includes("mac")) detectedOs.value = "macos";
		else detectedOs.value = "windows";

		try {
			const res = await fetch("https://api.github.com/repos/narr07/niluji/releases?per_page=10");
			if (!res.ok) return;
			const data = (await res.json()) as GithubRelease[];
			if (!Array.isArray(data) || data.length === 0) return;

			const mapped: LocalChangelogItem[] = data.map((rel, idx) => {
				const lines = (rel.body || "")
					.split(/\r?\n/)
					.map(l => l.replace(/^[-*]\s+/, "").trim())
					.filter(l => l.length > 0 && !l.startsWith("#"));

				const assets = (rel.assets || [])
					.filter(a => /\.(exe|msi|deb|AppImage|dmg)$/i.test(a.name))
					.map(a => ({
						label: a.name,
						icon: a.name.endsWith(".exe") || a.name.endsWith(".msi")
							? "i-lucide-monitor"
							: a.name.endsWith(".dmg")
								? "i-lucide-laptop"
								: "i-lucide-terminal",
						url: a.browser_download_url
					}));

				return {
					version: rel.tag_name,
					title: rel.name || `NILUJI ${rel.tag_name}`,
					date: rel.published_at ? rel.published_at.slice(0, 10) : "",
					badgeColor: idx === 0 ? "success" : "primary",
					htmlUrl: rel.html_url,
					assets,
					highlights: lines.length > 0 ? lines : [`Rilis resmi NILUJI ${rel.tag_name}.`]
				};
			});

			// Data GitHub dipakai apa adanya. Data bawaan hanya mengisi versi yang tidak dikembalikan API.
			for (const fb of fallbackReleases) {
				if (!mapped.some(m => m.version === fb.version)) {
					mapped.push({ ...fb, badgeColor: "primary" });
				}
			}
			liveReleases.value = mapped;
		} catch {
			// Gunakan daftar rilis bawaan bila sedang offline
		}
	});
</script>

<template>
	<UContainer class="py-10 space-y-12">
		<!-- Download Banner -->
		<UCard class="bg-elevated/40 border-primary/30">
			<div class="flex flex-col lg:flex-row lg:items-center justify-between gap-6">
				<div class="space-y-2 max-w-2xl">
					<div class="flex items-center gap-2">
						<UBadge color="success" variant="subtle" size="sm">
							Rilis Stabil Terbaru: {{ latestVersion }}
						</UBadge>
						<UBadge
							v-if="availableOsLabel"
							color="neutral"
							variant="outline"
							size="sm"
						>
							{{ availableOsLabel }}
						</UBadge>
					</div>
					<h1 class="text-2xl sm:text-3xl font-bold text-highlighted">
						Unduh Aplikasi Desktop NILUJI &amp; Template Ujian
					</h1>
					<p class="text-sm sm:text-base text-muted">
						Instal NILUJI pada komputer pengawas/proktor. Siswa cukup mengakses ujian melalui browser di HP atau laptop masing-masing menggunakan jaringan WiFi/LAN lokal tanpa internet.
					</p>
					<p
						v-if="missingOsLabel"
						class="text-sm text-muted"
					>
						Installer untuk {{ missingOsLabel }} belum tersedia pada rilis ini.
					</p>
				</div>

				<div class="flex flex-wrap items-center gap-3 shrink-0">
					<UButton
						:label="primaryDownloadLabel"
						icon="i-lucide-download"
						size="lg"
						color="primary"
						:to="primaryDownloadUrl"
						target="_blank"
					/>
					<UButton
						label="Semua Rilis di GitHub"
						icon="i-simple-icons-github"
						size="lg"
						color="neutral"
						variant="outline"
						to="https://github.com/narr07/niluji/releases"
						target="_blank"
					/>
				</div>
			</div>
		</UCard>

		<!-- Official Templates Section -->
		<div class="space-y-4">
			<div>
				<h2 class="text-xl font-bold text-highlighted">
					Template Resmi Siap Pakai
				</h2>
				<p class="text-sm text-muted">
					Unduh contoh berkas di bawah ini untuk mempercepat impor data siswa dan penyusunan naskah bank soal.
				</p>
			</div>

			<div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
				<UCard>
					<div class="space-y-3">
						<div class="flex items-center gap-2.5">
							<UIcon name="i-lucide-users" class="size-5 text-primary shrink-0" />
							<span class="font-semibold text-highlighted">Template Data Siswa (CSV)</span>
						</div>
						<p class="text-xs text-muted leading-relaxed">
							Format tabel 3 kolom (`NISN`, `Nama`, `Kelas`) untuk mengimpor ratusan siswa sekaligus di menu Siswa.
						</p>
						<UButton
							label="Unduh template-siswa.csv"
							icon="i-lucide-file-spreadsheet"
							size="xs"
							variant="soft"
							:to="templateUrl('template-siswa.csv')"
							external
							download
						/>
					</div>
				</UCard>

				<UCard>
					<div class="space-y-3">
						<div class="flex items-center gap-2.5">
							<UIcon name="i-lucide-file-code" class="size-5 text-primary shrink-0" />
							<span class="font-semibold text-highlighted">Template Bank Soal (Markdown)</span>
						</div>
						<p class="text-xs text-muted leading-relaxed">
							Contoh berkas `.md` berisi soal Pilihan Ganda empat dan lima opsi, kunci jawaban, skor, dan soal Esai. Bisa langsung dibuka lewat Import Soal > Dari file Markdown (.md).
						</p>
						<UButton
							label="Unduh template-soal.md"
							icon="i-lucide-file-text"
							size="xs"
							variant="soft"
							:to="templateUrl('template-soal.md')"
							external
							download
						/>
					</div>
				</UCard>

				<UCard>
					<div class="space-y-3">
						<div class="flex items-center gap-2.5">
							<UIcon name="i-lucide-book-open-check" class="size-5 text-primary shrink-0" />
							<span class="font-semibold text-highlighted">Panduan Format Word (.docx)</span>
						</div>
						<p class="text-xs text-muted leading-relaxed">
							Pelajari aturan penomoran soal, opsi A-E, dan tabel kunci jawaban terpisah untuk impor dari Microsoft Word.
						</p>
						<UButton
							label="Baca Panduan Impor Word"
							icon="i-lucide-arrow-right"
							size="xs"
							variant="soft"
							to="/panduan-guru/impor-dan-ekspor-soal"
						/>
					</div>
				</UCard>
			</div>
		</div>

		<USeparator />

		<!-- Changelog Timeline -->
		<div class="space-y-6">
			<div>
				<h2 class="text-xl font-bold text-highlighted">
					Riwayat Perubahan Versi (Changelog)
				</h2>
				<p class="text-sm text-muted">
					Daftar pembaruan fitur, peningkatan keamanan ujian, dan perbaikan bug pada setiap rilis NILUJI.
				</p>
			</div>

			<UChangelogVersions>
				<UChangelogVersion
					v-for="item in liveReleases"
					:key="item.version"
					:title="item.title"
					:date="item.date"
					:badge="{ label: item.version, color: item.badgeColor, variant: 'subtle' }"
				>
					<template #body>
						<div class="space-y-4">
							<ul class="text-sm space-y-2.5">
								<li
									v-for="(point, idx) in item.highlights"
									:key="idx"
									class="flex items-start gap-2.5"
								>
									<UIcon name="i-lucide-circle-check" class="size-4 mt-0.5 shrink-0 text-primary" />
									<span class="text-default leading-relaxed">{{ point }}</span>
								</li>
							</ul>

							<div v-if="item.assets.length" class="flex flex-wrap items-center gap-2 pt-2">
								<UButton
									v-for="asset in item.assets"
									:key="asset.label"
									:label="asset.label"
									:icon="asset.icon"
									:to="asset.url"
									target="_blank"
									size="xs"
									color="neutral"
									variant="outline"
								/>
							</div>
						</div>
					</template>
				</UChangelogVersion>
			</UChangelogVersions>
		</div>
	</UContainer>
</template>
