// Popup konfirmasi di tengah layar (mirip SweetAlert yang dipakai e-Rapor: "Sukses", "Selesai", "Gagal").
export interface SwalState {
	open: boolean
	type: "success" | "error" | "warning" | "info"
	title: string
	text?: string
	confirmText?: string
	cancelText?: string
	resolve?: (ok: boolean) => void
}

export function useSwal() {
	const state = useState<SwalState>("swal", () => ({ open: false, type: "info", title: "" }));

	function fire(opts: Omit<SwalState, "open" | "resolve">): Promise<boolean> {
		return new Promise((resolve) => {
			state.value = { ...opts, open: true, resolve };
		});
	}

	const success = (title: string, text?: string) => fire({ type: "success", title, text });
	const error = (title: string, text?: string) => fire({ type: "error", title, text });
	const confirm = (title: string, text?: string, confirmText = "Ya") =>
		fire({ type: "warning", title, text, confirmText, cancelText: "Batal" });

	function close(ok: boolean) {
		state.value.resolve?.(ok);
		state.value = { ...state.value, open: false, resolve: undefined };
	}

	return { state, fire, success, error, confirm, close };
}
