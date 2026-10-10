import { invoke } from "@tauri-apps/api/core";

export type UserLevel = "admin" | "guru" | "siswa";

export interface SessionUser {
	id: number
	username: string
	nama: string
	email: string | null
	level: UserLevel
	ptk_id: string | null
	peserta_didik_id: string | null
	last_login: string | null
}

export interface Session {
	user: SessionUser
	sekolahId: string | null
	sekolahNama: string | null
	semesterId: string | null
}

const STORAGE_KEY = "erapor:session";

function readStored(): Session | null {
	try {
		const raw = sessionStorage.getItem(STORAGE_KEY);
		return raw ? JSON.parse(raw) : null;
	}
	catch {
		return null;
	}
}

export const LEVEL_LABEL: Record<UserLevel, string> = {
	admin: "Admin",
	guru: "Guru",
	siswa: "Siswa"
};

export function useAuth() {
	const session = useState<Session | null>("session", () => readStored());
	// Guru yang tercatat sebagai wali kelas (rombel reguler) di semester login dapat menu Wali Kelas.
	const waliRombel = useState<{ rombongan_belajar_id: string, nama: string }[]>("wali-rombel", () => []);

	const user = computed(() => session.value?.user ?? null);
	const isWali = computed(() => waliRombel.value.length > 0);

	function persist() {
		try {
			if (session.value)
				sessionStorage.setItem(STORAGE_KEY, JSON.stringify(session.value));
			else
				sessionStorage.removeItem(STORAGE_KEY);
		}
		catch {}
	}

	// Satu-satunya jalan mengubah sesi yang sudah login (supaya penyimpanannya di satu tempat).
	function updateSession(patch: Partial<Omit<Session, "user">>) {
		if (!session.value)
			return;
		session.value = { ...session.value, ...patch };
		persist();
	}

	async function loadWali() {
		const s = session.value;
		if (!s || s.user.level !== "guru" || !s.user.ptk_id || !s.semesterId) {
			waliRombel.value = [];
			return;
		}
		waliRombel.value = await useDb().query(
			"SELECT rombongan_belajar_id, nama FROM rombel WHERE ptk_id = ? AND semester_id = ? AND jenis_rombel = '1' ORDER BY tingkat, nama",
			[s.user.ptk_id, s.semesterId]
		);
	}

	async function login(input: { username: string, password: string, sekolahId: string | null, semesterId: string | null }) {
		const u = await invoke<SessionUser>("auth_login", { username: input.username, password: input.password });
		const db = useDb();
		const sekolah = input.sekolahId
			? await db.first<{ nama: string }>("SELECT nama FROM sekolah WHERE sekolah_id = ?", [input.sekolahId])
			: undefined;
		session.value = {
			user: u,
			sekolahId: input.sekolahId,
			sekolahNama: sekolah?.nama ?? null,
			semesterId: input.semesterId
		};
		persist();
		await loadWali();
		return u;
	}

	async function logout() {
		const id = session.value?.user.id;
		if (id)
			await invoke("auth_logout", { userId: id }).catch(() => {});
		session.value = null;
		waliRombel.value = [];
		persist();
		await navigateTo("/e-rapor/login");
	}

	const headerSemester = computed(() => semesterLabel(session.value?.semesterId));

	return { session, user, isWali, waliRombel, login, logout, loadWali, updateSession, headerSemester };
}
