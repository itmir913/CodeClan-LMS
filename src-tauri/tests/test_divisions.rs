mod common;

use common::*;
use hyper::{Method, StatusCode};
use serde_json::json;
use tower::ServiceExt;

async fn app_with_admin() -> (axum::Router, String) {
    let pool = setup_test_db().await;
    let app = build_test_app(pool);
    setup_admin(&app).await;
    let cookie = teacher_login_cookie(&app, "admin", "password123").await;
    (app, cookie)
}

// ─── 분반 CRUD ─────────────────────────────────────────────

#[tokio::test]
async fn list_divisions_requires_auth() {
    let pool = setup_test_db().await;
    let app = build_test_app(pool);
    let req = get_request("/api/divisions", "");
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn create_and_list_divisions() {
    let (app, cookie) = app_with_admin().await;

    let id = create_division(&app, &cookie, "1반").await;
    assert!(id > 0);

    let req = get_request("/api/divisions", &cookie);
    let (status, body) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK);
    let list = body.as_array().unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["name"], json!("1반"));
}

#[tokio::test]
async fn create_division_requires_admin() {
    let (app, admin_cookie) = app_with_admin().await;

    // teacher 계정 생성
    let req = authed_request(
        Method::POST,
        "/api/teachers",
        &admin_cookie,
        json!({ "username": "teacher1", "name": "선생님", "password": "pass1234" }),
    );
    let (_, _) = response_json(&app, req).await;

    let teacher_cookie = teacher_login_cookie(&app, "teacher1", "pass1234").await;

    let req2 = authed_request(
        Method::POST,
        "/api/divisions",
        &teacher_cookie,
        json!({ "name": "2반" }),
    );
    let (status, _) = response_json(&app, req2).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn update_division_name() {
    let (app, cookie) = app_with_admin().await;
    let div_id = create_division(&app, &cookie, "원래이름").await;

    let req = authed_request(
        Method::PUT,
        &format!("/api/divisions/{div_id}"),
        &cookie,
        json!({ "name": "바뀐이름" }),
    );
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK);

    let req2 = get_request("/api/divisions", &cookie);
    let (_, body) = response_json(&app, req2).await;
    assert_eq!(body[0]["name"], json!("바뀐이름"));
}

#[tokio::test]
async fn delete_empty_division() {
    let (app, cookie) = app_with_admin().await;
    let div_id = create_division(&app, &cookie, "삭제할반").await;

    let req = authed_request(
        Method::DELETE,
        &format!("/api/divisions/{div_id}"),
        &cookie,
        json!({}),
    );
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK);

    let req2 = get_request("/api/divisions", &cookie);
    let (_, body) = response_json(&app, req2).await;
    assert_eq!(body.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn delete_division_with_students_returns_400() {
    let (app, cookie) = app_with_admin().await;
    let div_id = create_division(&app, &cookie, "학생있는반").await;
    add_student(&app, &cookie, div_id, "20240001", "학생", "pw123456").await;

    let req = authed_request(
        Method::DELETE,
        &format!("/api/divisions/{div_id}"),
        &cookie,
        json!({}),
    );
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// ─── 학생 관리 ─────────────────────────────────────────────

#[tokio::test]
async fn add_and_list_students() {
    let (app, cookie) = app_with_admin().await;
    let div_id = create_division(&app, &cookie, "1반").await;

    let sid = add_student(&app, &cookie, div_id, "20240001", "홍길동", "initpw1").await;
    assert!(sid > 0);

    let req = get_request(&format!("/api/divisions/{div_id}/students"), &cookie);
    let (status, body) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK);
    let list = body.as_array().unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["student_number"], json!("20240001"));
    assert_eq!(list[0]["password_reset_required"], json!(true));
}

