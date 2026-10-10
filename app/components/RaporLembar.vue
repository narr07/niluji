<script setup lang="ts">
	import type { Blok } from "./RaporBlok.vue";
	import type { RaporKelas, RaporSiswa } from "~/composables/useRapor";

	// Satu rapor siswa (format Laporan Hasil Belajar e-Rapor SD 2025), dipecah sendiri ke halaman kertas:
	// tiap potongan diukur di wadah tersembunyi, lalu disusun per halaman. Identitas diulang tiap halaman,
	// judul tabel mapel diulang kalau tabelnya bersambung, dan kaki halaman berisi
	// "Kelas | Nama | NISN" + "Halaman x dari y" per siswa. Halaman cetak tanpa margin (@page margin 0),
	// jadi browser tidak menambahkan tanggal/alamat di tepi kertas.
	const props = withDefaults(defineProps<{
		data: RaporKelas
		siswa: RaporSiswa
		tempat: string
		tanggal: string
		tandaTangan: boolean
		kertas?: { w: number, h: number } // mm
	}>(), { kertas: () => ({ w: 210, h: 297 }) });

	// Margin kertas (mm): kiri lebih lebar untuk jilid, bawah memberi ruang kaki halaman.
	const M = { atas: 12, kanan: 12, bawah: 18, kiri: 18 };
	const PX_PER_MM = 96 / 25.4;
	const strip = (v?: string | number | null) => (v === null || v === undefined || v === "" ? "-" : String(v));

	const blok = computed<Blok[]>(() => {
		const out: Blok[] = [{ jenis: "judul" }, { jenis: "thead" }];
		const satuKelompok = props.siswa.kelompok.length <= 1;
		for (const k of props.siswa.kelompok) {
			if (!satuKelompok)
				out.push({ jenis: "kelompok", nama: k.nama });
			k.mapel.forEach((m, i) => out.push({ jenis: "mapel", no: i + 1, mapel: m }));
		}
		const terakhir = out.at(-1);
		if (terakhir)
			terakhir.akhir = true;
		out.push({ jenis: "koku" }, { jenis: "ekskul" }, { jenis: "hadir" });
		if (props.data.semesterKe === 2)
			out.push({ jenis: "keputusan" });
		out.push({ jenis: "tanggapan" }, { jenis: "ttd" });
		return out;
	});

	// ===== Ukur & susun halaman =====
	const ukur = ref<HTMLElement>();
	const halaman = ref<number[][]>([]);
	const THEAD = -1; // penanda "ulangi judul tabel mapel" di awal halaman lanjutan

	// Tinggi asli (px CSS), tidak terpengaruh zoom pratinjau (transform: scale) di wadah luar.
	function tinggi(sel: string) {
		const el = ukur.value?.querySelector(sel) as HTMLElement | null;
		if (!el || !ukur.value?.offsetWidth)
			return 0;
		const skala = ukur.value.getBoundingClientRect().width / ukur.value.offsetWidth || 1;
		return el.getBoundingClientRect().height / skala;
	}

	let coba = 0;
	async function susun() {
		await nextTick();
		if (!ukur.value)
			return;
		// Tinggi 0 = wadah sedang tidak ditata (mis. tersembunyi). Jangan susun asal-asalan (semua jadi
		// satu halaman lalu menimpa kaki halaman); ulangi sebentar lagi.
		if (!tinggi("[data-u='ident']")) {
			if (coba++ < 20)
				setTimeout(susun, 150);
			return;
		}
		coba = 0;
		const tersedia = (props.kertas.h - M.atas - M.bawah) * PX_PER_MM - tinggi("[data-u='ident']") - 2;
		const h = blok.value.map((_, i) => tinggi(`[data-u='${i}']`));
		const hThead = h[1] ?? 0;
		const pages: number[][] = [[]];
		let pakai = 0;
		blok.value.forEach((b, i) => {
			const baris = b.jenis === "mapel" || b.jenis === "kelompok";
			// Judul kelompok/tabel jangan tertinggal sendirian di bawah halaman: ikutkan baris berikutnya.
			const ikut = (b.jenis === "kelompok" || b.jenis === "judul" || b.jenis === "thead") ? (h[i + 1] ?? 0) : 0;
			if (pakai + h[i]! + ikut > tersedia && pages.at(-1)!.length) {
				pages.push([]);
				pakai = 0;
				if (baris) {
					pages.at(-1)!.push(THEAD);
					pakai += hThead;
				}
			}
			pages.at(-1)!.push(i);
			pakai += h[i]!;
		});
		halaman.value = pages;
	}
	onMounted(susun);
	watch(() => [props.siswa, props.data, props.kertas.w, props.kertas.h, props.tandaTangan, props.tempat, props.tanggal], susun);

	const gayaHalaman = computed(() => ({
		width: `${props.kertas.w}mm`,
		height: `${props.kertas.h - 0.6}mm`,
		padding: `${M.atas}mm ${M.kanan}mm ${M.bawah}mm ${M.kiri}mm`
	}));
	const gayaKaki = { left: `${M.kiri}mm`, right: `${M.kanan}mm`, bottom: "9mm" };
</script>

