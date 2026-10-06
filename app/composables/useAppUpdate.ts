import { invoke } from "@tauri-apps/api/core";
import { relaunch } from "@tauri-apps/plugin-process";
import { check, type Update } from "@tauri-apps/plugin-updater";

// Update otomatis dari GitHub Releases (endpoint & kunci publik di tauri.conf.json → plugins.updater).
// State sengaja di luar fungsi supaya notifikasi, modal, dan tombol di Pengaturan → Tentang
// berbagi satu sumber yang sama.
type UpdateStatus = "idle" | "checking" | "available" | "uptodate" | "downloading" | "installing" | "error";

const status = ref<UpdateStatus>("idle");
const update = shallowRef<Update | null>(null);
const errorMessage = ref("");
const downloaded = ref(0);
const total = ref<number | null>(null);
const modalOpen = ref(false);
// Ujian yang sedang berjalan / siswa yang sedang login — update me-restart aplikasi, dan server
// ujian ikut mati, jadi update ditahan selama masih ada aktivitas ujian.
const busyReason = ref<string | null>(null);
let notified = false;

interface ExamRow { scheduledAt: number | null, windowEnd: number | null }

const refreshBusy = async () => {
	try {
		const [online, exams] = await Promise.all([
			invoke<unknown[]>("list_online_students"),
			invoke<ExamRow[]>("list_exams")
		]);
		const now = Date.now() / 1000;
		const running = exams.filter((e) => e.scheduledAt !== null && e.windowEnd !== null && e.scheduledAt <= now && now < e.windowEnd).length;
		busyReason.value = online.length
			? `${online.length} siswa sedang login ujian.`
			: running
				? `${running} ujian sedang berlangsung.`
				: null;
	} catch {
		busyReason.value = null;
	}
};

export const useAppUpdate = () => {
	const toast = useToast();

	const progress = computed(() => (total.value ? Math.min(100, Math.round((downloaded.value / total.value) * 100)) : null));

	/** `silent` = pengecekan otomatis saat aplikasi dibuka: gagal (mis. offline) tidak ditampilkan. */
	const checkForUpdate = async ({ silent = false } = {}) => {
		if (status.value === "checking" || status.value === "downloading" || status.value === "installing") return;
		status.value = "checking";
		errorMessage.value = "";
		try {
			update.value = await check();
			status.value = update.value ? "available" : "uptodate";
			if (update.value && !notified) {
				notified = true;
				toast.add({
					title: `Versi ${update.value.version} tersedia`,
					description: "Pembaruan aplikasi siap dipasang.",
					icon: "i-lucide-download",
					color: "primary",
					duration: 0,
					actions: [{ label: "Lihat pembaruan", icon: "i-lucide-sparkles", color: "primary", variant: "solid", onClick: () => openModal() }]
				});
			}
		} catch (error) {
			status.value = silent ? "idle" : "error";
			errorMessage.value = error instanceof Error ? error.message : String(error);
		}
	};

	const openModal = () => {
		modalOpen.value = true;
		void refreshBusy();
	};

	const installUpdate = async () => {
		if (!update.value) return;
		await refreshBusy();
		if (busyReason.value) return;

		// Backup dulu — data ada di %APPDATA% dan tidak disentuh installer, tapi ini jaring pengaman
		// kalau ada yang tidak terduga.
		try {
			await invoke("backup_now");
		} catch {
			// Backup gagal (mis. disk penuh) tidak menghalangi update.
		}

		status.value = "downloading";
		downloaded.value = 0;
		total.value = null;
		try {
			await update.value.downloadAndInstall((event) => {
				if (event.event === "Started") total.value = event.data.contentLength ?? null;
				else if (event.event === "Progress") downloaded.value += event.data.chunkLength;
				else if (event.event === "Finished") status.value = "installing";
			});
			// Di Windows installer menutup aplikasi sendiri; di OS lain restart manual.
			await relaunch();
		} catch (error) {
			status.value = "error";
			errorMessage.value = error instanceof Error ? error.message : String(error);
		}
	};

	return {
		status,
		update,
		errorMessage,
		progress,
		modalOpen,
		busyReason,
		checkForUpdate,
		openModal,
		installUpdate,
		refreshBusy
	};
};
