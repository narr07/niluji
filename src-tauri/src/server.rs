use crate::db::{self, Db};
use crate::exam_session::{self, ERR_SESSION};
use crate::rate_limit::RateLimiter;
use axum::{
	extract::{ConnectInfo, State},
	http::StatusCode,
	routing::{get, post},
	Json, Router
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs, UdpSocket};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tower_http::services::{ServeDir, ServeFile};

// Port 80 means students can open just the LAN IP with no ":port" suffix — much
// friendlier to type on a phone. Falls back to 8080+ if 80 is already taken (e.g. by
// IIS or Skype), then keeps scanning upward.
pub const PREFERRED_PORT: u16 = 80;
const PORT_SCAN_RANGE: u16 = 100;
const FALLBACK_PORT: u16 = 8080;

type ApiError = (StatusCode, String);

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
	pub logged_in_at: u64,
	#[serde(skip)]
	presence_token: String
}

pub fn list_online(state: &LoginState) -> Vec<OnlineStudent> {
	let mut list: Vec<OnlineStudent> = state.lock().unwrap().values().cloned().collect();
	list.sort_by_key(|student| std::cmp::Reverse(student.logged_in_at));
	list
}

#[derive(Clone)]
struct AppState {
	db: Db,
	login_state: LoginState,
	limiter: Arc<RateLimiter>
}

// Query SQLite bersifat blocking, jadi dijalankan di luar thread async supaya permintaan siswa
// lain tidak ikut tertahan.
async fn blocking<T, F>(task: F) -> Result<T, String>
where
	T: Send + 'static,
	F: FnOnce() -> Result<T, String> + Send + 'static
{
	tokio::task::spawn_blocking(task).await.map_err(|e| e.to_string())?
}

// 4xx berarti permintaan ditolak secara final, 5xx berarti aman untuk dicoba ulang oleh siswa.
fn exam_error(message: String) -> ApiError {
	let status = if message == ERR_SESSION {
		StatusCode::UNAUTHORIZED
	} else if exam_session::is_final_rejection(&message) {
		StatusCode::BAD_REQUEST
	} else {
		StatusCode::INTERNAL_SERVER_ERROR
	};
	(status, message)
}

fn ensure_not_limited(state: &AppState, ip: IpAddr, nisn: &str) -> Result<(), ApiError> {
	match state.limiter.check(ip, nisn) {
		Some(seconds) => Err((StatusCode::TOO_MANY_REQUESTS, format!("Terlalu banyak percobaan gagal. Coba lagi dalam {seconds} detik."))),
		None => Ok(())
	}
}

async fn health(State(state): State<AppState>) -> Json<Value> {
	let ok = blocking(move || Ok(db::ping(&state.db))).await.unwrap_or(false);
	Json(json!({ "status": if ok { "ok" } else { "db_error" } }))
}

#[derive(Deserialize)]
struct LoginPayload {
	nisn: String,
	name: String
}

