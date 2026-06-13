mod common;

use common::*;
use hyper::{Method, StatusCode};
use serde_json::json;

/// RUNNING 세션 + 학생까지 완전한 fixture
async fn attendance_fixture() -> (
    axum::Router,
    String, // admin_cookie
    String, // student_cookie
    i64,    // session_id
) {
    let pool = setup_test_db().await;
    let app = build_test_app(pool);
    setup_admin(&app).await;
    let admin = teacher_login_cookie(&app, "admin", "password123").await;

    let div_id = create_division(&app, &admin, "1반").await;
    add_student(&app, &admin, div_id, "20240001", "학생A", "pw1234").await;
    add_student(&app, &admin, div_id, "20240002", "학생B", "pw1234").await;

    let p_id = create_problem(&app, &admin, 1, "문제1", json!({"expected_output":"42"})).await;
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

    let sid = create_session(&app, &admin, aid, div_id).await;
    transition_session(&app, &admin, sid, "to_lobby").await;

    let s_cookie = student_login_cookie(&app, "20240001", "pw1234").await;
    (app, admin, s_cookie, sid)
}

// ─── 하트비트 기본 동작 ────────────────────────────────────

#[tokio::test]
async fn heartbeat_returns_ok() {
    let (app, _, s_cookie, sid) = attendance_fixture().await;

    let req = authed_request(
        Method::POST,
        "/api/student/heartbeat",
        &s_cookie,
        json!({ "context_type": "session", "context_id": sid }),
    );
    let (status, body) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["ok"], json!(true));
}

#[tokio::test]
async fn heartbeat_invalid_context_type_returns_400() {
    let (app, _, s_cookie, sid) = attendance_fixture().await;

    let req = authed_request(
        Method::POST,
        "/api/student/heartbeat",
        &s_cookie,
        json!({ "context_type": "invalid", "context_id": sid }),
    );
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn heartbeat_without_auth_returns_401() {
    let pool = setup_test_db().await;
    let app = build_test_app(pool);

    let req = authed_request(
        Method::POST,
        "/api/student/heartbeat",
        "cc_student=invalid_token",
        json!({ "context_type": "session", "context_id": 1 }),
    );
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// ─── 출결 현황 조회 ────────────────────────────────────────

#[tokio::test]
async fn attendance_shows_all_division_students() {
    let (app, admin, _, sid) = attendance_fixture().await;

    let req = get_request(&format!("/api/sessions/{sid}/attendance"), &admin);
    let (status, body) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let list = body.as_array().unwrap();
    assert_eq!(list.len(), 2, "학생이 2명이어야 함");
}

#[tokio::test]
async fn attendance_student_offline_before_heartbeat() {
    let (app, admin, _, sid) = attendance_fixture().await;

    let req = get_request(&format!("/api/sessions/{sid}/attendance"), &admin);
    let (_, body) = response_json(&app, req).await;

    let list = body.as_array().unwrap();
    // 하트비트 없음 → 모두 오프라인
    assert!(list.iter().all(|s| s["is_online"] == json!(false)));
}

#[tokio::test]
async fn attendance_student_online_after_heartbeat() {
    let (app, admin, s_cookie, sid) = attendance_fixture().await;

    // 학생A 하트비트 전송
    let hb = authed_request(
        Method::POST,
        "/api/student/heartbeat",
        &s_cookie,
        json!({ "context_type": "session", "context_id": sid }),
    );
    response_json(&app, hb).await;

    let req = get_request(&format!("/api/sessions/{sid}/attendance"), &admin);
    let (status, body) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let list = body.as_array().unwrap();
    let online_count = list.iter().filter(|s| s["is_online"] == json!(true)).count();
    assert_eq!(online_count, 1, "하트비트 보낸 학생 1명만 온라인");
}

#[tokio::test]
async fn heartbeat_idempotent_update() {
    let (app, admin, s_cookie, sid) = attendance_fixture().await;

    // 같은 학생이 하트비트를 두 번 보내도 중복 행이 생기지 않아야 함
    for _ in 0..2 {
        let hb = authed_request(
            Method::POST,
            "/api/student/heartbeat",
            &s_cookie,
            json!({ "context_type": "session", "context_id": sid }),
        );
        response_json(&app, hb).await;
    }

    let req = get_request(&format!("/api/sessions/{sid}/attendance"), &admin);
    let (_, body) = response_json(&app, req).await;
    let list = body.as_array().unwrap();

    // 학생은 여전히 2명 (중복 없음)
    assert_eq!(list.len(), 2);
    let online_count = list.iter().filter(|s| s["is_online"] == json!(true)).count();
    assert_eq!(online_count, 1);
}

#[tokio::test]
async fn attendance_lobby_join_is_not_late() {
    let (app, admin, s_cookie, sid) = attendance_fixture().await;

    // LOBBY 상태에서 하트비트 → is_late = false
    let hb = authed_request(
        Method::POST,
        "/api/student/heartbeat",
        &s_cookie,
        json!({ "context_type": "session", "context_id": sid }),
    );
    response_json(&app, hb).await;

    let req = get_request(&format!("/api/sessions/{sid}/attendance"), &admin);
    let (_, body) = response_json(&app, req).await;

    let list = body.as_array().unwrap();
    let s = list.iter().find(|s| s["is_online"] == json!(true)).unwrap();
    assert_eq!(s["is_late"], json!(false));
}

#[tokio::test]
async fn attendance_running_join_is_late() {
    let (app, admin, s_cookie, sid) = attendance_fixture().await;

    // RUNNING으로 전환 후 첫 하트비트 → is_late = true
    transition_session(&app, &admin, sid, "to_running").await;

    let hb = authed_request(
        Method::POST,
        "/api/student/heartbeat",
        &s_cookie,
        json!({ "context_type": "session", "context_id": sid }),
    );
    response_json(&app, hb).await;

    let req = get_request(&format!("/api/sessions/{sid}/attendance"), &admin);
    let (_, body) = response_json(&app, req).await;

    let list = body.as_array().unwrap();
    let s = list.iter().find(|s| s["is_online"] == json!(true)).unwrap();
    assert_eq!(s["is_late"], json!(true));
}