<template>
	<div class="rapor-buku">
		<!-- Wadah ukur: tidak terlihat, tidak ikut tercetak -->
		<div
			ref="ukur"
			class="rapor rapor-ukur"
			:style="{ width: `${kertas.w - M.kiri - M.kanan}mm` }"
			aria-hidden="true">
			<div data-u="ident" class="ukur-item">
				<RaporIdentitas :data="data" :siswa="siswa" />
			</div>
			<div
				v-for="(b, i) in blok"
				:key="i"
				:data-u="i"
				class="ukur-item">
				<RaporBlok
					:b="b"
					:data="data"
					:siswa="siswa"
					:tempat="tempat"
					:tanggal="tanggal"
					:tanda-tangan="tandaTangan" />
			</div>
		</div>

		<section
			v-for="(isi, p) in halaman"
			:key="p"
			class="rapor rapor-halaman"
			:style="gayaHalaman">
			<RaporIdentitas :data="data" :siswa="siswa" />
			<template v-for="i in isi" :key="i">
				<RaporBlok
					v-if="i === THEAD"
					:b="{ jenis: 'thead' }"
					:data="data"
					:siswa="siswa"
					:tempat="tempat"
					:tanggal="tanggal"
					:tanda-tangan="tandaTangan" />
				<RaporBlok
					v-else
					:b="blok[i]!"
					:data="data"
					:siswa="siswa"
					:tempat="tempat"
					:tanggal="tanggal"
					:tanda-tangan="tandaTangan" />
			</template>
			<div class="kaki-halaman" :style="gayaKaki">
				<span>{{ data.kelas.nama }} | {{ siswa.nama }} | {{ strip(siswa.nisn) }}</span>
				<span>Halaman {{ p + 1 }} dari {{ halaman.length }}</span>
			</div>
		</section>
	</div>
</template>

<style>
	/* Gaya kertas rapor: selalu hitam di atas putih (tidak ikut tema gelap aplikasi). */
	.rapor {
		font-family: Arial, Helvetica, sans-serif;
		font-size: 10.5pt;
		line-height: 1.35;
		color: #000;
		background: #fff;
	}
	.rapor-buku { display: flex; flex-direction: column; align-items: center; gap: 16px; }
	.rapor-ukur { position: absolute; left: -10000px; top: 0; visibility: hidden; pointer-events: none; }
	.rapor-ukur .ukur-item { display: flow-root; }
	.rapor-halaman { position: relative; box-sizing: border-box; overflow: hidden; box-shadow: 0 1px 4px rgb(0 0 0 / 0.25); }
	.rapor .kaki-halaman { position: absolute; display: flex; justify-content: space-between; border-top: 1px solid #000; padding-top: 3px; font-size: 8pt; font-style: italic; }
	.rapor .identitas { width: 100%; border-collapse: collapse; }
	.rapor .identitas td { padding: 1px 4px; vertical-align: top; }
	.rapor .identitas .label { width: 16%; white-space: nowrap; }
	.rapor hr { border: 0; border-top: 1.5px solid #000; margin: 6px 0 10px; }
	.rapor h1 { text-align: center; font-size: 12pt; font-weight: bold; margin: 0 0 8px; }
	.rapor .tbl { width: 100%; border-collapse: collapse; margin-bottom: 10px; }
	.rapor .tbl.baris { table-layout: fixed; margin-bottom: 0; margin-top: -1px; }
	.rapor .tbl.baris.akhir { margin-bottom: 10px; }
	.rapor .tbl th, .rapor .tbl td { border: 1px solid #000; padding: 4px 6px; vertical-align: top; text-align: left; }
	.rapor .tbl th { background: #e5e7eb; text-align: center; font-weight: bold; }
	.rapor .tbl col.no { width: 32px; }
	.rapor .tbl col.mapel { width: 30%; }
	.rapor .tbl col.nilai { width: 64px; }
	.rapor .tbl .tengah { text-align: center; }
	.rapor .tbl .capaian { text-align: left; }
	.rapor .tbl .kelompok td { font-weight: bold; background: #f3f4f6; }
	.rapor .tbl .isian { height: 40px; }
	.rapor .tbl .tanggapan { height: 60px; }
	.rapor .dua-kolom { display: flex; gap: 12px; }
	.rapor .dua-kolom .hadir { width: 40%; }
	.rapor .dua-kolom .catatan { flex: 1; }
	.rapor .ttd { display: grid; grid-template-columns: repeat(3, 1fr); gap: 12px; margin-top: 16px; }
	.rapor .ttd p { margin: 0; }
	.rapor .ttd .ruang { height: 64px; }
	.rapor .ttd .ttd-gambar { position: relative; }
	.rapor .ttd-img { position: absolute; left: 0; bottom: 0; height: 100%; max-width: 100%; object-fit: contain; }
	.rapor .stempel-img { position: absolute; left: -28px; bottom: -10px; height: 120%; opacity: 0.9; }
	.rapor .ttd .nama { font-weight: bold; text-decoration: underline; }
	.rapor * { print-color-adjust: exact; -webkit-print-color-adjust: exact; }
	@media print {
		.rapor-ukur { display: none !important; }
		.rapor-buku { display: block; gap: 0; }
		.rapor-halaman { box-shadow: none; }
		.rapor-halaman + .rapor-halaman { break-before: page; }
	}
</style>
