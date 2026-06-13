mod common;

use common::*;
use hyper::{Method, StatusCode};
use serde_json::json;

/// 분반·수행평가 연결까지 완료된 기본 fixture (세션 없음)
async fn student_fixture() -> (
    axum::Router,
    String, // admin_cookie
    String, // student_cookie
    i64,    // assessment_id
    i64,    // division_id
) {
    let pool = setup_test_db().await;
    let app = build_test_app(pool);
    setup_admin(&app).await;
    let admin = teacher_login_cookie(&app, "admin", "password123").await;

    let div_id = create_division(&app, &admin, "1반").await;
    add_student(&app, &admin, div_id, "20240001", "학생A", "pw1234").await;

    let p_id = create_problem(&app, &admin, 1, "문제1", json!({"expected_output": "42"})).await;
    let aid = create_assessment(&app, &admin, "수행평가1").await;

    let req = authed_request(
        Method::PUT,
        &format!("/api/assessments/{aid}/problems"),
        &admin,
        json!([{ "problem_id": p_id, "score": 100 }]),
    );
    response_json(&app, req).await;

    let req2 = authed_request(
        Method::POST,
        &format!("/api/assessments/{aid}/divisions"),
        &admin,
        json!({ "division_id": div_id }),
    );
    response_json(&app, req2).await;

    let s_cookie = student_login_cookie(&app, "20240001", "pw1234").await;
    (app, admin, s_cookie, aid, div_id)
}

// ─── 학생 수행평가 목록 조회 ────────────────────────────────

#[tokio::test]
async fn student_assessments_list_shows_linked_assessment() {
    let (app, _, s_cookie, aid, _) = student_fixture().await;

    let req = get_request("/api/student/assessments", &s_cookie);
    let (status, body) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let list = body.as_array().unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["id"], json!(aid));
    assert_eq!(list[0]["title"], json!("수행평가1"));
    assert_eq!(list[0]["problem_count"], json!(1));
}

#[tokio::test]
async fn student_assessments_no_session_shows_null() {
    let (app, _, s_cookie, _, _) = student_fixture().await;

    let req = get_request("/api/student/assessments", &s_cookie);
    let (status, body) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK, "{body}");

    assert!(body[0]["session_id"].is_null());
    assert!(body[0]["session_status"].is_null());
    assert_eq!(body[0]["is_result_released"], json!(false));
}

#[tokio::test]
async fn student_assessments_shows_running_session_status() {
    let (app, admin, s_cookie, aid, div_id) = student_fixture().await;

    let sid = create_running_session(&app, &admin, aid, div_id).await;

    let req = get_request("/api/student/assessments", &s_cookie);
    let (status, body) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK, "{body}");

    assert_eq!(body[0]["session_id"], json!(sid));
    assert_eq!(body[0]["session_status"], json!("RUNNING"));
    assert_eq!(body[0]["is_result_released"], json!(false));
}

#[tokio::test]
async fn student_assessments_unlinked_division_shows_empty() {
    let pool = setup_test_db().await;
    let app = build_test_app(pool);
    setup_admin(&app).await;
    let admin = teacher_login_cookie(&app, "admin", "password123").await;

    let div_id = create_division(&app, &admin, "1반").await;
    add_student(&app, &admin, div_id, "20240001", "학생A", "pw1234").await;

    // 수행평가 생성하지만 분반 연결 없음
    create_assessment(&app, &admin, "미연결평가").await;

    let s_cookie = student_login_cookie(&app, "20240001", "pw1234").await;

    let req = get_request("/api/student/assessments", &s_cookie);
    let (status, body) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body.as_array().unwrap().len(), 0);
}

// ─── 결과 공개 후 학생 is_result_released 반영 ─────────────

