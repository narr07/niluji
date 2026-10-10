// Mode aplikasi: "admin" (laptop server, ada Dapodik) atau "guru" (laptop guru, data hasil tarik
// dari admin). Mode guru diset otomatis saat pertama kali tarik data.
export type AppMode = "admin" | "guru";

export function useAppMode() {
	const mode = useState<AppMode>("app-mode", () => "admin");
	const loaded = useState("app-mode-loaded", () => false);
	// Laptop admin = mode admin dan sudah ada koneksi Web Service / data Dapodik. Di laptop ini
	// "Tarik data" dilarang: datanya akan tertimpa data satu guru saja.
	const adminLaptop = useState("admin-laptop", () => false);

	async function load() {
		try {
			const db = useDb();
			mode.value = ((await db.getSetting("mode")) as AppMode) || "admin";
			adminLaptop.value = mode.value === "admin"
				&& (await db.scalar<number>("SELECT (SELECT COUNT(*) FROM webservice) + (SELECT COUNT(*) FROM rombel)")) > 0;
		}
		catch {
			mode.value = "admin";
		}
		loaded.value = true;
	}

	async function set(m: AppMode) {
		await useDb().setSetting("mode", m);
		mode.value = m;
	}

	return { mode, loaded, load, set, adminLaptop, isGuruMode: computed(() => mode.value === "guru") };
}
