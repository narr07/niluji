import { getCurrentWebview } from "@tauri-apps/api/webview";

// Drag-and-drop file di aplikasi Tauri HARUS lewat API drag-drop Tauri sendiri
// (getCurrentWebview().onDragDropEvent), bukan event HTML5 `drop` biasa — browser sengaja
// menyembunyikan path asli file yang di-drop (cuma kasih nama + isi), padahal semua alur
// import di aplikasi ini butuh PATH file di disk (buat dikirim ke command Rust). API Tauri ini
// yang kasih path aslinya.
//
// Listener-nya aktif di level jendela (bukan per-elemen), makanya `isActive` dipakai buat
// membatasi supaya cuma modal yang lagi benar-benar terbuka yang bereaksi — modal lain yang
// kebetulan sama-sama ada di halaman (tapi tertutup) harus diam saja.
export const useFileDrop = (options: {
	isActive: () => boolean
	accept: (path: string) => boolean
	// Semua path yang lolos filter `accept` dari SATU aksi drop (bisa lebih dari satu kalau
	// beberapa file di-drag bareng) — pemanggil yang cuma butuh 1 file cukup ambil `paths[0]`.
	onDrop: (paths: string[]) => void
}) => {
	const isDragging = ref(false);
	let unlisten: (() => void) | null = null;

	onMounted(async () => {
		unlisten = await getCurrentWebview().onDragDropEvent((event) => {
			if (!options.isActive()) {
				isDragging.value = false;
				return;
			}
			if (event.payload.type === "enter" || event.payload.type === "over") {
				isDragging.value = true;
			} else if (event.payload.type === "drop") {
				isDragging.value = false;
				const paths = event.payload.paths.filter(options.accept);
				if (paths.length) options.onDrop(paths);
			} else {
				isDragging.value = false;
			}
		});
	});

	onUnmounted(() => {
		unlisten?.();
	});

	return { isDragging };
};
