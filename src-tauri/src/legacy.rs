// Migrasi folder data dari identifier lama. Sampai v1.0.1 aplikasi masih memakai identifier
// warisan template ("com.nicolaspadari.nuxtor"), jadi database, gambar soal, dan backup sekolah
// ada di %APPDATA%\com.nicolaspadari.nuxtor. Sejak identifier diganti, Tauri menunjuk folder
// baru — tanpa migrasi ini sekolah yang update akan melihat aplikasi kosong.
//
// Isinya DISALIN (bukan dipindah): folder lama tetap utuh sebagai cadangan kalau ada apa-apa.
// Hanya dijalankan sekali — kalau folder baru sudah punya database, tidak ada yang disentuh.
use std::path::Path;

const LEGACY_IDENTIFIER: &str = "com.nicolaspadari.nuxtor";

pub fn migrate_legacy_data_dir(data_dir: &Path) {
	if data_dir.join("cbt.sqlite").exists() {
		return;
	}
	let Some(legacy_dir) = data_dir.parent().map(|p| p.join(LEGACY_IDENTIFIER)) else {
		return;
	};
	if legacy_dir == data_dir || !legacy_dir.join("cbt.sqlite").exists() {
		return;
	}
	match copy_dir(&legacy_dir, data_dir) {
		Ok(()) => println!("Data lama dari {} disalin ke {}", legacy_dir.display(), data_dir.display()),
		Err(e) => eprintln!("Gagal menyalin data lama dari {}: {e}", legacy_dir.display())
	}
}

fn copy_dir(from: &Path, to: &Path) -> std::io::Result<()> {
	std::fs::create_dir_all(to)?;
	for entry in std::fs::read_dir(from)? {
		let entry = entry?;
		let target = to.join(entry.file_name());
		if entry.file_type()?.is_dir() {
			// Folder cache webview (localStorage dll.) tidak perlu ikut — bukan data sekolah.
			if entry.file_name() == "EBWebView" {
				continue;
			}
			copy_dir(&entry.path(), &target)?;
		} else {
			std::fs::copy(entry.path(), target)?;
		}
	}
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn copies_legacy_data_once() {
		let root = std::env::temp_dir().join(format!(
			"niluji_legacy_test_{}_{}",
			std::process::id(),
			std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
		));
		let legacy = root.join(LEGACY_IDENTIFIER);
		std::fs::create_dir_all(legacy.join("uploads")).unwrap();
		std::fs::write(legacy.join("cbt.sqlite"), b"lama").unwrap();
		std::fs::write(legacy.join("uploads").join("a.jpg"), b"gambar").unwrap();

		let new_dir = root.join("com.narr07.niluji");
		migrate_legacy_data_dir(&new_dir);
		assert_eq!(std::fs::read(new_dir.join("cbt.sqlite")).unwrap(), b"lama");
		assert!(new_dir.join("uploads").join("a.jpg").exists());
		assert!(legacy.join("cbt.sqlite").exists(), "folder lama harus tetap utuh");

		// Panggilan kedua tidak menimpa data baru.
		std::fs::write(new_dir.join("cbt.sqlite"), b"baru").unwrap();
		migrate_legacy_data_dir(&new_dir);
		assert_eq!(std::fs::read(new_dir.join("cbt.sqlite")).unwrap(), b"baru");

		let _ = std::fs::remove_dir_all(&root);
	}
}
