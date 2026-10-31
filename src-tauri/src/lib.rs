#[cfg(desktop)]
use tauri::{
	menu::{Menu, MenuItem},
	tray::TrayIconBuilder
};
use tauri::Manager;

mod backup;
mod db;
mod exam_session;
mod exams;
mod export;
mod import;
mod server;
mod settings;
mod students;
mod uploads;

fn validate_safe_user_file_path(path: &str) -> Result<std::path::PathBuf, String> {
	let p = std::path::Path::new(path);
	for comp in p.components() {
		if matches!(comp, std::path::Component::ParentDir) {
			return Err("Path file tidak diizinkan (mengandung '..').".into());
		}
	}
	let ext = p
		.extension()
		.and_then(|e| e.to_str())
		.map(|e| e.to_ascii_lowercase())
		.unwrap_or_default();
	const ALLOWED: &[&str] = &[
		"docx", "xlsx", "xls", "csv", "md", "zip", "jpg", "jpeg", "png", "gif", "webp"
	];
	if !ALLOWED.contains(&ext.as_str()) {
		return Err(format!("Ekstensi file '.{ext}' tidak diizinkan."));
	}
	Ok(p.to_path_buf())
}

#[tauri::command]
async fn get_server_info(
	port_state: tauri::State<'_, server::PortState>,
	db: tauri::State<'_, db::Db>
) -> Result<serde_json::Value, String> {
	let ips = server::local_lan_ips();
	let primary_ip = ips.first().cloned();
	Ok(serde_json::json!({
		"port": *port_state.lock().unwrap(),
		"lanIp": primary_ip,
		"lanIps": ips,
		"dbOk": db::ping(&db)
	}))
}

#[tauri::command]
async fn import_questions(
	path: String,
	opts: import::ImportOptions,
	db: tauri::State<'_, db::Db>
) -> Result<import::ImportSummary, String> {
	import::import_questions(&db, &path, opts)
}

#[tauri::command]
async fn read_file_bytes(path: String) -> Result<Vec<u8>, String> {
	let safe_path = validate_safe_user_file_path(&path)?;
	std::fs::read(&safe_path).map_err(|e| format!("Gagal membaca file: {e}"))
}

#[tauri::command]
async fn write_file_bytes(path: String, bytes: Vec<u8>) -> Result<(), String> {
	let safe_path = validate_safe_user_file_path(&path)?;
	std::fs::write(&safe_path, bytes).map_err(|e| format!("Gagal menyimpan file: {e}"))
}

#[tauri::command]
async fn list_questions(db: tauri::State<'_, db::Db>) -> Result<Vec<import::QuestionSummary>, String> {
	import::list_questions(&db)
}

#[tauri::command]
async fn get_dashboard_stats(db: tauri::State<'_, db::Db>) -> Result<import::DashboardStats, String> {
	import::dashboard_stats(&db)
}

#[tauri::command]
async fn list_online_students(login_state: tauri::State<'_, server::LoginState>) -> Result<Vec<server::OnlineStudent>, String> {
	Ok(server::list_online(&login_state))
}

#[tauri::command]
async fn list_subjects(db: tauri::State<'_, db::Db>) -> Result<Vec<exams::SubjectSummary>, String> {
	exams::list_subjects(&db)
}

