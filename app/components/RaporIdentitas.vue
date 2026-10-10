<script setup lang="ts">
	import type { RaporKelas, RaporSiswa } from "~/composables/useRapor";

	// Kepala tiap halaman rapor: identitas murid & kelas.
	defineProps<{ data: RaporKelas, siswa: RaporSiswa }>();
	const strip = (v?: string | number | null) => (v === null || v === undefined || v === "" ? "-" : String(v));
</script>

<template>
	<div>
		<table class="identitas">
			<tbody>
				<tr>
					<td class="label">
						Nama Murid
					</td><td>: {{ siswa.nama }}</td>
					<td class="label">
						Kelas
					</td><td>: {{ data.kelas.nama }}</td>
				</tr>
				<tr>
					<td class="label">
						NISN / NIS
					</td><td>: {{ strip(siswa.nisn) }} / {{ strip(siswa.nipd) }}</td>
					<td class="label">
						Fase
					</td><td>: {{ data.kelas.fase }}</td>
				</tr>
				<tr>
					<td class="label">
						Sekolah
					</td><td>: {{ data.sekolah.nama }}</td>
					<td class="label">
						Semester
					</td><td>: {{ data.semesterKe }}</td>
				</tr>
				<tr>
					<td class="label">
						Alamat
					</td><td>: {{ strip(data.sekolah.alamat) }}</td>
					<td class="label">
						Tahun Ajaran
					</td><td>: {{ data.tahunAjaran }}</td>
				</tr>
			</tbody>
		</table>
		<hr>
	</div>
</template>
