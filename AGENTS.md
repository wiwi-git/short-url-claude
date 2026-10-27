## 명령어

Rust 안정판 툴체인과 Dioxus 0.7 fullstack 프로젝트다.

- 설치: `rustup update stable && rustup target add wasm32-unknown-unknown && cargo install dioxus-cli --version 0.7.10 --locked`
- 개발 서버: `dx serve` (기본 `http://127.0.0.1:8080`, 클라이언트 wasm과 서버를 함께 빌드한다)
- 테스트: `cargo test` (테스트 빌드에서는 dev-dependency로 `server` 기능이 켜진다)

`dx`의 마이너 버전은 `Cargo.toml`의 `dioxus` 크레이트 버전(0.7.x)과 맞춘다. 커스텀 axum 라우트는 `src/lib.rs`의 `build_router`에 덧붙인다.

## Agent skills

### Issue tracker

Issues and specs live as local markdown files under `.scratch/<feature-slug>/`. See `docs/agents/issue-tracker.md`.

### Triage labels

Default five-role vocabulary (`needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`), recorded as a `Status:` line in each issue file. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: one `GLOSSARY.md` and `docs/adr/` at the repo root. See `docs/agents/domain.md`.
