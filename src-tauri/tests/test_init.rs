mod common;

use common::*;
use hyper::{Method, StatusCode};
use serde_json::json;

#[tokio::test]
async fn setup_status_needs_setup_initially() {
    let pool = setup_test_db().await;
    let app = build_test_app(pool);

    let req = get_request("/api/setup/status", "");
    let (status, body) = response_json(&app, req).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["needs_setup"], json!(true));
}

#[tokio::test]
async fn setup_complete_creates_admin_and_settings() {
    let pool = setup_test_db().await;
    let app = build_test_app(pool);

    let req = json_request(
        Method::POST,
        "/api/setup/complete",
        json!({
            "school_name": "한국고등학교",
            "admin_name": "홍길동",
            "admin_username": "admin",
            "admin_password": "securepass"
        }),
    );
    let (status, body) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["ok"], json!(true));

    // needs_setup이 false가 되어야 함
    let req2 = get_request("/api/setup/status", "");
    let (s2, b2) = response_json(&app, req2).await;
    assert_eq!(s2, StatusCode::OK);
    assert_eq!(b2["needs_setup"], json!(false));
}

#[tokio::test]
async fn setup_complete_rejects_short_password() {
    let pool = setup_test_db().await;
    let app = build_test_app(pool);

    let req = json_request(
        Method::POST,
        "/api/setup/complete",
        json!({
            "school_name": "학교",
            "admin_name": "관리자",
            "admin_username": "admin",
            "admin_password": "short"   // 7자 → 8자 미만
        }),
    );
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn setup_complete_rejects_empty_school_name() {
    let pool = setup_test_db().await;
    let app = build_test_app(pool);

    let req = json_request(
        Method::POST,
        "/api/setup/complete",
        json!({
            "school_name": "   ",
            "admin_name": "관리자",
            "admin_username": "admin",
            "admin_password": "password123"
        }),
    );
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn setup_complete_rejects_duplicate_call() {
    let pool = setup_test_db().await;
    let app = build_test_app(pool);

    setup_admin(&app).await;

    // 두 번째 호출은 거부되어야 함
    let req = json_request(
        Method::POST,
        "/api/setup/complete",
        json!({
            "school_name": "다른학교",
            "admin_name": "다른관리자",
            "admin_username": "admin2",
            "admin_password": "password456"
        }),
    );
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}
