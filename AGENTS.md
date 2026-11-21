## 명령어

Rust 안정판 툴체인과 Dioxus 0.7 fullstack 프로젝트다.

- 설치: `rustup update stable && rustup target add wasm32-unknown-unknown && cargo install dioxus-cli --version 0.7.10 --locked`
- 개발 서버: `dx serve` (`http://localhost:8080`, `BASE_URL` 기본값과 같은 주소다. 클라이언트 wasm과 서버를 함께 빌드한다). 저장소는 SQLite 파일 하나다(ADR 0004). `DATABASE_URL`이 없으면 작업 폴더의 `short-url.db`(`sqlite://short-url.db`)를 쓰고, 파일과 테이블이 없으면 만든다. 카카오 로그인 키 `KAKAO_REST_API_KEY`, `KAKAO_CLIENT_SECRET`이 환경에 있어야 서버가 뜬다(설정은 `docs/deploy/duse.md`). 카카오 Redirect URI에 `http://localhost:8080/auth/kakao/callback`을 등록해 두면 로컬에서도 실제로 로그인할 수 있다
- 배포: `scripts/deploy.sh` (duse 서버, `docs/deploy/duse.md`)
- 테스트: `cargo test` (카카오는 가짜를 꽂으므로 키가 필요 없다. 테스트 빌드에서는 dev-dependency로 `server` 기능이 켜진다. 테스트마다 임시 폴더에 새 SQLite 파일을 만들고 끝나면 지운다. 따로 띄울 DB가 없다)

`dx`의 마이너 버전은 `Cargo.toml`의 `dioxus` 크레이트 버전(0.7.x)과 맞춘다. 커스텀 axum 라우트는 `src/lib.rs`의 `build_router`에 덧붙인다.

## 배포 서버

운영 서버(duse.dev, OCI 도쿄 ARM VM)에 무언가를 올리거나 고치기 전에 `/Users/widaeyeon/claude-test/web-broadcast-proj/duse-server.md`를 먼저 읽고 그 규칙을 따른다. 그 문서가 서버 구성의 단일 원본이다(방송 프로젝트 폴더에만 있고 git이 추적하지 않는다). 서버 IP, 접속 방법, 비밀값을 이 저장소에 옮겨 적지 않는다. 서버 구성을 바꾸면 그 문서를 고친다.

## Agent skills

### Issue tracker

Issues and specs live as local markdown files under `.scratch/<feature-slug>/`. See `docs/agents/issue-tracker.md`.

### Triage labels

Default five-role vocabulary (`needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`), recorded as a `Status:` line in each issue file. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: one `GLOSSARY.md` and `docs/adr/` at the repo root. See `docs/agents/domain.md`.