async fn login(
	State(state): State<AppState>,
	ConnectInfo(peer): ConnectInfo<SocketAddr>,
	Json(payload): Json<LoginPayload>
) -> Result<Json<Value>, ApiError> {
	let ip = peer.ip();
	ensure_not_limited(&state, ip, &payload.nisn)?;

	let db = state.db.clone();
	let (nisn, name) = (payload.nisn.clone(), payload.name);
	let found = blocking(move || Ok(db::find_student(&db, &nisn, &name)))
		.await
		.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

	let Some(name) = found else {
		state.limiter.record_failure(ip, &payload.nisn);
		return Err((StatusCode::UNAUTHORIZED, exam_session::ERR_IDENTITY.to_string()));
	};

	let nisn = payload.nisn.trim().to_string();
	let presence_token = db::generate_secret();
	let logged_in_at = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
	state.login_state.lock().unwrap().insert(
		nisn.clone(),
		OnlineStudent { nisn, name: name.clone(), logged_in_at, presence_token: presence_token.clone() }
	);
	Ok(Json(json!({ "name": name, "presenceToken": presence_token })))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LogoutPayload {
	nisn: String,
	#[serde(default)]
	presence_token: String
}

async fn logout(State(state): State<AppState>, Json(payload): Json<LogoutPayload>) -> StatusCode {
	let mut online = state.login_state.lock().unwrap();
	let nisn = payload.nisn.trim();
	if online.get(nisn).is_some_and(|s| db::secrets_match(&s.presence_token, &payload.presence_token)) {
		online.remove(nisn);
	}
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
	ConnectInfo(peer): ConnectInfo<SocketAddr>,
	Json(payload): Json<JoinPayload>
) -> Result<Json<exam_session::JoinResponse>, ApiError> {
	let ip = peer.ip();
	ensure_not_limited(&state, ip, &payload.nisn)?;

	let db = state.db.clone();
	let nisn = payload.nisn.clone();
	let result = blocking(move || {
		exam_session::join(&db, &payload.token, &payload.nisn, &payload.name, payload.pin.as_deref(), payload.device_token.as_deref())
	})
	.await;

	match result {
		Ok(joined) => {
			state.limiter.clear_student(ip, &nisn);
			Ok(Json(joined))
		}
		Err(message) => {
			if exam_session::is_credential_error(&message) {
				state.limiter.record_failure(ip, &nisn);
			}
			Err((StatusCode::BAD_REQUEST, message))
		}
	}
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AnswerPayload {
	session_id: i64,
	session_token: String,
	question_id: i64,
	option_key: String,
	#[serde(default)]
	is_doubtful: Option<bool>
}

async fn exam_answer(
	State(state): State<AppState>,
	Json(payload): Json<AnswerPayload>
) -> Result<Json<exam_session::AnswerResponse>, ApiError> {
	blocking(move || {
		exam_session::answer(
			&state.db,
			payload.session_id,
			&payload.session_token,
			payload.question_id,
			&payload.option_key,
			payload.is_doubtful
		)
	})
	.await
	.map(Json)
	.map_err(exam_error)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SubmitPayload {
	session_id: i64,
	session_token: String
}

async fn exam_submit(
	State(state): State<AppState>,
	Json(payload): Json<SubmitPayload>
) -> Result<Json<exam_session::SubmitResponse>, ApiError> {
	blocking(move || exam_session::submit(&state.db, payload.session_id, &payload.session_token))
		.await
		.map(Json)
		.map_err(exam_error)
}

async fn exam_submit_pg(State(state): State<AppState>, Json(payload): Json<SubmitPayload>) -> Result<StatusCode, ApiError> {
	blocking(move || exam_session::submit_pg(&state.db, payload.session_id, &payload.session_token))
		.await
		.map(|_| StatusCode::OK)
		.map_err(exam_error)
}

fn is_valid_lan_ipv4(ip: &Ipv4Addr) -> bool {
	if ip.is_loopback() || ip.is_unspecified() || ip.is_broadcast() {
		return false;
	}
	let octets = ip.octets();
	// Exclude APIPA link-local 169.254.x.x
	!(octets[0] == 169 && octets[1] == 254)
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

fn router(state: AppState, static_dir: &Path, uploads_dir: &Path) -> Router {
	// SPA fallback: any path that isn't a real static file (e.g. a hidden page like
	// /cbt that the prerender crawler never linked to) serves index.html instead of 404.
	let spa = ServeDir::new(static_dir).not_found_service(ServeFile::new(static_dir.join("index.html")));

	Router::new()
		.route("/api/health", get(health))
		.route("/api/login", post(login))
		.route("/api/logout", post(logout))
		.route("/api/exam/join", post(exam_join))
		.route("/api/exam/answer", post(exam_answer))
		.route("/api/exam/submit-pg", post(exam_submit_pg))
		.route("/api/exam/submit", post(exam_submit))
		.with_state(state)
		.nest_service("/uploads", ServeDir::new(uploads_dir))
		.fallback_service(spa)
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

	let state = AppState { db, login_state, limiter: Arc::default() };
	let app = router(state, &static_dir, &uploads_dir);

	axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await.expect("CBT server crashed");
}

#[cfg(test)]
mod tests {
	use super::*;
	use axum::{
		body::Body,
		extract::connect_info::MockConnectInfo,
		http::{header, Request}
	};
	use http_body_util::BodyExt;
	use rusqlite::params;
	use tower::ServiceExt;

	struct TestApp {
		app: Router,
		db: Db,
		login_state: LoginState,
		dir: PathBuf,
		question_id: i64
	}

	impl Drop for TestApp {
		fn drop(&mut self) {
			let _ = std::fs::remove_dir_all(&self.dir);
		}
	}

	fn test_app() -> TestApp {
		let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
		let dir = std::env::temp_dir().join(format!("niluji_server_test_{}_{nanos}", std::process::id()));
		let db = db::open(&dir);
		let question_id = {
			let conn = db.lock().unwrap();
			conn.execute(
				"INSERT INTO students (nisn, name, class, pin) VALUES ('111', 'Siswa A', '6', '111111'), ('222', 'Siswa B', '6', '222222')",
				[]
			)
			.unwrap();
			conn.execute("INSERT INTO subjects (name) VALUES ('Uji Server')", []).unwrap();
			let subject = conn.last_insert_rowid();
			conn.execute("INSERT INTO questions (subject_id, class, question_text, answer_key, score) VALUES (?1, '6', 'Soal', 'A', 1)", params![subject]).unwrap();
			let question = conn.last_insert_rowid();
			conn.execute("INSERT INTO question_options (question_id, option_key, option_text) VALUES (?1, 'A', 'x'), (?1, 'B', 'y')", params![question]).unwrap();
			conn.execute(
				"INSERT INTO exams (subject_id, class, title, duration, question_count, token, require_pin) VALUES (?1, '6', 'Ujian', 30, 1, 'TOKEN1', 1)",
				params![subject]
			)
			.unwrap();
			question
		};
		let login_state = LoginState::default();
		let state = AppState { db: db.clone(), login_state: login_state.clone(), limiter: Arc::default() };
		let app = router(state, &dir, &dir).layer(MockConnectInfo(SocketAddr::from(([192, 168, 1, 50], 5000))));
		TestApp { app, db, login_state, dir, question_id }
	}

	async fn post_json(app: &Router, path: &str, body: Value) -> (StatusCode, Value, String) {
		let request = Request::post(path).header(header::CONTENT_TYPE, "application/json").body(Body::from(body.to_string())).unwrap();
		let response = app.clone().oneshot(request).await.unwrap();
		let status = response.status();
		let bytes = response.into_body().collect().await.unwrap().to_bytes();
		let text = String::from_utf8_lossy(&bytes).to_string();
		(status, serde_json::from_str(&text).unwrap_or(Value::Null), text)
	}

	async fn join(app: &Router, nisn: &str, name: &str, pin: &str, device: &str) -> Value {
		let body = json!({ "token": "token1", "nisn": nisn, "name": name, "pin": pin, "deviceToken": device });
		let (status, value, text) = post_json(app, "/api/exam/join", body).await;
		assert_eq!(status, StatusCode::OK, "{text}");
		value
	}

	#[tokio::test]
	async fn student_cannot_answer_or_submit_for_another_session_over_http() {
		let t = test_app();
		let victim = join(&t.app, "111", "Siswa A", "111111", "hp-a").await;
		let attacker = join(&t.app, "222", "Siswa B", "222222", "hp-b").await;
		let victim_id = victim["sessionId"].as_i64().unwrap();

		let forged = json!({
			"sessionId": victim_id,
			"sessionToken": attacker["sessionToken"],
			"questionId": t.question_id,
			"optionKey": "B"
		});
		assert_eq!(post_json(&t.app, "/api/exam/answer", forged).await.0, StatusCode::UNAUTHORIZED);

		let without_token = json!({ "sessionId": victim_id, "questionId": t.question_id, "optionKey": "B" });
		assert_eq!(post_json(&t.app, "/api/exam/answer", without_token).await.0, StatusCode::UNPROCESSABLE_ENTITY);

		let forged_submit = json!({ "sessionId": victim_id, "sessionToken": attacker["sessionToken"] });
		assert_eq!(post_json(&t.app, "/api/exam/submit", forged_submit.clone()).await.0, StatusCode::UNAUTHORIZED);
		assert_eq!(post_json(&t.app, "/api/exam/submit-pg", forged_submit).await.0, StatusCode::UNAUTHORIZED);

		let submitted: Option<i64> =
			t.db.lock().unwrap().query_row("SELECT submitted_at FROM exam_sessions WHERE id = ?1", params![victim_id], |r| r.get(0)).unwrap();
		assert!(submitted.is_none());
	}

	#[tokio::test]
	async fn owner_can_finish_the_exam_over_http_and_retry_submit() {
		let t = test_app();
		let joined = join(&t.app, "111", "Siswa A", "111111", "hp-a").await;
		assert!(joined["questions"][0].get("answerKey").is_none(), "kunci jawaban tidak boleh dikirim ke siswa");

		let auth = json!({ "sessionId": joined["sessionId"], "sessionToken": joined["sessionToken"] });
		let answer = json!({
			"sessionId": joined["sessionId"],
			"sessionToken": joined["sessionToken"],
			"questionId": t.question_id,
			"optionKey": "A"
		});
		assert_eq!(post_json(&t.app, "/api/exam/answer", answer.clone()).await.0, StatusCode::OK);

		let (status, value, _) = post_json(&t.app, "/api/exam/submit", auth.clone()).await;
		assert_eq!(status, StatusCode::OK);
		assert_eq!(value["score"], 100.0);
		assert_eq!(post_json(&t.app, "/api/exam/submit", auth).await.0, StatusCode::OK);

		let (status, _, text) = post_json(&t.app, "/api/exam/answer", answer).await;
		assert_eq!(status, StatusCode::BAD_REQUEST);
		assert_eq!(text, exam_session::ERR_SUBMITTED);
	}

	#[tokio::test]
	async fn repeated_wrong_pins_are_throttled() {
		let t = test_app();
		let body = json!({ "token": "TOKEN1", "nisn": "111", "name": "Siswa A", "pin": "000000", "deviceToken": "hp" });
		let mut last = StatusCode::OK;
		for _ in 0..8 {
			last = post_json(&t.app, "/api/exam/join", body.clone()).await.0;
			assert_ne!(last, StatusCode::OK);
		}
		assert_eq!(last, StatusCode::BAD_REQUEST);
		let correct = json!({ "token": "TOKEN1", "nisn": "111", "name": "Siswa A", "pin": "111111", "deviceToken": "hp" });
		assert_eq!(post_json(&t.app, "/api/exam/join", correct).await.0, StatusCode::TOO_MANY_REQUESTS);
	}

	#[tokio::test]
	async fn logout_requires_the_presence_token() {
		let t = test_app();
		let login_body = json!({ "nisn": "111", "name": "Siswa A" });
		let (status, login, _) = post_json(&t.app, "/api/login", login_body).await;
		assert_eq!(status, StatusCode::OK);

		post_json(&t.app, "/api/logout", json!({ "nisn": "111", "presenceToken": "salah" })).await;
		assert_eq!(list_online(&t.login_state).len(), 1);

		post_json(&t.app, "/api/logout", json!({ "nisn": "111", "presenceToken": login["presenceToken"] })).await;
		assert!(list_online(&t.login_state).is_empty());
	}

	#[test]
	fn only_final_rejections_stop_client_retries() {
		assert_eq!(exam_error(ERR_SESSION.to_string()).0, StatusCode::UNAUTHORIZED);
		assert_eq!(exam_error(exam_session::ERR_TIME_UP.to_string()).0, StatusCode::BAD_REQUEST);
		assert_eq!(exam_error("database is locked".to_string()).0, StatusCode::INTERNAL_SERVER_ERROR);
	}
}
