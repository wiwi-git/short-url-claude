#![cfg(feature = "server")]

//! 경계 2: HTTP 서버(얇게). 저장소를 못 쓰면 Shorten과 Follow가 503을 돌려주는지 확인한다.

mod common;

use dioxus::server::axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use tower::ServiceExt;

#[tokio::test]
async fn storage_failure_gives_503_for_shorten_and_follow() {
    let (app, db) = common::router().await;
    let res = app
        .clone()
        .oneshot(
            Request::post("/api/shorten")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"original_url":"https://example.com/a"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 서버가 쓰는 테이블을 지운다.
    db.break_storage().await;

    let res = app
        .clone()
        .oneshot(
            Request::post("/api/shorten")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"original_url":"https://example.com/b"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body: serde_json::Value = serde_json::from_str(&common::body_text(res).await).unwrap();
    assert_eq!(body["data"], "StorageUnavailable");

    let res = app
        .oneshot(Request::get("/1").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::SERVICE_UNAVAILABLE, "GET /1");
}