#[tokio::test]
async fn add_student_duplicate_number_returns_400() {
    let (app, cookie) = app_with_admin().await;
    let div_id = create_division(&app, &cookie, "1반").await;
    add_student(&app, &cookie, div_id, "20240001", "홍길동", "pw123").await;

    let req = authed_request(
        Method::POST,
        &format!("/api/divisions/{div_id}/students"),
        &cookie,
        json!({ "student_number": "20240001", "name": "다른학생", "password": "pw456" }),
    );
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn bulk_import_students() {
    let (app, cookie) = app_with_admin().await;
    let div_id = create_division(&app, &cookie, "1반").await;

    let req = authed_request(
        Method::POST,
        &format!("/api/divisions/{div_id}/students/bulk"),
        &cookie,
        json!([
            { "student_number": "20240001", "name": "학생A", "password": "pw1234" },
            { "student_number": "20240002", "name": "학생B", "password": "pw5678" },
            { "student_number": "20240001", "name": "중복학생", "password": "pw9012" }
        ]),
    );
    let (status, body) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["inserted"], json!(2));
    assert_eq!(body["skipped"], json!(1)); // 중복
}

#[tokio::test]
async fn delete_student() {
    let (app, cookie) = app_with_admin().await;
    let div_id = create_division(&app, &cookie, "1반").await;
    let sid = add_student(&app, &cookie, div_id, "20240001", "홍길동", "pw1234").await;

    let req = authed_request(
        Method::DELETE,
        &format!("/api/students/{sid}"),
        &cookie,
        json!({}),
    );
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK);

    // 목록에서 사라짐
    let req2 = get_request(&format!("/api/divisions/{div_id}/students"), &cookie);
    let (_, body) = response_json(&app, req2).await;
    assert_eq!(body.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn reset_student_password_sets_reset_flag() {
    let (app, cookie) = app_with_admin().await;
    let div_id = create_division(&app, &cookie, "1반").await;
    let sid = add_student(&app, &cookie, div_id, "20240001", "홍길동", "pw1234").await;

    // 학생이 먼저 비밀번호 변경 (reset_required = 0)
    let s_cookie = student_login_cookie(&app, "20240001", "pw1234").await;
    let req = authed_request(
        Method::POST,
        "/api/auth/student/change-password",
        &s_cookie,
        json!({ "current_password": "pw1234", "new_password": "newpass1" }),
    );
    response_json(&app, req).await;

    // 비밀번호 초기화 → reset_required = 1로 복귀
    let req2 = authed_request(
        Method::POST,
        &format!("/api/students/{sid}/reset-password"),
        &cookie,
        json!({ "new_password": "resetpw1" }),
    );
    let (status, _) = response_json(&app, req2).await;
    assert_eq!(status, StatusCode::OK);

    let s_cookie2 = student_login_cookie(&app, "20240001", "resetpw1").await;
    let req3 = get_request("/api/auth/student/me", &s_cookie2);
    let (_, body) = response_json(&app, req3).await;
    assert_eq!(body["password_reset_required"], json!(true));
}

// ─── 분반-교사 연결 ────────────────────────────────────────

#[tokio::test]
async fn set_and_get_division_teachers() {
    let (app, cookie) = app_with_admin().await;
    let div_id = create_division(&app, &cookie, "1반").await;

    // teacher 계정 생성
    let req = authed_request(
        Method::POST,
        "/api/teachers",
        &cookie,
        json!({ "username": "t1", "name": "교사1", "password": "pass1234" }),
    );
    let (_, tbody) = response_json(&app, req).await;
    let tid = tbody["id"].as_i64().unwrap();

    // 분반에 교사 연결
    let req2 = authed_request(
        Method::PUT,
        &format!("/api/divisions/{div_id}/teachers"),
        &cookie,
        json!({ "teacher_ids": [tid] }),
    );
    let (status, _) = response_json(&app, req2).await;
    assert_eq!(status, StatusCode::OK);

    // 조회
    let req3 = get_request(&format!("/api/divisions/{div_id}/teachers"), &cookie);
    let (s3, b3) = response_json(&app, req3).await;
    assert_eq!(s3, StatusCode::OK);
    let teachers = b3.as_array().unwrap();
    assert_eq!(teachers.len(), 1);
    assert_eq!(teachers[0]["username"], json!("t1"));
}
