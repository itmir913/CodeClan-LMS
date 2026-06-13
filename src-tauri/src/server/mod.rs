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
        // 대시보드
        .route("/dashboard", get(routes::dashboard::get_dashboard))
        // 교사 인증
        .route("/auth/login/teacher", post(routes::auth::teacher_login))
        .route("/auth/logout", post(routes::auth::logout))
        .route("/auth/me", get(routes::auth::me))
        .route("/auth/school-name", get(routes::auth::school_name))
        // 학생 인증
        .route("/auth/login/student", post(routes::auth::student_login))
        .route("/auth/logout/student", post(routes::auth::student_logout))
        .route("/auth/student/me", get(routes::auth::student_me))
        .route("/auth/student/change-password", post(routes::auth::student_change_password))
        // 분반 관리
        .route("/divisions", get(routes::divisions::get_divisions).post(routes::divisions::create_division))
        .route("/divisions/:id", axum::routing::put(routes::divisions::update_division).delete(routes::divisions::delete_division))
        .route("/divisions/:id/teachers", get(routes::divisions::get_division_teachers).put(routes::divisions::set_division_teachers))
        // 학생 관리
        .route("/divisions/:id/students", get(routes::divisions::get_students).post(routes::divisions::add_student))
        .route("/divisions/:id/students/bulk", post(routes::divisions::bulk_import_students))
        .route("/students/:id", axum::routing::delete(routes::divisions::delete_student))
        .route("/students/:id/reset-password", post(routes::divisions::reset_student_password))
        // 교사 계정 관리 (admin only)
        .route("/teachers", get(routes::teachers::get_teachers).post(routes::teachers::create_teacher))
        .route("/teachers/:id", axum::routing::put(routes::teachers::update_teacher).delete(routes::teachers::delete_teacher))
        // 문제 은행
        .route("/problems", get(routes::problems::list_problems).post(routes::problems::create_problem))
        .route("/problems/:id", get(routes::problems::get_problem).put(routes::problems::update_problem).delete(routes::problems::delete_problem))
        // 차시 관리
        .route("/lessons", get(routes::lessons::list_lessons).post(routes::lessons::create_lesson))
        .route("/lessons/:id", get(routes::lessons::get_lesson).put(routes::lessons::update_lesson).delete(routes::lessons::delete_lesson))
        .route("/lessons/:id/problems", axum::routing::put(routes::lessons::set_lesson_problems))
        .route("/lessons/:id/release", axum::routing::put(routes::lessons::toggle_release))
        // 수행평가 관리
        .route("/assessments", get(routes::assessments::list_assessments).post(routes::assessments::create_assessment))
        .route("/assessments/:id", get(routes::assessments::get_assessment).put(routes::assessments::update_assessment).delete(routes::assessments::delete_assessment))
        .route("/assessments/:id/problems", axum::routing::put(routes::assessments::set_assessment_problems))
        .route("/assessments/:id/divisions", post(routes::assessments::link_division))
        .route("/assessments/:id/divisions/:division_id", axum::routing::delete(routes::assessments::unlink_division))
        // 세션 관리
        .route("/sessions", get(routes::sessions::list_sessions).post(routes::sessions::create_session))
        .route("/sessions/:id/transition", post(routes::sessions::transition_session))
        .route("/sessions/:id/pause", post(routes::sessions::pause_session))
        .route("/sessions/:id/result-release", post(routes::sessions::toggle_result_release))
        // 학생 전용 라우트
        .route("/student/lessons", get(routes::student::get_lessons))
        .route("/student/assessments", get(routes::student::get_assessments))
        .route("/student/active-session", get(routes::student::get_active_session))
        .route("/student/session-problems", get(routes::submissions::get_session_problems))
        .route("/student/submit", post(routes::submissions::submit_answer))
        // 제출/채점 (교사)
        .route("/sessions/:id/submissions", get(routes::submissions::get_session_submissions))
        .route("/submissions/:id/grade", post(routes::submissions::grade_submission));

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
