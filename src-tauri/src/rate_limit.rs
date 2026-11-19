// Pembatas percobaan login di memori. Hanya kegagalan yang dihitung, jadi siswa yang masuk dengan
// benar tidak pernah terkena. Batas per perangkat + NISN sengaja dipakai sebagai pengganti batas
// per NISN saja, supaya seorang siswa tidak bisa mengunci akun temannya dari HP-nya sendiri.
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Mutex;
use std::time::{Duration, Instant};

const MAX_TRACKED_KEYS: usize = 10_000;

#[derive(Clone, Copy)]
struct Rule {
	max_failures: u32,
	window: Duration,
	block: Duration
}

const PER_DEVICE: Rule = Rule { max_failures: 30, window: Duration::from_secs(60), block: Duration::from_secs(60) };
const PER_DEVICE_AND_STUDENT: Rule = Rule { max_failures: 8, window: Duration::from_secs(300), block: Duration::from_secs(300) };

struct Bucket {
	failures: u32,
	window_started: Instant,
	blocked_until: Option<Instant>,
	expires_at: Instant
}

#[derive(Default)]
pub struct RateLimiter {
	buckets: Mutex<HashMap<String, Bucket>>
}

fn device_key(ip: IpAddr) -> String {
	format!("ip:{ip}")
}

fn student_key(ip: IpAddr, nisn: &str) -> String {
	format!("nisn:{ip}:{}", nisn.trim())
}

impl RateLimiter {
	// Mengembalikan sisa detik penantian bila perangkat ini sedang diblokir.
	pub fn check(&self, ip: IpAddr, nisn: &str) -> Option<u64> {
		self.check_at(ip, nisn, Instant::now())
	}

	pub fn record_failure(&self, ip: IpAddr, nisn: &str) {
		self.record_failure_at(ip, nisn, Instant::now());
	}

	pub fn clear_student(&self, ip: IpAddr, nisn: &str) {
		self.buckets.lock().unwrap().remove(&student_key(ip, nisn));
	}

	fn check_at(&self, ip: IpAddr, nisn: &str, now: Instant) -> Option<u64> {
		let buckets = self.buckets.lock().unwrap();
		[device_key(ip), student_key(ip, nisn)]
			.iter()
			.filter_map(|key| buckets.get(key)?.blocked_until)
			.filter(|until| *until > now)
			.map(|until| (until - now).as_secs() + 1)
			.max()
	}

	fn record_failure_at(&self, ip: IpAddr, nisn: &str, now: Instant) {
		let mut buckets = self.buckets.lock().unwrap();
		if buckets.len() >= MAX_TRACKED_KEYS {
			buckets.retain(|_, bucket| bucket.expires_at > now);
		}
		if buckets.len() >= MAX_TRACKED_KEYS {
			return;
		}
		Self::bump(&mut buckets, device_key(ip), PER_DEVICE, now);
		Self::bump(&mut buckets, student_key(ip, nisn), PER_DEVICE_AND_STUDENT, now);
	}

	fn bump(buckets: &mut HashMap<String, Bucket>, key: String, rule: Rule, now: Instant) {
		let bucket = buckets.entry(key).or_insert(Bucket { failures: 0, window_started: now, blocked_until: None, expires_at: now });
		if now.duration_since(bucket.window_started) > rule.window {
			bucket.failures = 0;
			bucket.window_started = now;
			bucket.blocked_until = None;
		}
		bucket.failures += 1;
		if bucket.failures >= rule.max_failures {
			bucket.blocked_until = Some(now + rule.block);
			bucket.failures = 0;
			bucket.window_started = now + rule.block;
		}
		bucket.expires_at = now + rule.window + rule.block;
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	const PHONE_A: IpAddr = IpAddr::V4(std::net::Ipv4Addr::new(192, 168, 1, 10));
	const PHONE_B: IpAddr = IpAddr::V4(std::net::Ipv4Addr::new(192, 168, 1, 11));

	#[test]
	fn repeated_failures_block_only_the_offending_device() {
		let limiter = RateLimiter::default();
		let now = Instant::now();
		for _ in 0..PER_DEVICE_AND_STUDENT.max_failures {
			assert!(limiter.check_at(PHONE_A, "111", now).is_none());
			limiter.record_failure_at(PHONE_A, "111", now);
		}
		assert!(limiter.check_at(PHONE_A, "111", now).is_some());
		assert!(limiter.check_at(PHONE_A, "222", now).is_none());
		assert!(limiter.check_at(PHONE_B, "111", now).is_none(), "pemilik akun di HP lain tidak ikut terkunci");
	}

	#[test]
	fn block_expires_after_its_duration() {
		let limiter = RateLimiter::default();
		let now = Instant::now();
		for _ in 0..PER_DEVICE_AND_STUDENT.max_failures {
			limiter.record_failure_at(PHONE_A, "111", now);
		}
		assert!(limiter.check_at(PHONE_A, "111", now + PER_DEVICE_AND_STUDENT.block + Duration::from_secs(1)).is_none());
	}

	#[test]
	fn guessing_across_many_students_blocks_the_device() {
		let limiter = RateLimiter::default();
		let now = Instant::now();
		for i in 0..PER_DEVICE.max_failures {
			limiter.record_failure_at(PHONE_A, &format!("nisn-{i}"), now);
		}
		assert!(limiter.check_at(PHONE_A, "siapa-saja", now).is_some());
	}

	#[test]
	fn success_clears_the_student_counter() {
		let limiter = RateLimiter::default();
		let now = Instant::now();
		for _ in 0..PER_DEVICE_AND_STUDENT.max_failures - 1 {
			limiter.record_failure_at(PHONE_A, "111", now);
		}
		limiter.clear_student(PHONE_A, "111");
		limiter.record_failure_at(PHONE_A, "111", now);
		assert!(limiter.check_at(PHONE_A, "111", now).is_none());
	}
}
