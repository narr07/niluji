// Mencocokkan tulisan kelas dari data sekolah ("IVA", "IV B", "4A", "Kelas 4-A", "V") dengan
// kelas yang terdaftar di Pengaturan > Data Kelas ("4", "5", "6").
//
// NILUJI memisahkan dua hal:
// - kelas (tingkat): dipakai bank soal, jadwal ujian, dan pemeriksaan kelas saat siswa masuk;
// - rombel: label rombongan belajar sekolah (IVA, IVB), dipakai untuk mengelompokkan siswa.
// Jadi satu STS Kelas 4 cukup dibuat sekali untuk semua rombel kelas 4.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassMatch {
	// Judul kelas terdaftar yang cocok. `None` bila tidak ada yang cocok.
	pub class: Option<String>,
	// Label rombel apa adanya (tanpa awalan "Kelas"), bila tulisan aslinya memuat rombel.
	pub rombel: Option<String>
}

const ROMAN: [(&str, u32); 12] = [
	("XII", 12),
	("XI", 11),
	("X", 10),
	("IX", 9),
	("VIII", 8),
	("VII", 7),
	("VI", 6),
	("V", 5),
	("IV", 4),
	("III", 3),
	("II", 2),
	("I", 1)
];

fn strip_kelas_prefix(raw: &str) -> &str {
	let trimmed = raw.trim();
	let lower = trimmed.to_lowercase();
	if lower.starts_with("kelas") {
		trimmed[5..].trim_start_matches(|c: char| c.is_whitespace() || matches!(c, ':' | '.' | '-'))
	} else {
		trimmed
	}
}

// Angka tingkat di awal tulisan beserta sisanya: "4A" -> (4, "A"), "IVB" -> (4, "B").
// Angka romawi dibaca serakus mungkin, jadi "VIA" adalah VI + A, bukan V + IA.
fn leading_level(text: &str) -> Option<(u32, &str)> {
	let digits: String = text.chars().take_while(char::is_ascii_digit).collect();
	if !digits.is_empty() {
		return digits.parse().ok().map(|n| (n, &text[digits.len()..]));
	}
	let upper = text.to_uppercase();
	ROMAN
		.iter()
		.filter(|(numeral, _)| upper.starts_with(numeral))
		.max_by_key(|(numeral, _)| numeral.len())
		.map(|(numeral, value)| (*value, &text[numeral.len()..]))
}

fn find_registered<'a>(name: &str, registered: &'a [String]) -> Option<&'a String> {
	registered.iter().find(|r| r.trim().eq_ignore_ascii_case(name.trim()))
}

pub fn match_class(raw: &str, registered: &[String]) -> ClassMatch {
	let none = ClassMatch { class: None, rombel: None };
	let raw = raw.trim();
	if raw.is_empty() {
		return none;
	}
	// Sekolah yang mendaftarkan rombel sebagai kelas sendiri ("IVA") tetap didukung.
	if let Some(found) = find_registered(raw, registered) {
		return ClassMatch { class: Some(found.clone()), rombel: None };
	}
	let body = strip_kelas_prefix(raw);
	if let Some(found) = find_registered(body, registered) {
		return ClassMatch { class: Some(found.clone()), rombel: None };
	}

	let Some((level, rest)) = leading_level(body) else {
		return none;
	};
	let suffix = rest.trim_start_matches(|c: char| c.is_whitespace() || matches!(c, '-' | '.' | '/' | '_'));
	// Sisa yang bukan label rombel pendek (mis. "4 jam") berarti tulisan ini bukan kelas.
	if suffix.chars().count() > 6 || !suffix.chars().all(|c| c.is_alphanumeric() || c == ' ') {
		return none;
	}
	let Some(found) = find_registered(&level.to_string(), registered) else {
		return none;
	};
	ClassMatch {
		class: Some(found.clone()),
		rombel: (!suffix.is_empty()).then(|| body.to_string())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn registered() -> Vec<String> {
		vec!["4".into(), "5".into(), "6".into()]
	}

	fn check(raw: &str) -> (Option<String>, Option<String>) {
		let m = match_class(raw, &registered());
		(m.class, m.rombel)
	}

	fn some(class: &str, rombel: Option<&str>) -> (Option<String>, Option<String>) {
		(Some(class.to_string()), rombel.map(String::from))
	}

	#[test]
	fn reads_roman_and_digit_levels_with_rombel() {
		assert_eq!(check("IVA"), some("4", Some("IVA")));
		assert_eq!(check("IV B"), some("4", Some("IV B")));
		assert_eq!(check("ivb"), some("4", Some("ivb")));
		assert_eq!(check("VIA"), some("6", Some("VIA")));
		assert_eq!(check("4A"), some("4", Some("4A")));
		assert_eq!(check("Kelas 4-A"), some("4", Some("4-A")));
		assert_eq!(check("5.2"), some("5", Some("5.2")));
	}

	#[test]
	fn reads_levels_without_rombel() {
		assert_eq!(check("V"), some("5", None));
		assert_eq!(check("VI"), some("6", None));
		assert_eq!(check("4"), some("4", None));
		assert_eq!(check("Kelas 6"), some("6", None));
		assert_eq!(check(" kelas IV "), some("4", None));
	}

	#[test]
	fn keeps_classes_registered_under_their_own_name() {
		let registered = vec!["IVA".to_string(), "IVB".to_string()];
		assert_eq!(match_class("iva", &registered), ClassMatch { class: Some("IVA".into()), rombel: None });
	}

	#[test]
	fn leaves_unknown_classes_unmatched() {
		for raw in ["", "III", "VIIA", "Paket B", "4 jam pelajaran", "Alumni"] {
			assert_eq!(check(raw), (None, None), "{raw}");
		}
	}
}
