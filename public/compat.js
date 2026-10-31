(function () {
	if (typeof window === "undefined") return;
	var noop = function () {};
	if (typeof window.matchMedia === "function") {
		var origMatchMedia = window.matchMedia.bind(window);
		window.matchMedia = function (query) {
			var mql = origMatchMedia(query) || { matches: false, media: query };
			if (typeof mql.addEventListener !== "function") {
				mql.addEventListener = typeof mql.addListener === "function"
					? function (_type, fn) { mql.addListener(fn); }
					: noop;
			}
			if (typeof mql.removeEventListener !== "function") {
				mql.removeEventListener = typeof mql.removeListener === "function"
					? function (_type, fn) { mql.removeListener(fn); }
					: noop;
			}
			return mql;
		};
	} else {
		window.matchMedia = function (query) {
			return {
				matches: false,
				media: query,
				onchange: null,
				addListener: noop,
				removeListener: noop,
				addEventListener: noop,
				removeEventListener: noop,
				dispatchEvent: function () { return false; }
			};
		};
	}
	if (typeof navigator !== "undefined" && navigator.permissions && typeof navigator.permissions.query === "function") {
		var origQuery = navigator.permissions.query.bind(navigator.permissions);
		navigator.permissions.query = function (desc) {
			return Promise.resolve(origQuery(desc)).then(function (status) {
				if (status && typeof status.addEventListener !== "function") {
					status.addEventListener = noop;
					status.removeEventListener = noop;
				}
				return status;
			});
		};
	}
})();

