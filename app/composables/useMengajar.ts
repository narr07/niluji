export interface Mengajar {
	id: number
	rombongan_belajar_id: string
	rombel: string
	tingkat: number
	kode_mapel: string
	mapel: string
	singkat: string
	terkunci: number
}

// Pembelajaran rapor yang diampu guru yang sedang login, di semester login.
export function useMengajar() {
	const { session, user } = useAuth();
	const db = useDb();
	const list = ref<Mengajar[]>([]);
	const loading = ref(true);

	async function load() {
		loading.value = true;
		list.value = user.value?.ptk_id
			? await db.query<Mengajar>(
				`SELECT pr.id, pr.rombongan_belajar_id, r.nama AS rombel, CAST(r.tingkat AS INTEGER) AS tingkat,
					pr.kode_mapel, m.nama AS mapel, m.singkat, pr.terkunci
				FROM pembelajaran_rapor pr JOIN rombel r USING (rombongan_belajar_id) JOIN mapel_rapor m ON m.kode = pr.kode_mapel
				WHERE pr.ptk_id = ? AND pr.semester_id = ? ORDER BY CAST(r.tingkat AS INTEGER), r.nama, m.urutan`,
				[user.value.ptk_id, session.value?.semesterId]
			)
			: [];
		loading.value = false;
	}

	const inputDibuka = ref(true);
	async function loadStatus() {
		inputDibuka.value = (await db.getSetting(`input_nilai_dibuka:${session.value?.semesterId}`)) !== "0";
	}

	onMounted(() => Promise.all([load(), loadStatus()]));

	return { list, loading, reload: load, inputDibuka, semesterKe: computed(() => semesterKe(session.value?.semesterId)) };
}
