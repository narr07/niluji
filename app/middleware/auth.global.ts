// Middleware ini hanya berlaku untuk rute e-Rapor (/e-rapor/*).
// Rute CBT Niluji (/, /soal, /ujian, /cbt, /hasil, /siswa) tidak terpengaruh.
const PUBLIC = new Set(["/e-rapor/login", "/e-rapor/sinkron"]);

const ADMIN_ONLY = [
	"/e-rapor/sesi-online",
	"/e-rapor/dapodik",
	"/e-rapor/pengguna",
	"/e-rapor/referensi",
	"/e-rapor/backup",
	"/e-rapor/kirim-nilai",
	"/e-rapor/pengaturan",
	"/e-rapor/status-penilaian",
	"/e-rapor/perkembangan-nilai",
	"/e-rapor/kokurikuler",
	"/e-rapor/cetak",
	"/e-rapor/transkrip",
	"/e-rapor/kartu-akun"
];

export default defineNuxtRouteMiddleware(async (to) => {
	// Hanya tangani rute yang diawali /e-rapor
	if (!to.path.startsWith("/e-rapor"))
		return;

	if (PUBLIC.has(to.path))
		return;

	const { session, loadWali, waliRombel } = useAuth();
	if (!session.value)
		return navigateTo("/e-rapor/login");

	if (session.value.user.level === "guru" && !waliRombel.value.length)
		await loadWali();

	if (session.value.user.level !== "admin" && ADMIN_ONLY.some(p => to.path.startsWith(p)))
		return navigateTo("/e-rapor");
});
