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
        .route("/setup/complete", post(routes::setup::complete))
        // Auth — teacher
        .route("/auth/login/teacher", post(routes::auth::login_teacher))
        .route("/auth/logout", post(routes::auth::logout_teacher))
        .route("/auth/me", get(routes::auth::me_teacher))
        .route("/auth/school-name", get(routes::auth::school_name))
        // Auth — student
        .route("/auth/login/student", post(routes::auth::login_student))
        .route("/auth/logout/student", post(routes::auth::logout_student))
        .route("/auth/student/me", get(routes::auth::me_student))
        .route("/auth/student/change-password", post(routes::auth::change_password_student));

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
