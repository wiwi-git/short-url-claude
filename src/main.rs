fn main() {
    #[cfg(not(feature = "server"))]
    dioxus::launch(short_url::App);

    #[cfg(feature = "server")]
    dioxus::serve(|| async move { Ok(short_url::build_router()) });
}