#[tauri::command]
async fn list_exams(db: tauri::State<'_, db::Db>) -> Result<Vec<exams::ExamSummary>, String> {
	exams::list_exams(&db)
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
async fn create_exam(
	subject_id: i64,
	class: Option<String>,
	jenis: Option<String>,
	title: String,
	duration: i64,
	scheduled_at: i64,
	window_end: i64,
	token: String,
	randomize_pg: bool,
	randomize_essay: bool,
	randomize_options: Option<bool>,
	require_pin: Option<bool>,
	pg_weight: Option<f64>,
	db: tauri::State<'_, db::Db>
) -> Result<i64, String> {
	exams::create_exam(
		&db,
		subject_id,
		class,
		jenis,
		title,
		duration,
		scheduled_at,
		window_end,
		token,
		randomize_pg,
		randomize_essay,
		randomize_options,
		require_pin,
		pg_weight
	)
}

#[tauri::command]
async fn list_classes(db: tauri::State<'_, db::Db>) -> Result<Vec<String>, String> {
	students::list_classes(&db)
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
async fn update_exam(
	id: i64,
	subject_id: i64,
	class: Option<String>,
	jenis: Option<String>,
	title: String,
	duration: i64,
	scheduled_at: i64,
	window_end: i64,
	token: String,
	randomize_pg: bool,
	randomize_essay: bool,
	randomize_options: Option<bool>,
	require_pin: Option<bool>,
	pg_weight: Option<f64>,
	db: tauri::State<'_, db::Db>
) -> Result<(), String> {
	exams::update_exam(
		&db,
		id,
		subject_id,
		class,
		jenis,
		title,
		duration,
		scheduled_at,
		window_end,
		token,
		randomize_pg,
		randomize_essay,
		randomize_options,
		require_pin,
		pg_weight
	)
}

#[tauri::command]
async fn delete_exam(id: i64, db: tauri::State<'_, db::Db>) -> Result<(), String> {
	exams::delete_exam(&db, id)
}

#[tauri::command]
async fn list_exam_sessions(db: tauri::State<'_, db::Db>) -> Result<Vec<exam_session::SessionProgress>, String> {
	exam_session::list_sessions(&db)
}

#[tauri::command]
async fn reset_exam_session(session_id: i64, db: tauri::State<'_, db::Db>) -> Result<(), String> {
	exam_session::reset(&db, session_id)
}

#[tauri::command]
async fn unlock_exam_session_device(session_id: i64, db: tauri::State<'_, db::Db>) -> Result<(), String> {
	exam_session::unlock_device(&db, session_id)
}

#[tauri::command]
async fn add_exam_session_time(session_id: i64, minutes: i64, db: tauri::State<'_, db::Db>) -> Result<i64, String> {
	exam_session::add_extra_time(&db, session_id, minutes)
}

#[tauri::command]
async fn force_submit_exam_session(session_id: i64, db: tauri::State<'_, db::Db>) -> Result<exam_session::SubmitResponse, String> {
	exam_session::force_submit(&db, session_id)
}

#[tauri::command]
async fn get_exam_session_detail(session_id: i64, db: tauri::State<'_, db::Db>) -> Result<exam_session::SessionDetail, String> {
	exam_session::session_detail(&db, session_id)
}

#[tauri::command]
async fn grade_essay_answer(session_id: i64, question_id: i64, score: f64, db: tauri::State<'_, db::Db>) -> Result<(), String> {
	exam_session::grade_essay_answer(&db, session_id, question_id, score)
}

#[tauri::command]
async fn get_exam_analytics(
	class: Option<String>,
	subject_id: i64,
	jenis: Option<String>,
	db: tauri::State<'_, db::Db>
) -> Result<exam_session::AnalyticsResponse, String> {
	exam_session::analytics(&db, class, subject_id, jenis)
}

#[tauri::command]
async fn parse_students_file(path: String) -> Result<Vec<students::StudentImportRow>, String> {
	students::parse_students_file(&path)
}

#[tauri::command]
async fn import_students_rows(
	rows: Vec<students::StudentImportRow>,
	db: tauri::State<'_, db::Db>
) -> Result<students::StudentImportSummary, String> {
	students::import_students_rows(&db, rows)
}

#[tauri::command]
async fn list_students(db: tauri::State<'_, db::Db>) -> Result<Vec<students::StudentRecord>, String> {
	students::list_students(&db)
}

#[tauri::command]
async fn create_student(nisn: String, name: String, class: Option<String>, db: tauri::State<'_, db::Db>) -> Result<i64, String> {
	students::create_student(&db, nisn, name, class)
}

#[tauri::command]
async fn update_student(id: i64, nisn: String, name: String, class: Option<String>, db: tauri::State<'_, db::Db>) -> Result<(), String> {
	students::update_student(&db, id, nisn, name, class)
}

#[tauri::command]
async fn delete_student(id: i64, db: tauri::State<'_, db::Db>) -> Result<(), String> {
	students::delete_student(&db, id)
}

#[tauri::command]
async fn regenerate_student_pins(class: Option<String>, db: tauri::State<'_, db::Db>) -> Result<usize, String> {
	students::regenerate_student_pins(&db, class)
}

#[tauri::command]
async fn list_schools(db: tauri::State<'_, db::Db>) -> Result<Vec<String>, String> {
	students::list_schools(&db)
}

#[tauri::command]
async fn students_by_school(school: String, db: tauri::State<'_, db::Db>) -> Result<Vec<students::StudentRecord>, String> {
	students::students_by_school(&db, &school)
}

#[tauri::command]
async fn tarik_data_siswa(csv_url: String, school: String, db: tauri::State<'_, db::Db>) -> Result<usize, String> {
	students::tarik_data_siswa(&db, &csv_url, &school).await
}

#[tauri::command]
async fn list_schools_from_source(csv_url: String) -> Result<Vec<String>, String> {
	students::list_schools_from_source(&csv_url).await
}

#[tauri::command]
async fn get_question(id: i64, db: tauri::State<'_, db::Db>) -> Result<import::QuestionDetail, String> {
	import::get_question(&db, id)
}

#[tauri::command]
async fn create_question(input: import::QuestionInput, db: tauri::State<'_, db::Db>) -> Result<i64, String> {
	import::create_question(&db, input)
}

#[tauri::command]
async fn update_question(id: i64, input: import::QuestionInput, db: tauri::State<'_, db::Db>) -> Result<(), String> {
	import::update_question(&db, id, input)
}

#[tauri::command]
async fn delete_question(id: i64, db: tauri::State<'_, db::Db>) -> Result<(), String> {
	import::delete_question(&db, id)
}

struct UploadsDir(std::path::PathBuf);
struct DataDir(std::path::PathBuf);

#[tauri::command]
async fn backup_now(db: tauri::State<'_, db::Db>, data_dir: tauri::State<'_, DataDir>) -> Result<backup::BackupInfo, String> {
	let path = backup::backup_now(&db, &data_dir.0)?;
	let metadata = std::fs::metadata(&path).map_err(|e| e.to_string())?;
	Ok(backup::BackupInfo {
		file_name: path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default(),
		path: path.to_string_lossy().to_string(),
		size_bytes: metadata.len(),
		created_at: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64
	})
}

#[tauri::command]
async fn list_backups(data_dir: tauri::State<'_, DataDir>) -> Result<Vec<backup::BackupInfo>, String> {
	backup::list_backups(&data_dir.0)
}

#[tauri::command]
async fn save_question_image(path: String, uploads_dir: tauri::State<'_, UploadsDir>) -> Result<uploads::SavedImage, String> {
	uploads::save_image(&uploads_dir.0, &path)
}

#[tauri::command]
async fn load_image_data_url(path: String, uploads_dir: tauri::State<'_, UploadsDir>) -> Result<String, String> {
	uploads::load_data_url(&uploads_dir.0, &path)
}

#[tauri::command]
async fn save_question_image_bytes(
	file_name: String,
	bytes: Vec<u8>,
	uploads_dir: tauri::State<'_, UploadsDir>
) -> Result<uploads::SavedImage, String> {
	uploads::save_image_bytes(&uploads_dir.0, &file_name, &bytes)
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
async fn export_jenis_soal(
	output_dir: String,
	kelas: String,
	subject_id: i64,
	jenis: String,
	pin: String,
	db: tauri::State<'_, db::Db>,
	uploads_dir: tauri::State<'_, UploadsDir>
) -> Result<export::ExportSummary, String> {
	export::export_jenis_soal(&db, &uploads_dir.0, &output_dir, &kelas, subject_id, &jenis, &pin)
}

#[tauri::command]
async fn get_school(db: tauri::State<'_, db::Db>) -> Result<settings::SchoolInfo, String> {
	settings::get_school(&db)
}

#[tauri::command]
async fn update_school(info: settings::SchoolInfo, db: tauri::State<'_, db::Db>) -> Result<(), String> {
	settings::update_school(&db, info)
}

#[tauri::command]
async fn list_classes_full(db: tauri::State<'_, db::Db>) -> Result<Vec<settings::ClassRecord>, String> {
	settings::list_classes_full(&db)
}

#[tauri::command]
async fn create_class(title: String, description: Option<String>, db: tauri::State<'_, db::Db>) -> Result<i64, String> {
	settings::create_class(&db, title, description)
}

#[tauri::command]
async fn update_class(id: i64, title: String, description: Option<String>, db: tauri::State<'_, db::Db>) -> Result<(), String> {
	settings::update_class(&db, id, title, description)
}

#[tauri::command]
async fn delete_class(id: i64, db: tauri::State<'_, db::Db>) -> Result<(), String> {
	settings::delete_class(&db, id)
}

#[tauri::command]
async fn reorder_classes(ordered_ids: Vec<i64>, db: tauri::State<'_, db::Db>) -> Result<(), String> {
	settings::reorder_classes(&db, ordered_ids)
}

#[tauri::command]
async fn create_subject(name: String, code: Option<String>, db: tauri::State<'_, db::Db>) -> Result<i64, String> {
	exams::create_subject(&db, name, code)
}

#[tauri::command]
async fn update_subject(id: i64, name: String, code: Option<String>, db: tauri::State<'_, db::Db>) -> Result<(), String> {
	exams::update_subject(&db, id, name, code)
}

#[tauri::command]
async fn delete_subject(id: i64, db: tauri::State<'_, db::Db>) -> Result<(), String> {
	exams::delete_subject(&db, id)
}

#[tauri::command]
async fn reorder_subjects(ordered_ids: Vec<i64>, db: tauri::State<'_, db::Db>) -> Result<(), String> {
	exams::reorder_subjects(&db, ordered_ids)
}

#[tauri::command]
async fn list_question_types(
	class: Option<String>,
	subject_id: Option<i64>,
	db: tauri::State<'_, db::Db>
) -> Result<Vec<settings::QuestionTypeRecord>, String> {
	settings::list_question_types(&db, class, subject_id)
}

#[tauri::command]
async fn create_question_type(
	name: String,
	description: Option<String>,
	class: Option<String>,
	subject_id: Option<i64>,
	db: tauri::State<'_, db::Db>
) -> Result<i64, String> {
	settings::create_question_type(&db, name, description, class, subject_id)
}

#[tauri::command]
async fn update_question_type(id: i64, name: String, description: Option<String>, db: tauri::State<'_, db::Db>) -> Result<(), String> {
	settings::update_question_type(&db, id, name, description)
}

#[tauri::command]
async fn delete_question_type(id: i64, db: tauri::State<'_, db::Db>) -> Result<(), String> {
	settings::delete_question_type(&db, id)
}

#[tauri::command]
async fn delete_question_type_scoped(
	id: i64,
	scope: String,
	current_class: String,
	current_subject_id: i64,
	db: tauri::State<'_, db::Db>
) -> Result<(), String> {
	settings::delete_question_type_scoped(&db, id, &scope, current_class, current_subject_id)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
	tauri::Builder::default()
		.setup(|app| {
			#[cfg(desktop)]
			{
				let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
				let menu = Menu::with_items(app, &[&quit_i])?;

				let _tray = TrayIconBuilder::new()
					.menu(&menu)
					.show_menu_on_left_click(true)
					.icon(app.default_window_icon().unwrap().clone())
					.on_menu_event(|app, event| match event.id.as_ref() {
						"quit" => {
							app.exit(0);
						}
						other => {
							println!("menu item {} not handled", other);
						}
					})
					.build(app)?;
			}

			let data_dir = app.path().app_data_dir().expect("no app data dir");
			let db = db::open(&data_dir);

			// Backup sekali tiap aplikasi dibuka — cukup buat jaga-jaga kalau ada apa-apa di
			// sesi berjalan, tanpa perlu penjadwal terpisah. Kegagalan backup TIDAK boleh
			// menghentikan aplikasi (mis. disk penuh) — cuma dicatat ke konsol.
			if let Err(e) = backup::backup_now(&db, &data_dir) {
				eprintln!("Backup otomatis saat start gagal: {e}");
			}

			// Debug builds read the live `dist/` folder directly so `bun run generate` is reflected
			// immediately. Release builds use the bundled resource copy (see tauri.conf.json).
			#[cfg(debug_assertions)]
			let static_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../dist");
			#[cfg(not(debug_assertions))]
			let static_dir = app.path().resource_dir().expect("no resource dir").join("dist");
			let uploads_dir = uploads::resolve_dir(&data_dir);
			std::fs::create_dir_all(&uploads_dir).expect("failed to create uploads dir");

			let port_state: server::PortState = Default::default();
			let login_state: server::LoginState = Default::default();
			tauri::async_runtime::spawn(server::serve(
				db.clone(),
				static_dir,
				uploads_dir.clone(),
				port_state.clone(),
				login_state.clone()
			));
			app.manage(db);
			app.manage(port_state);
			app.manage(login_state);
			app.manage(UploadsDir(uploads_dir));
			app.manage(DataDir(data_dir));

			Ok(())
		})
		.plugin(tauri_plugin_shell::init())
		.plugin(tauri_plugin_notification::init())
		.plugin(tauri_plugin_os::init())
		.plugin(tauri_plugin_fs::init())
		.plugin(tauri_plugin_store::Builder::new().build())
		.plugin(tauri_plugin_dialog::init())
		.plugin(tauri_plugin_http::init())
		.invoke_handler(tauri::generate_handler![
			get_server_info,
			import_questions,
			read_file_bytes,
			write_file_bytes,
			list_questions,
			get_dashboard_stats,
			list_online_students,
			list_subjects,
			list_exams,
			create_exam,
			list_classes,
			update_exam,
			delete_exam,
			list_exam_sessions,
			reset_exam_session,
			unlock_exam_session_device,
			add_exam_session_time,
			force_submit_exam_session,
			get_exam_session_detail,
			grade_essay_answer,
			get_exam_analytics,
			parse_students_file,
			import_students_rows,
			list_students,
			create_student,
			update_student,
			delete_student,
			regenerate_student_pins,
			list_schools,
			students_by_school,
			tarik_data_siswa,
			list_schools_from_source,
			get_question,
			create_question,
			update_question,
			delete_question,
			save_question_image,
			backup_now,
			list_backups,
			load_image_data_url,
			save_question_image_bytes,
			export_jenis_soal,
			get_school,
			update_school,
			list_classes_full,
			create_class,
			update_class,
			delete_class,
			reorder_classes,
			create_subject,
			update_subject,
			delete_subject,
			reorder_subjects,
			list_question_types,
			create_question_type,
			update_question_type,
			delete_question_type,
			delete_question_type_scoped
		])
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}
