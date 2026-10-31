export default defineNuxtPlugin(() => {
	if (typeof window === "undefined") return;
	const noop = () => {};
	if (typeof window.matchMedia === "function") {
		const origMatchMedia = window.matchMedia.bind(window);
		window.matchMedia = (query: string) => {
			const mql = origMatchMedia(query) || ({ matches: false, media: query } as MediaQueryList);
			if (typeof mql.addEventListener !== "function") {
				mql.addEventListener = typeof mql.addListener === "function"
					? (_type: string, fn: EventListenerOrEventListenerObject | null) => {
						if (typeof fn === "function") mql.addListener(fn as (this: MediaQueryList, ev: MediaQueryListEvent) => unknown);
					}
					: noop;
			}
			if (typeof mql.removeEventListener !== "function") {
				mql.removeEventListener = typeof mql.removeListener === "function"
					? (_type: string, fn: EventListenerOrEventListenerObject | null) => {
						if (typeof fn === "function") mql.removeListener(fn as (this: MediaQueryList, ev: MediaQueryListEvent) => unknown);
					}
					: noop;
			}
			return mql;
		};
	}
	if (typeof navigator !== "undefined" && navigator.permissions && typeof navigator.permissions.query === "function") {
		const origQuery = navigator.permissions.query.bind(navigator.permissions);
		navigator.permissions.query = (desc: PermissionDescriptor) => {
			return Promise.resolve(origQuery(desc)).then((status) => {
				if (status && typeof status.addEventListener !== "function") {
					status.addEventListener = noop;
					status.removeEventListener = noop;
				}
				return status;
			});
		};
	}
});
