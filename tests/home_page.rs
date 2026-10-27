#![cfg(feature = "server")]

use dioxus::server::axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use tower::ServiceExt;

#[tokio::test]
async fn home_page_shows_service_name() {
    // dx 없이 실행하면 실행 파일 옆에 public/ 이 없어 dioxus가 패닉한다.
    // 빈 디렉터리를 정적 자산 경로로 지정한다.
    let public = std::env::temp_dir().join("short-url-test-public");
    std::fs::create_dir_all(&public).unwrap();
    std::env::set_var("DIOXUS_PUBLIC_PATH", &public);

    let dir = tempfile::tempdir().unwrap();
    let shortener = short_url::shortener::Shortener::open(
        "http://localhost:8080",
        dir.path().join("links.db"),
    )
    .unwrap();
    let app = short_url::build_router(shortener);

    let res = app
        .oneshot(Request::get("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let html = strip_comments(&String::from_utf8(body.to_vec()).unwrap());
    assert!(html.contains("<h1>short-url</h1>"), "body was: {html}");
}

/// SSR 결과에 섞인 hydration 주석(`<!--node-id2-->` 등)을 지운다.
fn strip_comments(html: &str) -> String {
    let mut out = String::new();
    let mut rest = html;
    while let Some(start) = rest.find("<!--") {
        out.push_str(&rest[..start]);
        match rest[start..].find("-->") {
            Some(end) => rest = &rest[start + end + 3..],
            None => return out,
        }
    }
    out.push_str(rest);
    out
}
