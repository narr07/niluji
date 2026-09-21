#[cfg(desktop)]
use tauri::{
	menu::{Menu, MenuItem},
	tray::TrayIconBuilder
};
use tauri::Manager;

mod db;
mod exam_session;
mod exams;
mod export;
mod import;
mod server;
mod settings;
mod students;
mod uploads;

#[tauri::command]
fn get_server_info(port_state: tauri::State<server::PortState>, db: tauri::State<db::Db>) -> serde_json::Value {
	serde_json::json!({
		"port": *port_state.lock().unwrap(),
		"lanIp": server::local_lan_ip(),
		"dbOk": db::ping(&db)
	})
}

#[tauri::command]
fn import_questions(path: String, opts: import::ImportOptions, db: tauri::State<db::Db>) -> Result<import::ImportSummary, String> {
	import::import_questions(&db, &path, opts)
}

// Raw bytes of a user-picked file (e.g. from the file dialog) for parsing on the frontend —
// used by the Word (.docx) soal importer, which unzips/parses the file in JS via jszip rather
// than needing a docx-parsing crate on the Rust side. Plain std::fs::read, so it isn't subject
// to the webview's scoped fs:allow-document-read capability (that scoping only applies to
// direct frontend fs-plugin calls, not to file I/O done here in Rust).
#[tauri::command]
fn read_file_bytes(path: String) -> Result<Vec<u8>, String> {
	std::fs::read(&path).map_err(|e| format!("Gagal membaca file: {e}"))
}

// Sama alasannya dengan read_file_bytes di atas: ditulis di sisi Rust supaya tidak kena scoping
// fs:allow-document-write, karena tujuan tulisnya adalah path bebas yang dipilih user lewat
// dialog save (mis. export Hasil Ujian ke .xlsx), bukan folder tertentu yang sudah di-scope.
#[tauri::command]
fn write_file_bytes(path: String, bytes: Vec<u8>) -> Result<(), String> {
	std::fs::write(&path, bytes).map_err(|e| format!("Gagal menyimpan file: {e}"))
}

#[tauri::command]
fn list_questions(db: tauri::State<db::Db>) -> Result<Vec<import::QuestionSummary>, String> {
	import::list_questions(&db)
}

#[tauri::command]
fn get_dashboard_stats(db: tauri::State<db::Db>) -> Result<import::DashboardStats, String> {
	import::dashboard_stats(&db)
}

#[tauri::command]
fn list_online_students(login_state: tauri::State<server::LoginState>) -> Vec<server::OnlineStudent> {
	server::list_online(&login_state)
}

#[tauri::command]
fn list_subjects(db: tauri::State<db::Db>) -> Result<Vec<exams::SubjectSummary>, String> {
	exams::list_subjects(&db)
}

