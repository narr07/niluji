// Penanda posisi gambar yang disisipkan guru di dalam teks soal lewat tombol "Sisipkan Gambar"
// di toolbar editor (lihat SoalEditor.vue) — dipakai supaya gambar bisa muncul di TENGAH teks
// (mis. "Perhatikan gambar berikut! [[gambar]] Apa nama alat di atas?"), bukan cuma selalu di
// bawah semua teks. Soal lama yang belum pernah pakai tombol ini tidak punya penanda sama sekali
// di teksnya — itu sengaja, supaya pemanggil (cbt.vue dll) tahu kapan harus fallback ke perilaku
// lama (gambar tetap render di bawah teks, lewat elemen <img> terpisah).
//
// Yang benar-benar DISISIPKAN ke editor cuma "[[gambar]]" polos — tapi begitu UEditor
// menyerialisasi isinya ke markdown (setiap kali ada perubahan), tanda kurung siku ikut
// di-escape jadi "\[\[gambar\]\]" (aturan markdown umum, supaya "[...]" tidak kebaca sebagai
// awal link/gambar beneran). Jadi yang benar-benar TERSIMPAN di database bentuknya yang sudah
// di-escape itu — pola pencarian di bawah ini sengaja mentolerir backslash opsional di tiap
// kurung siku, supaya cocok baik untuk bentuk polos maupun yang sudah di-escape.
const IMAGE_PLACEHOLDER_SOURCE = "\\\\?\\[\\\\?\\[gambar\\\\?\\]\\\\?\\]";

export const hasSoalImagePlaceholder = (raw: string): boolean => new RegExp(IMAGE_PLACEHOLDER_SOURCE).test(raw ?? "");

export const insertSoalImagePlaceholder = "[[gambar]]";

// Konten soal disimpan sebagai markdown ringan (dari UEditor: **tebal**, *miring*, <u>garis
// bawah</u>, ~~coret~~, [[gambar]]). Untuk ditampilkan (bukan diedit) di halaman ujian siswa,
// tabel Bank Soal, dan halaman penilaian, teksnya di-escape dulu SEBELUM pola markdown diproses —
// jadi tag HTML apa pun yang kebetulan ada di teks soal tidak pernah dirender sebagai HTML asli,
// cuma 4 tag aman (strong/em/u/s) plus <img> dari penanda [[gambar]] yang sengaja disisipkan di
// sini yang bisa muncul.
export const renderSoalMarkdown = (raw: string, imageUrl?: string | null): string => {
	if (!raw) return "";

	const escaped = raw
		.replace(/&/g, "&amp;")
		.replace(/</g, "&lt;")
		.replace(/>/g, "&gt;");

	const withMarks = escaped
		.replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>")
		.replace(/(?<!\*)\*([^*\n]+?)\*(?!\*)/g, "<em>$1</em>")
		.replace(/&lt;u&gt;(.+?)&lt;\/u&gt;/g, "<u>$1</u>")
		.replace(/~~(.+?)~~/g, "<s>$1</s>");

	const imageTag = imageUrl ? `<img src="${imageUrl}" class="max-w-full rounded-md my-2" alt="">` : "";
	const withImage = withMarks.replace(new RegExp(IMAGE_PLACEHOLDER_SOURCE, "g"), imageTag);

	return withImage.replace(/\n/g, "<br>");
};

// Versi teks polos (tanpa tag) buat pratinjau singkat di tabel/daftar, biar tidak ada simbol
// markdown ("**", "~~", "[[gambar]]", dst) yang ikut kelihatan.
export const stripSoalMarkdown = (raw: string): string =>
	(raw ?? "")
		.replace(/\*\*(.+?)\*\*/g, "$1")
		.replace(/(?<!\*)\*([^*\n]+?)\*(?!\*)/g, "$1")
		.replace(/<u>(.+?)<\/u>/g, "$1")
		.replace(/~~(.+?)~~/g, "$1")
		.replace(new RegExp(IMAGE_PLACEHOLDER_SOURCE, "g"), "");
