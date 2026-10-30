#![cfg(feature = "server")]

//! 경계 2: HTTP 서버(얇게). Short Link 목록 서버 함수가 최신순 목록을 돌려주는지 확인한다.

mod common;

use dioxus::server::axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use tower::ServiceExt;

#[tokio::test]
async fn short_link_list_server_function_returns_short_links_newest_first() {
    let db = common::TestDb::new().await;
    let shortener = common::shortener(&db).await;
    shortener.shorten("https://example.com/a").await.unwrap();
    shortener.shorten("https://example.com/b").await.unwrap();
    let app = common::router_with(shortener);

    let res = app
        .oneshot(
            Request::post("/api/short-links")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"page":0}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body: serde_json::Value = serde_json::from_str(&common::body_text(res).await).unwrap();
    assert_eq!(
        body,
        serde_json::json!({
            "short_links": ["http://localhost:8080/2", "http://localhost:8080/1"],
            "has_next": false,
        })
    );
}
