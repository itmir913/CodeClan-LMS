mod common;
use axum::http::StatusCode;
use serde_json::json;

#[tokio::test]
async fn list_teachers_as_non_admin_returns_403() {
    let app = common::setup_app().await;
    app.do_setup().await;
    let admin_cookie = app.login_teacher("admin", "password123").await;
    app.create_teacher(&admin_cookie, "teacher1", "Teacher One", "teacherpw1")
        .await;
    let t_cookie = app.login_teacher("teacher1", "teacherpw1").await;

    let (status, body) = app.get("/api/admin/teachers", Some(&t_cookie)).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"], json!("ERR_FORBIDDEN"));
}

#[tokio::test]
async fn list_teachers_as_admin_returns_list() {
    let app = common::setup_app().await;
    app.do_setup().await;
    let admin_cookie = app.login_teacher("admin", "password123").await;

    let (status, body) = app.get("/api/admin/teachers", Some(&admin_cookie)).await;
    assert_eq!(status, StatusCode::OK);
    let list = body.as_array().unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["username"], json!("admin"));
}

#[tokio::test]
async fn create_teacher_success() {
    let app = common::setup_app().await;
    app.do_setup().await;
    let admin_cookie = app.login_teacher("admin", "password123").await;

    let (status, body, _) = app
        .post(
            "/api/admin/teachers",
            json!({ "username": "new_teacher", "name": "New Teacher", "password": "password123" }),
            Some(&admin_cookie),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["id"].as_i64().is_some());
}

#[tokio::test]
async fn create_teacher_duplicate_username_returns_error() {
    let app = common::setup_app().await;
    app.do_setup().await;
    let admin_cookie = app.login_teacher("admin", "password123").await;

    app.create_teacher(&admin_cookie, "teacher1", "Teacher One", "teacherpw1")
        .await;

    let (status, body, _) = app
        .post(
            "/api/admin/teachers",
            json!({ "username": "teacher1", "name": "Another", "password": "password123" }),
            Some(&admin_cookie),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], json!("ERR_USERNAME_TAKEN"));
}

#[tokio::test]
async fn delete_teacher_success() {
    let app = common::setup_app().await;
    app.do_setup().await;
    let admin_cookie = app.login_teacher("admin", "password123").await;
    let t_id = app
        .create_teacher(&admin_cookie, "teacher1", "Teacher One", "teacherpw1")
        .await;

    let (status, body) = app
        .delete(&format!("/api/admin/teachers/{t_id}"), Some(&admin_cookie))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["ok"], json!(true));
}

#[tokio::test]
async fn delete_self_returns_error() {
    let app = common::setup_app().await;
    app.do_setup().await;
    let admin_cookie = app.login_teacher("admin", "password123").await;
    let (_, me) = app.get("/api/auth/me", Some(&admin_cookie)).await;
    let admin_id = me["id"].as_i64().unwrap();

    let (status, body) = app
        .delete(
            &format!("/api/admin/teachers/{admin_id}"),
            Some(&admin_cookie),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], json!("ERR_CANNOT_DELETE_SELF"));
}

#[tokio::test]
async fn delete_last_admin_returns_error() {
    let app = common::setup_app().await;
    app.do_setup().await;
    let admin_cookie = app.login_teacher("admin", "password123").await;

    // Create a second teacher (non-admin) and try to delete the admin through another admin
    // Since we can't delete self, create another admin first, then have the second admin delete first
    let second_admin_id = app
        .create_teacher(&admin_cookie, "admin2", "Admin Two", "password123")
        .await;

    // Promote second to admin
    let (status, _) = app
        .put(
            &format!("/api/admin/teachers/{second_admin_id}"),
            json!({ "role": "admin" }),
            Some(&admin_cookie),
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    let admin2_cookie = app.login_teacher("admin2", "password123").await;
    let (_, me) = app.get("/api/auth/me", Some(&admin_cookie)).await;
    let admin1_id = me["id"].as_i64().unwrap();

    // admin2 deletes admin1 (succeeds - 2 admins exist)
    let (status, _) = app
        .delete(
            &format!("/api/admin/teachers/{admin1_id}"),
            Some(&admin2_cookie),
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    // Now only admin2 remains; trying to delete admin2 via another admin is impossible,
    // but we can test demoting last admin to teacher
    let (status2, body2) = app
        .put(
            &format!("/api/admin/teachers/{second_admin_id}"),
            json!({ "role": "teacher" }),
            Some(&admin2_cookie),
        )
        .await;
    assert_eq!(status2, StatusCode::BAD_REQUEST);
    assert_eq!(body2["error"], json!("ERR_LAST_ADMIN"));
}

#[tokio::test]
async fn demote_last_admin_role_returns_error() {
    let app = common::setup_app().await;
    app.do_setup().await;
    let admin_cookie = app.login_teacher("admin", "password123").await;
    let (_, me) = app.get("/api/auth/me", Some(&admin_cookie)).await;
    let admin_id = me["id"].as_i64().unwrap();

    let teacher_id = app
        .create_teacher(&admin_cookie, "teacher1", "Teacher One", "teacherpw1")
        .await;
    let _ = teacher_id;

    // There's only 1 admin; try to demote it
    let (status, body) = app
        .put(
            &format!("/api/admin/teachers/{admin_id}"),
            json!({ "role": "teacher" }),
            Some(&admin_cookie),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], json!("ERR_LAST_ADMIN"));
}

#[tokio::test]
async fn second_admin_can_be_demoted() {
    let app = common::setup_app().await;
    app.do_setup().await;
    let admin_cookie = app.login_teacher("admin", "password123").await;
    let second_id = app
        .create_teacher(&admin_cookie, "admin2", "Admin Two", "password123")
        .await;

    // Promote to admin
    let (s, _) = app
        .put(
            &format!("/api/admin/teachers/{second_id}"),
            json!({ "role": "admin" }),
            Some(&admin_cookie),
        )
        .await;
    assert_eq!(s, StatusCode::OK);

    // Now demote second admin (2 admins → ok)
    let (status, body) = app
        .put(
            &format!("/api/admin/teachers/{second_id}"),
            json!({ "role": "teacher" }),
            Some(&admin_cookie),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["ok"], json!(true));
}

#[tokio::test]
async fn create_subject_success() {
    let app = common::setup_app().await;
    app.do_setup().await;
    let admin_cookie = app.login_teacher("admin", "password123").await;

    let (status, body, _) = app
        .post(
            "/api/admin/subjects",
            json!({ "name": "Mathematics" }),
            Some(&admin_cookie),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["id"].as_i64().is_some());
}

#[tokio::test]
async fn create_subject_duplicate_name_returns_error() {
    let app = common::setup_app().await;
    app.do_setup().await;
    let admin_cookie = app.login_teacher("admin", "password123").await;
    app.create_subject(&admin_cookie, "Math").await;

    let (status, body, _) = app
        .post(
            "/api/admin/subjects",
            json!({ "name": "Math" }),
            Some(&admin_cookie),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], json!("ERR_SUBJECT_NAME_TAKEN"));
}

