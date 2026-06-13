/// 권한 제어 — 역할(admin/teacher)·분반 담당 여부에 따른 접근 제한 검증
mod common;

use common::*;
use hyper::{Method, StatusCode};
use serde_json::json;

/// admin + teacher1(1반 담당) + teacher2(미담당) + 1반·2반·학생 세팅
async fn setup_two_teacher_env() -> (
    axum::Router,
    String, // admin_cookie
    String, // teacher1_cookie (1반 담당)
    String, // teacher2_cookie (미담당)
    i64,    // div1_id
    i64,    // div2_id
) {
    let pool = setup_test_db().await;
    let app = build_test_app(pool);
    setup_admin(&app).await;
    let admin_cookie = teacher_login_cookie(&app, "admin", "password123").await;

    // 교사 계정 2개 생성
    let req1 = authed_request(
        Method::POST,
        "/api/teachers",
        &admin_cookie,
        json!({ "username": "t1", "name": "교사1", "password": "pw1234" }),
    );
    let (_, tb1) = response_json(&app, req1).await;
    let t1_id = tb1["id"].as_i64().unwrap();

    let req2 = authed_request(
        Method::POST,
        "/api/teachers",
        &admin_cookie,
        json!({ "username": "t2", "name": "교사2", "password": "pw5678" }),
    );
    response_json(&app, req2).await;

    // 분반 2개 생성
    let div1 = create_division(&app, &admin_cookie, "1반").await;
    let div2 = create_division(&app, &admin_cookie, "2반").await;

    // t1 → 1반 담당
    let req3 = authed_request(
        Method::PUT,
        &format!("/api/divisions/{div1}/teachers"),
        &admin_cookie,
        json!({ "teacher_ids": [t1_id] }),
    );
    response_json(&app, req3).await;

    let t1_cookie = teacher_login_cookie(&app, "t1", "pw1234").await;
    let t2_cookie = teacher_login_cookie(&app, "t2", "pw5678").await;

    (app, admin_cookie, t1_cookie, t2_cookie, div1, div2)
}

// ─── 분반 목록 접근 제어 ───────────────────────────────────

#[tokio::test]
async fn teacher_sees_only_assigned_divisions() {
    let (app, _, t1_cookie, t2_cookie, div1, _div2) =
        setup_two_teacher_env().await;

    // t1(1반 담당)은 1반만 보임
    let req = get_request("/api/divisions", &t1_cookie);
    let (status, body) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK);
    let list = body.as_array().unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["id"], json!(div1));

    // t2(담당 없음)는 0개
    let req2 = get_request("/api/divisions", &t2_cookie);
    let (_, b2) = response_json(&app, req2).await;
    assert_eq!(b2.as_array().unwrap().len(), 0);
}

// ─── 학생 목록 접근 제어 ───────────────────────────────────

#[tokio::test]
async fn teacher_cannot_view_unassigned_division_students() {
    let (app, admin_cookie, _t1_cookie, t2_cookie, div1, _div2) =
        setup_two_teacher_env().await;

    add_student(&app, &admin_cookie, div1, "20240001", "학생A", "pw1234").await;

    // t2는 1반 미담당 → 403
    let req = get_request(&format!("/api/divisions/{div1}/students"), &t2_cookie);
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn teacher_can_view_assigned_division_students() {
    let (app, admin_cookie, t1_cookie, _, div1, _) =
        setup_two_teacher_env().await;

    add_student(&app, &admin_cookie, div1, "20240001", "학생B", "pw1234").await;

    let req = get_request(&format!("/api/divisions/{div1}/students"), &t1_cookie);
    let (status, body) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().unwrap().len(), 1);
}

// ─── 세션 목록 접근 제어 ───────────────────────────────────

#[tokio::test]
async fn teacher_cannot_see_other_divisions_sessions() {
    let (app, admin_cookie, t1_cookie, _t2_cookie, div1, div2) =
        setup_two_teacher_env().await;

    // div2에 학생 추가
    add_student(&app, &admin_cookie, div2, "20240002", "학생C", "pw1234").await;

    // div2에 세션 생성 (admin)
    let pid = create_problem(&app, &admin_cookie, 1, "문제", json!({"expected_output":"x"})).await;
    let aid = create_assessment(&app, &admin_cookie, "평가").await;
    let req = authed_request(
        Method::PUT,
        &format!("/api/assessments/{aid}/problems"),
        &admin_cookie,
        json!([{ "problem_id": pid, "score": 100 }]),
    );
    response_json(&app, req).await;
    let req2 = authed_request(
        Method::POST,
        &format!("/api/assessments/{aid}/divisions"),
        &admin_cookie,
        json!({ "division_id": div2 }),
    );
    response_json(&app, req2).await;

    let _sid = create_session(&app, &admin_cookie, aid, div2).await;

    // t1(1반 담당)은 div2 세션을 볼 수 없음
    let req3 = get_request("/api/sessions", &t1_cookie);
    let (status, body) = response_json(&app, req3).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().unwrap().len(), 0, "t1은 2반 세션을 볼 수 없어야 함");
}

