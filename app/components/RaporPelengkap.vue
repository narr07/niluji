<script setup lang="ts">
	import type { PelengkapKelas, PelengkapSiswa } from "~/composables/useRapor";

	// Pelengkap rapor (padanan Cetak Nilai → Pelengkap Rapor e-Rapor): sampul, identitas sekolah,
	// identitas murid. Tiga halaman per siswa; `halaman` memilih yang ditampilkan (pratinjau) atau semua (cetak).
	const props = withDefaults(defineProps<{
		data: PelengkapKelas
		siswa: PelengkapSiswa
		tempat: string
		tanggal: string
		tandaTangan: boolean
		halaman?: 1 | 2 | 3
		kop?: Kop
		kertas?: { w: number, h: number } // mm
		kaki?: string // nama kelas untuk kaki halaman
	}>(), { kertas: () => ({ w: 210, h: 297 }) });

	// Tiap bagian = satu halaman kertas penuh (margin di dalam halaman, @page margin 0).
	const gayaHalaman = computed(() => ({ width: `${props.kertas.w}mm`, height: `${props.kertas.h - 0.6}mm` }));

	const strip = (v?: string | null) => (v?.trim() ? v : "-");
	const tampil = (n: number) => !props.halaman || props.halaman === n;

	const sekolahBaris = computed(() => [
		["Nama Sekolah", props.data.sekolah.nama],
		["NPSN", props.data.sekolah.npsn],
		["NIS/NSS/NDS", props.data.sekolah.nss],
		["Alamat Sekolah", props.data.sekolah.alamat],
		["Kode Pos", props.data.sekolah.kodePos],
		["Telepon", props.data.sekolah.telepon],
		["Kelurahan/Desa", props.data.sekolah.desa],
		["Kecamatan", props.data.sekolah.kecamatan],
		["Kabupaten/Kota", props.data.sekolah.kabupaten],
		["Provinsi", props.data.sekolah.provinsi],
		["Website", props.data.sekolah.website],
		["E-mail", props.data.sekolah.email]
	]);

	// [nomor, label, isi]; nomor kosong = sub-baris (a., b.); isi null = baris judul tanpa isi.
	const muridBaris = computed(() => {
		const s = props.siswa;
		return [
			["1.", "Nama Murid (Lengkap)", s.nama],
			["2.", "NISN / NIS", `${strip(s.nisn)} / ${strip(s.nipd)}`],
			["3.", "Tempat, Tanggal Lahir", tempatTanggal(s.tempatLahir, s.tanggalLahir)],
			["4.", "Jenis Kelamin", s.jenisKelamin],
			["5.", "Agama", s.agama],
			["6.", "Anak ke-", s.anakKe],
			["7.", "Pendidikan Sebelumnya", s.sekolahAsal],
			["8.", "Alamat Murid", s.alamat],
			["", "Telepon", s.telepon],
			["9.", "Nama Orang Tua", null],
			["", "a. Ayah", s.ayah],
			["", "b. Ibu", s.ibu],
			["10.", "Pekerjaan Orang Tua", null],
			["", "a. Ayah", s.kerjaAyah],
			["", "b. Ibu", s.kerjaIbu],
			["11.", "Alamat Orang Tua", s.alamat],
			["12.", "Wali Murid", null],
			["", "a. Nama", s.wali],
			["", "b. Pekerjaan", s.kerjaWali],
			["13.", "Diterima di sekolah ini", null],
			["", "Tanggal", tanggalIndo(s.tanggalMasuk)]
		] as [string, string, string | null][];
	});
</script>

