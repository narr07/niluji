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
		description: "Unduh installer aplikasi desktop NILUJI terbaru untuk Windows, Linux, dan macOS beserta catatan perubahan versi (changelog)."
	});

	const runtimeConfig = useRuntimeConfig();
	const baseURL = runtimeConfig.app.baseURL || "/";
	const templateUrl = (file: string) => `${baseURL}templates/${file}`.replace("//", "/");

	const detectedOs = ref<"windows" | "linux" | "macos">("windows");

	const fallbackReleases: LocalChangelogItem[] = [
		{
			version: "v1.1.0",
			title: "Keamanan Sesi, Kartu Peserta & Peningkatan UI CBT",
			date: "2026-10-31",
			badgeColor: "success",
			htmlUrl: "https://github.com/narr07/niluji/releases/latest",
			assets: [
				{ label: "Windows (.exe / .msi)", icon: "i-lucide-monitor", url: "https://github.com/narr07/niluji/releases/latest" },
				{ label: "Linux (.deb / .AppImage)", icon: "i-lucide-terminal", url: "https://github.com/narr07/niluji/releases/latest" },
				{ label: "macOS (.dmg)", icon: "i-lucide-laptop", url: "https://github.com/narr07/niluji/releases/latest" }
			],
			highlights: [
				"Kartu Peserta Ujian & PIN 6 Digit: setiap siswa otomatis memiliki PIN unik dan dapat dicetak bersama QR Code akses lokal.",
				"Kunci Perangkat (Single-Device Lock): mencegah satu akun siswa dikerjakan dari 2 perangkat sekaligus, dilengkapi tombol Buka Kunci Perangkat bagi pengawas.",
				"Aksi Pengawas Real-Time: tambah waktu ujian (+10 menit) dan paksa kumpulkan (Force Submit) langsung dari tabel Hasil Ujian.",
				"Acak Opsi Jawaban PG (A-D) secara deterministik per sesi siswa dan penggabungan otomatis Nilai PG + Nilai Esai sesuai bobot persentase.",
				"Portal CBT Siswa diperbarui: tombol Ragu-ragu (bookmark), pengatur ukuran teks (A-/A/A+), indikator status simpan real-time, pintasan keyboard (A-D, R, Panah), dan bilah navigasi bawah khusus HP.",
				"Analisis Pengecoh (Distractor Analysis) per butir soal dan fitur Akses Cepat Paket Soal di halaman Bank Soal."
			]
		},
		{
			version: "v1.0.1",
			title: "Perbaikan Stabilitas, Impor Markdown & Backup Otomatis",
			date: "2026-09-22",
			badgeColor: "primary",
			htmlUrl: "https://github.com/narr07/niluji/releases/tag/v1.0.1",
			assets: [],
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

	const liveReleases = ref<LocalChangelogItem[]>(fallbackReleases);
	const latestVersion = computed(() => liveReleases.value[0]?.version || "v1.1.0");

	const primaryDownloadLabel = computed(() => {
		if (detectedOs.value === "linux") return `Unduh untuk Linux (${latestVersion.value})`;
		if (detectedOs.value === "macos") return `Unduh untuk macOS (${latestVersion.value})`;
		return `Unduh untuk Windows (${latestVersion.value})`;
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

			for (const fb of fallbackReleases) {
				if (!mapped.some(m => m.version === fb.version)) {
					mapped.push(fb);
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
						<UBadge color="neutral" variant="outline" size="sm">
							Windows • Linux • macOS
						</UBadge>
					</div>
					<h1 class="text-2xl sm:text-3xl font-bold text-highlighted">
						Unduh Aplikasi Desktop NILUJI &amp; Template Ujian
					</h1>
					<p class="text-sm sm:text-base text-muted">
						Instal NILUJI pada komputer pengawas/proktor. Siswa cukup mengakses ujian melalui browser di HP atau laptop masing-masing menggunakan jaringan WiFi/LAN lokal tanpa internet.
					</p>
				</div>

				<div class="flex flex-wrap items-center gap-3 shrink-0">
					<UButton
						:label="primaryDownloadLabel"
						icon="i-lucide-download"
						size="lg"
						color="primary"
						to="https://github.com/narr07/niluji/releases/latest"
						target="_blank"
					/>
					<UButton
						label="Semua Sistem Operasi"
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
							Contoh berkas `.md` lengkap dengan soal Pilihan Ganda, opsi bergambar (`![opsi-A]`), kunci jawaban, dan soal Esai.
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
