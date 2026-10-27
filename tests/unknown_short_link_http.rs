#![cfg(feature = "server")]

//! 경계 2: HTTP 서버(얇게). `/`와 올바른 Short Link가 아닌 주소가 404 페이지를 받는지 확인한다.

mod common;

use dioxus::server::axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use tower::ServiceExt;

async fn assert_not_found_page(app: &dioxus::server::axum::Router, path: &str) {
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
    let html = common::body_text(res).await;
    assert!(html.contains("없는 링크"), "GET {path} body was: {html}");
    assert!(html.contains(r#"href="/""#), "GET {path} body was: {html}");
}

#[tokio::test]
async fn unknown_link_number_and_invalid_path_get_the_404_page() {
    let (app, _dir) = common::router();

    for path in ["/1", "/007", "/abc"] {
        assert_not_found_page(&app, path).await;
    }
}

#[tokio::test]
async fn paths_with_more_than_one_segment_get_the_404_page() {
    let (app, _dir) = common::router();

    for path in ["/a/b", "/1/", "/1/2", "/api", "/api/unknown"] {
        assert_not_found_page(&app, path).await;
    }
}
