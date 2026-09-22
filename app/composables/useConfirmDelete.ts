// Dialog konfirmasi hapus yang seragam di seluruh aplikasi — dipakai lewat `await confirmDelete(...)`
// sebagai pengganti `confirm()` bawaan browser (gampang ke-klik tanpa sengaja/tidak kebaca jelas).
// Statenya sengaja di LUAR fungsi (bukan di dalam useConfirmDelete()) supaya semua pemanggil
// berbagi satu instance modal yang sama (dipasang sekali di app.vue), bukan bikin modal baru
// tiap komponen yang butuh konfirmasi hapus.
const open = ref(false);
const title = ref("");
const description = ref("");
const confirmLabel = ref("Hapus");
let resolver: ((value: boolean) => void) | null = null;

export const useConfirmDeleteState = () => ({ open, title, description, confirmLabel });

export const resolveConfirmDelete = (value: boolean) => {
	open.value = false;
	resolver?.(value);
	resolver = null;
};

export interface ConfirmDeleteOptions {
	title: string
	description?: string
	confirmLabel?: string
}

export const confirmDelete = (options: ConfirmDeleteOptions): Promise<boolean> => {
	title.value = options.title;
	description.value = options.description ?? "";
	confirmLabel.value = options.confirmLabel ?? "Hapus";
	open.value = true;
	return new Promise<boolean>((resolve) => {
		resolver = resolve;
	});
};
