pub mod routes;
pub mod state;

use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

use state::AppState;

pub async fn start(state: AppState) -> anyhow::Result<()> {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let api = Router::new()
        // 초기 설정
        .route("/setup/status", get(routes::setup::get_status))
        .route("/setup/complete", post(routes::setup::complete))
        // 인증
        .route("/auth/login/teacher", post(routes::auth::teacher_login))
        .route("/auth/logout", post(routes::auth::logout))
        .route("/auth/me", get(routes::auth::me))
        .route("/auth/school-name", get(routes::auth::school_name));

    let app = Router::new()
        .nest("/api", api)
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("Axum server listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
