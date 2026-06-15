mod common;
use axum::http::StatusCode;
use serde_json::json;

#[tokio::test]
async fn get_status_initial_needs_setup() {
    let app = common::setup_app().await;
    let (status, body) = app.get("/api/setup/status", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["needs_setup"], json!(true));
}

#[tokio::test]
async fn complete_setup_success() {
    let app = common::setup_app().await;
    let (status, body, _) = app
        .post(
            "/api/setup/complete",
            json!({
                "school_name": "Test School",
                "admin_name": "Admin",
                "admin_username": "admin",
                "admin_password": "password123",
                "locale": "ko"
            }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["ok"], json!(true));
}

#[tokio::test]
async fn complete_setup_twice_returns_error() {
    let app = common::setup_app().await;
    app.do_setup().await;

    let (status, body, _) = app
        .post(
            "/api/setup/complete",
            json!({
                "school_name": "Another School",
                "admin_name": "Admin2",
                "admin_username": "admin2",
                "admin_password": "password123",
                "locale": "ko"
            }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], json!("ERR_SETUP_ALREADY_COMPLETE"));
}

#[tokio::test]
async fn complete_setup_empty_school_name() {
    let app = common::setup_app().await;
    let (status, body, _) = app
        .post(
            "/api/setup/complete",
            json!({
                "school_name": "   ",
                "admin_name": "Admin",
                "admin_username": "admin",
                "admin_password": "password123",
                "locale": "ko"
            }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], json!("ERR_SCHOOL_NAME_REQUIRED"));
}

#[tokio::test]
async fn complete_setup_password_too_short() {
    let app = common::setup_app().await;
    let (status, body, _) = app
        .post(
            "/api/setup/complete",
            json!({
                "school_name": "Test School",
                "admin_name": "Admin",
                "admin_username": "admin",
                "admin_password": "1234567",
                "locale": "ko"
            }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], json!("ERR_PASSWORD_TOO_SHORT"));
}

#[tokio::test]
async fn complete_setup_invalid_locale_defaults_to_en() {
    let app = common::setup_app().await;
    let (status, body, _) = app
        .post(
            "/api/setup/complete",
            json!({
                "school_name": "Test School",
                "admin_name": "Admin",
                "admin_username": "admin",
                "admin_password": "password123",
                "locale": "!!!invalid!!!"
            }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "Expected setup to succeed: {body}");

    let (status2, body2) = app.get("/api/setup/status", None).await;
    assert_eq!(status2, StatusCode::OK);
    assert_eq!(body2["locale"], json!("en"));
}

#[tokio::test]
async fn get_status_after_setup_needs_setup_false() {
    let app = common::setup_app().await;
    app.do_setup().await;

    let (status, body) = app.get("/api/setup/status", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["needs_setup"], json!(false));
}
