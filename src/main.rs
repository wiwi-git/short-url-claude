fn main() {
    #[cfg(not(feature = "server"))]
    dioxus::launch(short_url::App);

    #[cfg(feature = "server")]
    dioxus::serve(|| async move {
        let kakao = std::sync::Arc::new(short_url::kakao_from_env()?);
        Ok(short_url::build_router(
            short_url::shortener_from_env().await?,
            kakao,
        ))
    });
}
