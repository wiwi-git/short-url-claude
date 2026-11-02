#![cfg(feature = "server")]

//! 경계 2: HTTP 서버(얇게). My Links 목록 서버 함수가 로그인한 User의 Short Link만 최신순으로 돌려주는지,
//! Remove 서버 함수가 내 링크에서 빼는지 확인한다.

mod common;

use dioxus::server::axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use tower::ServiceExt;

fn post_json(path: &str, body: &'static str, cookie: Option<&str>) -> Request<Body> {
    let mut req = Request::post(path).header(header::CONTENT_TYPE, "application/json");
    if let Some(cookie) = cookie {
        req = req.header(header::COOKIE, cookie);
    }
    req.body(Body::from(body)).unwrap()
}

#[tokio::test]
async fn short_link_list_server_function_returns_only_my_links_newest_first() {
    let db = common::TestDb::new().await;
    let shortener = common::shortener(&db).await;
    let me = shortener.sign_in(1, "나").await.unwrap();
    let other = shortener.sign_in(2, "남").await.unwrap();
    shortener
        .shorten(&me, "https://example.com/a")
        .await
        .unwrap();
    shortener
        .shorten(&other, "https://example.com/other")
        .await
        .unwrap();
    shortener
        .shorten(&me, "https://example.com/b")
        .await
        .unwrap();
    let cookie = common::session_cookie(&shortener, &me).await;
    let app = common::router_with(shortener);

    let res = app
        .oneshot(post_json(
            "/api/short-links",
            r#"{"page":0}"#,
            Some(&cookie),
        ))
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body: serde_json::Value = serde_json::from_str(&common::body_text(res).await).unwrap();
    assert_eq!(
        body,
        serde_json::json!({
            "short_links": [
                {"link_number": 3, "short_link": "http://localhost:8080/3"},
                {"link_number": 1, "short_link": "http://localhost:8080/1"},
            ],
            "has_next": false,
        })
    );
}

#[tokio::test]
async fn remove_server_function_takes_the_link_out_of_my_links() {
    let (app, _db, cookie) = common::signed_in_router().await;
    let res = app
        .clone()
        .oneshot(post_json(
            "/api/shorten",
            r#"{"original_url":"https://example.com/a"}"#,
            Some(&cookie),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let res = app
        .clone()
        .oneshot(post_json(
            "/api/remove",
            r#"{"link_number":1}"#,
            Some(&cookie),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let res = app
        .clone()
        .oneshot(post_json(
            "/api/short-links",
            r#"{"page":0}"#,
            Some(&cookie),
        ))
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_str(&common::body_text(res).await).unwrap();
    assert_eq!(body["short_links"], serde_json::json!([]));

    let res = app
        .oneshot(Request::get("/1").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(
        res.status(),
        StatusCode::NOT_FOUND,
        "아무도 담지 않은 Short Link는 지워진다"
    );
}

#[tokio::test]
async fn server_functions_refuse_requests_without_a_session() {
    let (app, _db, _cookie) = common::signed_in_router().await;

    for (path, body) in [
        (
            "/api/shorten",
            r#"{"original_url":"https://example.com/a"}"#,
        ),
        ("/api/short-links", r#"{"page":0}"#),
        ("/api/remove", r#"{"link_number":1}"#),
    ] {
        for cookie in [None, Some("session=not-a-session")] {
            let res = app
                .clone()
                .oneshot(post_json(path, body, cookie))
                .await
                .unwrap();
            assert_eq!(res.status(), StatusCode::UNAUTHORIZED, "{path} {cookie:?}");
        }
    }
}
