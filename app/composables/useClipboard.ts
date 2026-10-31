export const useClipboard = (options?: { copiedDuring?: number }) => {
	const copiedDuring = options?.copiedDuring ?? 1500;
	const copied = ref(false);
	const text = ref("");
	let timer: ReturnType<typeof setTimeout> | null = null;

	const copy = async (value?: string) => {
		if (value == null || typeof window === "undefined") return;
		try {
			if (navigator?.clipboard?.writeText) {
				await navigator.clipboard.writeText(value);
			} else {
				const el = document.createElement("textarea");
				el.value = value;
				el.style.position = "fixed";
				el.style.opacity = "0";
				el.setAttribute("readonly", "");
				document.body.appendChild(el);
				el.select();
				document.execCommand("copy");
				el.remove();
			}
		} catch {
			const el = document.createElement("textarea");
			el.value = value;
			el.style.position = "fixed";
			el.style.opacity = "0";
			el.setAttribute("readonly", "");
			document.body.appendChild(el);
			el.select();
			document.execCommand("copy");
			el.remove();
		}
		text.value = value;
		copied.value = true;
		if (timer) clearTimeout(timer);
		timer = setTimeout(() => {
			copied.value = false;
		}, copiedDuring);
	};

	return {
		copy,
		copied: readonly(copied),
		text: readonly(text),
		isSupported: computed(() => true)
	};
};

