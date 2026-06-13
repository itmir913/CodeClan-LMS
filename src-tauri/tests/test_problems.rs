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
async fn list_problems_empty_initially() {
    let (app, cookie) = app_with_admin().await;

    let req = get_request("/api/problems", &cookie);
    let (status, body) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn create_and_get_problem() {
    let (app, cookie) = app_with_admin().await;

    let pid = create_problem(
        &app,
        &cookie,
        1,
        "덧셈 문제",
        json!({ "expected_output": "42" }),
    )
    .await;
    assert!(pid > 0);

    let req = get_request(&format!("/api/problems/{pid}"), &cookie);
    let (status, body) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["title"], json!("덧셈 문제"));
    assert_eq!(body["problem_type"], json!(1));
}

#[tokio::test]
async fn list_problems_with_type_filter() {
    let (app, cookie) = app_with_admin().await;

    create_problem(&app, &cookie, 1, "유형1 문제", json!({})).await;
    create_problem(&app, &cookie, 2, "유형2 문제", json!({})).await;
    create_problem(&app, &cookie, 1, "유형1 두번째", json!({})).await;

    let req = get_request("/api/problems?problem_type=1", &cookie);
    let (status, body) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn create_problem_invalid_type_returns_400() {
    let (app, cookie) = app_with_admin().await;

    let req = authed_request(
        Method::POST,
        "/api/problems",
        &cookie,
        json!({ "problem_type": 99, "title": "잘못된유형" }),
    );
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn create_problem_invalid_json_type_config_returns_400() {
    let (app, cookie) = app_with_admin().await;

    let req = authed_request(
        Method::POST,
        "/api/problems",
        &cookie,
        json!({ "problem_type": 1, "title": "문제", "type_config": "not_valid_json" }),
    );
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn update_problem() {
    let (app, cookie) = app_with_admin().await;

    let pid = create_problem(&app, &cookie, 1, "원래제목", json!({})).await;

    let req = authed_request(
        Method::PUT,
        &format!("/api/problems/{pid}"),
        &cookie,
        json!({ "title": "수정된제목", "type_config": "{\"expected_output\": \"hello\"}" }),
    );
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK);

    let req2 = get_request(&format!("/api/problems/{pid}"), &cookie);
    let (_, body) = response_json(&app, req2).await;
    assert_eq!(body["title"], json!("수정된제목"));
}

#[tokio::test]
async fn delete_problem() {
    let (app, cookie) = app_with_admin().await;
    let pid = create_problem(&app, &cookie, 1, "삭제할문제", json!({})).await;

    let req = authed_request(
        Method::DELETE,
        &format!("/api/problems/{pid}"),
        &cookie,
        json!({}),
    );
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK);

    let req2 = get_request(&format!("/api/problems/{pid}"), &cookie);
    let (s2, _) = response_json(&app, req2).await;
    assert_eq!(s2, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_problem_assigned_to_assessment_returns_400() {
    let (app, cookie) = app_with_admin().await;

    let pid = create_problem(&app, &cookie, 1, "배정된문제", json!({})).await;
    let aid = create_assessment(&app, &cookie, "수행평가1").await;

    // 수행평가에 문제 배정
    let req = authed_request(
        Method::PUT,
        &format!("/api/assessments/{aid}/problems"),
        &cookie,
        json!([{ "problem_id": pid, "order_no": 1, "score": 10 }]),
    );
    response_json(&app, req).await;

    // 배정된 문제 삭제 시도 → 400
    let req2 = authed_request(
        Method::DELETE,
        &format!("/api/problems/{pid}"),
        &cookie,
        json!({}),
    );
    let (status, _) = response_json(&app, req2).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn get_nonexistent_problem_returns_404() {
    let (app, cookie) = app_with_admin().await;

    let req = get_request("/api/problems/99999", &cookie);
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
