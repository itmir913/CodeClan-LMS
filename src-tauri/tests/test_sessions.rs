mod common;

use common::*;
use hyper::{Method, StatusCode};
use serde_json::json;

async fn app_with_session_fixture() -> (axum::Router, String, i64, i64, i64) {
    let pool = setup_test_db().await;
    let app = build_test_app(pool);
    setup_admin(&app).await;
    let cookie = teacher_login_cookie(&app, "admin", "password123").await;

    let div_id = create_division(&app, &cookie, "1반").await;
    add_student(&app, &cookie, div_id, "20240001", "학생A", "pw1234").await;
    let p_id = create_problem(&app, &cookie, 1, "문제1", json!({"expected_output":"42"})).await;
    let aid = create_assessment(&app, &cookie, "수행평가1").await;

    let req = authed_request(
        Method::PUT,
        &format!("/api/assessments/{aid}/problems"),
        &cookie,
        json!([{ "problem_id": p_id, "score": 100 }]),
    );
    response_json(&app, req).await;

    let req2 = authed_request(
        Method::POST,
        &format!("/api/assessments/{aid}/divisions"),
        &cookie,
        json!({ "division_id": div_id }),
    );
    response_json(&app, req2).await;

    let sid = create_session(&app, &cookie, aid, div_id).await;
    (app, cookie, div_id, aid, sid)
}

// ─── 세션 생성 ─────────────────────────────────────────────

#[tokio::test]
async fn create_session_status_created() {
    let (app, cookie, div_id, aid, sid) = app_with_session_fixture().await;
    assert!(sid > 0);

    let req = get_request("/api/sessions", &cookie);
    let (status, body) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK);
    let sessions = body.as_array().unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0]["status"], json!("CREATED"));

    let _ = (div_id, aid); // suppress unused warnings
}

#[tokio::test]
async fn create_second_active_session_for_same_division_returns_400() {
    let (app, cookie, div_id, aid, _) = app_with_session_fixture().await;

    // 세션을 LOBBY로 전환
    let sid = create_session(&app, &cookie, aid, div_id).await;
    transition_session(&app, &cookie, sid, "to_lobby").await;

    // 같은 분반에 새 세션 생성 시도 → 400
    let req = authed_request(
        Method::POST,
        "/api/sessions",
        &cookie,
        json!({ "assessment_id": aid, "division_id": div_id }),
    );
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// ─── 세션 상태 전환 ────────────────────────────────────────

#[tokio::test]
async fn session_transition_created_to_lobby() {
    let (app, cookie, _, _, sid) = app_with_session_fixture().await;

    transition_session(&app, &cookie, sid, "to_lobby").await;

    let req = get_request("/api/sessions", &cookie);
    let (_, body) = response_json(&app, req).await;
    assert_eq!(body[0]["status"], json!("LOBBY"));
}

#[tokio::test]
async fn session_transition_lobby_back_to_created() {
    let (app, cookie, _, _, sid) = app_with_session_fixture().await;
    transition_session(&app, &cookie, sid, "to_lobby").await;

    transition_session(&app, &cookie, sid, "to_created").await;

    let req = get_request("/api/sessions", &cookie);
    let (_, body) = response_json(&app, req).await;
    assert_eq!(body[0]["status"], json!("CREATED"));
}

#[tokio::test]
async fn session_transition_lobby_to_running_sets_start_at() {
    let (app, cookie, _, _, sid) = app_with_session_fixture().await;
    transition_session(&app, &cookie, sid, "to_lobby").await;

    let uri = format!("/api/sessions/{sid}/transition");
    let req = authed_request(Method::POST, &uri, &cookie, json!({ "action": "to_running" }));
    let (status, body) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["status"], json!("RUNNING"));
}

#[tokio::test]
async fn session_transition_running_to_closed() {
    let (app, cookie, div_id, aid, _) = app_with_session_fixture().await;
    let sid = create_running_session(&app, &cookie, aid, div_id).await;

    let uri = format!("/api/sessions/{sid}/transition");
    let req = authed_request(Method::POST, &uri, &cookie, json!({ "action": "close" }));
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK);

    let req2 = get_request("/api/sessions", &cookie);
    let (_, b2) = response_json(&app, req2).await;
    let running_session = b2.as_array().unwrap().iter().find(|s| s["id"] == json!(sid)).unwrap().clone();
    assert_eq!(running_session["status"], json!("CLOSED"));
}

#[tokio::test]
async fn invalid_transition_returns_400() {
    let (app, cookie, _, _, sid) = app_with_session_fixture().await;

    // CREATED → close 는 허용 안 됨
    let uri = format!("/api/sessions/{sid}/transition");
    let req = authed_request(Method::POST, &uri, &cookie, json!({ "action": "close" }));
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// ─── 일시정지 ─────────────────────────────────────────────

#[tokio::test]
async fn pause_running_session_toggles_paused() {
    let (app, cookie, div_id, aid, _) = app_with_session_fixture().await;
    let sid = create_running_session(&app, &cookie, aid, div_id).await;

    let uri = format!("/api/sessions/{sid}/pause");
    let req = authed_request(Method::POST, &uri, &cookie, json!({}));
    let (status, body) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["is_paused"], json!(true));

    // 다시 → 재개
    let req2 = authed_request(Method::POST, &uri, &cookie, json!({}));
    let (_, b2) = response_json(&app, req2).await;
    assert_eq!(b2["is_paused"], json!(false));
}

#[tokio::test]
async fn pause_non_running_session_returns_400() {
    let (app, cookie, _, _, sid) = app_with_session_fixture().await;
    // CREATED 상태에서 pause 시도
    let uri = format!("/api/sessions/{sid}/pause");
    let req = authed_request(Method::POST, &uri, &cookie, json!({}));
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// ─── 결과 공개 ─────────────────────────────────────────────

#[tokio::test]
async fn result_release_on_closed_session() {
    let (app, cookie, div_id, aid, _) = app_with_session_fixture().await;
    let sid = create_running_session(&app, &cookie, aid, div_id).await;

    // CLOSED로 전환
    let close_req = authed_request(
        Method::POST,
        &format!("/api/sessions/{sid}/transition"),
        &cookie,
        json!({ "action": "close" }),
    );
    response_json(&app, close_req).await;

    let uri = format!("/api/sessions/{sid}/result-release");
    let req = authed_request(Method::POST, &uri, &cookie, json!({}));
    let (status, body) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["is_result_released"], json!(true));

    // 다시 → 비공개
    let req2 = authed_request(Method::POST, &uri, &cookie, json!({}));
    let (_, b2) = response_json(&app, req2).await;
    assert_eq!(b2["is_result_released"], json!(false));
}

#[tokio::test]
async fn result_release_on_non_closed_session_returns_400() {
    let (app, cookie, div_id, aid, _) = app_with_session_fixture().await;
    let sid = create_running_session(&app, &cookie, aid, div_id).await;
    // RUNNING 상태에서 result-release 시도
    let uri = format!("/api/sessions/{sid}/result-release");
    let req = authed_request(Method::POST, &uri, &cookie, json!({}));
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}
