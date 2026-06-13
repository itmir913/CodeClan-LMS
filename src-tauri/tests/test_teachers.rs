mod common;

use common::*;
use hyper::{Method, StatusCode};
use serde_json::json;

async fn app_with_admin() -> (axum::Router, String) {
    let pool = setup_test_db().await;
    let app = build_test_app(pool);
    setup_admin(&app).await;
    let cookie = teacher_login_cookie(&app, "admin", "password123").await;
    (app, cookie)
}

#[tokio::test]
async fn list_teachers_admin_only() {
    let (app, cookie) = app_with_admin().await;

    let req = get_request("/api/teachers", &cookie);
    let (status, body) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK);
    // admin 자신 포함 1명
    assert_eq!(body.as_array().unwrap().len(), 1);
    assert_eq!(body[0]["role"], json!("admin"));
}

#[tokio::test]
async fn list_teachers_teacher_role_returns_403() {
    let (app, admin_cookie) = app_with_admin().await;

    // teacher 계정 생성
    let req = authed_request(
        Method::POST,
        "/api/teachers",
        &admin_cookie,
        json!({ "username": "t1", "name": "교사1", "password": "pass1234" }),
    );
    response_json(&app, req).await;

    let t_cookie = teacher_login_cookie(&app, "t1", "pass1234").await;
    let req2 = get_request("/api/teachers", &t_cookie);
    let (status, _) = response_json(&app, req2).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn create_teacher_by_admin() {
    let (app, cookie) = app_with_admin().await;

    let req = authed_request(
        Method::POST,
        "/api/teachers",
        &cookie,
        json!({ "username": "t2", "name": "교사2", "password": "pass1234", "role": "teacher" }),
    );
    let (status, body) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["username"], json!("t2"));
    assert_eq!(body["role"], json!("teacher"));
}

#[tokio::test]
async fn create_teacher_by_non_admin_returns_403() {
    let (app, admin_cookie) = app_with_admin().await;

    // teacher 계정 생성
    let req = authed_request(
        Method::POST,
        "/api/teachers",
        &admin_cookie,
        json!({ "username": "t1", "name": "교사1", "password": "pass1234" }),
    );
    response_json(&app, req).await;

    let t_cookie = teacher_login_cookie(&app, "t1", "pass1234").await;

    let req2 = authed_request(
        Method::POST,
        "/api/teachers",
        &t_cookie,
        json!({ "username": "t3", "name": "교사3", "password": "pass1234" }),
    );
    let (status, _) = response_json(&app, req2).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn create_teacher_duplicate_username_returns_400() {
    let (app, cookie) = app_with_admin().await;

    let req = authed_request(
        Method::POST,
        "/api/teachers",
        &cookie,
        json!({ "username": "t1", "name": "교사1", "password": "pass1234" }),
    );
    response_json(&app, req).await;

    let req2 = authed_request(
        Method::POST,
        "/api/teachers",
        &cookie,
        json!({ "username": "t1", "name": "다른교사", "password": "pass5678" }),
    );
    let (status, _) = response_json(&app, req2).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn update_teacher_info() {
    let (app, cookie) = app_with_admin().await;

    let req = authed_request(
        Method::POST,
        "/api/teachers",
        &cookie,
        json!({ "username": "t1", "name": "교사1", "password": "pass1234" }),
    );
    let (_, tbody) = response_json(&app, req).await;
    let tid = tbody["id"].as_i64().unwrap();

    let req2 = authed_request(
        Method::PUT,
        &format!("/api/teachers/{tid}"),
        &cookie,
        json!({ "name": "수정된이름", "role": "admin" }),
    );
    let (status, _) = response_json(&app, req2).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn delete_teacher() {
    let (app, cookie) = app_with_admin().await;

    let req = authed_request(
        Method::POST,
        "/api/teachers",
        &cookie,
        json!({ "username": "t1", "name": "교사1", "password": "pass1234" }),
    );
    let (_, tbody) = response_json(&app, req).await;
    let tid = tbody["id"].as_i64().unwrap();

    let req2 = authed_request(
        Method::DELETE,
        &format!("/api/teachers/{tid}"),
        &cookie,
        json!({}),
    );
    let (status, _) = response_json(&app, req2).await;
    assert_eq!(status, StatusCode::OK);

    // 목록에서 사라짐 (admin만 남음)
    let req3 = get_request("/api/teachers", &cookie);
    let (_, b3) = response_json(&app, req3).await;
    assert_eq!(b3.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn delete_self_returns_400() {
    let (app, cookie) = app_with_admin().await;

    // admin 자신의 id 조회
    let req = get_request("/api/auth/me", &cookie);
    let (_, me) = response_json(&app, req).await;
    let admin_id = me["id"].as_i64().unwrap();

    let req2 = authed_request(
        Method::DELETE,
        &format!("/api/teachers/{admin_id}"),
        &cookie,
        json!({}),
    );
    let (status, _) = response_json(&app, req2).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}
