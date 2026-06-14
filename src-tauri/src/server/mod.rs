pub mod routes;
pub mod state;

use axum::{routing::{get, post}, Router};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

use state::AppState;

pub fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let api = Router::new()
        .route("/setup/status", get(routes::setup::get_status))
        .route("/setup/complete", post(routes::setup::complete));

    Router::new()
        .nest("/api", api)
        .layer(cors)
        .with_state(state)
}

pub async fn start(state: AppState) -> anyhow::Result<()> {
    let app = build_router(state);
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("Axum server listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
