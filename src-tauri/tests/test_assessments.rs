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

#[tokio::test]
async fn create_and_list_assessments() {
    let (app, cookie) = app_with_admin().await;

    let aid = create_assessment(&app, &cookie, "중간고사 수행평가").await;
    assert!(aid > 0);

    let req = get_request("/api/assessments", &cookie);
    let (status, body) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().unwrap().len(), 1);
    assert_eq!(body[0]["title"], json!("중간고사 수행평가"));
}

#[tokio::test]
async fn get_assessment_detail() {
    let (app, cookie) = app_with_admin().await;
    let aid = create_assessment(&app, &cookie, "기말고사").await;

    let req = get_request(&format!("/api/assessments/{aid}"), &cookie);
    let (status, body) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["title"], json!("기말고사"));
    assert!(body["problems"].is_array());
    assert!(body["divisions"].is_array());
}

#[tokio::test]
async fn update_assessment() {
    let (app, cookie) = app_with_admin().await;
    let aid = create_assessment(&app, &cookie, "원래제목").await;

    let req = authed_request(
        Method::PUT,
        &format!("/api/assessments/{aid}"),
        &cookie,
        json!({ "title": "바뀐제목" }),
    );
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn delete_assessment() {
    let (app, cookie) = app_with_admin().await;
    let aid = create_assessment(&app, &cookie, "삭제할평가").await;

    let req = authed_request(Method::DELETE, &format!("/api/assessments/{aid}"), &cookie, json!({}));
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK);

    let req2 = get_request("/api/assessments", &cookie);
    let (_, b2) = response_json(&app, req2).await;
    assert_eq!(b2.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn set_assessment_problems_with_scores() {
    let (app, cookie) = app_with_admin().await;
    let aid = create_assessment(&app, &cookie, "평가1").await;
    let p1 = create_problem(&app, &cookie, 1, "문제1", json!({})).await;
    let p2 = create_problem(&app, &cookie, 2, "문제2", json!({})).await;

    let req = authed_request(
        Method::PUT,
        &format!("/api/assessments/{aid}/problems"),
        &cookie,
        json!([
            { "problem_id": p1, "score": 30 },
            { "problem_id": p2, "score": 70 }
        ]),
    );
    let (status, body) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let req2 = get_request(&format!("/api/assessments/{aid}"), &cookie);
    let (_, b2) = response_json(&app, req2).await;
    let problems = b2["problems"].as_array().unwrap();
    assert_eq!(problems.len(), 2);
    // 배점 확인
    let scores: Vec<i64> = problems.iter().map(|p| p["score"].as_i64().unwrap()).collect();
    assert!(scores.contains(&30));
    assert!(scores.contains(&70));
}

#[tokio::test]
async fn link_and_unlink_division() {
    let (app, cookie) = app_with_admin().await;
    let aid = create_assessment(&app, &cookie, "평가1").await;
    let div_id = create_division(&app, &cookie, "1반").await;

    // 분반 연결
    let req = authed_request(
        Method::POST,
        &format!("/api/assessments/{aid}/divisions"),
        &cookie,
        json!({ "division_id": div_id }),
    );
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK);

    let req2 = get_request(&format!("/api/assessments/{aid}"), &cookie);
    let (_, b2) = response_json(&app, req2).await;
    assert_eq!(b2["divisions"].as_array().unwrap().len(), 1);

    // 분반 연결 해제
    let req3 = authed_request(
        Method::DELETE,
        &format!("/api/assessments/{aid}/divisions/{div_id}"),
        &cookie,
        json!({}),
    );
    let (s3, _) = response_json(&app, req3).await;
    assert_eq!(s3, StatusCode::OK);

    let req4 = get_request(&format!("/api/assessments/{aid}"), &cookie);
    let (_, b4) = response_json(&app, req4).await;
    assert_eq!(b4["divisions"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn update_locked_assessment_returns_400() {
    let (app, cookie) = app_with_admin().await;

    let div_id = create_division(&app, &cookie, "1반").await;
    add_student(&app, &cookie, div_id, "20240001", "학생A", "pw1234").await;
    let aid = create_assessment(&app, &cookie, "잠금평가").await;
    let p1 = create_problem(&app, &cookie, 1, "문제1", json!({"expected_output": "1"})).await;

    // 문항 배정
    let req = authed_request(
        Method::PUT,
        &format!("/api/assessments/{aid}/problems"),
        &cookie,
        json!([{ "problem_id": p1, "score": 100 }]),
    );
    response_json(&app, req).await;

    // 분반 연결
    let req2 = authed_request(
        Method::POST,
        &format!("/api/assessments/{aid}/divisions"),
        &cookie,
        json!({ "division_id": div_id }),
    );
    response_json(&app, req2).await;

    // 세션 생성 → RUNNING
    create_running_session(&app, &cookie, aid, div_id).await;

    // 잠금 상태에서 수정 시도 → 400
    let req3 = authed_request(
        Method::PUT,
        &format!("/api/assessments/{aid}"),
        &cookie,
        json!({ "title": "수정시도" }),
    );
    let (status, _) = response_json(&app, req3).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}
