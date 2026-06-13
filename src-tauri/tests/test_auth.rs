mod common;

use common::*;
use hyper::{Method, StatusCode};
use serde_json::json;

// ─── 교사 인증 ─────────────────────────────────────────────

#[tokio::test]
async fn teacher_login_success_sets_cookie() {
    let pool = setup_test_db().await;
    let app = build_test_app(pool);
    setup_admin(&app).await;

    let req = json_request(
        Method::POST,
        "/api/auth/login/teacher",
        json!({ "username": "admin", "password": "password123" }),
    );
    let resp = app.clone().oneshot(req).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let set_cookie = resp.headers().get("set-cookie").expect("set-cookie 없음");
    assert!(set_cookie.to_str().unwrap().contains("cc_session="));
}

#[tokio::test]
async fn teacher_login_wrong_password_returns_401() {
    let pool = setup_test_db().await;
    let app = build_test_app(pool);
    setup_admin(&app).await;

    let req = json_request(
        Method::POST,
        "/api/auth/login/teacher",
        json!({ "username": "admin", "password": "wrongpassword" }),
    );
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn teacher_login_unknown_user_returns_401() {
    let pool = setup_test_db().await;
    let app = build_test_app(pool);
    setup_admin(&app).await;

    let req = json_request(
        Method::POST,
        "/api/auth/login/teacher",
        json!({ "username": "nobody", "password": "password123" }),
    );
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn teacher_me_returns_user_info() {
    let pool = setup_test_db().await;
    let app = build_test_app(pool);
    setup_admin(&app).await;

    let cookie = teacher_login_cookie(&app, "admin", "password123").await;
    let req = get_request("/api/auth/me", &cookie);
    let (status, body) = response_json(&app, req).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["username"], json!("admin"));
    assert_eq!(body["role"], json!("admin"));
}

#[tokio::test]
async fn teacher_me_without_cookie_returns_401() {
    let pool = setup_test_db().await;
    let app = build_test_app(pool);
    setup_admin(&app).await;

    let req = get_request("/api/auth/me", "");
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn teacher_logout_invalidates_session() {
    let pool = setup_test_db().await;
    let app = build_test_app(pool);
    setup_admin(&app).await;

    let cookie = teacher_login_cookie(&app, "admin", "password123").await;

    // 로그아웃
    let req = authed_request(Method::POST, "/api/auth/logout", &cookie, json!({}));
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK);

    // 이후 me 호출 → 401
    let req2 = get_request("/api/auth/me", &cookie);
    let (status2, _) = response_json(&app, req2).await;
    assert_eq!(status2, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn school_name_returns_configured_name() {
    let pool = setup_test_db().await;
    let app = build_test_app(pool);
    setup_admin(&app).await;

    let req = get_request("/api/auth/school-name", "");
    let (status, body) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["school_name"], json!("테스트고등학교"));
}

// ─── 학생 인증 ─────────────────────────────────────────────

/// 학생 계정을 세팅하는 헬퍼 (admin 로그인 → 분반 생성 → 학생 추가)
async fn setup_student(
    app: &axum::Router,
) -> (String /* admin_cookie */, i64 /* div_id */, i64 /* student_id */) {
    setup_admin(app).await;
    let admin_cookie = teacher_login_cookie(app, "admin", "password123").await;
    let div_id = create_division(app, &admin_cookie, "1반").await;
    let sid = add_student(app, &admin_cookie, div_id, "20240001", "김학생", "student_pw").await;
    (admin_cookie, div_id, sid)
}

#[tokio::test]
async fn student_login_success_sets_cookie() {
    let pool = setup_test_db().await;
    let app = build_test_app(pool);
    setup_student(&app).await;

    let req = json_request(
        Method::POST,
        "/api/auth/login/student",
        json!({ "student_number": "20240001", "password": "student_pw" }),
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let set_cookie = resp.headers().get("set-cookie").expect("set-cookie 없음");
    assert!(set_cookie.to_str().unwrap().contains("cc_student="));
}

#[tokio::test]
async fn student_login_wrong_password_returns_401() {
    let pool = setup_test_db().await;
    let app = build_test_app(pool);
    setup_student(&app).await;

    let req = json_request(
        Method::POST,
        "/api/auth/login/student",
        json!({ "student_number": "20240001", "password": "wrongpw" }),
    );
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn student_login_empty_password_hash_returns_400() {
    // password_hash가 빈 문자열인 학생은 400 반환
    let pool = setup_test_db().await;

    // DB에 직접 빈 해시 학생 삽입
    let div_id: i64 = sqlx::query_scalar::<_, i64>("INSERT INTO divisions (name) VALUES ('테스트반') RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO students (division_id, student_number, name, password_hash) VALUES (?, '20240099', '빈해시학생', '')")
        .bind(div_id)
        .execute(&pool)
        .await
        .unwrap();

    let app = build_test_app(pool);

    // setup 없이도 학생 로그인 시도 가능
    let req = json_request(
        Method::POST,
        "/api/auth/login/student",
        json!({ "student_number": "20240099", "password": "anypassword" }),
    );
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn student_me_returns_user_info_with_reset_flag() {
    let pool = setup_test_db().await;
    let app = build_test_app(pool);
    setup_student(&app).await;

    let cookie = student_login_cookie(&app, "20240001", "student_pw").await;
    let req = get_request("/api/auth/student/me", &cookie);
    let (status, body) = response_json(&app, req).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["student_number"], json!("20240001"));
    // add_student는 password_reset_required=1로 생성
    assert_eq!(body["password_reset_required"], json!(true));
}

#[tokio::test]
async fn student_me_without_cookie_returns_401() {
    let pool = setup_test_db().await;
    let app = build_test_app(pool);

    let req = get_request("/api/auth/student/me", "");
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn student_change_password_success() {
    let pool = setup_test_db().await;
    let app = build_test_app(pool);
    setup_student(&app).await;

    let cookie = student_login_cookie(&app, "20240001", "student_pw").await;

    // 비밀번호 변경
    let req = authed_request(
        Method::POST,
        "/api/auth/student/change-password",
        &cookie,
        json!({
            "current_password": "student_pw",
            "new_password": "new_secure_pw"
        }),
    );
    let (status, body) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK, "{body}");

    // 새 비밀번호로 로그인 가능
    let req2 = json_request(
        Method::POST,
        "/api/auth/login/student",
        json!({ "student_number": "20240001", "password": "new_secure_pw" }),
    );
    let (s2, _) = response_json(&app, req2).await;
    assert_eq!(s2, StatusCode::OK);
}

#[tokio::test]
async fn student_change_password_wrong_current_returns_401() {
    let pool = setup_test_db().await;
    let app = build_test_app(pool);
    setup_student(&app).await;

    let cookie = student_login_cookie(&app, "20240001", "student_pw").await;

    let req = authed_request(
        Method::POST,
        "/api/auth/student/change-password",
        &cookie,
        json!({
            "current_password": "wrongpw",
            "new_password": "new_secure_pw"
        }),
    );
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn student_change_password_too_short_returns_400() {
    let pool = setup_test_db().await;
    let app = build_test_app(pool);
    setup_student(&app).await;

    let cookie = student_login_cookie(&app, "20240001", "student_pw").await;

    let req = authed_request(
        Method::POST,
        "/api/auth/student/change-password",
        &cookie,
        json!({
            "current_password": "student_pw",
            "new_password": "abc"  // 3자 → 6자 미만
        }),
    );
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn student_logout_invalidates_session() {
    let pool = setup_test_db().await;
    let app = build_test_app(pool);
    setup_student(&app).await;

    let cookie = student_login_cookie(&app, "20240001", "student_pw").await;

    // 로그아웃
    let req = authed_request(Method::POST, "/api/auth/logout/student", &cookie, json!({}));
    let (status, _) = response_json(&app, req).await;
    assert_eq!(status, StatusCode::OK);

    // 이후 me 호출 → 401
    let req2 = get_request("/api/auth/student/me", &cookie);
    let (status2, _) = response_json(&app, req2).await;
    assert_eq!(status2, StatusCode::UNAUTHORIZED);
}

// tower::ServiceExt를 사용하기 위해 import
use tower::ServiceExt;
