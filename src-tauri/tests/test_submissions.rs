mod common;

use common::*;
use hyper::{Method, StatusCode};
use serde_json::json;

/// 수행평가 + 세션 + 학생까지 완전한 fixture 구성
async fn full_fixture(
    problem_type: i64,
    type_config: serde_json::Value,
) -> (axum::Router, String /* admin_cookie */, String /* student_cookie */, i64 /* session_id */, i64 /* problem_id */) {
    let pool = setup_test_db().await;
    let app = build_test_app(pool);
    setup_admin(&app).await;
    let admin_cookie = teacher_login_cookie(&app, "admin", "password123").await;

    let div_id = create_division(&app, &admin_cookie, "1반").await;
    add_student(&app, &admin_cookie, div_id, "20240001", "학생A", "pw1234").await;

    let p_id = create_problem(&app, &admin_cookie, problem_type, "문제1", type_config).await;
    let aid = create_assessment(&app, &admin_cookie, "수행평가1").await;

    let req = authed_request(
        Method::PUT,
        &format!("/api/assessments/{aid}/problems"),
        &admin_cookie,
        json!([{ "problem_id": p_id, "score": 100 }]),
    );
    response_json(&app, req).await;

    let req2 = authed_request(
        Method::POST,
        &format!("/api/assessments/{aid}/divisions"),
        &admin_cookie,
        json!({ "division_id": div_id }),
    );
    response_json(&app, req2).await;

    let sid = create_running_session(&app, &admin_cookie, aid, div_id).await;
    let student_cookie = student_login_cookie(&app, "20240001", "pw1234").await;

    (app, admin_cookie, student_cookie, sid, p_id)
}

// ─── session-problems 조회 ─────────────────────────────────

#[tokio::test]
async fn get_session_problems_running_session() {
    let (app, _, student_cookie, _, _) = full_fixture(1, json!({"expected_output": "42"})).await;

    let req = get_request("/api/student/session-problems", &student_cookie);
    let (status, body) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let probs = body.as_array().unwrap();
    assert_eq!(probs.len(), 1);
    assert!(probs[0]["submission_id"].is_null()); // 아직 제출 없음
}

#[tokio::test]
async fn get_session_problems_no_running_session_returns_404() {
    let pool = setup_test_db().await;
    let app = build_test_app(pool);
    setup_admin(&app).await;
    let cookie = teacher_login_cookie(&app, "admin", "password123").await;
    let div_id = create_division(&app, &cookie, "1반").await;
    add_student(&app, &cookie, div_id, "20240001", "학생A", "pw1234").await;
    let s_cookie = student_login_cookie(&app, "20240001", "pw1234").await;

    let req = get_request("/api/student/session-problems", &s_cookie);
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ─── 유형① 자동채점 ────────────────────────────────────────

#[tokio::test]
async fn submit_type1_correct_answer_returns_ac() {
    let (app, _, s_cookie, _, p_id) =
        full_fixture(1, json!({"expected_output": "42"})).await;

    let req = authed_request(
        Method::POST,
        "/api/student/submit",
        &s_cookie,
        json!({ "problem_id": p_id, "content": "42" }),
    );
    let (status, body) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["verdict"], json!("AC"));
    assert_eq!(body["score"], json!(100));
}

#[tokio::test]
async fn submit_type1_wrong_answer_returns_wa() {
    let (app, _, s_cookie, _, p_id) =
        full_fixture(1, json!({"expected_output": "42"})).await;

    let req = authed_request(
        Method::POST,
        "/api/student/submit",
        &s_cookie,
        json!({ "problem_id": p_id, "content": "43" }),
    );
    let (_, body) = response_json(&app, req).await;
    assert_eq!(body["verdict"], json!("WA"));
    assert_eq!(body["score"], json!(0));
}

#[tokio::test]
async fn submit_type1_no_expected_output_returns_null_verdict() {
    let (app, _, s_cookie, _, p_id) = full_fixture(1, json!({})).await;

    let req = authed_request(
        Method::POST,
        "/api/student/submit",
        &s_cookie,
        json!({ "problem_id": p_id, "content": "anything" }),
    );
    let (status, body) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["verdict"].is_null());
}

// ─── 유형② PENDING ─────────────────────────────────────────

#[tokio::test]
async fn submit_type2_returns_pending() {
    let (app, _, s_cookie, _, p_id) = full_fixture(2, json!({})).await;

    let req = authed_request(
        Method::POST,
        "/api/student/submit",
        &s_cookie,
        json!({ "problem_id": p_id, "content": "print('hello')", "language": "python" }),
    );
    let (status, body) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["verdict"], json!("PENDING"));
}

