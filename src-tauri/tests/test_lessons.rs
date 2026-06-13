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

// ─── 차시 CRUD ─────────────────────────────────────────────

#[tokio::test]
async fn create_and_list_lessons() {
    let (app, cookie) = app_with_admin().await;

    let req = authed_request(
        Method::POST,
        "/api/lessons",
        &cookie,
        json!({ "title": "1차시: Hello World", "order_no": 1 }),
    );
    let (status, body) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let lid = body["id"].as_i64().unwrap();

    let req2 = get_request("/api/lessons", &cookie);
    let (s2, b2) = response_json(&app, req2).await;
    assert_eq!(s2, StatusCode::OK);
    assert_eq!(b2.as_array().unwrap().len(), 1);
    assert_eq!(b2[0]["id"], json!(lid));
}

#[tokio::test]
async fn get_lesson_detail() {
    let (app, cookie) = app_with_admin().await;

    let req = authed_request(
        Method::POST,
        "/api/lessons",
        &cookie,
        json!({ "title": "2차시", "description": "설명" }),
    );
    let (_, b) = response_json(&app, req).await;
    let lid = b["id"].as_i64().unwrap();

    let req2 = get_request(&format!("/api/lessons/{lid}"), &cookie);
    let (status, body) = response_json(&app, req2).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["title"], json!("2차시"));
    assert_eq!(body["description"], json!("설명"));
    assert!(body["problems"].is_array());
}

#[tokio::test]
async fn update_lesson() {
    let (app, cookie) = app_with_admin().await;

    let req = authed_request(
        Method::POST,
        "/api/lessons",
        &cookie,
        json!({ "title": "원래제목" }),
    );
    let (_, b) = response_json(&app, req).await;
    let lid = b["id"].as_i64().unwrap();

    let req2 = authed_request(
        Method::PUT,
        &format!("/api/lessons/{lid}"),
        &cookie,
        json!({ "title": "수정된제목", "order_no": 5 }),
    );
    let (status, _) = response_json(&app, req2).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn delete_lesson() {
    let (app, cookie) = app_with_admin().await;

    let req = authed_request(
        Method::POST,
        "/api/lessons",
        &cookie,
        json!({ "title": "삭제할차시" }),
    );
    let (_, b) = response_json(&app, req).await;
    let lid = b["id"].as_i64().unwrap();

    let req2 = authed_request(Method::DELETE, &format!("/api/lessons/{lid}"), &cookie, json!({}));
    let (status, _) = response_json(&app, req2).await;
    assert_eq!(status, StatusCode::OK);

    let req3 = get_request("/api/lessons", &cookie);
    let (_, b3) = response_json(&app, req3).await;
    assert_eq!(b3.as_array().unwrap().len(), 0);
}

// ─── 차시 문항 배정 ────────────────────────────────────────

#[tokio::test]
async fn set_lesson_problems() {
    let (app, cookie) = app_with_admin().await;

    let req = authed_request(Method::POST, "/api/lessons", &cookie, json!({ "title": "1차시" }));
    let (_, lb) = response_json(&app, req).await;
    let lid = lb["id"].as_i64().unwrap();

    let p1 = create_problem(&app, &cookie, 1, "문제1", json!({})).await;
    let p2 = create_problem(&app, &cookie, 2, "문제2", json!({})).await;

    let req2 = authed_request(
        Method::PUT,
        &format!("/api/lessons/{lid}/problems"),
        &cookie,
        json!({ "problem_ids": [p1, p2] }),
    );
    let (status, _) = response_json(&app, req2).await;
    assert_eq!(status, StatusCode::OK);

    let req3 = get_request(&format!("/api/lessons/{lid}"), &cookie);
    let (_, b3) = response_json(&app, req3).await;
    assert_eq!(b3["problems"].as_array().unwrap().len(), 2);
}

// ─── 분반별 공개 토글 ─────────────────────────────────────

#[tokio::test]
async fn toggle_lesson_release() {
    let (app, cookie) = app_with_admin().await;

    let req = authed_request(Method::POST, "/api/lessons", &cookie, json!({ "title": "1차시" }));
    let (_, lb) = response_json(&app, req).await;
    let lid = lb["id"].as_i64().unwrap();

    let div_id = create_division(&app, &cookie, "1반").await;

    // 공개
    let req2 = authed_request(
        Method::PUT,
        &format!("/api/lessons/{lid}/release"),
        &cookie,
        json!({ "division_id": div_id, "is_released": true }),
    );
    let (status, body) = response_json(&app, req2).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body["released_at"].is_string());

    // 비공개
    let req3 = authed_request(
        Method::PUT,
        &format!("/api/lessons/{lid}/release"),
        &cookie,
        json!({ "division_id": div_id, "is_released": false }),
    );
    let (s3, b3) = response_json(&app, req3).await;
    assert_eq!(s3, StatusCode::OK);
    assert!(b3["released_at"].is_null());
}

#[tokio::test]
async fn student_sees_only_released_lessons() {
    let (app, cookie) = app_with_admin().await;

    let div_id = create_division(&app, &cookie, "1반").await;
    let _ = add_student(&app, &cookie, div_id, "20240001", "학생A", "pw1234").await;

    let l1_req = authed_request(Method::POST, "/api/lessons", &cookie, json!({ "title": "공개차시" }));
    let (_, l1b) = response_json(&app, l1_req).await;
    let l1_id = l1b["id"].as_i64().unwrap();

    let l2_req = authed_request(Method::POST, "/api/lessons", &cookie, json!({ "title": "비공개차시" }));
    response_json(&app, l2_req).await;

    // l1만 공개
    let rel_req = authed_request(
        Method::PUT,
        &format!("/api/lessons/{l1_id}/release"),
        &cookie,
        json!({ "division_id": div_id, "is_released": true }),
    );
    response_json(&app, rel_req).await;

    let s_cookie = student_login_cookie(&app, "20240001", "pw1234").await;
    let req = get_request("/api/student/lessons", &s_cookie);
    let (status, body) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK);
    let list = body.as_array().unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["title"], json!("공개차시"));
}
