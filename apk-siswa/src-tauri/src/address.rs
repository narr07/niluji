//! Membaca alamat server dari ketikan siswa atau isi QR kartu peserta.
//!
//! Hasilnya selalu `http://host[:port]` tanpa path. Path dan query lain dibuang supaya
//! aplikasi hanya pernah membuka `/cbt` di server itu, kecuali token ujian dari QR.

use std::net::IpAddr;
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerAddress {
	pub host: String,
	pub port: u16,
	pub token: Option<String>,
}

impl ServerAddress {
	/// Alamat dasar server, misalnya `http://192.168.1.10` atau `http://192.168.1.10:8080`.
	pub fn base_url(&self) -> String {
		if self.port == 80 {
			format!("http://{}", self.host)
		} else {
			format!("http://{}:{}", self.host, self.port)
		}
	}

	/// Alamat portal ujian yang dibuka di WebView.
	pub fn exam_url(&self) -> String {
		match &self.token {
			Some(token) => format!("{}/cbt?token={}", self.base_url(), token),
			None => format!("{}/cbt", self.base_url()),
		}
	}

	/// Hanya halaman di host dan port ini yang boleh dibuka selama ujian.
	pub fn allows(&self, url: &Url) -> bool {
		url.scheme() == "http"
			&& url.host_str().map(|h| h.eq_ignore_ascii_case(&self.host)).unwrap_or(false)
			&& url.port_or_known_default() == Some(self.port)
	}
}

pub const ERR_EMPTY: &str = "Isi alamat server ujian terlebih dahulu.";
pub const ERR_INVALID: &str = "Alamat server tidak dikenali. Contoh yang benar: 192.168.1.10 atau 192.168.1.10:8080.";
pub const ERR_HTTPS: &str = "Server NILUJI di ruang ujian memakai http, bukan https.";
pub const ERR_PUBLIC: &str = "Alamat ini bukan alamat jaringan lokal. Pakai alamat yang tertera di kartu peserta atau layar pengawas.";

pub fn parse(input: &str) -> Result<ServerAddress, String> {
	let trimmed = input.trim();
	if trimmed.is_empty() {
		return Err(ERR_EMPTY.into());
	}
	if trimmed.to_ascii_lowercase().starts_with("https://") {
		return Err(ERR_HTTPS.into());
	}
	let with_scheme = if trimmed.contains("://") { trimmed.to_string() } else { format!("http://{trimmed}") };
	let url = Url::parse(&with_scheme).map_err(|_| ERR_INVALID.to_string())?;
	if url.scheme() != "http" || !url.username().is_empty() || url.password().is_some() {
		return Err(ERR_INVALID.into());
	}
	let host = match url.host() {
		Some(url::Host::Ipv4(ip)) => {
			if !is_local(IpAddr::V4(ip)) {
				return Err(ERR_PUBLIC.into());
			}
			ip.to_string()
		}
		Some(url::Host::Domain(name)) => {
			// Nama host hanya diterima bila jelas milik jaringan lokal, misalnya `proktor.local`.
			let name = name.to_ascii_lowercase();
			let local_name = !name.contains('.') || name.ends_with(".local") || name.ends_with(".lan") || name.ends_with(".home.arpa");
			if !local_name || (name == "localhost" && cfg!(mobile)) {
				return Err(ERR_PUBLIC.into());
			}
			name
		}
		_ => return Err(ERR_INVALID.into()),
	};
	let port = url.port_or_known_default().unwrap_or(80);
	let token = url
		.query_pairs()
		.find(|(key, _)| key == "token")
		.map(|(_, value)| value.trim().to_ascii_uppercase())
		.filter(|value| !value.is_empty() && value.len() <= 32 && value.chars().all(|c| c.is_ascii_alphanumeric()));
	Ok(ServerAddress { host, port, token })
}

/// Alamat privat (RFC 1918), CGNAT hotspot, link-local, dan loopback untuk uji di laptop.
fn is_local(ip: IpAddr) -> bool {
	match ip {
		IpAddr::V4(v4) => {
			let o = v4.octets();
			v4.is_private() || v4.is_link_local() || (o[0] == 100 && (64..=127).contains(&o[1])) || (v4.is_loopback() && !cfg!(mobile))
		}
		IpAddr::V6(_) => false,
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn reads_plain_ip() {
		let a = parse("192.168.1.10").unwrap();
		assert_eq!(a.base_url(), "http://192.168.1.10");
		assert_eq!(a.exam_url(), "http://192.168.1.10/cbt");
	}

	#[test]
	fn reads_qr_from_card_with_port() {
		let a = parse("http://10.0.0.5:8080/cbt").unwrap();
		assert_eq!(a.base_url(), "http://10.0.0.5:8080");
		assert_eq!(a.port, 8080);
		assert_eq!(a.token, None);
	}

	#[test]
	fn keeps_only_a_safe_token() {
		assert_eq!(parse("http://192.168.1.10/cbt?token=abc123").unwrap().token.as_deref(), Some("ABC123"));
		assert_eq!(parse("http://192.168.1.10/cbt?token=<script>").unwrap().token, None);
	}

	#[test]
	fn drops_other_paths() {
		assert_eq!(parse("192.168.1.10/admin?x=1").unwrap().exam_url(), "http://192.168.1.10/cbt");
	}

	#[test]
	fn rejects_public_and_odd_addresses() {
		assert_eq!(parse("8.8.8.8").unwrap_err(), ERR_PUBLIC);
		assert_eq!(parse("google.com").unwrap_err(), ERR_PUBLIC);
		assert_eq!(parse("https://192.168.1.10").unwrap_err(), ERR_HTTPS);
		assert_eq!(parse("  ").unwrap_err(), ERR_EMPTY);
		assert_eq!(parse("ftp://192.168.1.10").unwrap_err(), ERR_INVALID);
		assert_eq!(parse("http://user:pass@192.168.1.10").unwrap_err(), ERR_INVALID);
		assert!(parse("[fe80::1]").is_err());
	}

	#[test]
	fn accepts_local_names_and_hotspot_ranges() {
		assert!(parse("proktor.local").is_ok());
		assert!(parse("100.80.1.2").is_ok());
		assert!(parse("172.20.10.1").is_ok());
	}

	#[test]
	fn navigation_is_limited_to_the_chosen_server() {
		let a = parse("192.168.1.10").unwrap();
		assert!(a.allows(&Url::parse("http://192.168.1.10/cbt?token=X").unwrap()));
		assert!(a.allows(&Url::parse("http://192.168.1.10:80/uploads/a.png").unwrap()));
		assert!(!a.allows(&Url::parse("http://192.168.1.10:8080/cbt").unwrap()));
		assert!(!a.allows(&Url::parse("http://192.168.1.11/cbt").unwrap()));
		assert!(!a.allows(&Url::parse("https://192.168.1.10/cbt").unwrap()));
	}
}
