import Sortable from "sortablejs";

// Urut ulang lewat drag & drop, sama seperti niluji (Pengaturan → Kelas/Pelajaran):
// - Hanya bisa diseret dari pegangan [data-drag-handle] (ikon grip), supaya input di baris tetap bisa diklik.
// - forceFallback: WebView Tauri sering gagal dengan drag bawaan HTML5 (kursor 🚫 terus); SortableJS
//   pakai simulasi drag sendiri.
// - DOM yang sudah dipindah SortableJS dikembalikan dulu, baru data diubah, supaya Vue yang
//   menggambar ulang urutannya (kalau tidak, DOM dan data bisa tidak sinkron).
//
// Pakai sebagai ref fungsi: :ref="el => pasang(el, (dari, ke) => ...)"
export function useDragUrut() {
	const aktif = new Map<Element, Sortable>();

	// Bersihkan instance untuk elemen yang sudah hilang dari halaman (mis. kelompok diganti nama).
	// Harus SETELAH Vue selesai memasang: callback ref dipanggil saat elemen baru dibuat, ketika
	// leluhurnya belum masuk dokumen — kalau dicek saat itu, daftar lain ikut dianggap "hilang".
	let bersihTerjadwal = false;
	function jadwalkanBersih() {
		if (bersihTerjadwal)
			return;
		bersihTerjadwal = true;
		nextTick(() => {
			bersihTerjadwal = false;
			for (const [node, s] of aktif) {
				if (!node.isConnected) {
					s.destroy();
					aktif.delete(node);
				}
			}
		});
	}

	function pasang(el: unknown, onPindah: (dari: number, ke: number) => void) {
		jadwalkanBersih();
		if (!(el instanceof HTMLElement) || aktif.has(el))
			return;
		aktif.set(el, Sortable.create(el, {
			handle: "[data-drag-handle]",
			animation: 150,
			forceFallback: true,
			onUpdate: (e) => {
				const { item, from, oldIndex, newIndex } = e;
				if (oldIndex === undefined || newIndex === undefined || oldIndex === newIndex)
					return;
				item.remove();
				from.insertBefore(item, from.children[oldIndex] ?? null);
				onPindah(oldIndex, newIndex);
			}
		}));
	}

	onBeforeUnmount(() => {
		aktif.forEach(s => s.destroy());
		aktif.clear();
	});

	return { pasang };
}

// Pindahkan satu elemen array dari indeks `dari` ke `ke` (mengembalikan array baru).
export function pindahkan<T>(arr: T[], dari: number, ke: number): T[] {
	const out = [...arr];
	const [x] = out.splice(dari, 1);
	out.splice(ke, 0, x as T);
	return out;
}
