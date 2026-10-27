use dioxus::fullstack::{AsStatusCode, StatusCode};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

pub mod shortener;

use shortener::Rejection;

pub const SERVICE_NAME: &str = "short-url";

#[component]
pub fn App() -> Element {
    let mut original_url = use_signal(String::new);
    let mut outcome = use_signal(|| None::<Result<String, ShortenError>>);

    rsx! {
        document::Title { "{SERVICE_NAME}" }
        main {
            h1 { "{SERVICE_NAME}" }
            form {
                onsubmit: move |e| async move {
                    e.prevent_default();
                    outcome.set(Some(shorten(original_url()).await));
                },
                input {
                    r#type: "text",
                    placeholder: "https://...",
                    value: "{original_url}",
                    oninput: move |e| original_url.set(e.value()),
                }
                button { r#type: "submit", "줄이기" }
            }
            match outcome() {
                Some(Ok(short_link)) => rsx! {
                    p {
                        a { href: "{short_link}", "{short_link}" }
                        button {
                            r#type: "button",
                            onclick: move |_| {
                                let short_link = short_link.clone();
                                async move { copy_to_clipboard(short_link).await }
                            },
                            "복사"
                        }
                    }
                },
                Some(Err(e)) => rsx! { p { class: "error", "{e}" } },
                None => rsx! {},
            }
        }
    }
}

async fn copy_to_clipboard(text: String) {
    let eval = document::eval("await navigator.clipboard.writeText(await dioxus.recv());");
    if eval.send(text).is_ok() {
        let _ = eval.await;
    }
}

/// Shorten 서버 함수가 실패한 이유. 화면은 이 값의 문구를 그대로 보여 준다.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ShortenError {
    Rejected(Rejection),
    Server(String),
}

impl std::fmt::Display for ShortenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShortenError::Rejected(rejection) => rejection.fmt(f),
            ShortenError::Server(_) => f.write_str("서버와 통신하지 못했어요"),
        }
    }
}

impl std::error::Error for ShortenError {}

impl From<ServerFnError> for ShortenError {
    fn from(e: ServerFnError) -> Self {
        ShortenError::Server(e.to_string())
    }
}

impl AsStatusCode for ShortenError {
    fn as_status_code(&self) -> StatusCode {
        match self {
            ShortenError::Rejected(Rejection::StorageUnavailable) => {
                StatusCode::SERVICE_UNAVAILABLE
            }
            ShortenError::Server(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[cfg(feature = "server")]
type ShortenerExtension = dioxus::server::axum::Extension<std::sync::Arc<shortener::Shortener>>;

/// Original URL을 Shorten해 Short Link를 돌려준다.
#[post("/api/shorten", shortener: ShortenerExtension)]
pub async fn shorten(original_url: String) -> Result<String, ShortenError> {
    shortener
        .shorten(&original_url)
        .map_err(ShortenError::Rejected)
}

/// `BASE_URL`(기본 `http://localhost:8080`)과 `DATABASE_PATH`(기본 `short-url.db`)로
/// Shortener를 연다. 저장소 파일이 없으면 만든다.
#[cfg(feature = "server")]
pub fn shortener_from_env() -> Result<shortener::Shortener, rusqlite::Error> {
    let base_url =
        std::env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let db_path = std::env::var("DATABASE_PATH").unwrap_or_else(|_| "short-url.db".to_string());
    shortener::Shortener::open(&base_url, db_path)
}

/// 서버 라우터. Dioxus 앱(SSR, server function, 정적 자산)에 Follow 라우트 `/{n}`을 더한다.
#[cfg(feature = "server")]
pub fn build_router(shortener: shortener::Shortener) -> dioxus::server::axum::Router {
    use dioxus::server::axum::{routing::get, Extension};
    dioxus::server::router(App)
        .route("/{link_number}", get(follow))
        .layer(Extension(std::sync::Arc::new(shortener)))
}

#[cfg(feature = "server")]
async fn follow(
    dioxus::server::axum::Extension(shortener): ShortenerExtension,
    dioxus::server::axum::extract::Path(path): dioxus::server::axum::extract::Path<String>,
) -> dioxus::server::axum::response::Response {
    use dioxus::server::axum::{
        http::header::LOCATION,
        response::{Html, IntoResponse},
    };
    match shortener.follow(&path) {
        Some(original_url) => (StatusCode::FOUND, [(LOCATION, original_url)]).into_response(),
        None => (StatusCode::NOT_FOUND, Html(NOT_FOUND_PAGE)).into_response(),
    }
}

/// 없는 Short Link나 올바른 Link Number가 아닌 경로에 보여 주는 404 페이지.
#[cfg(feature = "server")]
const NOT_FOUND_PAGE: &str = r#"<!DOCTYPE html>
<html lang="ko">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>없는 링크 - short-url</title>
</head>
<body>
<main>
<h1>없는 링크예요</h1>
<p>이 Short Link는 발급된 적이 없어요. 주소를 다시 확인해 주세요.</p>
<p><a href="/">첫 화면으로 가서 직접 줄이기</a></p>
</main>
</body>
</html>
"#;
