#![cfg(feature = "server")]

//! 경계 2: HTTP 서버(얇게). Shorten 서버 함수가 거절 사유를 돌려주는지 확인한다.

use dioxus::server::axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use http_body_util::BodyExt;
use short_url::shortener::Shortener;
use tower::ServiceExt;

#[tokio::test]
async fn shorten_server_function_returns_the_rejection_reason() {
    // dx 없이 실행하면 실행 파일 옆에 public/ 이 없어 dioxus가 패닉한다.
    let public = std::env::temp_dir().join("short-url-test-public");
    std::fs::create_dir_all(&public).unwrap();
    std::env::set_var("DIOXUS_PUBLIC_PATH", &public);

    let dir = tempfile::tempdir().unwrap();
    let shortener = Shortener::open("http://localhost:8080", dir.path().join("links.db")).unwrap();
    let app = short_url::build_router(shortener);

    let res = app
        .oneshot(
            Request::post("/api/shorten")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"original_url":"ftp://example.com/a"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        body["data"],
        serde_json::json!({ "Rejected": "UnsupportedScheme" })
    );
    assert_eq!(
        body["message"],
        "http:// 또는 https://로 시작하는 주소만 줄일 수 있어요"
    );
}
