pub mod db;
pub mod error;
pub mod server;

use server::state::AppState;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    Manager, WindowEvent,
};

pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "codeclan_lms=info,axum=info".into()),
        )
        .init();

    tauri::Builder::default()
        .setup(|app| {
            let exe_dir = std::env::current_exe()
                .expect("current_exe 경로 조회 실패")
                .parent()
                .expect("exe 부모 디렉토리 없음")
                .to_path_buf();
            let data_dir = exe_dir.join("data");
            let db_path = data_dir.join("codeclan.db");

            tauri::async_runtime::spawn(async move {
                let pool = db::init(&db_path)
                    .await
                    .expect("Database initialization failed");
                let state = AppState { db: pool, data_dir };
                server::start(state)
                    .await
                    .expect("Axum server failed");
            });

            let menu_open = MenuItem::with_id(app, "open", "열기", true, None::<&str>)?;
            let menu_quit = MenuItem::with_id(app, "quit", "LMS 종료", true, None::<&str>)?;
            let tray_menu = Menu::with_items(app, &[&menu_open, &menu_quit])?;

            let tray_result = TrayIconBuilder::new()
                .tooltip("CodeClan LMS")
                .menu(&tray_menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::DoubleClick { .. } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app);

            if let Err(e) = tray_result {
                tracing::warn!("Tray icon not available: {e}");
            }

            let window = app.get_webview_window("main").unwrap();
            let win_clone = window.clone();
            window.on_window_event(move |event| {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = win_clone.hide();
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
