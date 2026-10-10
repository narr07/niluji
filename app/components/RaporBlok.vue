<script setup lang="ts">
	import type { RaporKelas, RaporMapel, RaporSiswa } from "~/composables/useRapor";

	// Satu potongan rapor yang tidak boleh terbelah halaman. RaporLembar mengukur tinggi tiap potongan
	// lalu menyusunnya ke halaman. Baris mapel dibuat tabel sendiri-sendiri dengan lebar kolom tetap
	// supaya garisnya tetap lurus walau tabelnya bersambung ke halaman berikutnya.
	export type JenisBlok = "judul" | "thead" | "kelompok" | "mapel" | "koku" | "ekskul" | "hadir" | "keputusan" | "tanggapan" | "ttd";
	export interface Blok { jenis: JenisBlok, nama?: string, no?: number, mapel?: RaporMapel, akhir?: boolean }

	defineProps<{ b: Blok, data: RaporKelas, siswa: RaporSiswa, tempat: string, tanggal: string, tandaTangan: boolean }>();

	const strip = (v?: string | number | null) => (v === null || v === undefined || v === "" ? "-" : String(v));
	const hari = (v: number | null) => (v === null ? "- hari" : `${v} hari`);

	function keputusan(r: RaporKelas, s: RaporSiswa) {
		if (s.naik === null)
			return "";
		const t = r.kelas.tingkat;
		if (t >= 6)
			return s.naik ? "LULUS" : "TIDAK LULUS";
		return s.naik
			? `Naik ke Kelas ${t + 1} (${angkaKata(t + 1)})`
			: `Tinggal di Kelas ${t} (${angkaKata(t)})`;
	}
</script>

<template>
	<h1 v-if="b.jenis === 'judul'">
		LAPORAN HASIL BELAJAR
	</h1>

	<table v-else-if="b.jenis === 'thead' || b.jenis === 'kelompok' || b.jenis === 'mapel'" class="tbl baris" :class="{ akhir: b.akhir }">
		<colgroup>
			<col class="no">
			<col class="mapel">
			<col class="nilai">
			<col>
		</colgroup>
		<thead v-if="b.jenis === 'thead'">
			<tr>
				<th>No</th>
				<th>Mata Pelajaran</th>
				<th>Nilai Akhir</th>
				<th>Capaian Kompetensi</th>
			</tr>
		</thead>
		<tbody v-else>
			<tr v-if="b.jenis === 'kelompok'" class="kelompok">
				<td colspan="4">
					{{ b.nama }}
				</td>
			</tr>
			<tr v-else-if="b.mapel">
				<td class="tengah">
					{{ b.no }}
				</td>
				<td>{{ b.mapel.nama }}</td>
				<td class="tengah">
					{{ strip(b.mapel.nilai) }}
				</td>
				<td class="capaian">
					{{ b.mapel.capaian || "-" }}
				</td>
			</tr>
		</tbody>
	</table>

	<table v-else-if="b.jenis === 'koku'" class="tbl">
		<thead>
			<tr><th>Kokurikuler</th></tr>
		</thead>
		<tbody>
			<tr>
				<td class="isian">
					{{ siswa.kokurikuler || "-" }}
				</td>
			</tr>
		</tbody>
	</table>

	<table v-else-if="b.jenis === 'ekskul'" class="tbl">
		<colgroup>
			<col class="no">
			<col class="mapel">
			<col>
		</colgroup>
		<thead>
			<tr>
				<th>No</th>
				<th>Ekstrakurikuler</th>
				<th>Keterangan</th>
			</tr>
		</thead>
		<tbody>
			<tr v-for="(x, i) in siswa.ekskul" :key="x.nama">
				<td class="tengah">
					{{ i + 1 }}
				</td><td>{{ x.nama }}</td><td>{{ x.keterangan }}</td>
			</tr>
			<tr v-if="!siswa.ekskul.length">
				<td class="tengah">
					-
				</td><td>-</td><td>-</td>
			</tr>
		</tbody>
	</table>

	<div v-else-if="b.jenis === 'hadir'" class="dua-kolom">
		<table class="tbl hadir">
			<thead>
				<tr><th colspan="2">Ketidakhadiran</th></tr>
			</thead>
			<tbody>
				<tr><td>Sakit</td><td>: {{ hari(siswa.sakit) }}</td></tr>
				<tr><td>Izin</td><td>: {{ hari(siswa.izin) }}</td></tr>
				<tr><td>Tanpa Keterangan</td><td>: {{ hari(siswa.alpa) }}</td></tr>
			</tbody>
		</table>
		<table class="tbl catatan">
			<thead>
				<tr><th>Catatan Wali Kelas</th></tr>
			</thead>
			<tbody>
				<tr>
					<td class="isian">
						{{ siswa.catatan || "-" }}
					</td>
				</tr>
			</tbody>
		</table>
	</div>

	<table v-else-if="b.jenis === 'keputusan'" class="tbl">
		<thead>
			<tr><th>Keputusan</th></tr>
		</thead>
		<tbody>
			<tr>
				<td class="isian">
					Berdasarkan pencapaian seluruh kompetensi, murid dinyatakan:
					<b>{{ keputusan(data, siswa) || "...................." }}</b>
				</td>
			</tr>
		</tbody>
	</table>

	<table v-else-if="b.jenis === 'tanggapan'" class="tbl">
		<thead>
			<tr><th>Tanggapan Orang Tua/Wali Murid</th></tr>
		</thead>
		<tbody>
			<tr><td class="tanggapan" /></tr>
		</tbody>
	</table>

	<div v-else-if="b.jenis === 'ttd'" class="ttd">
		<div>
			<p>&nbsp;</p>
			<p>Orang Tua Murid</p>
			<p class="ruang" />
			<p class="garis">
				..............................
			</p>
		</div>
		<div>
			<p>&nbsp;</p>
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
		<div>
			<p>{{ [tempat.trim(), tanggalIndo(tanggal)].filter(Boolean).join(", ") || "...................., ...................." }}</p>
			<p>Wali Kelas</p>
			<div class="ruang ttd-gambar">
				<img
					v-if="tandaTangan && data.ttdWali"
					:src="data.ttdWali"
					alt=""
					class="ttd-img">
			</div>
			<p class="nama">
				{{ tandaTangan && data.wali ? data.wali.nama : ".............................." }}
			</p>
			<p v-if="tandaTangan && data.wali">
				NIP. {{ strip(data.wali.nip) }}
			</p>
		</div>
	</div>
</template>
