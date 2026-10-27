//! HTTP 테스트(경계 2)가 함께 쓰는 준비 코드.

#![allow(dead_code)]

use dioxus::server::axum::{body::Body, http::Response, Router};
use http_body_util::BodyExt;
use short_url::shortener::Shortener;

/// 임시 SQLite를 붙인 실제 서버 라우터. 돌려준 `TempDir`가 살아 있는 동안 저장소가 남는다.
pub fn router() -> (Router, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let shortener = Shortener::open("http://localhost:8080", dir.path().join("links.db")).unwrap();
    (router_with(shortener), dir)
}

/// 주어진 Shortener로 서버 라우터를 만든다.
pub fn router_with(shortener: Shortener) -> Router {
    // dx 없이 실행하면 실행 파일 옆에 public/ 이 없어 dioxus가 패닉한다.
    // 빈 디렉터리를 정적 자산 경로로 지정한다.
    let public = std::env::temp_dir().join("short-url-test-public");
    std::fs::create_dir_all(&public).unwrap();
    std::env::set_var("DIOXUS_PUBLIC_PATH", &public);
    short_url::build_router(shortener)
}

/// 응답 본문을 문자열로 읽는다.
pub async fn body_text(res: Response<Body>) -> String {
    let body = res.into_body().collect().await.unwrap().to_bytes();
    String::from_utf8(body.to_vec()).unwrap()
}