#[tokio::test]
async fn sessions_filter_by_status() {
    let (app, admin_cookie, _, _, div1, div2) = setup_two_teacher_env().await;

    add_student(&app, &admin_cookie, div1, "20240001", "학생A", "pw1234").await;
    add_student(&app, &admin_cookie, div2, "20240002", "학생B", "pw1234").await;

    let pid = create_problem(&app, &admin_cookie, 1, "문제", json!({"expected_output":"x"})).await;
    let aid1 = create_assessment(&app, &admin_cookie, "평가1").await;
    let aid2 = create_assessment(&app, &admin_cookie, "평가2").await;

    for aid in [aid1, aid2] {
        let req = authed_request(
            Method::PUT,
            &format!("/api/assessments/{aid}/problems"),
            &admin_cookie,
            json!([{ "problem_id": pid, "score": 100 }]),
        );
        response_json(&app, req).await;
    }

    let req = authed_request(
        Method::POST,
        &format!("/api/assessments/{aid1}/divisions"),
        &admin_cookie,
        json!({ "division_id": div1 }),
    );
    response_json(&app, req).await;

    let req2 = authed_request(
        Method::POST,
        &format!("/api/assessments/{aid2}/divisions"),
        &admin_cookie,
        json!({ "division_id": div2 }),
    );
    response_json(&app, req2).await;

    // div1 세션 → LOBBY, div2 세션 → CREATED
    let sid1 = create_session(&app, &admin_cookie, aid1, div1).await;
    let _sid2 = create_session(&app, &admin_cookie, aid2, div2).await;
    transition_session(&app, &admin_cookie, sid1, "to_lobby").await;

    // status=LOBBY 필터
    let req3 = get_request("/api/sessions?status=LOBBY", &admin_cookie);
    let (status, body) = response_json(&app, req3).await;
    assert_eq!(status, StatusCode::OK);
    let sessions = body.as_array().unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0]["status"], json!("LOBBY"));
}

#[tokio::test]
async fn sessions_filter_by_division_id() {
    let (app, admin_cookie, _, _, div1, div2) = setup_two_teacher_env().await;

    add_student(&app, &admin_cookie, div1, "20240001", "학생A", "pw1234").await;
    add_student(&app, &admin_cookie, div2, "20240002", "학생B", "pw1234").await;

    let pid = create_problem(&app, &admin_cookie, 1, "문제", json!({"expected_output":"x"})).await;
    let aid1 = create_assessment(&app, &admin_cookie, "평가1").await;
    let aid2 = create_assessment(&app, &admin_cookie, "평가2").await;

    for (aid, div_id) in [(aid1, div1), (aid2, div2)] {
        let req = authed_request(
            Method::PUT,
            &format!("/api/assessments/{aid}/problems"),
            &admin_cookie,
            json!([{ "problem_id": pid, "score": 100 }]),
        );
        response_json(&app, req).await;
        let req2 = authed_request(
            Method::POST,
            &format!("/api/assessments/{aid}/divisions"),
            &admin_cookie,
            json!({ "division_id": div_id }),
        );
        response_json(&app, req2).await;
        create_session(&app, &admin_cookie, aid, div_id).await;
    }

    let req = get_request(&format!("/api/sessions?division_id={div1}"), &admin_cookie);
    let (status, body) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK);
    let sessions = body.as_array().unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0]["division_id"], json!(div1));
}

// ─── 세션 INDIVIDUAL 타입 ─────────────────────────────────

#[tokio::test]
async fn create_individual_session_with_target_students() {
    let (app, admin_cookie, _, _, div1, _) = setup_two_teacher_env().await;

    let s1 = add_student(&app, &admin_cookie, div1, "20240001", "학생A", "pw1234").await;
    let _s2 = add_student(&app, &admin_cookie, div1, "20240002", "학생B", "pw5678").await;

    let pid = create_problem(&app, &admin_cookie, 1, "문제", json!({"expected_output":"x"})).await;
    let aid = create_assessment(&app, &admin_cookie, "개별평가").await;
    let req = authed_request(
        Method::PUT,
        &format!("/api/assessments/{aid}/problems"),
        &admin_cookie,
        json!([{ "problem_id": pid, "score": 100 }]),
    );
    response_json(&app, req).await;
    let req2 = authed_request(
        Method::POST,
        &format!("/api/assessments/{aid}/divisions"),
        &admin_cookie,
        json!({ "division_id": div1 }),
    );
    response_json(&app, req2).await;

    // INDIVIDUAL 타입, s1만 대상
    let req3 = authed_request(
        Method::POST,
        "/api/sessions",
        &admin_cookie,
        json!({
            "assessment_id": aid,
            "division_id": div1,
            "target_type": "INDIVIDUAL",
            "student_ids": [s1]
        }),
    );
    let (status, body) = response_json(&app, req3).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["target_type"], json!("INDIVIDUAL"));
    assert_eq!(body["student_count"], json!(1));
}

