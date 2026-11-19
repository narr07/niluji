//! Memastikan alamat yang dituju benar-benar server NILUJI sebelum portal dibuka.
//!
//! Memakai HTTP/1.1 sederhana lewat `TcpStream` supaya tidak butuh pustaka HTTP tambahan.
//! Fungsi ini memblokir, jadi selalu dipanggil dari `spawn_blocking`.

use crate::address::ServerAddress;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

const TIMEOUT: Duration = Duration::from_secs(4);
const MAX_RESPONSE: usize = 8 * 1024;

pub const ERR_UNREACHABLE: &str = "Server tidak bisa dihubungi. Pastikan HP tersambung ke WiFi ruang ujian dan alamatnya sama dengan yang ditampilkan pengawas.";
pub const ERR_NOT_NILUJI: &str = "Alamat ini bisa dihubungi, tetapi bukan server NILUJI. Periksa lagi alamat dan nomor port-nya.";
pub const ERR_DB: &str = "Server NILUJI ditemukan, tetapi database-nya sedang bermasalah. Beri tahu pengawas.";

pub fn check(address: &ServerAddress) -> Result<(), String> {
	let targets: Vec<_> = (address.host.as_str(), address.port)
		.to_socket_addrs()
		.map_err(|_| ERR_UNREACHABLE.to_string())?
		.collect();
	let mut stream = targets
		.iter()
		.find_map(|target| TcpStream::connect_timeout(target, TIMEOUT).ok())
		.ok_or_else(|| ERR_UNREACHABLE.to_string())?;
	stream.set_read_timeout(Some(TIMEOUT)).ok();
	stream.set_write_timeout(Some(TIMEOUT)).ok();

	let host_header = if address.port == 80 { address.host.clone() } else { format!("{}:{}", address.host, address.port) };
	let request = format!("GET /api/health HTTP/1.1\r\nHost: {host_header}\r\nAccept: application/json\r\nConnection: close\r\n\r\n");
	stream.write_all(request.as_bytes()).map_err(|_| ERR_UNREACHABLE.to_string())?;

	let mut response = Vec::new();
	let mut buffer = [0u8; 1024];
	loop {
		match stream.read(&mut buffer) {
			Ok(0) => break,
			Ok(n) => {
				response.extend_from_slice(&buffer[..n]);
				if response.len() > MAX_RESPONSE {
					break;
				}
			}
			Err(_) if !response.is_empty() => break,
			Err(_) => return Err(ERR_UNREACHABLE.into()),
		}
	}
	interpret(&String::from_utf8_lossy(&response))
}

/// Memeriksa jawaban `/api/health`: `{"status":"ok"}` atau `{"status":"db_error"}`.
fn interpret(response: &str) -> Result<(), String> {
	let (head, body) = response.split_once("\r\n\r\n").ok_or_else(|| ERR_NOT_NILUJI.to_string())?;
	let status_line = head.lines().next().unwrap_or_default();
	if !status_line.starts_with("HTTP/1.") || status_line.split_whitespace().nth(1) != Some("200") {
		return Err(ERR_NOT_NILUJI.into());
	}
	let compact: String = body.chars().filter(|c| !c.is_whitespace()).collect();
	if compact.contains(r#""status":"ok""#) {
		Ok(())
	} else if compact.contains(r#""status":"db_error""#) {
		Err(ERR_DB.into())
	} else {
		Err(ERR_NOT_NILUJI.into())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::net::TcpListener;
	use std::thread;

	fn serve_once(reply: &'static str) -> u16 {
		let listener = TcpListener::bind("127.0.0.1:0").unwrap();
		let port = listener.local_addr().unwrap().port();
		thread::spawn(move || {
			if let Ok((mut socket, _)) = listener.accept() {
				let mut buf = [0u8; 1024];
				let _ = socket.read(&mut buf);
				let _ = socket.write_all(reply.as_bytes());
			}
		});
		port
	}

	fn local(port: u16) -> ServerAddress {
		ServerAddress { host: "127.0.0.1".into(), port, token: None }
	}

	#[test]
	fn accepts_a_niluji_server() {
		let port = serve_once("HTTP/1.1 200 OK\r\ncontent-type: application/json\r\n\r\n{\"status\":\"ok\"}");
		assert_eq!(check(&local(port)), Ok(()));
	}

	#[test]
	fn reports_database_problems() {
		let port = serve_once("HTTP/1.1 200 OK\r\n\r\n{\"status\": \"db_error\"}");
		assert_eq!(check(&local(port)).unwrap_err(), ERR_DB);
	}

	#[test]
	fn rejects_other_web_servers() {
		let port = serve_once("HTTP/1.1 404 Not Found\r\n\r\nnot here");
		assert_eq!(check(&local(port)).unwrap_err(), ERR_NOT_NILUJI);
		let port = serve_once("HTTP/1.1 200 OK\r\n\r\n<html>router</html>");
		assert_eq!(check(&local(port)).unwrap_err(), ERR_NOT_NILUJI);
	}

	#[test]
	fn reports_unreachable_servers() {
		let listener = TcpListener::bind("127.0.0.1:0").unwrap();
		let port = listener.local_addr().unwrap().port();
		drop(listener);
		assert_eq!(check(&local(port)).unwrap_err(), ERR_UNREACHABLE);
	}
}
