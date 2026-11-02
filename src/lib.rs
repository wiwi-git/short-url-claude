use dioxus::fullstack::{AsStatusCode, StatusCode};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
pub mod kakao;
pub mod shortener;

use shortener::{Rejection, ShortLinkPage};

pub const SERVICE_NAME: &str = "short-url";

/// 첫 화면 스타일. 가운데 정렬한 한 단 구성에, 눈이 편한 차분한 청록-파랑 계열을 쓴다.
/// 카카오 로그인 버튼만 카카오 안내에 맞춘 노란색이다.
const STYLE: &str = r#"
:root {
  --bg: #f5f8f9;
  --card: #ffffff;
  --text: #24323a;
  --muted: #6b7c85;
  --border: #e1e9ec;
  --link: #2a7f93;
  --link-hover: #1d5f70;
  --accent: linear-gradient(135deg, #3a95a8, #4a7fb5);
  --title: linear-gradient(135deg, #15586a, #22487a);
  --error: #c8504f;
}
@media (prefers-color-scheme: dark) {
  :root {
    --bg: #141a1d;
    --card: #1c2428;
    --text: #e3ebee;
    --muted: #93a4ab;
    --border: #2b373c;
    --link: #79bccb;
    --link-hover: #a3d4df;
    --accent: linear-gradient(135deg, #3f8e9f, #4d79a8);
    --title: linear-gradient(135deg, #5fb3c6, #6d9fd6);
    --error: #e07a78;
  }
}
* { box-sizing: border-box; }
body {
  margin: 0;
  background: var(--bg);
  color: var(--text);
  font-family: -apple-system, BlinkMacSystemFont, "Apple SD Gothic Neo", "Pretendard", "Noto Sans KR", sans-serif;
}
main {
  max-width: 560px;
  margin: 0 auto;
  padding: 56px 16px;
  text-align: center;
}
h1 {
  margin: 0 0 28px;
  font-size: 2.4rem;
  background: var(--title);
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
}
h2 { margin: 0 0 12px; font-size: 1.1rem; color: var(--muted); font-weight: 600; }
a { color: var(--link); font-weight: 600; text-decoration: none; }
a:hover { color: var(--link-hover); text-decoration: underline; }
form { display: flex; gap: 8px; }
input {
  flex: 1;
  min-width: 0;
  padding: 12px 14px;
  border: 1px solid var(--border);
  border-radius: 12px;
  background: var(--card);
  color: var(--text);
  font-size: 1rem;
}
input:focus { outline: 2px solid var(--link); outline-offset: 1px; }
button {
  padding: 12px 18px;
  border: 0;
  border-radius: 12px;
  background: var(--accent);
  color: #fff;
  font-size: 1rem;
  font-weight: 600;
  cursor: pointer;
}
button:hover { filter: brightness(1.08); }
button:disabled { opacity: 0.4; cursor: default; filter: none; }
.result {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  margin: 20px 0 0;
  padding: 14px;
  border: 1px solid var(--border);
  border-radius: 12px;
  background: var(--card);
  word-break: break-all;
}
.result button { padding: 8px 14px; font-size: 0.9rem; }
.error { margin: 16px 0 0; color: var(--error); }
section {
  margin-top: 40px;
  padding: 20px;
  border: 1px solid var(--border);
  border-radius: 16px;
  background: var(--card);
}
ul { list-style: none; margin: 0; padding: 0; }
li {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 8px 0;
  border-bottom: 1px solid var(--border);
}
li:last-child { border-bottom: 0; }
nav { display: flex; justify-content: center; gap: 8px; margin-top: 16px; }
nav button { padding: 8px 16px; font-size: 0.9rem; }
.empty { color: var(--muted); margin: 0; }
.quiet {
  padding: 4px 10px;
  border: 1px solid var(--border);
  background: transparent;
  color: var(--muted);
  font-size: 0.8rem;
}
.login {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 16px;
}
.login h1 { margin-bottom: 36px; }
a.kakao {
  display: inline-block;
  padding: 14px 32px;
  border-radius: 12px;
  background: #fee500;
  color: rgba(0, 0, 0, 0.85);
}
a.kakao:hover { color: rgba(0, 0, 0, 0.85); text-decoration: none; filter: brightness(0.97); }
.account {
  position: absolute;
  top: 16px;
  right: 16px;
  display: flex;
  align-items: center;
  gap: 10px;
  color: var(--muted);
  font-weight: 600;
}
.account form { display: block; }
.account .quiet { font-size: 0.85rem; padding: 6px 12px; }
"#;

#[component]
pub fn App() -> Element {
    rsx! {
        document::Title { "{SERVICE_NAME}" }
        document::Style { {STYLE} }
        SuspenseBoundary {
            fallback: |_| rsx! {},
            Home {}
        }
    }
}

/// `/` 한 주소에서 로그인 상태에 따라 로그인 화면과 줄이기 화면 중 하나를 보여 준다.
#[component]
fn Home() -> Element {
    let me = use_server_future(current_user)?;
    match me() {
        Some(Ok(Some(nickname))) => rsx! { ShortenScreen { nickname } },
        Some(Ok(None)) => rsx! { LoginScreen {} },
        Some(Err(_)) => rsx! {
            main {
                h1 { "{SERVICE_NAME}" }
                p { class: "error", "지금은 열 수 없어요. 잠시 뒤 다시 시도해 주세요" }
            }
        },
        None => rsx! {},
    }
}

/// 로그인 전 화면: 가운데에 사이트 이름과 카카오 로그인 버튼만 있다.
#[component]
fn LoginScreen() -> Element {
    rsx! {
        main { class: "login",
            h1 { "{SERVICE_NAME}" }
            a { class: "kakao", href: "/auth/kakao/login", "카카오 로그인" }
        }
    }
}

/// 로그인 후 화면: 줄이기와 My Links. 오른쪽 위에 닉네임과 로그아웃 버튼이 있다.
#[component]
fn ShortenScreen(nickname: String) -> Element {
    let mut original_url = use_signal(String::new);
    let mut outcome = use_signal(|| None::<Result<String, ShortenError>>);
    // My Links 목록이 보여 주는 페이지(0부터)와, 줄이기나 Remove로 목록이 바뀌었을 때 다시 부르는 신호.
    let mut list_page = use_signal(|| 0u32);
    let mut list_version = use_signal(|| 0u32);

    rsx! {
        header { class: "account",
            span { "{nickname}" }
            form { method: "post", action: "/auth/logout",
                button { class: "quiet", r#type: "submit", "로그아웃" }
            }
        }
        main {
            h1 { "{SERVICE_NAME}" }
            form {
                onsubmit: move |e| async move {
                    e.prevent_default();
                    let result = shorten(original_url()).await;
                    if result.is_ok() {
                        list_page.set(0);
                        list_version += 1;
                    }
                    outcome.set(Some(result));
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
                    p { class: "result",
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
            SuspenseBoundary {
                fallback: |_| rsx! { p { "내 단축 링크를 불러오는 중이에요" } },
                ShortLinkList { page: list_page, version: list_version }
            }
        }
    }
}

/// 로그인한 User의 My Links. 담은 순서의 최신순, 한 페이지 50개, "이전"/"다음"으로 넘긴다 (주소는 그대로).
/// 각 Short Link 옆의 "삭제"는 Remove(내 링크에서 빼기)다.
#[component]
fn ShortLinkList(page: Signal<u32>, version: Signal<u32>) -> Element {
    let list = use_server_future(move || {
        let page = page();
        version();
        short_links(page)
    })?;

    let content = match list() {
        Some(Ok(ShortLinkPage {
            short_links,
            has_next,
        })) => {
            let current = page();
            rsx! {
                if short_links.is_empty() {
                    p { class: "empty", "아직 만든 단축 링크가 없어요" }
                } else {
                    ul {
                        for link in short_links {
                            li { key: "{link.link_number}",
                                a { href: "{link.short_link}", "{link.short_link}" }
                                button {
                                    class: "quiet",
                                    r#type: "button",
                                    onclick: move |_| async move {
                                        if remove(link.link_number).await.is_ok() {
                                            version += 1;
                                        }
                                    },
                                    "삭제"
                                }
                            }
                        }
                    }
                }
                if current > 0 || has_next {
                    nav {
                        button {
                            r#type: "button",
                            disabled: current == 0,
                            onclick: move |_| page -= 1,
                            "이전"
                        }
                        button {
                            r#type: "button",
                            disabled: !has_next,
                            onclick: move |_| page += 1,
                            "다음"
                        }
                    }
                }
            }
        }
        Some(Err(_)) => rsx! { p { class: "error", "내 단축 링크를 불러오지 못했어요" } },
        None => rsx! {},
    };

    rsx! {
        section {
            h2 { "내 단축 링크" }
            {content}
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
    /// 로그인하지 않았거나 세션이 만료됐다 (401).
    SignedOut,
    /// 저장소를 쓸 수 없다 (503).
    StorageUnavailable,
    /// 서버 함수 호출 자체가 실패했다.
    Server(String),
}

impl std::fmt::Display for ShortenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShortenError::Rejected(rejection) => rejection.fmt(f),
            ShortenError::SignedOut => {
                f.write_str("로그인이 풀렸어요. 새로 고침한 뒤 다시 로그인해 주세요")
            }
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
            ShortenError::SignedOut => StatusCode::UNAUTHORIZED,
            ShortenError::StorageUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            ShortenError::Server(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[cfg(feature = "server")]
use dioxus::server::axum::{
    extract::{Path, Query},
    http::{
        header::{COOKIE, LOCATION, SET_COOKIE},
        HeaderMap,
    },
    response::{AppendHeaders, Html, IntoResponse, Response},
    routing::{get, post},
    Extension, Json, Router,
};
#[cfg(feature = "server")]
use shortener::{Shortener, User};

#[cfg(feature = "server")]
type ShortenerExtension = Extension<std::sync::Arc<Shortener>>;
#[cfg(feature = "server")]
type KakaoExtension = Extension<std::sync::Arc<dyn kakao::Kakao>>;

/// 세션 토큰을 담는 쿠키 이름.
#[cfg(feature = "server")]
const SESSION_COOKIE: &str = "session";
/// 카카오 로그인 state를 콜백까지 들고 있는 쿠키 이름.
#[cfg(feature = "server")]
const STATE_COOKIE: &str = "kakao_state";

/// 요청 쿠키에서 `name`의 값을 찾는다.
#[cfg(feature = "server")]
fn cookie_value(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get_all(COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(';'))
        .find_map(|pair| {
            let (key, value) = pair.trim().split_once('=')?;
            (key == name && !value.is_empty()).then(|| value.to_string())
        })
}

/// 세션 쿠키의 User. 쿠키가 없거나 세션이 만료됐으면 `Ok(None)`.
#[cfg(feature = "server")]
async fn signed_in_user(
    shortener: &Shortener,
    headers: &HeaderMap,
) -> Result<Option<User>, shortener::StorageError> {
    match cookie_value(headers, SESSION_COOKIE) {
        Some(token) => shortener.user_for_session(&token).await,
        None => Ok(None),
    }
}

/// 서버 함수가 상태 코드와 함께 돌려주는 오류.
#[cfg(feature = "server")]
fn server_error(code: StatusCode, message: &str) -> ServerFnError {
    ServerFnError::ServerError {
        message: message.to_string(),
        code: code.as_u16(),
        details: None,
    }
}

/// 서버 함수에서 로그인한 User를 찾는다. 없으면 401, 저장소 오류면 503.
#[cfg(feature = "server")]
async fn require_user(shortener: &Shortener, headers: &HeaderMap) -> Result<User, ServerFnError> {
    match signed_in_user(shortener, headers).await {
        Ok(Some(user)) => Ok(user),
        Ok(None) => Err(server_error(StatusCode::UNAUTHORIZED, "로그인이 필요해요")),
        Err(e) => {
            dioxus::logger::tracing::error!("세션 확인 실패: {e}");
            Err(server_error(
                StatusCode::SERVICE_UNAVAILABLE,
                "지금은 로그인을 확인할 수 없어요",
            ))
        }
    }
}

/// 지금 로그인한 User의 닉네임. 로그인하지 않았으면 `None`.
#[post("/api/me", shortener: ShortenerExtension, headers: HeaderMap)]
pub async fn current_user() -> Result<Option<String>, ServerFnError> {
    match signed_in_user(&shortener, &headers).await {
        Ok(user) => Ok(user.map(|user| user.nickname)),
        Err(e) => {
            dioxus::logger::tracing::error!("세션 확인 실패: {e}");
            Err(server_error(
                StatusCode::SERVICE_UNAVAILABLE,
                "지금은 로그인을 확인할 수 없어요",
            ))
        }
    }
}

/// 로그인한 User가 Original URL을 Shorten해 Short Link를 돌려받는다.
#[post("/api/shorten", shortener: ShortenerExtension, headers: HeaderMap)]
pub async fn shorten(original_url: String) -> Result<String, ShortenError> {
    let user = match signed_in_user(&shortener, &headers).await {
        Ok(Some(user)) => user,
        Ok(None) => return Err(ShortenError::SignedOut),
        Err(e) => {
            dioxus::logger::tracing::error!("세션 확인 실패: {e}");
            return Err(ShortenError::StorageUnavailable);
        }
    };
    shortener
        .shorten(&user, &original_url)
        .await
        .map_err(|e| match e {
            shortener::ShortenError::Rejected(rejection) => ShortenError::Rejected(rejection),
            shortener::ShortenError::Storage(e) => {
                dioxus::logger::tracing::error!("Shorten 실패: {e}");
                ShortenError::StorageUnavailable
            }
        })
}

/// 로그인한 User의 My Links에서 `page`번째 페이지(0부터)를 최신순으로 돌려준다.
#[post("/api/short-links", shortener: ShortenerExtension, headers: HeaderMap)]
pub async fn short_links(page: u32) -> Result<ShortLinkPage, ServerFnError> {
    let user = require_user(&shortener, &headers).await?;
    shortener.my_links(&user, page).await.map_err(|e| {
        dioxus::logger::tracing::error!("My Links 읽기 실패: {e}");
        server_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "내 단축 링크를 불러오지 못했어요",
        )
    })
}

/// 로그인한 User의 My Links에서 Short Link를 뺀다(Remove).
#[post("/api/remove", shortener: ShortenerExtension, headers: HeaderMap)]
pub async fn remove(link_number: i64) -> Result<(), ServerFnError> {
    let user = require_user(&shortener, &headers).await?;
    shortener.remove(&user, link_number).await.map_err(|e| {
        dioxus::logger::tracing::error!("Remove 실패: {e}");
        server_error(StatusCode::SERVICE_UNAVAILABLE, "지금은 뺄 수 없어요")
    })
}

/// `DATABASE_URL`이 없을 때 붙는 로컬 개발용 Postgres (AGENTS.md의 docker 명령으로 띄운다).
#[cfg(feature = "server")]
pub const LOCAL_DATABASE_URL: &str = "postgres://postgres:postgres@localhost:5432/short_url";

/// `BASE_URL` 환경 변수. 없으면 로컬 개발 주소 `http://localhost:8080`.
#[cfg(feature = "server")]
fn base_url_from_env() -> String {
    std::env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string())
}

/// `BASE_URL`(기본 `http://localhost:8080`)과 `DATABASE_URL`(기본 [`LOCAL_DATABASE_URL`])로
/// Shortener를 연다. 테이블이 없으면 만든다. 배포 환경에서는 `DATABASE_URL`을 꼭 설정한다.
#[cfg(feature = "server")]
pub async fn shortener_from_env() -> Result<Shortener, shortener::OpenError> {
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| LOCAL_DATABASE_URL.to_string());
    Shortener::open(&base_url_from_env(), &database_url).await
}

/// 꼭 있어야 하는 환경 변수가 없다.
#[cfg(feature = "server")]
#[derive(Debug)]
pub struct MissingEnv(&'static str);

#[cfg(feature = "server")]
impl std::fmt::Display for MissingEnv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "환경 변수 {}가 없어요. 카카오 디벨로퍼스의 앱 키를 넣어 주세요 (docs/deploy/render.md)",
            self.0
        )
    }
}

#[cfg(feature = "server")]
impl std::error::Error for MissingEnv {}

/// `KAKAO_REST_API_KEY`와 `KAKAO_CLIENT_SECRET`으로 카카오 로그인을 준비한다.
/// Redirect URI는 `BASE_URL` + `/auth/kakao/callback`이다. 키가 없으면 실패한다.
#[cfg(feature = "server")]
pub fn kakao_from_env() -> Result<kakao::KakaoApi, MissingEnv> {
    let var = |name: &'static str| {
        std::env::var(name)
            .ok()
            .filter(|v| !v.trim().is_empty())
            .ok_or(MissingEnv(name))
    };
    let redirect_uri = format!(
        "{}/auth/kakao/callback",
        base_url_from_env().trim_end_matches('/')
    );
    Ok(kakao::KakaoApi::new(
        var("KAKAO_REST_API_KEY")?,
        var("KAKAO_CLIENT_SECRET")?,
        redirect_uri,
    ))
}

/// 서버 라우터. `/`는 첫 화면(SSR, 로그인 상태에 따라 로그인 화면 또는 줄이기 화면),
/// `/auth/...`는 카카오 로그인과 로그아웃, `/api/stats`는 통계, `/{경로 조각 하나}`는 Follow,
/// 그 밖의 주소는 404 페이지다.
/// `dioxus::server::router`는 모든 GET을 SSR로 받는 fallback을 달기 때문에 쓰지 않고,
/// 같은 부품(server function, 정적 자산)으로 직접 조립한다.
#[cfg(feature = "server")]
pub fn build_router(shortener: Shortener, kakao: std::sync::Arc<dyn kakao::Kakao>) -> Router {
    use dioxus::server::{render_handler, DioxusRouterExt, FullstackState, ServeConfig};
    Router::new()
        .register_server_functions()
        .serve_static_assets()
        .route("/", get(render_handler))
        .route("/auth/kakao/login", get(kakao_login))
        .route("/auth/kakao/callback", get(kakao_callback))
        .route("/auth/logout", post(logout))
        .route("/api/stats", get(stats))
        .route("/{link_number_text}", get(follow))
        .fallback(not_found)
        .with_state(FullstackState::new(ServeConfig::new(), App))
        .layer(Extension(std::sync::Arc::new(shortener)))
        .layer(Extension(kakao))
}

/// `Set-Cookie` 값. 서비스 주소가 https면 `Secure`를 붙인다. `max_age`가 0이면 쿠키를 지운다.
#[cfg(feature = "server")]
fn set_cookie(shortener: &Shortener, name: &str, value: &str, path: &str, max_age: u64) -> String {
    let secure = if shortener.base_url().starts_with("https://") {
        "; Secure"
    } else {
        ""
    };
    format!("{name}={value}; Path={path}; Max-Age={max_age}; HttpOnly; SameSite=Lax{secure}")
}

/// 카카오 로그인 state 쿠키가 사는 시간(초).
#[cfg(feature = "server")]
const STATE_MAX_AGE: u64 = 10 * 60;

/// 로그인 버튼: state를 쿠키에 넣고 카카오 인가 주소로 보낸다.
#[cfg(feature = "server")]
async fn kakao_login(
    Extension(shortener): ShortenerExtension,
    Extension(kakao): KakaoExtension,
) -> Response {
    let state = shortener::random_token();
    (
        StatusCode::FOUND,
        AppendHeaders([
            (
                SET_COOKIE,
                set_cookie(
                    &shortener,
                    STATE_COOKIE,
                    &state,
                    "/auth/kakao",
                    STATE_MAX_AGE,
                ),
            ),
            (LOCATION, kakao.login_url(&state)),
        ]),
    )
        .into_response()
}

/// 카카오 콜백: state가 맞고 인가 코드가 있으면 User로 로그인시키고 세션 쿠키를 심는다.
/// 취소, 거절, state 불일치, 카카오나 저장소 실패는 모두 세션 없이 `/`(로그인 화면)로 보낸다.
#[cfg(feature = "server")]
async fn kakao_callback(
    Extension(shortener): ShortenerExtension,
    Extension(kakao): KakaoExtension,
    Query(params): Query<std::collections::HashMap<String, String>>,
    headers: HeaderMap,
) -> Response {
    let clear_state = set_cookie(&shortener, STATE_COOKIE, "", "/auth/kakao", 0);
    let state_matches = matches!(
        (params.get("state"), cookie_value(&headers, STATE_COOKIE)),
        (Some(got), Some(expected)) if *got == expected
    );
    let token = match params.get("code") {
        Some(code) if state_matches => sign_in_with_kakao(&shortener, kakao.as_ref(), code).await,
        _ => None,
    };
    let mut cookies = vec![clear_state];
    if let Some(token) = token {
        cookies.push(set_cookie(
            &shortener,
            SESSION_COOKIE,
            &token,
            "/",
            shortener::SESSION_DAYS * 24 * 60 * 60,
        ));
    }
    redirect_home(cookies)
}

/// 인가 코드로 카카오 프로필을 받아 User로 로그인하고 세션 토큰을 돌려준다. 실패하면 `None`.
#[cfg(feature = "server")]
async fn sign_in_with_kakao(
    shortener: &Shortener,
    kakao: &dyn kakao::Kakao,
    code: &str,
) -> Option<String> {
    let profile = kakao
        .profile(code)
        .await
        .map_err(|e| dioxus::logger::tracing::error!("카카오 로그인 실패: {e}"))
        .ok()?;
    let session = async {
        let user = shortener
            .sign_in(profile.kakao_id, &profile.nickname)
            .await?;
        shortener.start_session(&user).await
    };
    session
        .await
        .map_err(|e| dioxus::logger::tracing::error!("로그인 저장 실패: {e}"))
        .ok()
}

/// 로그아웃: 세션을 닫고 쿠키를 지운 뒤 `/`(로그인 화면)로 보낸다.
#[cfg(feature = "server")]
async fn logout(Extension(shortener): ShortenerExtension, headers: HeaderMap) -> Response {
    if let Some(token) = cookie_value(&headers, SESSION_COOKIE) {
        if let Err(e) = shortener.end_session(&token).await {
            dioxus::logger::tracing::error!("로그아웃 실패: {e}");
        }
    }
    redirect_home(vec![set_cookie(&shortener, SESSION_COOKIE, "", "/", 0)])
}

/// 쿠키를 심으며 `/`로 302.
#[cfg(feature = "server")]
fn redirect_home(cookies: Vec<String>) -> Response {
    let mut res = (StatusCode::FOUND, [(LOCATION, "/")]).into_response();
    for cookie in cookies {
        if let Ok(value) = cookie.parse() {
            res.headers_mut().append(SET_COOKIE, value);
        }
    }
    res
}

/// 통계: 로그인 없이 User 수와 Short Link 수를 JSON으로 돌려준다.
#[cfg(feature = "server")]
async fn stats(Extension(shortener): ShortenerExtension) -> Response {
    match shortener.stats().await {
        Ok(stats) => Json(stats).into_response(),
        Err(e) => {
            dioxus::logger::tracing::error!("통계 읽기 실패: {e}");
            StatusCode::SERVICE_UNAVAILABLE.into_response()
        }
    }
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
<style>{STYLE}</style>
</head>
<body>
<main>
<h1>없는 링크예요</h1>
<p>이 Short Link는 없거나 지워졌어요. 주소를 다시 확인해 주세요.</p>
<p><a href="/">첫 화면으로</a></p>
</main>
</body>
</html>
"#
    )
}