// ─── 비밀번호 초기화 권한 ─────────────────────────────────

#[tokio::test]
async fn assigned_teacher_can_reset_student_password() {
    let (app, admin_cookie, t1_cookie, _, div1, _) =
        setup_two_teacher_env().await;

    let sid = add_student(&app, &admin_cookie, div1, "20240001", "학생A", "pw1234").await;

    // t1(1반 담당)이 학생 비밀번호 초기화
    let req = authed_request(
        Method::POST,
        &format!("/api/students/{sid}/reset-password"),
        &t1_cookie,
        json!({ "new_password": "resetpw1" }),
    );
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn unassigned_teacher_cannot_reset_student_password() {
    let (app, admin_cookie, _, t2_cookie, div1, _) =
        setup_two_teacher_env().await;

    let sid = add_student(&app, &admin_cookie, div1, "20240001", "학생A", "pw1234").await;

    // t2(담당 없음)가 1반 학생 비밀번호 초기화 시도 → 403
    let req = authed_request(
        Method::POST,
        &format!("/api/students/{sid}/reset-password"),
        &t2_cookie,
        json!({ "new_password": "resetpw1" }),
    );
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

// ─── 제출 채점 권한 ────────────────────────────────────────

#[tokio::test]
async fn unassigned_teacher_cannot_grade_submission() {
    let (app, admin_cookie, _, t2_cookie, div1, _) =
        setup_two_teacher_env().await;

    add_student(&app, &admin_cookie, div1, "20240001", "학생A", "pw1234").await;

    let pid = create_problem(&app, &admin_cookie, 3, "보고서", json!({})).await;
    let aid = create_assessment(&app, &admin_cookie, "수행평가").await;
    let req = authed_request(
        Method::PUT,
        &format!("/api/assessments/{aid}/problems"),
        &admin_cookie,
        json!([{ "problem_id": pid, "score": 100 }]),
    );
    response_json(&app, req).await;
    let req2 = authed_request(
        Method::POST,
        &format!("/api/assessments/{aid}/divisions"),
        &admin_cookie,
        json!({ "division_id": div1 }),
    );
    response_json(&app, req2).await;

    create_running_session(&app, &admin_cookie, aid, div1).await;

    let s_cookie = student_login_cookie(&app, "20240001", "pw1234").await;
    let req3 = authed_request(
        Method::POST,
        "/api/student/submit",
        &s_cookie,
        json!({ "problem_id": pid, "content": "보고서 내용" }),
    );
    let (_, sb) = response_json(&app, req3).await;
    let sub_id = sb["submission_id"].as_i64().unwrap();

    // t2(미담당)가 채점 시도 → 403
    let req4 = authed_request(
        Method::POST,
        &format!("/api/submissions/{sub_id}/grade"),
        &t2_cookie,
        json!({ "score": 80 }),
    );
    let (status, _) = response_json(&app, req4).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn unassigned_teacher_cannot_view_session_submissions() {
    let (app, admin_cookie, _, t2_cookie, div1, _) =
        setup_two_teacher_env().await;

    add_student(&app, &admin_cookie, div1, "20240001", "학생A", "pw1234").await;

    let pid = create_problem(&app, &admin_cookie, 1, "문제", json!({"expected_output":"x"})).await;
    let aid = create_assessment(&app, &admin_cookie, "수행평가").await;
    let req = authed_request(
        Method::PUT,
        &format!("/api/assessments/{aid}/problems"),
        &admin_cookie,
        json!([{ "problem_id": pid, "score": 100 }]),
    );
    response_json(&app, req).await;
    let req2 = authed_request(
        Method::POST,
        &format!("/api/assessments/{aid}/divisions"),
        &admin_cookie,
        json!({ "division_id": div1 }),
    );
    response_json(&app, req2).await;

    let sid = create_running_session(&app, &admin_cookie, aid, div1).await;

    // t2(미담당)가 세션 제출 목록 조회 → 403
    let req3 = get_request(&format!("/api/sessions/{sid}/submissions"), &t2_cookie);
    let (status, _) = response_json(&app, req3).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}