#[tauri::command]
fn list_exams(db: tauri::State<db::Db>) -> Result<Vec<exams::ExamSummary>, String> {
	exams::list_exams(&db)
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
fn create_exam(
	subject_id: i64,
	class: Option<String>,
	jenis: Option<String>,
	title: String,
	duration: i64,
	scheduled_at: i64,
	window_end: i64,
	token: String,
	db: tauri::State<db::Db>
) -> Result<i64, String> {
	exams::create_exam(&db, subject_id, class, jenis, title, duration, scheduled_at, window_end, token)
}

#[tauri::command]
fn list_classes(db: tauri::State<db::Db>) -> Result<Vec<String>, String> {
	students::list_classes(&db)
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
fn update_exam(
	id: i64,
	subject_id: i64,
	class: Option<String>,
	jenis: Option<String>,
	title: String,
	duration: i64,
	scheduled_at: i64,
	window_end: i64,
	token: String,
	db: tauri::State<db::Db>
) -> Result<(), String> {
	exams::update_exam(&db, id, subject_id, class, jenis, title, duration, scheduled_at, window_end, token)
}

#[tauri::command]
fn delete_exam(id: i64, db: tauri::State<db::Db>) -> Result<(), String> {
	exams::delete_exam(&db, id)
}

#[tauri::command]
fn list_exam_sessions(db: tauri::State<db::Db>) -> Result<Vec<exam_session::SessionProgress>, String> {
	exam_session::list_sessions(&db)
}

#[tauri::command]
fn reset_exam_session(session_id: i64, db: tauri::State<db::Db>) -> Result<(), String> {
	exam_session::reset(&db, session_id)
}

#[tauri::command]
fn get_exam_session_detail(session_id: i64, db: tauri::State<db::Db>) -> Result<exam_session::SessionDetail, String> {
	exam_session::session_detail(&db, session_id)
}

#[tauri::command]
fn grade_essay_answer(session_id: i64, question_id: i64, score: f64, db: tauri::State<db::Db>) -> Result<(), String> {
	exam_session::grade_essay_answer(&db, session_id, question_id, score)
}

#[tauri::command]
fn get_exam_analytics(
	class: Option<String>,
	subject_id: i64,
	jenis: Option<String>,
	db: tauri::State<db::Db>
) -> Result<exam_session::AnalyticsResponse, String> {
	exam_session::analytics(&db, class, subject_id, jenis)
}

#[tauri::command]
fn parse_students_file(path: String) -> Result<Vec<students::StudentImportRow>, String> {
	students::parse_students_file(&path)
}

#[tauri::command]
fn import_students_rows(
	rows: Vec<students::StudentImportRow>,
	db: tauri::State<db::Db>
) -> Result<students::StudentImportSummary, String> {
	students::import_students_rows(&db, rows)
}

#[tauri::command]
fn list_students(db: tauri::State<db::Db>) -> Result<Vec<students::StudentRecord>, String> {
	students::list_students(&db)
}

#[tauri::command]
fn create_student(nisn: String, name: String, class: Option<String>, db: tauri::State<db::Db>) -> Result<i64, String> {
	students::create_student(&db, nisn, name, class)
}

#[tauri::command]
fn update_student(id: i64, nisn: String, name: String, class: Option<String>, db: tauri::State<db::Db>) -> Result<(), String> {
	students::update_student(&db, id, nisn, name, class)
}

#[tauri::command]
fn delete_student(id: i64, db: tauri::State<db::Db>) -> Result<(), String> {
	students::delete_student(&db, id)
}

#[tauri::command]
fn list_schools(db: tauri::State<db::Db>) -> Result<Vec<String>, String> {
	students::list_schools(&db)
}

#[tauri::command]
fn students_by_school(school: String, db: tauri::State<db::Db>) -> Result<Vec<students::StudentRecord>, String> {
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
fn get_question(id: i64, db: tauri::State<db::Db>) -> Result<import::QuestionDetail, String> {
	import::get_question(&db, id)
}

#[tauri::command]
fn create_question(input: import::QuestionInput, db: tauri::State<db::Db>) -> Result<i64, String> {
	import::create_question(&db, input)
}

#[tauri::command]
fn update_question(id: i64, input: import::QuestionInput, db: tauri::State<db::Db>) -> Result<(), String> {
	import::update_question(&db, id, input)
}

#[tauri::command]
fn delete_question(id: i64, db: tauri::State<db::Db>) -> Result<(), String> {
	import::delete_question(&db, id)
}

struct UploadsDir(std::path::PathBuf);

#[tauri::command]
fn save_question_image(path: String, uploads_dir: tauri::State<UploadsDir>) -> Result<uploads::SavedImage, String> {
	uploads::save_image(&uploads_dir.0, &path)
}

#[tauri::command]
fn load_image_data_url(path: String, uploads_dir: tauri::State<UploadsDir>) -> Result<String, String> {
	uploads::load_data_url(&uploads_dir.0, &path)
}

#[tauri::command]
fn save_question_image_bytes(file_name: String, bytes: Vec<u8>, uploads_dir: tauri::State<UploadsDir>) -> Result<uploads::SavedImage, String> {
	uploads::save_image_bytes(&uploads_dir.0, &file_name, &bytes)
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
fn export_jenis_soal(
	output_dir: String,
	kelas: String,
	subject_id: i64,
	jenis: String,
	pin: String,
	db: tauri::State<db::Db>,
	uploads_dir: tauri::State<UploadsDir>
) -> Result<export::ExportSummary, String> {
	export::export_jenis_soal(&db, &uploads_dir.0, &output_dir, &kelas, subject_id, &jenis, &pin)
}

#[tauri::command]
fn get_school(db: tauri::State<db::Db>) -> Result<settings::SchoolInfo, String> {
	settings::get_school(&db)
}

#[tauri::command]
fn update_school(info: settings::SchoolInfo, db: tauri::State<db::Db>) -> Result<(), String> {
	settings::update_school(&db, info)
}

#[tauri::command]
fn list_classes_full(db: tauri::State<db::Db>) -> Result<Vec<settings::ClassRecord>, String> {
	settings::list_classes_full(&db)
}

#[tauri::command]
fn create_class(title: String, description: Option<String>, db: tauri::State<db::Db>) -> Result<i64, String> {
	settings::create_class(&db, title, description)
}

#[tauri::command]
fn update_class(id: i64, title: String, description: Option<String>, db: tauri::State<db::Db>) -> Result<(), String> {
	settings::update_class(&db, id, title, description)
}

#[tauri::command]
fn delete_class(id: i64, db: tauri::State<db::Db>) -> Result<(), String> {
	settings::delete_class(&db, id)
}

#[tauri::command]
fn reorder_classes(ordered_ids: Vec<i64>, db: tauri::State<db::Db>) -> Result<(), String> {
	settings::reorder_classes(&db, ordered_ids)
}

#[tauri::command]
fn create_subject(name: String, code: Option<String>, db: tauri::State<db::Db>) -> Result<i64, String> {
	exams::create_subject(&db, name, code)
}

#[tauri::command]
fn update_subject(id: i64, name: String, code: Option<String>, db: tauri::State<db::Db>) -> Result<(), String> {
	exams::update_subject(&db, id, name, code)
}

#[tauri::command]
fn delete_subject(id: i64, db: tauri::State<db::Db>) -> Result<(), String> {
	exams::delete_subject(&db, id)
}

#[tauri::command]
fn reorder_subjects(ordered_ids: Vec<i64>, db: tauri::State<db::Db>) -> Result<(), String> {
	exams::reorder_subjects(&db, ordered_ids)
}

#[tauri::command]
fn list_question_types(class: Option<String>, subject_id: Option<i64>, db: tauri::State<db::Db>) -> Result<Vec<settings::QuestionTypeRecord>, String> {
	settings::list_question_types(&db, class, subject_id)
}

#[tauri::command]
fn create_question_type(
	name: String,
	description: Option<String>,
	class: Option<String>,
	subject_id: Option<i64>,
	db: tauri::State<db::Db>
) -> Result<i64, String> {
	settings::create_question_type(&db, name, description, class, subject_id)
}

#[tauri::command]
fn update_question_type(id: i64, name: String, description: Option<String>, db: tauri::State<db::Db>) -> Result<(), String> {
	settings::update_question_type(&db, id, name, description)
}

#[tauri::command]
fn delete_question_type(id: i64, db: tauri::State<db::Db>) -> Result<(), String> {
	settings::delete_question_type(&db, id)
}

#[tauri::command]
fn delete_question_type_scoped(
	id: i64,
	scope: String,
	current_class: String,
	current_subject_id: i64,
	db: tauri::State<db::Db>
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
			get_exam_session_detail,
			grade_essay_answer,
			get_exam_analytics,
			parse_students_file,
			import_students_rows,
			list_students,
			create_student,
			update_student,
			delete_student,
			list_schools,
			students_by_school,
			tarik_data_siswa,
			list_schools_from_source,
			get_question,
			create_question,
			update_question,
			delete_question,
			save_question_image,
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
