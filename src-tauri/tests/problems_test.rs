mod common;
use axum::http::StatusCode;
use serde_json::json;

async fn setup_with_auth(app: &common::TestApp) -> String {
    app.do_setup().await;
    app.login_teacher("admin", "password123").await
}

#[tokio::test]
async fn list_problems_without_auth_returns_401() {
    let app = common::setup_app().await;
    app.do_setup().await;

    let (status, body) = app.get("/api/problems", None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"], json!("ERR_UNAUTHORIZED"));
}

#[tokio::test]
async fn create_short_answer_success() {
    let app = common::setup_app().await;
    let cookie = setup_with_auth(&app).await;

    let (status, body, _) = app
        .post(
            "/api/problems",
            json!({
                "type": "short_answer",
                "title": "Q1",
                "answer": "42"
            }),
            Some(&cookie),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "Create problem failed: {body}");
    assert!(body["id"].as_i64().is_some());
}

#[tokio::test]
async fn create_short_answer_no_answer_returns_error() {
    let app = common::setup_app().await;
    let cookie = setup_with_auth(&app).await;

    let (status, body, _) = app
        .post(
            "/api/problems",
            json!({ "type": "short_answer", "title": "Q1" }),
            Some(&cookie),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], json!("ERR_PROBLEM_ANSWER_REQUIRED"));
}

#[tokio::test]
async fn create_multiple_choice_success() {
    let app = common::setup_app().await;
    let cookie = setup_with_auth(&app).await;

    let (status, body, _) = app
        .post(
            "/api/problems",
            json!({
                "type": "multiple_choice",
                "title": "Which is correct?",
                "choices": [
                    { "content": "Option A", "is_correct": true },
                    { "content": "Option B", "is_correct": false }
                ]
            }),
            Some(&cookie),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "Create MC failed: {body}");
    assert!(body["id"].as_i64().is_some());
}

#[tokio::test]
async fn create_multiple_choice_too_few_choices() {
    let app = common::setup_app().await;
    let cookie = setup_with_auth(&app).await;

    let (status, body, _) = app
        .post(
            "/api/problems",
            json!({
                "type": "multiple_choice",
                "title": "Q",
                "choices": [{ "content": "Only one", "is_correct": true }]
            }),
            Some(&cookie),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], json!("ERR_PROBLEM_CHOICES_MIN"));
}

#[tokio::test]
async fn create_multiple_choice_no_correct_answer() {
    let app = common::setup_app().await;
    let cookie = setup_with_auth(&app).await;

    let (status, body, _) = app
        .post(
            "/api/problems",
            json!({
                "type": "multiple_choice",
                "title": "Q",
                "choices": [
                    { "content": "A", "is_correct": false },
                    { "content": "B", "is_correct": false }
                ]
            }),
            Some(&cookie),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], json!("ERR_PROBLEM_ANSWER_REQUIRED"));
}

#[tokio::test]
async fn create_code_submit_success_and_files_created() {
    let app = common::setup_app().await;
    let cookie = setup_with_auth(&app).await;

    let (status, body, _) = app
        .post(
            "/api/problems",
            json!({
                "type": "code_submit",
                "title": "Sum Problem",
                "test_cases": [
                    { "input": "1 2", "expected_output": "3", "is_sample": true }
                ]
            }),
            Some(&cookie),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "Create code_submit failed: {body}");
    let problem_id = body["id"].as_i64().unwrap();

    // Verify files created
    let tc_dir = app.data_dir.join("problems").join(problem_id.to_string());
    assert!(
        tc_dir.join("1.in").exists(),
        "Test case input file should exist"
    );
    assert!(
        tc_dir.join("1.out").exists(),
        "Test case output file should exist"
    );
}

