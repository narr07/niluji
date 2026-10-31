use crate::db::{self, Db};
use axum::{
	extract::State,
	http::StatusCode,
	routing::{get, post},
	Json, Router
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs, UdpSocket};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tower_http::services::{ServeDir, ServeFile};

// Port 80 means students can open just the LAN IP with no ":port" suffix — much
// friendlier to type on a phone. Falls back to 8080+ if 80 is already taken (e.g. by
// IIS or Skype), then keeps scanning upward.
pub const PREFERRED_PORT: u16 = 80;
const PORT_SCAN_RANGE: u16 = 100;
const FALLBACK_PORT: u16 = 8080;

// Set once the server has actually bound; None until then (or if every port in range is taken).
pub type PortState = Arc<Mutex<Option<u16>>>;

// ponytail: in-memory presence, keyed by NISN, so re-login just refreshes the timestamp.
// Resets on app restart, no history kept — good enough for "who's online right now".
pub type LoginState = Arc<Mutex<HashMap<String, OnlineStudent>>>;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OnlineStudent {
	pub nisn: String,
	pub name: String,
	pub logged_in_at: u64
}

pub fn list_online(state: &LoginState) -> Vec<OnlineStudent> {
	let mut list: Vec<OnlineStudent> = state.lock().unwrap().values().cloned().collect();
	list.sort_by(|a, b| b.logged_in_at.cmp(&a.logged_in_at));
	list
}

#[derive(Clone)]
struct AppState {
	db: Db,
	login_state: LoginState
}

async fn health(State(state): State<AppState>) -> Json<Value> {
	Json(json!({ "status": if db::ping(&state.db) { "ok" } else { "db_error" } }))
}

#[derive(Deserialize)]
struct LoginPayload {
	nisn: String,
	name: String
}

async fn login(State(state): State<AppState>, Json(payload): Json<LoginPayload>) -> Result<Json<Value>, StatusCode> {
	match db::find_student(&state.db, &payload.nisn, &payload.name) {
		Some(name) => {
			let logged_in_at = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
			state.login_state.lock().unwrap().insert(
				payload.nisn.clone(),
				OnlineStudent { nisn: payload.nisn, name: name.clone(), logged_in_at }
			);
			Ok(Json(json!({ "name": name })))
		}
		None => Err(StatusCode::UNAUTHORIZED)
	}
}

#[derive(Deserialize)]
struct LogoutPayload {
	nisn: String
}

