// Konten soal disimpan sebagai markdown ringan (dari UEditor: **tebal**, *miring*, <u>garis
// bawah</u>, ~~coret~~). Untuk ditampilkan (bukan diedit) di halaman ujian siswa, tabel Bank
// Soal, dan halaman penilaian, teksnya di-escape dulu SEBELUM pola markdown diproses — jadi
// tag HTML apa pun yang kebetulan ada di teks soal tidak pernah dirender sebagai HTML asli,
// cuma 4 tag aman (strong/em/u/s) yang sengaja disisipkan di sini yang bisa muncul.
export const renderSoalMarkdown = (raw: string): string => {
	if (!raw) return "";

	const escaped = raw
		.replace(/&/g, "&amp;")
		.replace(/</g, "&lt;")
		.replace(/>/g, "&gt;");

	return escaped
		.replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>")
		.replace(/(?<!\*)\*([^*\n]+?)\*(?!\*)/g, "<em>$1</em>")
		.replace(/&lt;u&gt;(.+?)&lt;\/u&gt;/g, "<u>$1</u>")
		.replace(/~~(.+?)~~/g, "<s>$1</s>")
		.replace(/\n/g, "<br>");
};

// Versi teks polos (tanpa tag) buat pratinjau singkat di tabel/daftar, biar tidak ada simbol
// markdown ("**", "~~", dst) yang ikut kelihatan.
export const stripSoalMarkdown = (raw: string): string =>
	(raw ?? "")
		.replace(/\*\*(.+?)\*\*/g, "$1")
		.replace(/(?<!\*)\*([^*\n]+?)\*(?!\*)/g, "$1")
		.replace(/<u>(.+?)<\/u>/g, "$1")
		.replace(/~~(.+?)~~/g, "$1");