#[tokio::test]
async fn delete_subject_success() {
    let app = common::setup_app().await;
    app.do_setup().await;
    let admin_cookie = app.login_teacher("admin", "password123").await;
    let subject_id = app.create_subject(&admin_cookie, "Math").await;

    let (status, body) = app
        .delete(
            &format!("/api/admin/subjects/{subject_id}"),
            Some(&admin_cookie),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["ok"], json!(true));
}

#[tokio::test]
async fn delete_subject_in_use_returns_error() {
    let app = common::setup_app().await;
    app.do_setup().await;
    let admin_cookie = app.login_teacher("admin", "password123").await;
    let subject_id = app.create_subject(&admin_cookie, "Math").await;
    app.create_class(&admin_cookie, "1반", subject_id).await;

    let (status, body) = app
        .delete(
            &format!("/api/admin/subjects/{subject_id}"),
            Some(&admin_cookie),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], json!("ERR_SUBJECT_IN_USE"));
}

#[tokio::test]
async fn import_teachers_success() {
    let app = common::setup_app().await;
    app.do_setup().await;
    let admin_cookie = app.login_teacher("admin", "password123").await;

    let (status, body, _) = app
        .post(
            "/api/admin/teachers/import",
            json!([
                { "username": "t1", "name": "Teacher 1", "password": "password123" },
                { "username": "t2", "name": "Teacher 2", "password": "password456" }
            ]),
            Some(&admin_cookie),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "Import failed: {body}");
    assert_eq!(body["imported"], json!(2));
}

#[tokio::test]
async fn import_teachers_short_password_returns_error() {
    let app = common::setup_app().await;
    app.do_setup().await;
    let admin_cookie = app.login_teacher("admin", "password123").await;

    let (status, body, _) = app
        .post(
            "/api/admin/teachers/import",
            json!([
                { "username": "t1", "name": "Teacher 1", "password": "short" }
            ]),
            Some(&admin_cookie),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], json!("ERR_IMPORT_PASSWORD_TOO_SHORT"));
}

#[tokio::test]
async fn import_subjects_success() {
    let app = common::setup_app().await;
    app.do_setup().await;
    let admin_cookie = app.login_teacher("admin", "password123").await;

    let (status, body, _) = app
        .post(
            "/api/admin/subjects/import",
            json!([{ "name": "Math" }, { "name": "Science" }]),
            Some(&admin_cookie),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["imported"], json!(2));
}

#[tokio::test]
async fn import_subjects_duplicate_returns_error() {
    let app = common::setup_app().await;
    app.do_setup().await;
    let admin_cookie = app.login_teacher("admin", "password123").await;
    app.create_subject(&admin_cookie, "Math").await;

    let (status, body, _) = app
        .post(
            "/api/admin/subjects/import",
            json!([{ "name": "Science" }, { "name": "Math" }]),
            Some(&admin_cookie),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], json!("ERR_IMPORT_DUPLICATE"));

    // Rollback: "Science" should not exist
    let (_, subjects) = app.get("/api/subjects", Some(&admin_cookie)).await;
    let names: Vec<&str> = subjects
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|s| s["name"].as_str())
        .collect();
    assert!(!names.contains(&"Science"), "Rollback failed: Science should not exist");
}
