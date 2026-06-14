mod db;
pub mod error;
pub mod server;

use server::state::AppState;
use tauri::{
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
            let db_path = exe_dir.join("data").join("codeclan.db");

            tauri::async_runtime::spawn(async move {
                let pool = db::init(&db_path)
                    .await
                    .expect("Database initialization failed");
                let state = AppState { db: pool };
                server::start(state)
                    .await
                    .expect("Axum server failed");
            });

            let tray_result = TrayIconBuilder::new()
                .tooltip("CodeClan LMS")
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
