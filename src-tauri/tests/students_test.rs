mod common;
use axum::http::StatusCode;
use serde_json::json;

async fn setup_with_class(app: &common::TestApp) -> (String, i64) {
    app.do_setup().await;
    let admin_cookie = app.login_teacher("admin", "password123").await;
    let subject_id = app.create_subject(&admin_cookie, "Science").await;
    let class_id = app.create_class(&admin_cookie, "Class 1", subject_id).await;
    (admin_cookie, class_id)
}

#[tokio::test]
async fn list_students_without_auth_returns_401() {
    let app = common::setup_app().await;
    let (_, class_id) = setup_with_class(&app).await;

    let (status, body) = app
        .get(&format!("/api/classes/{class_id}/students"), None)
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"], json!("ERR_UNAUTHORIZED"));
}

#[tokio::test]
async fn add_student_success_and_username_format() {
    let app = common::setup_app().await;
    let (cookie, class_id) = setup_with_class(&app).await;

    let (status, body, _) = app
        .post(
            &format!("/api/classes/{class_id}/students"),
            json!({ "name": "Kim Chulsoo", "grade": 1, "class_no": 2, "number": 3 }),
            Some(&cookie),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "Add student failed: {body}");
    let student_id = body["id"].as_i64().unwrap();

    // Verify username = "10203" (1 + 02 + 03)
    let (_, students) = app
        .get(&format!("/api/classes/{class_id}/students"), Some(&cookie))
        .await;
    let list = students.as_array().unwrap();
    let student = list
        .iter()
        .find(|s| s["id"].as_i64() == Some(student_id))
        .expect("Student not found in list");
    assert_eq!(student["username"], json!("10203"));
}

#[tokio::test]
async fn add_student_grade_out_of_range() {
    let app = common::setup_app().await;
    let (cookie, class_id) = setup_with_class(&app).await;

    let (status, body, _) = app
        .post(
            &format!("/api/classes/{class_id}/students"),
            json!({ "name": "Kim", "grade": 7, "class_no": 1, "number": 1 }),
            Some(&cookie),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], json!("ERR_STUDENT_GRADE_INVALID"));
}

#[tokio::test]
async fn add_student_class_no_out_of_range() {
    let app = common::setup_app().await;
    let (cookie, class_id) = setup_with_class(&app).await;

    let (status, body, _) = app
        .post(
            &format!("/api/classes/{class_id}/students"),
            json!({ "name": "Kim", "grade": 1, "class_no": 100, "number": 1 }),
            Some(&cookie),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], json!("ERR_STUDENT_CLASS_NO_INVALID"));
}

#[tokio::test]
async fn add_student_number_out_of_range() {
    let app = common::setup_app().await;
    let (cookie, class_id) = setup_with_class(&app).await;

    let (status, body, _) = app
        .post(
            &format!("/api/classes/{class_id}/students"),
            json!({ "name": "Kim", "grade": 1, "class_no": 1, "number": 100 }),
            Some(&cookie),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], json!("ERR_STUDENT_NUMBER_INVALID"));
}

#[tokio::test]
async fn add_student_duplicate_returns_error() {
    let app = common::setup_app().await;
    let (cookie, class_id) = setup_with_class(&app).await;

    app.add_student(&cookie, class_id, "Kim", 1, 1, 1).await;

    let (status, body, _) = app
        .post(
            &format!("/api/classes/{class_id}/students"),
            json!({ "name": "Lee", "grade": 1, "class_no": 1, "number": 1 }),
            Some(&cookie),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], json!("ERR_STUDENT_ALREADY_EXISTS"));
}

#[tokio::test]
async fn bulk_add_students_inserted_and_skipped_counts() {
    let app = common::setup_app().await;
    let (cookie, class_id) = setup_with_class(&app).await;

    // Add one student first
    app.add_student(&cookie, class_id, "Kim", 1, 1, 1).await;

    // bulk_add: 1 new + 1 duplicate
    let (status, body, _) = app
        .post(
            &format!("/api/classes/{class_id}/students/bulk"),
            json!([
                { "name": "Lee", "grade": 1, "class_no": 1, "number": 2 },
                { "name": "Kim", "grade": 1, "class_no": 1, "number": 1 }  // duplicate
            ]),
            Some(&cookie),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "Bulk add failed: {body}");
    assert_eq!(body["inserted"], json!(1));
    assert_eq!(body["skipped"], json!(1));
}

