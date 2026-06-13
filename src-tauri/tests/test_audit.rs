mod common;

use common::*;
use hyper::StatusCode;
use serde_json::json;

async fn audit_fixture() -> (axum::Router, String) {
    let pool = setup_test_db().await;
    let app = build_test_app(pool);
    setup_admin(&app).await;
    let admin = teacher_login_cookie(&app, "admin", "password123").await;
    (app, admin)
}

// ─── 감사 로그 기본 조회 ───────────────────────────────────

#[tokio::test]
async fn audit_logs_empty_on_fresh_db() {
    let (app, admin) = audit_fixture().await;

    // setup/complete 자체도 audit_log를 남기지 않음 — 빈 배열 반환
    let req = get_request("/api/audit-logs", &admin);
    let (status, body) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn audit_logs_records_assessment_create() {
    let (app, admin) = audit_fixture().await;

    create_assessment(&app, &admin, "수행평가A").await;

    let req = get_request("/api/audit-logs", &admin);
    let (status, body) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let list = body.as_array().unwrap();
    assert!(!list.is_empty());
    assert_eq!(list[0]["action_type"], json!("assessment_create"));
    assert_eq!(list[0]["target_type"], json!("assessment"));
}

#[tokio::test]
async fn audit_logs_filter_by_action_type() {
    let (app, admin) = audit_fixture().await;

    create_assessment(&app, &admin, "수행평가B").await;
    // 분반 생성도 로그 남김
    create_division(&app, &admin, "1반").await;

    // assessment_create 만 필터
    let req = get_request("/api/audit-logs?action_type=assessment_create", &admin);
    let (status, body) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let list = body.as_array().unwrap();
    assert!(list.iter().all(|r| r["action_type"] == json!("assessment_create")));
}

#[tokio::test]
async fn audit_logs_requires_teacher_auth() {
    let (app, _) = audit_fixture().await;

    let req = get_request("/api/audit-logs", "cc_session=invalid");
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn audit_logs_pagination_limit() {
    let (app, admin) = audit_fixture().await;

    // 로그 5개 생성
    for i in 0..5 {
        create_assessment(&app, &admin, &format!("평가{i}")).await;
    }

    let req = get_request("/api/audit-logs?limit=3", &admin);
    let (status, body) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body.as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn audit_logs_actor_name_populated() {
    let (app, admin) = audit_fixture().await;

    create_assessment(&app, &admin, "평가X").await;

    let req = get_request("/api/audit-logs", &admin);
    let (_, body) = response_json(&app, req).await;
    let list = body.as_array().unwrap();
    assert!(!list[0]["actor_name"].is_null(), "actor_name이 null이면 안 됨");
}
