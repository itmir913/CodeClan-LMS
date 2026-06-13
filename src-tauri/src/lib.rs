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
            let app_data_dir = app.path().app_data_dir()?;
            let db_path = app_data_dir.join("codeclan.db");

            // Axum 서버를 별도 tokio 태스크로 실행
            tauri::async_runtime::spawn(async move {
                let pool = db::init(&db_path)
                    .await
                    .expect("Database initialization failed");
                let state = AppState { db: pool };
                server::start(state)
                    .await
                    .expect("Axum server failed");
            });

            // 트레이 아이콘 설정 (아이콘 파일이 없어도 실패하지 않음)
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
                tracing::warn!("Tray icon not available (run `cargo tauri icon` to generate): {e}");
            }

            // 닫기 버튼 → 트레이로 숨김 (종료 아님)
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
