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
    /// 입력이 거절됐다 (400).
    Rejected(Rejection),
    /// 저장소를 쓸 수 없다 (503).
    StorageUnavailable,
    /// 서버 함수 호출 자체가 실패했다.
    Server(String),
}

impl std::fmt::Display for ShortenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShortenError::Rejected(rejection) => rejection.fmt(f),
            ShortenError::StorageUnavailable => {
                f.write_str("지금은 줄일 수 없어요. 잠시 뒤 다시 시도해 주세요")
            }
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
            ShortenError::Rejected(_) => StatusCode::BAD_REQUEST,
            ShortenError::StorageUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            ShortenError::Server(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[cfg(feature = "server")]
use dioxus::server::axum::{
    extract::Path,
    http::header::LOCATION,
    response::{Html, IntoResponse, Response},
    routing::get,
    Extension, Router,
};

#[cfg(feature = "server")]
type ShortenerExtension = Extension<std::sync::Arc<shortener::Shortener>>;

/// Original URL을 Shorten해 Short Link를 돌려준다.
#[post("/api/shorten", shortener: ShortenerExtension)]
pub async fn shorten(original_url: String) -> Result<String, ShortenError> {
    shortener.shorten(&original_url).await.map_err(|e| match e {
        shortener::ShortenError::Rejected(rejection) => ShortenError::Rejected(rejection),
        shortener::ShortenError::Storage(e) => {
            dioxus::logger::tracing::error!("Shorten 실패: {e}");
            ShortenError::StorageUnavailable
        }
    })
}

/// `DATABASE_URL`이 없을 때 붙는 로컬 개발용 Postgres (AGENTS.md의 docker 명령으로 띄운다).
#[cfg(feature = "server")]
pub const LOCAL_DATABASE_URL: &str = "postgres://postgres:postgres@localhost:5432/short_url";

/// `BASE_URL`(기본 `http://localhost:8080`)과 `DATABASE_URL`(기본 [`LOCAL_DATABASE_URL`])로
/// Shortener를 연다. 테이블이 없으면 만든다. 배포 환경에서는 `DATABASE_URL`을 꼭 설정한다.
#[cfg(feature = "server")]
pub async fn shortener_from_env() -> Result<shortener::Shortener, shortener::OpenError> {
    let base_url =
        std::env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| LOCAL_DATABASE_URL.to_string());
    shortener::Shortener::open(&base_url, &database_url).await
}

/// 서버 라우터. `/`는 첫 화면(SSR), `/{경로 조각 하나}`는 Follow, 그 밖의 주소는 404 페이지다.
/// `dioxus::server::router`는 모든 GET을 SSR로 받는 fallback을 달기 때문에 쓰지 않고,
/// 같은 부품(server function, 정적 자산)으로 직접 조립한다.
#[cfg(feature = "server")]
pub fn build_router(shortener: shortener::Shortener) -> Router {
    use dioxus::server::{render_handler, DioxusRouterExt, FullstackState, ServeConfig};
    Router::new()
        .register_server_functions()
        .serve_static_assets()
        .route("/", get(render_handler))
        .route("/{link_number_text}", get(follow))
        .fallback(not_found)
        .with_state(FullstackState::new(ServeConfig::new(), App))
        .layer(Extension(std::sync::Arc::new(shortener)))
}

#[cfg(feature = "server")]
async fn follow(
    Extension(shortener): ShortenerExtension,
    Path(link_number_text): Path<String>,
) -> Response {
    match shortener.follow(&link_number_text).await {
        Ok(Some(original_url)) => (StatusCode::FOUND, [(LOCATION, original_url)]).into_response(),
        Ok(None) => not_found().await,
        Err(e) => {
            dioxus::logger::tracing::error!("Follow 실패: {e}");
            StatusCode::SERVICE_UNAVAILABLE.into_response()
        }
    }
}

#[cfg(feature = "server")]
async fn not_found() -> Response {
    (StatusCode::NOT_FOUND, Html(not_found_page())).into_response()
}

/// 없는 Short Link나 올바른 Short Link가 아닌 주소에 보여 주는 404 페이지.
#[cfg(feature = "server")]
fn not_found_page() -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="ko">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>없는 링크 - {SERVICE_NAME}</title>
</head>
<body>
<main>
<h1>없는 링크예요</h1>
<p>이 Short Link는 발급된 적이 없어요. 주소를 다시 확인해 주세요.</p>
<p><a href="/">첫 화면으로 가서 직접 줄이기</a></p>
</main>
</body>
</html>
"#
    )
}
