mod common;
use axum::http::StatusCode;
use serde_json::json;

#[tokio::test]
async fn list_classes_without_auth_returns_401() {
    let app = common::setup_app().await;
    app.do_setup().await;

    let (status, body) = app.get("/api/classes", None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"], json!("ERR_UNAUTHORIZED"));
}

#[tokio::test]
async fn create_class_success() {
    let app = common::setup_app().await;
    app.do_setup().await;
    let cookie = app.login_teacher("admin", "password123").await;
    let subject_id = app.create_subject(&cookie, "Math").await;

    let (status, body, _) = app
        .post(
            "/api/classes",
            json!({ "name": "1반", "subject_id": subject_id }),
            Some(&cookie),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["id"].as_i64().is_some());
}

#[tokio::test]
async fn create_class_empty_name_returns_error() {
    let app = common::setup_app().await;
    app.do_setup().await;
    let cookie = app.login_teacher("admin", "password123").await;
    let subject_id = app.create_subject(&cookie, "Math").await;

    let (status, body, _) = app
        .post(
            "/api/classes",
            json!({ "name": "   ", "subject_id": subject_id }),
            Some(&cookie),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], json!("ERR_CLASS_NAME_REQUIRED"));
}

#[tokio::test]
async fn create_class_invalid_subject_returns_error() {
    let app = common::setup_app().await;
    app.do_setup().await;
    let cookie = app.login_teacher("admin", "password123").await;

    let (status, body, _) = app
        .post(
            "/api/classes",
            json!({ "name": "1반", "subject_id": 9999 }),
            Some(&cookie),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], json!("ERR_SUBJECT_NOT_FOUND"));
}

#[tokio::test]
async fn list_classes_teacher_only_sees_own() {
    let app = common::setup_app().await;
    app.do_setup().await;
    let admin_cookie = app.login_teacher("admin", "password123").await;
    let subject_id = app.create_subject(&admin_cookie, "Math").await;

    // Create teacher1
    let t1_id = app
        .create_teacher(&admin_cookie, "teacher1", "Teacher One", "teacherpw1")
        .await;
    let _ = t1_id;
    let t1_cookie = app.login_teacher("teacher1", "teacherpw1").await;

    // Create teacher2
    app.create_teacher(&admin_cookie, "teacher2", "Teacher Two", "teacherpw2")
        .await;
    let t2_cookie = app.login_teacher("teacher2", "teacherpw2").await;

    // Teacher1 creates a class
    app.create_class(&t1_cookie, "Class A", subject_id).await;

    // Teacher2 should not see teacher1's class
    let (status, body) = app.get("/api/classes", Some(&t2_cookie)).await;
    assert_eq!(status, StatusCode::OK);
    let classes = body.as_array().unwrap();
    assert!(classes.is_empty(), "Teacher2 should see 0 classes, got: {classes:?}");

    // Teacher1 should see their own class
    let (_, body2) = app.get("/api/classes", Some(&t1_cookie)).await;
    let classes2 = body2.as_array().unwrap();
    assert_eq!(classes2.len(), 1);
}

#[tokio::test]
async fn admin_sees_all_classes() {
    let app = common::setup_app().await;
    app.do_setup().await;
    let admin_cookie = app.login_teacher("admin", "password123").await;
    let subject_id = app.create_subject(&admin_cookie, "Math").await;

    app.create_teacher(&admin_cookie, "teacher1", "Teacher One", "teacherpw1")
        .await;
    let t1_cookie = app.login_teacher("teacher1", "teacherpw1").await;
    app.create_class(&t1_cookie, "Class A", subject_id).await;
    app.create_class(&admin_cookie, "Class B", subject_id).await;

    let (status, body) = app.get("/api/classes", Some(&admin_cookie)).await;
    assert_eq!(status, StatusCode::OK);
    let classes = body.as_array().unwrap();
    assert_eq!(classes.len(), 2);
}

#[tokio::test]
async fn get_other_teacher_class_returns_403() {
    let app = common::setup_app().await;
    app.do_setup().await;
    let admin_cookie = app.login_teacher("admin", "password123").await;
    let subject_id = app.create_subject(&admin_cookie, "Math").await;

    app.create_teacher(&admin_cookie, "teacher1", "Teacher One", "teacherpw1")
        .await;
    let t1_cookie = app.login_teacher("teacher1", "teacherpw1").await;
    let class_id = app.create_class(&t1_cookie, "Class A", subject_id).await;

    app.create_teacher(&admin_cookie, "teacher2", "Teacher Two", "teacherpw2")
        .await;
    let t2_cookie = app.login_teacher("teacher2", "teacherpw2").await;

    let (status, body) = app
        .get(&format!("/api/classes/{class_id}"), Some(&t2_cookie))
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"], json!("ERR_FORBIDDEN"));
}

