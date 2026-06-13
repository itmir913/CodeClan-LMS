use axum::Router;
use axum::body::Body;
use http_body_util::BodyExt;
use hyper::{Method, Request, StatusCode};
use serde_json::{json, Value};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use tower::ServiceExt;

use codeclan_lms_lib::server::{build_router, state::AppState};

pub async fn setup_test_db() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("인메모리 DB 연결 실패");

    sqlx::query("PRAGMA journal_mode=WAL")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("PRAGMA foreign_keys=ON")
        .execute(&pool)
        .await
        .unwrap();

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("마이그레이션 실패");

    pool
}

pub fn build_test_app(pool: SqlitePool) -> Router {
    build_router(AppState { db: pool })
}

pub async fn response_json(app: &Router, req: Request<Body>) -> (StatusCode, Value) {
    let resp = app.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, json)
}

pub fn json_request(method: Method, uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

pub fn authed_request(method: Method, uri: &str, cookie: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .header("cookie", cookie)
        .body(Body::from(body.to_string()))
        .unwrap()
}

pub fn get_request(uri: &str, cookie: &str) -> Request<Body> {
    Request::builder()
        .method(Method::GET)
        .uri(uri)
        .header("cookie", cookie)
        .body(Body::empty())
        .unwrap()
}

/// setup/complete 실행 → admin 계정 생성, settings 행 삽입
pub async fn setup_admin(app: &Router) {
    let req = json_request(
        Method::POST,
        "/api/setup/complete",
        json!({
            "school_name": "테스트고등학교",
            "admin_name": "관리자",
            "admin_username": "admin",
            "admin_password": "password123"
        }),
    );
    let (status, body) = response_json(app, req).await;
    assert_eq!(status, StatusCode::OK, "setup_admin 실패: {body}");
}

/// 교사 로그인 → "cc_session=<token>" 쿠키 문자열 반환
pub async fn teacher_login_cookie(app: &Router, username: &str, password: &str) -> String {
    let req = json_request(
        Method::POST,
        "/api/auth/login/teacher",
        json!({ "username": username, "password": password }),
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "교사 로그인 실패");

    resp.headers()
        .get("set-cookie")
        .expect("set-cookie 헤더 없음")
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_string()
}

/// 학생 로그인 → "cc_student=<token>" 쿠키 문자열 반환
pub async fn student_login_cookie(app: &Router, student_number: &str, password: &str) -> String {
    let req = json_request(
        Method::POST,
        "/api/auth/login/student",
        json!({ "student_number": student_number, "password": password }),
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "학생 로그인 실패 (학번: {student_number})");

    resp.headers()
        .get("set-cookie")
        .expect("set-cookie 헤더 없음")
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_string()
}

/// 분반 생성 → division id 반환 (admin 쿠키 필요)
pub async fn create_division(app: &Router, cookie: &str, name: &str) -> i64 {
    let req = authed_request(
        Method::POST,
        "/api/divisions",
        cookie,
        json!({ "name": name }),
    );
    let (status, body) = response_json(app, req).await;
    assert_eq!(status, StatusCode::OK, "분반 생성 실패: {body}");
    body["id"].as_i64().expect("id 필드 없음")
}

/// 학생 추가 → student id 반환 (admin 쿠키 필요)
pub async fn add_student(
    app: &Router,
    cookie: &str,
    division_id: i64,
    student_number: &str,
    name: &str,
    password: &str,
) -> i64 {
    let uri = format!("/api/divisions/{division_id}/students");
    let req = authed_request(
        Method::POST,
        &uri,
        cookie,
        json!({
            "student_number": student_number,
            "name": name,
            "password": password
        }),
    );
    let (status, body) = response_json(app, req).await;
    assert_eq!(status, StatusCode::OK, "학생 추가 실패: {body}");
    body["id"].as_i64().expect("id 필드 없음")
}

/// 문제 생성 → problem id 반환
pub async fn create_problem(
    app: &Router,
    cookie: &str,
    problem_type: i64,
    title: &str,
    type_config: Value,
) -> i64 {
    let req = authed_request(
        Method::POST,
        "/api/problems",
        cookie,
        json!({
            "problem_type": problem_type,
            "title": title,
            "type_config": type_config.to_string()
        }),
    );
    let (status, body) = response_json(app, req).await;
    assert_eq!(status, StatusCode::OK, "문제 생성 실패: {body}");
    body["id"].as_i64().expect("id 필드 없음")
}

/// 수행평가 생성 → assessment id 반환
pub async fn create_assessment(app: &Router, cookie: &str, title: &str) -> i64 {
    let req = authed_request(
        Method::POST,
        "/api/assessments",
        cookie,
        json!({ "title": title }),
    );
    let (status, body) = response_json(app, req).await;
    assert_eq!(status, StatusCode::OK, "수행평가 생성 실패: {body}");
    body["id"].as_i64().expect("id 필드 없음")
}

/// 세션 생성 → session id 반환 (CREATED 상태)
pub async fn create_session(app: &Router, cookie: &str, assessment_id: i64, division_id: i64) -> i64 {
    let req = authed_request(
        Method::POST,
        "/api/sessions",
        cookie,
        json!({
            "assessment_id": assessment_id,
            "division_id": division_id
        }),
    );
    let (status, body) = response_json(app, req).await;
    assert_eq!(status, StatusCode::OK, "세션 생성 실패: {body}");
    body["id"].as_i64().expect("id 필드 없음")
}

/// 세션 상태 전환 헬퍼
pub async fn transition_session(app: &Router, cookie: &str, session_id: i64, action: &str) {
    let uri = format!("/api/sessions/{session_id}/transition");
    let req = authed_request(
        Method::POST,
        &uri,
        cookie,
        json!({ "action": action }),
    );
    let (status, body) = response_json(app, req).await;
    assert_eq!(status, StatusCode::OK, "세션 전환 실패({action}): {body}");
}

/// CREATED → LOBBY → RUNNING 까지 전환하고 session_id 반환
pub async fn create_running_session(
    app: &Router,
    cookie: &str,
    assessment_id: i64,
    division_id: i64,
) -> i64 {
    let sid = create_session(app, cookie, assessment_id, division_id).await;
    transition_session(app, cookie, sid, "to_lobby").await;
    transition_session(app, cookie, sid, "to_running").await;
    sid
}
