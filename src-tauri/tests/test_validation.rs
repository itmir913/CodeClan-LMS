/// 입력값 검증 엣지 케이스 — 모든 모듈의 빈 값·범위 초과·잘못된 타입 검증
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

// ─── setup 검증 ────────────────────────────────────────────

#[tokio::test]
async fn setup_rejects_empty_admin_name() {
    let pool = setup_test_db().await;
    let app = build_test_app(pool);

    let req = json_request(
        Method::POST,
        "/api/setup/complete",
        json!({
            "school_name": "학교",
            "admin_name": "   ",
            "admin_username": "admin",
            "admin_password": "password123"
        }),
    );
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn setup_rejects_empty_admin_username() {
    let pool = setup_test_db().await;
    let app = build_test_app(pool);

    let req = json_request(
        Method::POST,
        "/api/setup/complete",
        json!({
            "school_name": "학교",
            "admin_name": "관리자",
            "admin_username": "   ",
            "admin_password": "password123"
        }),
    );
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// ─── division 검증 ─────────────────────────────────────────

#[tokio::test]
async fn create_division_empty_name_returns_400() {
    let (app, cookie) = app_with_admin().await;

    let req = authed_request(
        Method::POST,
        "/api/divisions",
        &cookie,
        json!({ "name": "   " }),
    );
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn update_nonexistent_division_returns_404() {
    let (app, cookie) = app_with_admin().await;

    let req = authed_request(
        Method::PUT,
        "/api/divisions/99999",
        &cookie,
        json!({ "name": "없는분반" }),
    );
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn bulk_import_empty_list_returns_400() {
    let (app, cookie) = app_with_admin().await;
    let div_id = create_division(&app, &cookie, "1반").await;

    let req = authed_request(
        Method::POST,
        &format!("/api/divisions/{div_id}/students/bulk"),
        &cookie,
        json!([]),
    );
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn add_student_empty_fields_returns_400() {
    let (app, cookie) = app_with_admin().await;
    let div_id = create_division(&app, &cookie, "1반").await;

    let req = authed_request(
        Method::POST,
        &format!("/api/divisions/{div_id}/students"),
        &cookie,
        json!({ "student_number": "", "name": "학생", "password": "pw123" }),
    );
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// ─── teacher 검증 ─────────────────────────────────────────

#[tokio::test]
async fn delete_nonexistent_teacher_returns_404() {
    let (app, cookie) = app_with_admin().await;

    let req = authed_request(
        Method::DELETE,
        "/api/teachers/99999",
        &cookie,
        json!({}),
    );
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn create_teacher_invalid_role_returns_400() {
    let (app, cookie) = app_with_admin().await;

    let req = authed_request(
        Method::POST,
        "/api/teachers",
        &cookie,
        json!({ "username": "t1", "name": "교사", "password": "pw1234", "role": "superuser" }),
    );
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn create_teacher_empty_fields_returns_400() {
    let (app, cookie) = app_with_admin().await;

    let req = authed_request(
        Method::POST,
        "/api/teachers",
        &cookie,
        json!({ "username": "", "name": "교사", "password": "pw1234" }),
    );
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// ─── problem 검증 ─────────────────────────────────────────

#[tokio::test]
async fn create_problem_empty_title_returns_400() {
    let (app, cookie) = app_with_admin().await;

    let req = authed_request(
        Method::POST,
        "/api/problems",
        &cookie,
        json!({ "problem_type": 1, "title": "   " }),
    );
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn create_problem_type_zero_returns_400() {
    let (app, cookie) = app_with_admin().await;

    let req = authed_request(
        Method::POST,
        "/api/problems",
        &cookie,
        json!({ "problem_type": 0, "title": "유형0문제" }),
    );
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn create_problem_type_five_returns_400() {
    let (app, cookie) = app_with_admin().await;

    let req = authed_request(
        Method::POST,
        "/api/problems",
        &cookie,
        json!({ "problem_type": 5, "title": "유형5문제" }),
    );
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn list_problems_text_search() {
    let (app, cookie) = app_with_admin().await;

    create_problem(&app, &cookie, 1, "사과 문제", json!({})).await;
    create_problem(&app, &cookie, 1, "바나나 문제", json!({})).await;
    create_problem(&app, &cookie, 2, "사과 유형2", json!({})).await;

    let req = get_request("/api/problems?q=%EC%82%AC%EA%B3%BC", &cookie); // "사과" URL 인코딩
    let (status, body) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn delete_problem_assigned_to_lesson_returns_400() {
    let (app, cookie) = app_with_admin().await;

    let pid = create_problem(&app, &cookie, 1, "차시배정문제", json!({})).await;

    // 차시 생성 후 문제 배정
    let req = authed_request(Method::POST, "/api/lessons", &cookie, json!({ "title": "1차시" }));
    let (_, lb) = response_json(&app, req).await;
    let lid = lb["id"].as_i64().unwrap();

    let req2 = authed_request(
        Method::PUT,
        &format!("/api/lessons/{lid}/problems"),
        &cookie,
        json!({ "problem_ids": [pid] }),
    );
    response_json(&app, req2).await;

    // 배정된 문제 삭제 시도 → 400
    let req3 = authed_request(
        Method::DELETE,
        &format!("/api/problems/{pid}"),
        &cookie,
        json!({}),
    );
    let (status, _) = response_json(&app, req3).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// ─── lesson 검증 ───────────────────────────────────────────

#[tokio::test]
async fn get_lessons_requires_auth() {
    let pool = setup_test_db().await;
    let app = build_test_app(pool);

    let req = get_request("/api/lessons", "");
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn get_nonexistent_lesson_returns_404() {
    let (app, cookie) = app_with_admin().await;

    let req = get_request("/api/lessons/99999", &cookie);
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn set_lesson_problems_replaces_existing() {
    let (app, cookie) = app_with_admin().await;

    let req = authed_request(Method::POST, "/api/lessons", &cookie, json!({ "title": "1차시" }));
    let (_, lb) = response_json(&app, req).await;
    let lid = lb["id"].as_i64().unwrap();

    let p1 = create_problem(&app, &cookie, 1, "문제1", json!({})).await;
    let p2 = create_problem(&app, &cookie, 1, "문제2", json!({})).await;
    let p3 = create_problem(&app, &cookie, 1, "문제3", json!({})).await;

    // p1, p2 배정
    let req2 = authed_request(
        Method::PUT,
        &format!("/api/lessons/{lid}/problems"),
        &cookie,
        json!({ "problem_ids": [p1, p2] }),
    );
    response_json(&app, req2).await;

    // p3만 재배정 (p1, p2 제거 확인)
    let req3 = authed_request(
        Method::PUT,
        &format!("/api/lessons/{lid}/problems"),
        &cookie,
        json!({ "problem_ids": [p3] }),
    );
    response_json(&app, req3).await;

    let req4 = get_request(&format!("/api/lessons/{lid}"), &cookie);
    let (_, b4) = response_json(&app, req4).await;
    let probs = b4["problems"].as_array().unwrap();
    assert_eq!(probs.len(), 1);
    assert_eq!(probs[0]["problem_id"], json!(p3));
}

// ─── assessment 검증 ───────────────────────────────────────

#[tokio::test]
async fn create_assessment_empty_title_returns_400() {
    let (app, cookie) = app_with_admin().await;

    let req = authed_request(
        Method::POST,
        "/api/assessments",
        &cookie,
        json!({ "title": "   " }),
    );
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn delete_locked_assessment_returns_400() {
    let (app, cookie) = app_with_admin().await;

    let div_id = create_division(&app, &cookie, "1반").await;
    add_student(&app, &cookie, div_id, "20240001", "학생A", "pw1234").await;
    let aid = create_assessment(&app, &cookie, "잠금평가").await;
    let pid = create_problem(&app, &cookie, 1, "문제1", json!({"expected_output":"1"})).await;

    let req = authed_request(
        Method::PUT,
        &format!("/api/assessments/{aid}/problems"),
        &cookie,
        json!([{ "problem_id": pid, "score": 100 }]),
    );
    response_json(&app, req).await;

    let req2 = authed_request(
        Method::POST,
        &format!("/api/assessments/{aid}/divisions"),
        &cookie,
        json!({ "division_id": div_id }),
    );
    response_json(&app, req2).await;

    create_running_session(&app, &cookie, aid, div_id).await;

    // RUNNING 중 삭제 → 400
    let req3 = authed_request(
        Method::DELETE,
        &format!("/api/assessments/{aid}"),
        &cookie,
        json!({}),
    );
    let (status, _) = response_json(&app, req3).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn set_assessment_problems_locked_returns_400() {
    let (app, cookie) = app_with_admin().await;

    let div_id = create_division(&app, &cookie, "1반").await;
    add_student(&app, &cookie, div_id, "20240001", "학생A", "pw1234").await;
    let aid = create_assessment(&app, &cookie, "잠금평가").await;
    let pid = create_problem(&app, &cookie, 1, "문제1", json!({"expected_output":"1"})).await;

    let req = authed_request(
        Method::PUT,
        &format!("/api/assessments/{aid}/problems"),
        &cookie,
        json!([{ "problem_id": pid, "score": 100 }]),
    );
    response_json(&app, req).await;

    let req2 = authed_request(
        Method::POST,
        &format!("/api/assessments/{aid}/divisions"),
        &cookie,
        json!({ "division_id": div_id }),
    );
    response_json(&app, req2).await;

    create_running_session(&app, &cookie, aid, div_id).await;

    // 잠금 상태에서 문항 재배정 → 400
    let pid2 = create_problem(&app, &cookie, 1, "문제2", json!({})).await;
    let req3 = authed_request(
        Method::PUT,
        &format!("/api/assessments/{aid}/problems"),
        &cookie,
        json!([{ "problem_id": pid2, "score": 50 }]),
    );
    let (status, _) = response_json(&app, req3).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn unlink_division_with_running_session_returns_400() {
    let (app, cookie) = app_with_admin().await;

    let div_id = create_division(&app, &cookie, "1반").await;
    add_student(&app, &cookie, div_id, "20240001", "학생A", "pw1234").await;
    let aid = create_assessment(&app, &cookie, "수행평가").await;
    let pid = create_problem(&app, &cookie, 1, "문제1", json!({"expected_output":"1"})).await;

    let req = authed_request(
        Method::PUT,
        &format!("/api/assessments/{aid}/problems"),
        &cookie,
        json!([{ "problem_id": pid, "score": 100 }]),
    );
    response_json(&app, req).await;

    let req2 = authed_request(
        Method::POST,
        &format!("/api/assessments/{aid}/divisions"),
        &cookie,
        json!({ "division_id": div_id }),
    );
    response_json(&app, req2).await;

    create_running_session(&app, &cookie, aid, div_id).await;

    // RUNNING 중 분반 연결 해제 → 400
    let req3 = authed_request(
        Method::DELETE,
        &format!("/api/assessments/{aid}/divisions/{div_id}"),
        &cookie,
        json!({}),
    );
    let (status, _) = response_json(&app, req3).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}