#[tokio::test]
async fn update_other_teacher_class_returns_403() {
    let app = common::setup_app().await;
    app.do_setup().await;
    let admin_cookie = app.login_teacher("admin", "password123").await;
    let subject_id = app.create_subject(&admin_cookie, "Math").await;

    app.create_teacher(&admin_cookie, "teacher1", "Teacher One", "teacherpw1")
        .await;
    let t1_cookie = app.login_teacher("teacher1", "teacherpw1").await;
    let class_id = app.create_class(&t1_cookie, "Class A", subject_id).await;

    app.create_teacher(&admin_cookie, "teacher2", "Teacher Two", "teacherpw2")
        .await;
    let t2_cookie = app.login_teacher("teacher2", "teacherpw2").await;

    let (status, body) = app
        .put(
            &format!("/api/classes/{class_id}"),
            json!({ "name": "Renamed", "subject_id": subject_id }),
            Some(&t2_cookie),
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"], json!("ERR_FORBIDDEN"));
}

#[tokio::test]
async fn delete_other_teacher_class_returns_403() {
    let app = common::setup_app().await;
    app.do_setup().await;
    let admin_cookie = app.login_teacher("admin", "password123").await;
    let subject_id = app.create_subject(&admin_cookie, "Math").await;

    app.create_teacher(&admin_cookie, "teacher1", "Teacher One", "teacherpw1")
        .await;
    let t1_cookie = app.login_teacher("teacher1", "teacherpw1").await;
    let class_id = app.create_class(&t1_cookie, "Class A", subject_id).await;

    app.create_teacher(&admin_cookie, "teacher2", "Teacher Two", "teacherpw2")
        .await;
    let t2_cookie = app.login_teacher("teacher2", "teacherpw2").await;

    let (status, body) = app
        .delete(&format!("/api/classes/{class_id}"), Some(&t2_cookie))
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"], json!("ERR_FORBIDDEN"));
}

#[tokio::test]
async fn admin_can_update_any_class() {
    let app = common::setup_app().await;
    app.do_setup().await;
    let admin_cookie = app.login_teacher("admin", "password123").await;
    let subject_id = app.create_subject(&admin_cookie, "Math").await;

    app.create_teacher(&admin_cookie, "teacher1", "Teacher One", "teacherpw1")
        .await;
    let t1_cookie = app.login_teacher("teacher1", "teacherpw1").await;
    let class_id = app.create_class(&t1_cookie, "Class A", subject_id).await;

    let (status, body) = app
        .put(
            &format!("/api/classes/{class_id}"),
            json!({ "name": "Updated by Admin", "subject_id": subject_id }),
            Some(&admin_cookie),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["ok"], json!(true));
}

#[tokio::test]
async fn admin_can_delete_any_class() {
    let app = common::setup_app().await;
    app.do_setup().await;
    let admin_cookie = app.login_teacher("admin", "password123").await;
    let subject_id = app.create_subject(&admin_cookie, "Math").await;

    app.create_teacher(&admin_cookie, "teacher1", "Teacher One", "teacherpw1")
        .await;
    let t1_cookie = app.login_teacher("teacher1", "teacherpw1").await;
    let class_id = app.create_class(&t1_cookie, "Class A", subject_id).await;

    let (status, body) = app
        .delete(&format!("/api/classes/{class_id}"), Some(&admin_cookie))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["ok"], json!(true));
}

#[tokio::test]
async fn get_nonexistent_class_returns_404() {
    let app = common::setup_app().await;
    app.do_setup().await;
    let admin_cookie = app.login_teacher("admin", "password123").await;

    let (status, body) = app
        .get("/api/classes/9999", Some(&admin_cookie))
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"], json!("ERR_NOT_FOUND"));
}

#[tokio::test]
async fn get_deleted_class_returns_404() {
    let app = common::setup_app().await;
    app.do_setup().await;
    let admin_cookie = app.login_teacher("admin", "password123").await;
    let subject_id = app.create_subject(&admin_cookie, "Math").await;
    let class_id = app.create_class(&admin_cookie, "Class A", subject_id).await;

    app.delete(&format!("/api/classes/{class_id}"), Some(&admin_cookie))
        .await;

    let (status, body) = app
        .get(&format!("/api/classes/{class_id}"), Some(&admin_cookie))
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"], json!("ERR_NOT_FOUND"));
}
