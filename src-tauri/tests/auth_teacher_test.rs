mod common;
use axum::http::StatusCode;
use serde_json::json;

#[tokio::test]
async fn login_success_returns_user_and_cookie() {
    let app = common::setup_app().await;
    app.do_setup().await;

    let (status, body, cookie) = app
        .post(
            "/api/auth/login/teacher",
            json!({ "username": "admin", "password": "password123" }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["user"]["username"], json!("admin"));
    assert_eq!(body["user"]["role"], json!("admin"));
    assert!(cookie.is_some(), "Expected cc_session cookie");
}

#[tokio::test]
async fn login_fail_unknown_username() {
    let app = common::setup_app().await;
    app.do_setup().await;

    let (status, body, _) = app
        .post(
            "/api/auth/login/teacher",
            json!({ "username": "nobody", "password": "password123" }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], json!("ERR_INVALID_CREDENTIALS"));
}

#[tokio::test]
async fn login_fail_wrong_password() {
    let app = common::setup_app().await;
    app.do_setup().await;

    let (status, body, _) = app
        .post(
            "/api/auth/login/teacher",
            json!({ "username": "admin", "password": "wrongpassword" }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], json!("ERR_INVALID_CREDENTIALS"));
}

#[tokio::test]
async fn me_returns_user_info() {
    let app = common::setup_app().await;
    app.do_setup().await;
    let cookie = app.login_teacher("admin", "password123").await;

    let (status, body) = app.get("/api/auth/me", Some(&cookie)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["username"], json!("admin"));
    assert_eq!(body["role"], json!("admin"));
}

#[tokio::test]
async fn me_without_cookie_returns_unauthorized() {
    let app = common::setup_app().await;
    app.do_setup().await;

    let (status, body) = app.get("/api/auth/me", None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"], json!("ERR_UNAUTHORIZED"));
}

#[tokio::test]
async fn me_after_logout_returns_unauthorized() {
    let app = common::setup_app().await;
    app.do_setup().await;
    let cookie = app.login_teacher("admin", "password123").await;

    let (status, _, _) = app.post("/api/auth/logout", json!({}), Some(&cookie)).await;
    assert_eq!(status, StatusCode::OK);

    let (status2, body2) = app.get("/api/auth/me", Some(&cookie)).await;
    assert_eq!(status2, StatusCode::UNAUTHORIZED);
    assert_eq!(body2["error"], json!("ERR_UNAUTHORIZED"));
}

#[tokio::test]
async fn update_name_success() {
    let app = common::setup_app().await;
    app.do_setup().await;
    let cookie = app.login_teacher("admin", "password123").await;

    let (status, body) = app
        .put(
            "/api/auth/me",
            json!({ "name": "New Name" }),
            Some(&cookie),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["ok"], json!(true));

    let (_, me) = app.get("/api/auth/me", Some(&cookie)).await;
    assert_eq!(me["name"], json!("New Name"));
}

#[tokio::test]
async fn update_name_empty_returns_error() {
    let app = common::setup_app().await;
    app.do_setup().await;
    let cookie = app.login_teacher("admin", "password123").await;

    let (status, body) = app
        .put("/api/auth/me", json!({ "name": "   " }), Some(&cookie))
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], json!("ERR_NAME_REQUIRED"));
}

#[tokio::test]
async fn change_password_success() {
    let app = common::setup_app().await;
    app.do_setup().await;
    let cookie = app.login_teacher("admin", "password123").await;

    let (status, body) = app
        .put(
            "/api/auth/me/password",
            json!({ "current_password": "password123", "new_password": "newpassword456" }),
            Some(&cookie),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["ok"], json!(true));

    // Can now login with new password
    let new_cookie = app.login_teacher("admin", "newpassword456").await;
    let (_, me) = app.get("/api/auth/me", Some(&new_cookie)).await;
    assert_eq!(me["username"], json!("admin"));
}

#[tokio::test]
async fn change_password_wrong_current() {
    let app = common::setup_app().await;
    app.do_setup().await;
    let cookie = app.login_teacher("admin", "password123").await;

    let (status, body) = app
        .put(
            "/api/auth/me/password",
            json!({ "current_password": "wrongpassword", "new_password": "newpassword456" }),
            Some(&cookie),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], json!("ERR_INVALID_CREDENTIALS"));
}

#[tokio::test]
async fn change_password_too_short() {
    let app = common::setup_app().await;
    app.do_setup().await;
    let cookie = app.login_teacher("admin", "password123").await;

    let (status, body) = app
        .put(
            "/api/auth/me/password",
            json!({ "current_password": "password123", "new_password": "short" }),
            Some(&cookie),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], json!("ERR_PASSWORD_TOO_SHORT"));
}

#[tokio::test]
async fn school_name_returns_name() {
    let app = common::setup_app().await;
    app.do_setup().await;

    let (status, body) = app.get("/api/auth/school-name", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["school_name"], json!("Test School"));
}
