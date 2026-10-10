<script setup lang="ts">
	// Satu lembar transkrip nilai ijazah SD.
	defineProps<{
		sekolah: { nama: string, npsn: string }
		kepsek: { nama: string, nip: string | null } | null
		gambar: GambarRapor
		mapel: { kode: string, nama: string }[]
		siswa: TranskripBaris & { tempatLahir: string, tanggalLahir: string }
		atur: { tempat: string, tanggal: string, rataRata: boolean, tandaTangan: boolean }
		angka: (v: number | null | undefined) => string
		rata: number | null
		kop?: Kop
	}>();
</script>

<template>
	<div class="transkrip">
		<KopSurat v-if="kop" :kop="kop" />
		<h1>TRANSKRIP NILAI</h1>
		<p class="nomor">
			Nomor: {{ siswa.noTranskrip || "...................................." }}
		</p>
		<table class="identitas">
			<tbody>
				<tr><td>Satuan Pendidikan</td><td>:</td><td>{{ sekolah.nama }}</td></tr>
				<tr><td>Nomor Pokok Sekolah Nasional</td><td>:</td><td>{{ sekolah.npsn }}</td></tr>
				<tr><td>Nama Lengkap</td><td>:</td><td>{{ siswa.nama }}</td></tr>
				<tr><td>Tempat, Tanggal Lahir</td><td>:</td><td>{{ tempatTanggal(siswa.tempatLahir, siswa.tanggalLahir) }}</td></tr>
				<tr><td>Nomor Induk Siswa Nasional</td><td>:</td><td>{{ siswa.nisn || "-" }}</td></tr>
				<tr><td>Nomor Ijazah</td><td>:</td><td>{{ siswa.noIjazah || "-" }}</td></tr>
				<tr><td>Tanggal Kelulusan</td><td>:</td><td>{{ tanggalIndo(siswa.tanggalLulus) || "-" }}</td></tr>
			</tbody>
		</table>
		<table class="nilai">
			<thead>
				<tr>
					<th class="no">
						No
					</th>
					<th>Mata Pelajaran</th>
					<th class="angka">
						Nilai
					</th>
				</tr>
			</thead>
			<tbody>
				<tr v-for="(m, i) in mapel" :key="m.kode">
					<td class="tengah">
						{{ i + 1 }}
					</td>
					<td>{{ m.nama }}</td>
					<td class="tengah">
						{{ angka(siswa.nilai[m.kode]) }}
					</td>
				</tr>
				<tr v-if="atur.rataRata" class="rata">
					<td colspan="2" class="tengah">
						Rata-rata
					</td>
					<td class="tengah">
						{{ angka(rata) }}
					</td>
				</tr>
			</tbody>
		</table>
		<div class="ttd">
			<p>{{ [atur.tempat.trim(), tanggalIndo(atur.tanggal)].filter(Boolean).join(", ") }}</p>
			<p>Kepala Sekolah,</p>
			<div class="ruang ttd-gambar">
				<img
					v-if="atur.tandaTangan && gambar.ttdKepsek"
					:src="gambar.ttdKepsek"
					alt=""
					class="ttd-img">
				<img
					v-if="atur.tandaTangan && gambar.stempel"
					:src="gambar.stempel"
					alt=""
					class="stempel-img">
			</div>
			<p class="nama">
				{{ atur.tandaTangan && kepsek ? kepsek.nama : ".............................." }}
			</p>
			<p v-if="atur.tandaTangan && kepsek">
				NIP. {{ kepsek.nip || "-" }}
			</p>
		</div>
	</div>
</template>

<style>
	/* Satu halaman F4 penuh; margin di dalam halaman (@page margin 0), jadi browser tidak mencetak tanggal/URL. */
	.transkrip { width: 215mm; height: 329.4mm; box-sizing: border-box; padding: 15mm 15mm 15mm 20mm; overflow: hidden; font-family: Arial, Helvetica, sans-serif; font-size: 11pt; line-height: 1.45; color: #000; background: #fff; box-shadow: 0 1px 4px rgb(0 0 0 / 0.25); }
	.transkrip h1 { text-align: center; font-size: 14pt; font-weight: bold; margin: 0; }
	.transkrip .nomor { text-align: center; margin: 2px 0 18px; }
	.transkrip .identitas { border-collapse: collapse; margin-bottom: 14px; }
	.transkrip .identitas td { padding: 2px 6px 2px 0; vertical-align: top; }
	.transkrip .identitas td:first-child { width: 240px; }
	.transkrip .nilai { width: 100%; border-collapse: collapse; }
	.transkrip .nilai th, .transkrip .nilai td { border: 1px solid #000; padding: 4px 8px; }
	.transkrip .nilai th { background: #e5e7eb; }
	.transkrip .nilai .no { width: 40px; }
	.transkrip .nilai .angka { width: 90px; }
	.transkrip .nilai .tengah { text-align: center; }
	.transkrip .nilai .rata td { font-weight: bold; }
	.transkrip .ttd { margin: 28px 0 0 auto; width: 260px; break-inside: avoid; }
	.transkrip .ttd p { margin: 0; }
	.transkrip .ttd .ruang { height: 70px; position: relative; }
	.transkrip .ttd .ttd-img { position: absolute; left: 0; bottom: 0; height: 100%; max-width: 100%; object-fit: contain; }
	.transkrip .ttd .stempel-img { position: absolute; left: -28px; bottom: -10px; height: 120%; opacity: 0.9; }
	.transkrip .ttd .nama { font-weight: bold; text-decoration: underline; }
	.transkrip * { print-color-adjust: exact; -webkit-print-color-adjust: exact; }
</style>