#[tokio::test]
async fn get_short_answer_problem_returns_type_fields() {
    let app = common::setup_app().await;
    let cookie = setup_with_auth(&app).await;

    let (_, body, _) = app
        .post(
            "/api/problems",
            json!({ "type": "short_answer", "title": "Q1", "answer": "hello", "case_sensitive": true }),
            Some(&cookie),
        )
        .await;
    let id = body["id"].as_i64().unwrap();

    let (status, detail) = app.get(&format!("/api/problems/{id}"), Some(&cookie)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(detail["type"], json!("short_answer"));
    assert_eq!(detail["answer"], json!("hello"));
    assert_eq!(detail["case_sensitive"], json!(true));
}

#[tokio::test]
async fn get_multiple_choice_problem_returns_choices() {
    let app = common::setup_app().await;
    let cookie = setup_with_auth(&app).await;

    let (_, body, _) = app
        .post(
            "/api/problems",
            json!({
                "type": "multiple_choice",
                "title": "Q",
                "choices": [
                    { "content": "A", "is_correct": true },
                    { "content": "B", "is_correct": false }
                ]
            }),
            Some(&cookie),
        )
        .await;
    let id = body["id"].as_i64().unwrap();

    let (status, detail) = app.get(&format!("/api/problems/{id}"), Some(&cookie)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(detail["type"], json!("multiple_choice"));
    let choices = detail["choices"].as_array().unwrap();
    assert_eq!(choices.len(), 2);
    assert!(choices.iter().any(|c| c["is_correct"] == json!(true)));
}

#[tokio::test]
async fn get_code_submit_problem_returns_test_cases() {
    let app = common::setup_app().await;
    let cookie = setup_with_auth(&app).await;

    let (_, body, _) = app
        .post(
            "/api/problems",
            json!({
                "type": "code_submit",
                "title": "Sum",
                "test_cases": [
                    { "input": "1 2", "expected_output": "3", "is_sample": true, "explanation": "basic" }
                ]
            }),
            Some(&cookie),
        )
        .await;
    let id = body["id"].as_i64().unwrap();

    let (status, detail) = app.get(&format!("/api/problems/{id}"), Some(&cookie)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(detail["type"], json!("code_submit"));
    let tcs = detail["test_cases"].as_array().unwrap();
    assert_eq!(tcs.len(), 1);
    assert_eq!(tcs[0]["input"], json!("1 2"));
    assert_eq!(tcs[0]["expected_output"], json!("3"));
    assert_eq!(tcs[0]["is_sample"], json!(true));
}

#[tokio::test]
async fn update_problem_success() {
    let app = common::setup_app().await;
    let cookie = setup_with_auth(&app).await;

    let (_, body, _) = app
        .post(
            "/api/problems",
            json!({ "type": "short_answer", "title": "Old Title", "answer": "old" }),
            Some(&cookie),
        )
        .await;
    let id = body["id"].as_i64().unwrap();

    let (status, upd_body) = app
        .put(
            &format!("/api/problems/{id}"),
            json!({ "type": "short_answer", "title": "New Title", "answer": "new" }),
            Some(&cookie),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "Update failed: {upd_body}");

    let (_, detail) = app.get(&format!("/api/problems/{id}"), Some(&cookie)).await;
    assert_eq!(detail["title"], json!("New Title"));
    assert_eq!(detail["answer"], json!("new"));
}

#[tokio::test]
async fn update_problem_type_mismatch_returns_error() {
    let app = common::setup_app().await;
    let cookie = setup_with_auth(&app).await;

    let (_, body, _) = app
        .post(
            "/api/problems",
            json!({ "type": "short_answer", "title": "Q1", "answer": "42" }),
            Some(&cookie),
        )
        .await;
    let id = body["id"].as_i64().unwrap();

    let (status, body) = app
        .put(
            &format!("/api/problems/{id}"),
            json!({
                "type": "multiple_choice",
                "title": "Q1",
                "choices": [
                    { "content": "A", "is_correct": true },
                    { "content": "B", "is_correct": false }
                ]
            }),
            Some(&cookie),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], json!("ERR_PROBLEM_TYPE_MISMATCH"));
}

#[tokio::test]
async fn delete_problem_success() {
    let app = common::setup_app().await;
    let cookie = setup_with_auth(&app).await;

    let (_, body, _) = app
        .post(
            "/api/problems",
            json!({ "type": "short_answer", "title": "Q1", "answer": "42" }),
            Some(&cookie),
        )
        .await;
    let id = body["id"].as_i64().unwrap();

    let (status, del_body) = app
        .delete(&format!("/api/problems/{id}"), Some(&cookie))
        .await;
    assert_eq!(status, StatusCode::OK, "Delete failed: {del_body}");

    let (status2, body2) = app.get(&format!("/api/problems/{id}"), Some(&cookie)).await;
    assert_eq!(status2, StatusCode::NOT_FOUND);
    assert_eq!(body2["error"], json!("ERR_NOT_FOUND"));
}

#[tokio::test]
async fn get_nonexistent_problem_returns_404() {
    let app = common::setup_app().await;
    let cookie = setup_with_auth(&app).await;

    let (status, body) = app.get("/api/problems/9999", Some(&cookie)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"], json!("ERR_NOT_FOUND"));
}

#[tokio::test]
async fn delete_code_submit_removes_tc_directory() {
    let app = common::setup_app().await;
    let cookie = setup_with_auth(&app).await;

    let (_, body, _) = app
        .post(
            "/api/problems",
            json!({
                "type": "code_submit",
                "title": "Sum",
                "test_cases": [{ "input": "1 2", "expected_output": "3" }]
            }),
            Some(&cookie),
        )
        .await;
    let id = body["id"].as_i64().unwrap();
    let tc_dir = app.data_dir.join("problems").join(id.to_string());
    assert!(tc_dir.exists(), "TC directory should exist before delete");

    let (status, _) = app
        .delete(&format!("/api/problems/{id}"), Some(&cookie))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert!(!tc_dir.exists(), "TC directory should be removed after delete");
}