#[tokio::test]
async fn student_assessments_shows_result_released_flag() {
    let (app, admin, s_cookie, aid, div_id) = student_fixture().await;

    let sid = create_running_session(&app, &admin, aid, div_id).await;

    // 세션 닫기
    let close_req = authed_request(
        Method::POST,
        &format!("/api/sessions/{sid}/transition"),
        &admin,
        json!({ "action": "close" }),
    );
    response_json(&app, close_req).await;

    // 결과 공개
    let release_req = authed_request(
        Method::POST,
        &format!("/api/sessions/{sid}/result-release"),
        &admin,
        json!({}),
    );
    response_json(&app, release_req).await;

    let req = get_request("/api/student/assessments", &s_cookie);
    let (status, body) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body[0]["is_result_released"], json!(true));
    assert_eq!(body[0]["session_status"], json!("CLOSED"));
}

// ─── 결과 상세 조회 (session result problems) ──────────────

#[tokio::test]
async fn student_result_problems_unreleased_returns_404() {
    let (app, admin, s_cookie, aid, div_id) = student_fixture().await;

    let sid = create_running_session(&app, &admin, aid, div_id).await;

    // 세션 종료 (결과 미공개)
    let close_req = authed_request(
        Method::POST,
        &format!("/api/sessions/{sid}/transition"),
        &admin,
        json!({ "action": "close" }),
    );
    response_json(&app, close_req).await;

    let req = get_request(&format!("/api/student/sessions/{sid}/problems"), &s_cookie);
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn student_result_problems_shows_problems_after_release() {
    let (app, admin, s_cookie, aid, div_id) = student_fixture().await;

    let sid = create_running_session(&app, &admin, aid, div_id).await;

    let close_req = authed_request(
        Method::POST,
        &format!("/api/sessions/{sid}/transition"),
        &admin,
        json!({ "action": "close" }),
    );
    response_json(&app, close_req).await;

    let release_req = authed_request(
        Method::POST,
        &format!("/api/sessions/{sid}/result-release"),
        &admin,
        json!({}),
    );
    response_json(&app, release_req).await;

    let req = get_request(&format!("/api/student/sessions/{sid}/problems"), &s_cookie);
    let (status, body) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let list = body.as_array().unwrap();
    assert_eq!(list.len(), 1, "문제 1개 반환");
    assert_eq!(list[0]["title"], json!("문제1"));
}

#[tokio::test]
async fn student_assessments_my_score_reflects_graded_submission() {
    let (app, admin, s_cookie, aid, div_id) = student_fixture().await;

    let sid = create_running_session(&app, &admin, aid, div_id).await;

    // 학생이 제출
    let submit_req = authed_request(
        Method::POST,
        "/api/student/submit",
        &s_cookie,
        json!({ "problem_id": 1, "content": "42" }),
    );
    response_json(&app, submit_req).await;

    // 제출 ID 조회
    let subs_req = authed_request(
        Method::GET,
        &format!("/api/sessions/{sid}/submissions"),
        &admin,
        json!({}),
    );
    let (_, subs) = response_json(&app, subs_req).await;
    let sub_id = subs[0]["id"].as_i64().unwrap();

    // 교사 채점
    let grade_req = authed_request(
        Method::POST,
        &format!("/api/submissions/{sub_id}/grade"),
        &admin,
        json!({ "score": 80 }),
    );
    response_json(&app, grade_req).await;

    // 세션 종료 + 결과 공개
    let close_req = authed_request(
        Method::POST,
        &format!("/api/sessions/{sid}/transition"),
        &admin,
        json!({ "action": "close" }),
    );
    response_json(&app, close_req).await;

    let release_req = authed_request(
        Method::POST,
        &format!("/api/sessions/{sid}/result-release"),
        &admin,
        json!({}),
    );
    response_json(&app, release_req).await;

    // 학생 수행평가 목록에서 my_score 확인
    let req = get_request("/api/student/assessments", &s_cookie);
    let (status, body) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body[0]["my_score"], json!(80));
    assert_eq!(body[0]["total_max_score"], json!(100));
}
