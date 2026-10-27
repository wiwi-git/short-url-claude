#![cfg(feature = "server")]

//! 경계 2: HTTP 서버(얇게). 없는 Short Link와 잘못된 경로가 404 페이지를 받는지 확인한다.

use dioxus::server::axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use http_body_util::BodyExt;
use short_url::shortener::Shortener;
use tower::ServiceExt;

#[tokio::test]
async fn unknown_link_number_and_invalid_path_get_the_404_page() {
    // dx 없이 실행하면 실행 파일 옆에 public/ 이 없어 dioxus가 패닉한다.
    let public = std::env::temp_dir().join("short-url-test-public");
    std::fs::create_dir_all(&public).unwrap();
    std::env::set_var("DIOXUS_PUBLIC_PATH", &public);

    let dir = tempfile::tempdir().unwrap();
    let shortener =
        Shortener::open("http://localhost:8080", dir.path().join("links.db")).unwrap();
    let app = short_url::build_router(shortener);

    for path in ["/1", "/007", "/abc"] {
        let res = app
            .clone()
            .oneshot(Request::get(path).body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::NOT_FOUND, "GET {path}");
        assert!(
            res.headers()[header::CONTENT_TYPE]
                .to_str()
                .unwrap()
                .starts_with("text/html"),
            "GET {path}"
        );
        let body = res.into_body().collect().await.unwrap().to_bytes();
        let html = String::from_utf8(body.to_vec()).unwrap();
        assert!(html.contains("없는 링크"), "GET {path} body was: {html}");
        assert!(html.contains(r#"href="/""#), "GET {path} body was: {html}");
    }
}
