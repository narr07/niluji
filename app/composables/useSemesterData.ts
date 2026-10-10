// Pola halaman referensi: muat data untuk semester login, dengan loading + pesan error, dan muat
// ulang otomatis kalau semester berganti. Tanpa semester (belum sinkron), data kosong tanpa query.
export function useSemesterData<T>(fetcher: (semesterId: string) => Promise<T[]>, judulError = "Gagal memuat data") {
	const { session } = useAuth();
	const toast = useToast();

	const rows = ref<T[]>([]) as Ref<T[]>;
	const loading = ref(true);
	const semesterId = computed(() => session.value?.semesterId ?? "");

	async function reload() {
		if (!semesterId.value) {
			rows.value = [];
			loading.value = false;
			return;
		}
		loading.value = true;
		try {
			rows.value = await fetcher(semesterId.value);
		}
		catch (e) {
			toast.add({ title: judulError, description: pesanError(e), color: "error" });
		}
		finally {
			loading.value = false;
		}
	}

	watch(semesterId, reload, { immediate: true });

	return { rows, loading, semesterId, reload };
}
