## 명령어

Rust 안정판 툴체인과 Dioxus 0.7 fullstack 프로젝트다.

- 설치: `rustup update stable && rustup target add wasm32-unknown-unknown && cargo install dioxus-cli --version 0.7.10 --locked`
- 로컬 DB: `docker run -d --name short-url-postgres -e POSTGRES_PASSWORD=postgres -e POSTGRES_DB=short_url -p 5432:5432 postgres:17` (한 번 만든 뒤에는 `docker start short-url-postgres`). 저장소는 Postgres뿐이다(ADR 0002). `dx serve`와 `cargo test` 모두 이 DB가 떠 있어야 한다
- 개발 서버: `dx serve` (`http://localhost:8080`, `BASE_URL` 기본값과 같은 주소다. 클라이언트 wasm과 서버를 함께 빌드한다). `DATABASE_URL`이 없으면 위 로컬 DB(`postgres://postgres:postgres@localhost:5432/short_url`)에 붙고, 테이블이 없으면 만든다
- 테스트: `cargo test` (테스트 빌드에서는 dev-dependency로 `server` 기능이 켜진다. 테스트마다 `TEST_DATABASE_URL`(기본값은 위 로컬 DB)에 새 데이터베이스를 만들고 끝나면 지운다)

`dx`의 마이너 버전은 `Cargo.toml`의 `dioxus` 크레이트 버전(0.7.x)과 맞춘다. 커스텀 axum 라우트는 `src/lib.rs`의 `build_router`에 덧붙인다.

## Agent skills

### Issue tracker

Issues and specs live as local markdown files under `.scratch/<feature-slug>/`. See `docs/agents/issue-tracker.md`.

### Triage labels

Default five-role vocabulary (`needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`), recorded as a `Status:` line in each issue file. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: one `GLOSSARY.md` and `docs/adr/` at the repo root. See `docs/agents/domain.md`.
