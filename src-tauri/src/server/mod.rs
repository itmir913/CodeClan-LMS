pub mod routes;
pub mod state;

use axum::{routing::{delete, get, post, put}, Router};
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
        .route("/auth/me", get(routes::auth::me_teacher).put(routes::auth::update_teacher_name))
        .route("/auth/me/password", put(routes::auth::change_password_teacher))
        .route("/auth/school-name", get(routes::auth::school_name))
        // Auth — student
        .route("/auth/login/student", post(routes::auth::login_student))
        .route("/auth/logout/student", post(routes::auth::logout_student))
        .route("/auth/student/me", get(routes::auth::me_student))
        .route("/auth/student/change-password", post(routes::auth::change_password_student))
        // Subjects (교사·admin 공용 조회)
        .route("/subjects", get(routes::admin::list_subjects))
        // Classes
        .route("/classes", get(routes::classes::list_classes).post(routes::classes::create_class))
        .route(
            "/classes/:id",
            get(routes::classes::get_class)
                .put(routes::classes::update_class)
                .delete(routes::classes::delete_class),
        )
        // Students
        .route(
            "/classes/:id/students",
            get(routes::students::list_students).post(routes::students::add_student),
        )
        .route("/classes/:id/students/bulk", post(routes::students::bulk_add_students))
        .route("/students/:id", delete(routes::students::delete_student))
        .route("/students/:id/reset-password", post(routes::students::reset_student_password))
        // Admin — teachers
        .route(
            "/admin/teachers",
            get(routes::admin::list_teachers).post(routes::admin::create_teacher),
        )
        .route(
            "/admin/teachers/:id",
            put(routes::admin::update_teacher).delete(routes::admin::delete_teacher),
        )
        // Admin — subjects
        .route("/admin/subjects", post(routes::admin::create_subject))
        .route("/admin/subjects/:id", delete(routes::admin::delete_subject))
        // Problems
        .route("/problems", get(routes::problems::list_problems).post(routes::problems::create_problem))
        .route(
            "/problems/:id",
            get(routes::problems::get_problem)
                .put(routes::problems::update_problem)
                .delete(routes::problems::delete_problem),
        );

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
