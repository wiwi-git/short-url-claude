#![cfg(feature = "server")]

//! 경계 2: HTTP 서버(얇게). Shorten 서버 함수가 거절 사유를 돌려주는지 확인한다.

mod common;

use dioxus::server::axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use tower::ServiceExt;

#[tokio::test]
async fn shorten_server_function_returns_the_rejection_reason() {
    let (app, _db) = common::router().await;

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
    let body: serde_json::Value = serde_json::from_str(&common::body_text(res).await).unwrap();
    assert_eq!(
        body["data"],
        serde_json::json!({ "Rejected": "UnsupportedScheme" })
    );
    assert_eq!(
        body["message"],
        "http:// 또는 https://로 시작하는 주소만 줄일 수 있어요"
    );
}
