#![cfg(feature = "server")]

//! 서버 렌더링 스모크 확인: `/`가 200과 서비스 이름을 돌려주는지만 본다. 화면 테스트가 아니다.

mod common;

use dioxus::server::axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

#[tokio::test]
async fn home_page_shows_service_name() {
    let (app, _db) = common::router().await;

    let res = app
        .oneshot(Request::get("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let html = strip_comments(&common::body_text(res).await);
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
