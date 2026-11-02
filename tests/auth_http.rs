#![cfg(feature = "server")]

//! 경계 2: HTTP 서버(얇게), 가짜 카카오. 로그인 화면, 카카오 로그인 흐름, 로그아웃, 통계 API를 확인한다.

mod common;

use dioxus::server::axum::{
    body::Body,
    http::{header, Request, Response, StatusCode},
    Router,
};
use tower::ServiceExt;

async fn get(app: &Router, path: &str, cookie: Option<&str>) -> Response<Body> {
    let mut req = Request::get(path);
    if let Some(cookie) = cookie {
        req = req.header(header::COOKIE, cookie);
    }
    app.clone()
        .oneshot(req.body(Body::empty()).unwrap())
        .await
        .unwrap()
}

/// 응답의 `Set-Cookie` 중 `name`의 것 (`name=value; ...` 전체).
fn set_cookie(res: &Response<Body>, name: &str) -> Option<String> {
    res.headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .map(|v| v.to_str().unwrap().to_string())
        .find(|v| v.starts_with(&format!("{name}=")))
}

/// `Set-Cookie` 값에서 브라우저가 돌려보낼 `name=value` 부분.
fn cookie_pair(set_cookie: &str) -> String {
    set_cookie.split(';').next().unwrap().to_string()
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

async fn home_html(app: &Router, cookie: Option<&str>) -> String {
    let res = get(app, "/", cookie).await;
    assert_eq!(res.status(), StatusCode::OK);
    strip_comments(&common::body_text(res).await)
}

/// 로그인 버튼 → 카카오 → 콜백을 가짜 카카오로 거쳐, 콜백 응답을 돌려준다.
async fn sign_in_through_kakao(app: &Router, code: &str) -> Response<Body> {
    let res = get(app, "/auth/kakao/login", None).await;
    let location = res.headers()[header::LOCATION]
        .to_str()
        .unwrap()
        .to_string();
    let state = location.split("state=").nth(1).unwrap().to_string();
    let state_cookie = cookie_pair(&set_cookie(&res, "kakao_state").unwrap());
    get(
        app,
        &format!("/auth/kakao/callback?code={code}&state={state}"),
        Some(&state_cookie),
    )
    .await
}

#[tokio::test]
async fn signed_out_home_shows_only_the_title_and_the_login_button() {
    let (app, _db) = common::router().await;

    let html = home_html(&app, None).await;

    assert!(html.contains("<h1>short-url</h1>"), "body was: {html}");
    assert!(
        html.contains(r#"href="/auth/kakao/login""#),
        "body was: {html}"
    );
    assert!(html.contains("카카오 로그인"), "body was: {html}");
    assert!(!html.contains("<form"), "줄이기 폼이 없어야 한다: {html}");
    assert!(!html.contains("내 단축 링크"), "목록이 없어야 한다: {html}");
}

#[tokio::test]
async fn login_button_redirects_to_kakao_with_a_state_cookie() {
    let (app, _db) = common::router().await;

    let res = get(&app, "/auth/kakao/login", None).await;

    assert_eq!(res.status(), StatusCode::FOUND);
    let location = res.headers()[header::LOCATION].to_str().unwrap();
    assert!(location.starts_with(common::FAKE_LOGIN_URL), "{location}");
    let state = location.split("state=").nth(1).unwrap();
    let cookie = set_cookie(&res, "kakao_state").unwrap();
    assert_eq!(cookie_pair(&cookie), format!("kakao_state={state}"));
    assert!(cookie.contains("HttpOnly"), "{cookie}");
}

#[tokio::test]
async fn kakao_callback_signs_in_and_home_shows_nickname_logout_and_my_links() {
    let (app, _db) = common::router().await;

    let res = sign_in_through_kakao(&app, "7:카카오친구").await;

    assert_eq!(res.status(), StatusCode::FOUND);
    assert_eq!(res.headers()[header::LOCATION], "/");
    let session = set_cookie(&res, "session").expect("세션 쿠키");
    assert!(session.contains("HttpOnly"), "{session}");
    assert!(session.contains("SameSite=Lax"), "{session}");
    assert!(
        session.contains(&format!("Max-Age={}", 30 * 24 * 60 * 60)),
        "{session}"
    );
    assert!(
        !session.contains("Secure"),
        "http 서비스 주소에는 Secure를 붙이지 않는다"
    );

    let html = home_html(&app, Some(&cookie_pair(&session))).await;
    assert!(html.contains("카카오친구"), "body was: {html}");
    assert!(
        html.contains(r#"action="/auth/logout""#),
        "body was: {html}"
    );
    assert!(html.contains("로그아웃"), "body was: {html}");
    assert!(html.contains("<form"), "줄이기 폼: {html}");
    assert!(html.contains("내 단축 링크"), "body was: {html}");
}

#[tokio::test]
async fn kakao_callback_without_matching_state_or_code_does_not_sign_in() {
    let (app, _db) = common::router().await;
    let login = get(&app, "/auth/kakao/login", None).await;
    let state_cookie = cookie_pair(&set_cookie(&login, "kakao_state").unwrap());
    let state = state_cookie.trim_start_matches("kakao_state=").to_string();

    for (path, cookie) in [
        // state가 쿠키와 다르다.
        (
            "/auth/kakao/callback?code=7:친구&state=other".to_string(),
            Some(state_cookie.clone()),
        ),
        // state 쿠키가 없다.
        (
            format!("/auth/kakao/callback?code=7:친구&state={state}"),
            None,
        ),
        // 사용자가 동의를 취소했다.
        (
            format!("/auth/kakao/callback?error=access_denied&state={state}"),
            Some(state_cookie.clone()),
        ),
        // 카카오가 코드를 받아 주지 않았다.
        (
            format!("/auth/kakao/callback?code=broken&state={state}"),
            Some(state_cookie.clone()),
        ),
    ] {
        let res = get(&app, &path, cookie.as_deref()).await;
        assert_eq!(res.status(), StatusCode::FOUND, "{path}");
        assert_eq!(res.headers()[header::LOCATION], "/", "{path}");
        assert_eq!(set_cookie(&res, "session"), None, "{path}");
    }
}

#[tokio::test]
async fn logout_ends_the_session_and_home_shows_the_login_screen_again() {
    let (app, _db) = common::router().await;
    let session = cookie_pair(
        &set_cookie(
            &sign_in_through_kakao(&app, "7:카카오친구").await,
            "session",
        )
        .unwrap(),
    );

    let res = app
        .clone()
        .oneshot(
            Request::post("/auth/logout")
                .header(header::COOKIE, &session)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::FOUND);
    assert_eq!(res.headers()[header::LOCATION], "/");
    assert!(set_cookie(&res, "session").unwrap().contains("Max-Age=0"));
    let html = home_html(&app, Some(&session)).await;
    assert!(
        html.contains("카카오 로그인"),
        "예전 쿠키로는 로그인 화면: {html}"
    );
    assert!(!html.contains("카카오친구"), "body was: {html}");
}

#[tokio::test]
async fn session_cookie_is_secure_when_the_service_address_is_https() {
    let db = common::TestDb::new().await;
    let shortener = short_url::shortener::Shortener::open("https://sho.rt", &db.url)
        .await
        .unwrap();
    let app = common::router_with(shortener);

    let res = sign_in_through_kakao(&app, "7:친구").await;

    assert!(set_cookie(&res, "session").unwrap().contains("Secure"));
}

#[tokio::test]
async fn follow_and_stats_work_without_signing_in() {
    let db = common::TestDb::new().await;
    let shortener = common::shortener(&db).await;
    let alice = shortener.sign_in(1, "앨리스").await.unwrap();
    shortener.sign_in(2, "밥").await.unwrap();
    shortener
        .shorten(&alice, "https://example.com/a")
        .await
        .unwrap();
    shortener
        .shorten(&alice, "https://example.com/b")
        .await
        .unwrap();
    let app = common::router_with(shortener);

    let res = get(&app, "/1", None).await;
    assert_eq!(res.status(), StatusCode::FOUND);
    assert_eq!(res.headers()[header::LOCATION], "https://example.com/a");

    let res = get(&app, "/api/stats", None).await;
    assert_eq!(res.status(), StatusCode::OK);
    let body: serde_json::Value = serde_json::from_str(&common::body_text(res).await).unwrap();
    assert_eq!(body, serde_json::json!({ "users": 2, "short_links": 2 }));
}