async fn logout(State(state): State<AppState>, Json(payload): Json<LogoutPayload>) -> StatusCode {
	state.login_state.lock().unwrap().remove(&payload.nisn);
	StatusCode::OK
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct JoinPayload {
	token: String,
	nisn: String,
	name: String,
	#[serde(default)]
	pin: Option<String>,
	#[serde(default)]
	device_token: Option<String>
}

async fn exam_join(
	State(state): State<AppState>,
	Json(payload): Json<JoinPayload>
) -> Result<Json<crate::exam_session::JoinResponse>, (StatusCode, String)> {
	crate::exam_session::join(
		&state.db,
		&payload.token,
		&payload.nisn,
		&payload.name,
		payload.pin.as_deref(),
		payload.device_token.as_deref()
	)
	.map(Json)
	.map_err(|e| (StatusCode::BAD_REQUEST, e))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AnswerPayload {
	session_id: i64,
	question_id: i64,
	option_key: String,
	#[serde(default)]
	is_doubtful: Option<bool>
}

async fn exam_answer(
	State(state): State<AppState>,
	Json(payload): Json<AnswerPayload>
) -> Result<Json<crate::exam_session::AnswerResponse>, (StatusCode, String)> {
	crate::exam_session::answer(
		&state.db,
		payload.session_id,
		payload.question_id,
		&payload.option_key,
		payload.is_doubtful
	)
	.map(Json)
	.map_err(|e| (StatusCode::BAD_REQUEST, e))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SubmitPayload {
	session_id: i64
}

async fn exam_submit(
	State(state): State<AppState>,
	Json(payload): Json<SubmitPayload>
) -> Result<Json<crate::exam_session::SubmitResponse>, (StatusCode, String)> {
	crate::exam_session::submit(&state.db, payload.session_id)
		.map(Json)
		.map_err(|e| (StatusCode::BAD_REQUEST, e))
}

async fn exam_submit_pg(State(state): State<AppState>, Json(payload): Json<SubmitPayload>) -> Result<StatusCode, (StatusCode, String)> {
	crate::exam_session::submit_pg(&state.db, payload.session_id)
		.map(|_| StatusCode::OK)
		.map_err(|e| (StatusCode::BAD_REQUEST, e))
}

fn is_valid_lan_ipv4(ip: &Ipv4Addr) -> bool {
	if ip.is_loopback() || ip.is_unspecified() || ip.is_broadcast() {
		return false;
	}
	let octets = ip.octets();
	// Exclude APIPA link-local 169.254.x.x
	if octets[0] == 169 && octets[1] == 254 {
		return false;
	}
	true
}

fn ipv4_priority(ip: &Ipv4Addr) -> u8 {
	let o = ip.octets();
	if o[0] == 192 && o[1] == 168 {
		0
	} else if o[0] == 10 {
		1
	} else if o[0] == 172 && (16..=31).contains(&o[1]) {
		2
	} else {
		3
	}
}

pub fn local_lan_ips() -> Vec<String> {
	let mut candidates: Vec<Ipv4Addr> = Vec::new();
	let mut push_ip = |ip: Ipv4Addr| {
		if is_valid_lan_ipv4(&ip) && !candidates.contains(&ip) {
			candidates.push(ip);
		}
	};

	// 1. Probe common routes via UDP (works even offline when a local gateway or subnet exists)
	for target in ["8.8.8.8:80", "192.168.1.1:80", "192.168.0.1:80", "10.0.0.1:80", "172.16.0.1:80"] {
		if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
			if socket.connect(target).is_ok() {
				if let Ok(addr) = socket.local_addr() {
					if let IpAddr::V4(v4) = addr.ip() {
						push_ip(v4);
					}
				}
			}
		}
	}

	// 2. Resolve local hostname addresses (enumerates all active NIC IPv4s on Windows/macOS/Linux)
	if let Ok(host) = std::env::var("COMPUTERNAME").or_else(|_| std::env::var("HOSTNAME")) {
		if let Ok(addrs) = (host.as_str(), 0u16).to_socket_addrs() {
			for sock in addrs {
				if let IpAddr::V4(v4) = sock.ip() {
					push_ip(v4);
				}
			}
		}
	}

	candidates.sort_by_key(ipv4_priority);
	candidates.into_iter().map(|ip| ip.to_string()).collect()
}

pub fn local_lan_ip() -> Option<String> {
	local_lan_ips().into_iter().next()
}

async fn bind_first_available(preferred: u16) -> Option<(tokio::net::TcpListener, u16)> {
	for port in preferred..preferred.saturating_add(PORT_SCAN_RANGE) {
		let addr = SocketAddr::from(([0, 0, 0, 0], port));
		if let Ok(listener) = tokio::net::TcpListener::bind(addr).await {
			return Some((listener, port));
		}
	}
	None
}

pub async fn serve(db: Db, static_dir: PathBuf, uploads_dir: PathBuf, port_state: PortState, login_state: LoginState) {
	let bound = match bind_first_available(PREFERRED_PORT).await {
		Some(result) => Some(result),
		None => bind_first_available(FALLBACK_PORT).await
	};
	let Some((listener, port)) = bound else {
		eprintln!("[cbt-server] no free port found, server not started");
		return;
	};

	*port_state.lock().unwrap() = Some(port);
	println!("[cbt-server] listening on 0.0.0.0:{port} (LAN IP: {:?})", local_lan_ip());

	// SPA fallback: any path that isn't a real static file (e.g. a hidden page like
	// /cbt that the prerender crawler never linked to) serves index.html instead of 404.
	let spa = ServeDir::new(&static_dir).not_found_service(ServeFile::new(static_dir.join("index.html")));

	let app = Router::new()
		.route("/api/health", get(health))
		.route("/api/login", post(login))
		.route("/api/logout", post(logout))
		.route("/api/exam/join", post(exam_join))
		.route("/api/exam/answer", post(exam_answer))
		.route("/api/exam/submit-pg", post(exam_submit_pg))
		.route("/api/exam/submit", post(exam_submit))
		.with_state(AppState { db, login_state })
		.nest_service("/uploads", ServeDir::new(&uploads_dir))
		.fallback_service(spa);

	axum::serve(listener, app).await.expect("CBT server crashed");
}
