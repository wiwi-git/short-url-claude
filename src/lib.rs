use dioxus::prelude::*;

pub const SERVICE_NAME: &str = "short-url";

#[component]
pub fn App() -> Element {
    rsx! {
        document::Title { "{SERVICE_NAME}" }
        main {
            h1 { "{SERVICE_NAME}" }
        }
    }
}

/// 서버 라우터. Dioxus 앱(SSR, server function, 정적 자산) 위에
/// 이후 티켓의 커스텀 axum 라우트(예: `/{n}`)를 여기에 덧붙인다.
#[cfg(feature = "server")]
pub fn build_router() -> dioxus::server::axum::Router {
    dioxus::server::router(App)
}