#[tokio::test]
async fn import_students_success() {
    let app = common::setup_app().await;
    let (cookie, class_id) = setup_with_class(&app).await;

    let (status, body, _) = app
        .post(
            &format!("/api/classes/{class_id}/students/import"),
            json!([
                { "name": "Kim", "grade": 1, "class_no": 1, "number": 1 },
                { "name": "Lee", "grade": 1, "class_no": 1, "number": 2 }
            ]),
            Some(&cookie),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "Import failed: {body}");
    assert_eq!(body["imported"], json!(2));
}

#[tokio::test]
async fn import_students_duplicate_rollsback_all() {
    let app = common::setup_app().await;
    let (cookie, class_id) = setup_with_class(&app).await;

    // Pre-existing student
    app.add_student(&cookie, class_id, "Kim", 1, 1, 1).await;

    let (status, body, _) = app
        .post(
            &format!("/api/classes/{class_id}/students/import"),
            json!([
                { "name": "New", "grade": 1, "class_no": 1, "number": 99 },  // new
                { "name": "Kim", "grade": 1, "class_no": 1, "number": 1 }    // duplicate
            ]),
            Some(&cookie),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], json!("ERR_IMPORT_DUPLICATE"));

    // Rollback: student 1/1/99 should not exist
    let (_, students) = app
        .get(&format!("/api/classes/{class_id}/students"), Some(&cookie))
        .await;
    let list = students.as_array().unwrap();
    assert_eq!(list.len(), 1, "Rollback failed: expected only the original student");
}

#[tokio::test]
async fn delete_student_success() {
    let app = common::setup_app().await;
    let (cookie, class_id) = setup_with_class(&app).await;
    let student_id = app.add_student(&cookie, class_id, "Kim", 1, 1, 1).await;

    let (status, body) = app
        .delete(&format!("/api/students/{student_id}"), Some(&cookie))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["ok"], json!(true));
}

#[tokio::test]
async fn delete_nonexistent_student_returns_404() {
    let app = common::setup_app().await;
    let (cookie, _) = setup_with_class(&app).await;

    let (status, body) = app
        .delete("/api/students/9999", Some(&cookie))
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"], json!("ERR_NOT_FOUND"));
}

#[tokio::test]
async fn delete_other_teacher_student_returns_403() {
    let app = common::setup_app().await;
    app.do_setup().await;
    let admin_cookie = app.login_teacher("admin", "password123").await;
    let subject_id = app.create_subject(&admin_cookie, "Math").await;

    app.create_teacher(&admin_cookie, "teacher1", "Teacher One", "teacherpw1")
        .await;
    let t1_cookie = app.login_teacher("teacher1", "teacherpw1").await;
    let class_id = app.create_class(&t1_cookie, "Class A", subject_id).await;
    let student_id = app
        .add_student(&t1_cookie, class_id, "Kim", 1, 1, 1)
        .await;

    app.create_teacher(&admin_cookie, "teacher2", "Teacher Two", "teacherpw2")
        .await;
    let t2_cookie = app.login_teacher("teacher2", "teacherpw2").await;

    let (status, body) = app
        .delete(&format!("/api/students/{student_id}"), Some(&t2_cookie))
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"], json!("ERR_FORBIDDEN"));
}

#[tokio::test]
async fn reset_password_marks_reset_required_and_allows_login() {
    let app = common::setup_app().await;
    let (cookie, class_id) = setup_with_class(&app).await;
    let student_id = app.add_student(&cookie, class_id, "Kim", 1, 1, 1).await;

    // username = "10101"
    app.reset_student_password(&cookie, student_id).await;

    // Can login with username as password
    let (status, body, s_cookie) = app
        .post(
            "/api/auth/login/student",
            json!({ "username": "10101", "password": "10101" }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "Student login after reset failed: {body}");
    assert_eq!(body["user"]["password_reset_required"], json!(true));
    assert!(s_cookie.is_some());
}
