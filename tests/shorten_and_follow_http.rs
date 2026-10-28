#![cfg(feature = "server")]

//! 경계 2: HTTP 서버(얇게). 새 Postgres 데이터베이스를 붙인 실제 라우터에 요청을 보낸다.

mod common;

use dioxus::server::axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use tower::ServiceExt;

#[tokio::test]
async fn shortened_link_follows_to_the_original_url_with_302() {
    let (app, _db) = common::router().await;

    let res = app
        .clone()
        .oneshot(
            Request::post("/api/shorten")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    r#"{"original_url":"https://example.com/a?b=1"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let short_link: String = serde_json::from_str(&common::body_text(res).await).unwrap();
    assert_eq!(short_link, "http://localhost:8080/1");

    let res = app
        .oneshot(Request::get("/1").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::FOUND);
    assert_eq!(res.headers()[header::LOCATION], "https://example.com/a?b=1");
}