// ─── 재제출 is_latest 관리 ────────────────────────────────

#[tokio::test]
async fn resubmit_updates_is_latest_flag() {
    let (app, _, s_cookie, _, p_id) =
        full_fixture(1, json!({"expected_output": "42"})).await;

    // 첫 제출
    let req1 = authed_request(
        Method::POST,
        "/api/student/submit",
        &s_cookie,
        json!({ "problem_id": p_id, "content": "wrong" }),
    );
    let (_, b1) = response_json(&app, req1).await;
    let sub1_id = b1["submission_id"].as_i64().unwrap();

    // 재제출
    let req2 = authed_request(
        Method::POST,
        "/api/student/submit",
        &s_cookie,
        json!({ "problem_id": p_id, "content": "42" }),
    );
    let (_, b2) = response_json(&app, req2).await;
    let sub2_id = b2["submission_id"].as_i64().unwrap();

    assert_ne!(sub1_id, sub2_id);

    // session-problems 조회 시 최신 제출(AC)이 나타나야 함
    let req3 = get_request("/api/student/session-problems", &s_cookie);
    let (_, b3) = response_json(&app, req3).await;
    let probs = b3.as_array().unwrap();
    assert_eq!(probs[0]["verdict"], json!("AC"));
    assert_eq!(probs[0]["submission_id"], json!(sub2_id));
}

// ─── 교사 제출 목록 조회 ───────────────────────────────────

#[tokio::test]
async fn teacher_can_view_session_submissions() {
    let (app, admin_cookie, s_cookie, sid, p_id) =
        full_fixture(1, json!({"expected_output": "42"})).await;

    // 학생 제출
    let req = authed_request(
        Method::POST,
        "/api/student/submit",
        &s_cookie,
        json!({ "problem_id": p_id, "content": "42" }),
    );
    response_json(&app, req).await;

    // 교사 조회
    let req2 = get_request(&format!("/api/sessions/{sid}/submissions"), &admin_cookie);
    let (status, body) = response_json(&app, req2).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let list = body.as_array().unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["verdict"], json!("AC"));
}

// ─── 수동 채점 ─────────────────────────────────────────────

#[tokio::test]
async fn teacher_grade_submission() {
    let (app, admin_cookie, s_cookie, sid, p_id) = full_fixture(3, json!({})).await;

    // 학생 제출 (유형③ → verdict null)
    let req = authed_request(
        Method::POST,
        "/api/student/submit",
        &s_cookie,
        json!({ "problem_id": p_id, "content": "보고서 내용" }),
    );
    let (_, sb) = response_json(&app, req).await;
    let sub_id = sb["submission_id"].as_i64().unwrap();

    // 교사 채점
    let req2 = authed_request(
        Method::POST,
        &format!("/api/submissions/{sub_id}/grade"),
        &admin_cookie,
        json!({ "score": 85 }),
    );
    let (status, body) = response_json(&app, req2).await;
    assert_eq!(status, StatusCode::OK, "{body}");

    // 제출 목록에서 GRADED 확인
    let req3 = get_request(&format!("/api/sessions/{sid}/submissions"), &admin_cookie);
    let (_, b3) = response_json(&app, req3).await;
    let sub = &b3.as_array().unwrap()[0];
    assert_eq!(sub["verdict"], json!("GRADED"));
    assert_eq!(sub["score"], json!(85));
}

// ─── active-session ────────────────────────────────────────

#[tokio::test]
async fn student_sees_active_running_session() {
    let (app, _, s_cookie, sid, _) =
        full_fixture(1, json!({"expected_output": "42"})).await;

    let req = get_request("/api/student/active-session", &s_cookie);
    let (status, body) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK);
    // active-session은 Option<SessionRow> 반환 (null이면 없음, object면 있음)
    assert!(body.is_object(), "active session should be an object, got: {body}");
    assert_eq!(body["id"], json!(sid));

    let _ = sid;
}

#[tokio::test]
async fn student_no_active_session_returns_null() {
    let pool = setup_test_db().await;
    let app = build_test_app(pool);
    setup_admin(&app).await;
    let cookie = teacher_login_cookie(&app, "admin", "password123").await;
    let div_id = create_division(&app, &cookie, "1반").await;
    add_student(&app, &cookie, div_id, "20240001", "학생A", "pw1234").await;
    let s_cookie = student_login_cookie(&app, "20240001", "pw1234").await;

    let req = get_request("/api/student/active-session", &s_cookie);
    let (status, body) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.is_null(), "no active session should return null, got: {body}");
}
