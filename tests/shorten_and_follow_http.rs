#![cfg(feature = "server")]

//! 경계 2: HTTP 서버(얇게). 임시 SQLite를 붙인 실제 라우터에 요청을 보낸다.

use dioxus::server::axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use http_body_util::BodyExt;
use short_url::shortener::Shortener;
use tower::ServiceExt;

#[tokio::test]
async fn shortened_link_follows_to_the_original_url_with_302() {
    // dx 없이 실행하면 실행 파일 옆에 public/ 이 없어 dioxus가 패닉한다.
    let public = std::env::temp_dir().join("short-url-test-public");
    std::fs::create_dir_all(&public).unwrap();
    std::env::set_var("DIOXUS_PUBLIC_PATH", &public);

    let dir = tempfile::tempdir().unwrap();
    let shortener =
        Shortener::open("http://localhost:8080", dir.path().join("links.db")).unwrap();
    let app = short_url::build_router(shortener);

    let res = app
        .clone()
        .oneshot(
            Request::post("/api/shorten")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"original_url":"https://example.com/a?b=1"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let short_link: String = serde_json::from_slice(&body).unwrap();
    assert_eq!(short_link, "http://localhost:8080/1");

    let res = app
        .oneshot(Request::get("/1").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::FOUND);
    assert_eq!(res.headers()[header::LOCATION], "https://example.com/a?b=1");
}
