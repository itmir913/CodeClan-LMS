mod common;
use axum::http::StatusCode;
use serde_json::json;

async fn setup_with_student(app: &common::TestApp) -> (String, i64, String) {
    app.do_setup().await;
    let admin_cookie = app.login_teacher("admin", "password123").await;
    let subject_id = app.create_subject(&admin_cookie, "Science").await;
    let class_id = app.create_class(&admin_cookie, "Class 1", subject_id).await;
    let student_id = app
        .add_student(&admin_cookie, class_id, "Kim Chulsoo", 1, 1, 1)
        .await;
    // username = "10101"
    (admin_cookie, student_id, "10101".to_string())
}

#[tokio::test]
async fn student_login_without_password_reset_fails() {
    let app = common::setup_app().await;
    let (_, _, username) = setup_with_student(&app).await;

    // Initial state: password_hash = '' → ERR_INVALID_CREDENTIALS
    let (status, body, _) = app.login_student_raw(&username, &username).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], json!("ERR_INVALID_CREDENTIALS"));
}

#[tokio::test]
async fn student_login_after_reset_succeeds_with_username_as_password() {
    let app = common::setup_app().await;
    let (admin_cookie, student_id, username) = setup_with_student(&app).await;
    app.reset_student_password(&admin_cookie, student_id).await;

    let (status, body, cookie) = app.login_student_raw(&username, &username).await;
    assert_eq!(status, StatusCode::OK, "Student login failed: {body}");
    assert_eq!(body["user"]["username"], json!(username));
    assert_eq!(body["user"]["password_reset_required"], json!(true));
    assert!(cookie.is_some(), "Expected cc_session cookie");
}

#[tokio::test]
async fn student_me_without_cookie_returns_unauthorized() {
    let app = common::setup_app().await;
    app.do_setup().await;

    let (status, body) = app.get("/api/auth/student/me", None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"], json!("ERR_UNAUTHORIZED"));
}

#[tokio::test]
async fn student_change_password_when_reset_required_skips_current() {
    let app = common::setup_app().await;
    let (admin_cookie, student_id, username) = setup_with_student(&app).await;
    app.reset_student_password(&admin_cookie, student_id).await;
    let student_cookie = app.login_student(&username, &username).await;

    // password_reset_required=true → no need for current_password
    let (status, body, _) = app
        .post(
            "/api/auth/student/change-password",
            json!({ "new_password": "newpassword456" }),
            Some(&student_cookie),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "Change password failed: {body}");
    assert_eq!(body["ok"], json!(true));
}

#[tokio::test]
async fn student_change_password_when_reset_not_required_needs_current() {
    let app = common::setup_app().await;
    let (admin_cookie, student_id, username) = setup_with_student(&app).await;
    app.reset_student_password(&admin_cookie, student_id).await;

    // First change: skip current (reset_required=true)
    let student_cookie = app.login_student(&username, &username).await;
    let (status, _, _) = app
        .post(
            "/api/auth/student/change-password",
            json!({ "new_password": "newpassword456" }),
            Some(&student_cookie),
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    // Login again with new password
    let student_cookie2 = app.login_student(&username, "newpassword456").await;

    // Second change: password_reset_required=false → current_password required
    let (status2, body2, _) = app
        .post(
            "/api/auth/student/change-password",
            json!({ "new_password": "anotherpass789" }),
            Some(&student_cookie2),
        )
        .await;
    assert_eq!(status2, StatusCode::BAD_REQUEST);
    assert_eq!(body2["error"], json!("ERR_CURRENT_PASSWORD_REQUIRED"));
}

#[tokio::test]
async fn student_password_reset_required_becomes_false_after_change() {
    let app = common::setup_app().await;
    let (admin_cookie, student_id, username) = setup_with_student(&app).await;
    app.reset_student_password(&admin_cookie, student_id).await;
    let student_cookie = app.login_student(&username, &username).await;

    app.post(
        "/api/auth/student/change-password",
        json!({ "new_password": "newpassword456" }),
        Some(&student_cookie),
    )
    .await;

    // Login again and verify password_reset_required=false
    let student_cookie2 = app.login_student(&username, "newpassword456").await;
    let (_, me) = app.get("/api/auth/student/me", Some(&student_cookie2)).await;
    assert_eq!(me["password_reset_required"], json!(false));
}

#[tokio::test]
async fn student_logout_clears_session() {
    let app = common::setup_app().await;
    let (admin_cookie, student_id, username) = setup_with_student(&app).await;
    app.reset_student_password(&admin_cookie, student_id).await;
    let student_cookie = app.login_student(&username, &username).await;

    let (status, _, _) = app
        .post("/api/auth/logout/student", json!({}), Some(&student_cookie))
        .await;
    assert_eq!(status, StatusCode::OK);

    let (status2, body2) = app
        .get("/api/auth/student/me", Some(&student_cookie))
        .await;
    assert_eq!(status2, StatusCode::UNAUTHORIZED);
    assert_eq!(body2["error"], json!("ERR_UNAUTHORIZED"));
}
