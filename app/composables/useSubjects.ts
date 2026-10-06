import { invoke } from "@tauri-apps/api/core";

export interface SubjectInfo {
	id: number
	name: string
	code: string | null
}

// Cache mapel bersama untuk seluruh aplikasi. URL halaman memakai NAMA LENGKAP mapel, sedangkan
// judul/breadcrumb menampilkan KODE-nya ("PAI") — kalau tiap halaman mengambil list_subjects
// sendiri, judulnya sempat menampilkan nama panjang dulu lalu berkedip jadi kode. Dengan cache
// ini (dimuat sekali di layout), kodenya sudah tersedia saat halaman dibuka.
// Pola stale-while-revalidate: tiap pemanggilan useSubjects() langsung memakai cache, sambil
// memuat ulang di belakang layar supaya perubahan dari Pengaturan → Mata Pelajaran ikut terbawa.
const subjects = ref<SubjectInfo[] | null>(null);
let inflight: Promise<void> | null = null;

const refreshSubjects = () => {
	inflight ??= invoke<SubjectInfo[]>("list_subjects")
		.then((list) => {
			subjects.value = list;
		})
		.catch(() => {
			// Biarkan cache lama (atau null) — halaman tetap jalan dengan nama dari URL.
		})
		.finally(() => {
			inflight = null;
		});
	return inflight;
};

export const useSubjects = () => {
	void refreshSubjects();
	return { subjects, refreshSubjects };
};

/**
 * Mapel & label singkat (kode, atau nama kalau kodenya kosong) untuk nama mapel dari URL.
 * `label` bernilai null selama cache belum termuat — tampilkan judul tanpa mapel dulu, jangan
 * nama panjangnya, supaya tidak berkedip.
 */
export const useSubjectByName = (name: MaybeRefOrGetter<string>) => {
	const { subjects } = useSubjects();
	const subject = computed(() => subjects.value?.find((s) => s.name === toValue(name)));
	const label = computed(() => (subjects.value ? subject.value?.code || toValue(name) : null));
	return { subject, label };
};