<template>
	<div class="pelengkap">
		<!-- 1. Sampul -->
		<section v-if="tampil(1)" class="halaman sampul" :style="gayaHalaman">
			<img
				v-if="data.gambar.logo"
				:src="data.gambar.logo"
				alt="Logo"
				class="logo">
			<h1 :class="{ 'dengan-logo': data.gambar.logo }">
				RAPOR MURID<br>SEKOLAH DASAR<br>(SD)
			</h1>
			<div class="kotak-isi">
				<p>Nama Murid</p>
				<div class="kotak">
					{{ siswa.nama }}
				</div>
				<p>NISN / NIS</p>
				<div class="kotak">
					{{ strip(siswa.nisn) }} / {{ strip(siswa.nipd) }}
				</div>
			</div>
			<p class="kementerian">
				KEMENTERIAN PENDIDIKAN DASAR DAN MENENGAH<br>REPUBLIK INDONESIA
			</p>
		</section>

		<!-- 2. Identitas sekolah -->
		<section v-if="tampil(2)" class="halaman" :style="gayaHalaman">
			<KopSurat v-if="kop" :kop="kop" />
			<h2>RAPOR MURID<br>SEKOLAH DASAR (SD)</h2>
			<table class="isian">
				<tbody>
					<tr v-for="[label, isi] in sekolahBaris" :key="label">
						<td class="label">
							{{ label }}
						</td>
						<td class="titik">
							:
						</td>
						<td>{{ strip(isi) }}</td>
					</tr>
				</tbody>
			</table>
			<div class="kaki-halaman">
				<span>{{ kaki ? `${kaki} | ` : "" }}{{ siswa.nama }} | {{ strip(siswa.nisn) }}</span><span>Halaman 2 dari 3</span>
			</div>
		</section>

		<!-- 3. Identitas murid -->
		<section v-if="tampil(3)" class="halaman" :style="gayaHalaman">
			<h2>IDENTITAS MURID</h2>
			<table class="isian">
				<tbody>
					<tr v-for="([no, label, isi], i) in muridBaris" :key="i">
						<td class="no">
							{{ no }}
						</td>
						<td class="label" :class="{ sub: !no }">
							{{ label }}
						</td>
						<td class="titik">
							{{ isi === null ? "" : ":" }}
						</td>
						<td>{{ isi === null ? "" : strip(isi) }}</td>
					</tr>
				</tbody>
			</table>
			<div class="ttd-identitas">
				<div class="foto">
					<img v-if="siswa.foto" :src="siswa.foto" :alt="`Foto ${siswa.nama}`">
					<template v-else>
						Pas foto<br>3 × 4
					</template>
				</div>
				<div>
					<p>{{ [tempat.trim(), tanggalIndo(tanggal)].filter(Boolean).join(", ") || "...................., ...................." }}</p>
					<p>Kepala Sekolah</p>
					<div class="ruang ttd-gambar">
						<img
							v-if="tandaTangan && data.gambar.ttdKepsek"
							:src="data.gambar.ttdKepsek"
							alt=""
							class="ttd-img">
						<img
							v-if="tandaTangan && data.gambar.stempel"
							:src="data.gambar.stempel"
							alt=""
							class="stempel-img">
					</div>
					<p class="nama">
						{{ tandaTangan && data.kepsek ? data.kepsek.nama : ".............................." }}
					</p>
					<p v-if="tandaTangan && data.kepsek">
						NIP. {{ strip(data.kepsek.nip) }}
					</p>
				</div>
			</div>
			<div class="kaki-halaman">
				<span>{{ kaki ? `${kaki} | ` : "" }}{{ siswa.nama }} | {{ strip(siswa.nisn) }}</span><span>Halaman 3 dari 3</span>
			</div>
		</section>
	</div>
</template>

<style>
	/* Kertas pelengkap: hitam di atas putih, tidak ikut tema gelap. */
	.pelengkap { font-family: Arial, Helvetica, sans-serif; font-size: 11pt; line-height: 1.5; color: #000; background: #fff; }
	.pelengkap { display: flex; flex-direction: column; align-items: center; gap: 16px; }
	.pelengkap .halaman { position: relative; box-sizing: border-box; padding: 15mm 15mm 18mm 20mm; overflow: hidden; background: #fff; box-shadow: 0 1px 4px rgb(0 0 0 / 0.25); }
	.pelengkap .kaki-halaman { position: absolute; left: 20mm; right: 15mm; bottom: 9mm; display: flex; justify-content: space-between; border-top: 1px solid #000; padding-top: 3px; font-size: 8pt; font-style: italic; }
	@media print {
		.pelengkap { display: block; }
		.pelengkap .halaman { box-shadow: none; }
		.pelengkap .halaman + .halaman { break-before: page; }
	}
	.pelengkap h1 { text-align: center; font-size: 20pt; font-weight: bold; line-height: 1.4; margin: 40px 0 80px; }
	.pelengkap h1.dengan-logo { margin-top: 24px; }
	.pelengkap .logo { display: block; width: 4.5cm; max-height: 5cm; object-fit: contain; margin: 24px auto 0; }
	.pelengkap .ttd-identitas .foto img { width: 100%; height: 100%; object-fit: cover; }
	.pelengkap .ttd-gambar { position: relative; }
	.pelengkap .ttd-img { position: absolute; left: 0; bottom: 0; height: 100%; max-width: 100%; object-fit: contain; }
	.pelengkap .stempel-img { position: absolute; left: -28px; bottom: -10px; height: 120%; opacity: 0.9; }
	.pelengkap h2 { text-align: center; font-size: 13pt; font-weight: bold; margin: 0 0 24px; }
	.pelengkap .sampul { text-align: center; }
	.pelengkap .kotak-isi { width: 70%; margin: 0 auto 120px; }
	.pelengkap .kotak-isi p { margin: 12px 0 4px; }
	.pelengkap .kotak { border: 1.5px solid #000; padding: 8px; font-weight: bold; font-size: 12pt; }
	.pelengkap .kementerian { font-weight: bold; font-size: 12pt; }
	.pelengkap .isian { width: 100%; border-collapse: collapse; }
	.pelengkap .isian td { padding: 3px 4px; vertical-align: top; }
	.pelengkap .isian .no { width: 32px; }
	.pelengkap .isian .label { width: 38%; }
	.pelengkap .isian .label.sub { padding-left: 16px; }
	.pelengkap .isian .titik { width: 12px; }
	.pelengkap .ttd-identitas { display: flex; justify-content: flex-end; gap: 48px; margin-top: 32px; break-inside: avoid; }
	.pelengkap .ttd-identitas p { margin: 0; }
	.pelengkap .ttd-identitas .foto { width: 3cm; height: 4cm; border: 1px solid #000; display: flex; align-items: center; justify-content: center; text-align: center; font-size: 9pt; color: #555; }
	.pelengkap .ttd-identitas .ruang { height: 64px; }
	.pelengkap .ttd-identitas .nama { font-weight: bold; text-decoration: underline; }
</style>
