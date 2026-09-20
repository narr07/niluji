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
use std::net::{SocketAddr, UdpSocket};
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
	name: String
}

async fn exam_join(
	State(state): State<AppState>,
	Json(payload): Json<JoinPayload>
) -> Result<Json<crate::exam_session::JoinResponse>, (StatusCode, String)> {
	crate::exam_session::join(&state.db, &payload.token, &payload.nisn, &payload.name)
		.map(Json)
		.map_err(|e| (StatusCode::BAD_REQUEST, e))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AnswerPayload {
	session_id: i64,
	question_id: i64,
	option_key: String
}

async fn exam_answer(State(state): State<AppState>, Json(payload): Json<AnswerPayload>) -> Result<StatusCode, (StatusCode, String)> {
	crate::exam_session::answer(&state.db, payload.session_id, payload.question_id, &payload.option_key)
		.map(|_| StatusCode::OK)
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

// ponytail: picks the interface used to reach 8.8.8.8 (no packet actually sent);
// wrong on multi-NIC machines that route the exam LAN through a different interface.
pub fn local_lan_ip() -> Option<String> {
	let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
	socket.connect("8.8.8.8:80").ok()?;
	socket.local_addr().ok().map(|addr| addr.ip().to_string())
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
