#![allow(dead_code, unused_imports)]

use axum::body::Body;
use axum::http::{HeaderMap, Request, StatusCode};
use axum::Router;
use http_body_util::BodyExt;
use serde_json::{json, Value};
use std::path::PathBuf;
use tempfile::TempDir;
use tower::ServiceExt;

use codeclan_lms_lib::{
    db,
    server::{self, state::AppState},
};

pub struct TestApp {
    pub router: Router,
    pub data_dir: PathBuf,
    pub _tmp: TempDir,
}

pub async fn setup_app() -> TestApp {
    let tmp = tempfile::tempdir().unwrap();
    let db_path = tmp.path().join("test.db");
    let pool = db::init(&db_path).await.unwrap();
    let data_dir = tmp.path().join("data");
    let state = AppState {
        db: pool,
        data_dir: data_dir.clone(),
    };
    let router = server::build_router(state);
    TestApp {
        router,
        data_dir,
        _tmp: tmp,
    }
}

impl TestApp {
    async fn raw_request(&self, req: Request<Body>) -> (StatusCode, Value, HeaderMap) {
        let resp = self.router.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let headers = resp.headers().clone();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let body: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
        (status, body, headers)
    }

    // Extract "cc_session=TOKEN" from Set-Cookie header (returns None on logout/empty)
    fn extract_cookie(headers: &HeaderMap) -> Option<String> {
        let sc = headers.get("set-cookie")?.to_str().ok()?;
        let token = sc.split(';').next()?.trim().to_string();
        if token == "cc_session=" || token == "cc_session" {
            None
        } else {
            Some(token)
        }
    }

    pub async fn get(&self, path: &str, cookie: Option<&str>) -> (StatusCode, Value) {
        let mut b = Request::builder().method("GET").uri(path);
        if let Some(c) = cookie {
            b = b.header("Cookie", c);
        }
        let (s, v, _) = self.raw_request(b.body(Body::empty()).unwrap()).await;
        (s, v)
    }

    pub async fn post(
        &self,
        path: &str,
        body: Value,
        cookie: Option<&str>,
    ) -> (StatusCode, Value, Option<String>) {
        let mut b = Request::builder()
            .method("POST")
            .uri(path)
            .header("Content-Type", "application/json");
        if let Some(c) = cookie {
            b = b.header("Cookie", c);
        }
        let (s, v, h) = self
            .raw_request(b.body(Body::from(body.to_string())).unwrap())
            .await;
        (s, v, Self::extract_cookie(&h))
    }

    pub async fn put(
        &self,
        path: &str,
        body: Value,
        cookie: Option<&str>,
    ) -> (StatusCode, Value) {
        let mut b = Request::builder()
            .method("PUT")
            .uri(path)
            .header("Content-Type", "application/json");
        if let Some(c) = cookie {
            b = b.header("Cookie", c);
        }
        let (s, v, _) = self
            .raw_request(b.body(Body::from(body.to_string())).unwrap())
            .await;
        (s, v)
    }

    pub async fn delete(&self, path: &str, cookie: Option<&str>) -> (StatusCode, Value) {
        let mut b = Request::builder().method("DELETE").uri(path);
        if let Some(c) = cookie {
            b = b.header("Cookie", c);
        }
        let (s, v, _) = self.raw_request(b.body(Body::empty()).unwrap()).await;
        (s, v)
    }

    // ── High-level helpers ────────────────────────────────────────────────────

    pub async fn do_setup(&self) {
        let (status, body, _) = self
            .post(
                "/api/setup/complete",
                json!({
                    "school_name": "Test School",
                    "admin_name": "Admin",
                    "admin_username": "admin",
                    "admin_password": "password123",
                    "locale": "ko"
                }),
                None,
            )
            .await;
        assert_eq!(status, StatusCode::OK, "Setup failed: {body}");
    }

    pub async fn login_teacher(&self, username: &str, password: &str) -> String {
        let (status, body, cookie) = self
            .post(
                "/api/auth/login/teacher",
                json!({ "username": username, "password": password }),
                None,
            )
            .await;
        assert_eq!(status, StatusCode::OK, "Teacher login failed: {body}");
        cookie.expect("Expected cc_session cookie from teacher login")
    }

    pub async fn login_student_raw(
        &self,
        username: &str,
        password: &str,
    ) -> (StatusCode, Value, Option<String>) {
        self.post(
            "/api/auth/login/student",
            json!({ "username": username, "password": password }),
            None,
        )
        .await
    }

    pub async fn login_student(&self, username: &str, password: &str) -> String {
        let (status, body, cookie) = self.login_student_raw(username, password).await;
        assert_eq!(status, StatusCode::OK, "Student login failed: {body}");
        cookie.expect("Expected cc_session cookie from student login")
    }

    pub async fn create_subject(&self, admin_cookie: &str, name: &str) -> i64 {
        let (status, body, _) = self
            .post(
                "/api/admin/subjects",
                json!({ "name": name }),
                Some(admin_cookie),
            )
            .await;
        assert_eq!(status, StatusCode::OK, "Create subject failed: {body}");
        body["id"].as_i64().unwrap()
    }

    pub async fn create_class(
        &self,
        teacher_cookie: &str,
        name: &str,
        subject_id: i64,
    ) -> i64 {
        let (status, body, _) = self
            .post(
                "/api/classes",
                json!({ "name": name, "subject_id": subject_id }),
                Some(teacher_cookie),
            )
            .await;
        assert_eq!(status, StatusCode::OK, "Create class failed: {body}");
        body["id"].as_i64().unwrap()
    }

    pub async fn add_student(
        &self,
        teacher_cookie: &str,
        class_id: i64,
        name: &str,
        grade: i64,
        class_no: i64,
        number: i64,
    ) -> i64 {
        let (status, body, _) = self
            .post(
                &format!("/api/classes/{class_id}/students"),
                json!({ "name": name, "grade": grade, "class_no": class_no, "number": number }),
                Some(teacher_cookie),
            )
            .await;
        assert_eq!(status, StatusCode::OK, "Add student failed: {body}");
        body["id"].as_i64().unwrap()
    }

    pub async fn reset_student_password(&self, teacher_cookie: &str, student_id: i64) {
        let (status, body, _) = self
            .post(
                &format!("/api/students/{student_id}/reset-password"),
                json!({}),
                Some(teacher_cookie),
            )
            .await;
        assert_eq!(status, StatusCode::OK, "Reset password failed: {body}");
    }

    pub async fn create_teacher(
        &self,
        admin_cookie: &str,
        username: &str,
        name: &str,
        password: &str,
    ) -> i64 {
        let (status, body, _) = self
            .post(
                "/api/admin/teachers",
                json!({ "username": username, "name": name, "password": password }),
                Some(admin_cookie),
            )
            .await;
        assert_eq!(status, StatusCode::OK, "Create teacher failed: {body}");
        body["id"].as_i64().unwrap()
    }
}
