# 02: Shorten과 Follow 기본 흐름 (tracer bullet)

**What to build:** 방문자가 첫 화면에서 올바른 Original URL을 넣고 "줄이기"를 누르면 `BASE_URL/1`, `/2`, ... 형태의 Short Link와 "복사" 버튼을 받는다. 그 Short Link를 열면 302로 Original URL에 도착한다. Short Link는 SQLite 파일에 저장되어 서버를 재시작해도 남는다. 명세의 Shortener 모듈(Shorten, Follow 두 인터페이스)과 두 테스트 경계를 처음으로 세우는 티켓이다. 명세: `.scratch/short-url-mvp/spec.md`

**Blocked by:** 01 (개발 환경과 빈 Dioxus fullstack 앱)

**Status:** resolved

- [x] Shortener 모듈이 Shorten(입력 문자열 → Short Link)과 Follow(경로 조각 → Original URL 또는 없음)를 공개한다
- [x] Link Number는 1부터 차례로 발급된다 (ADR 0001)
- [x] Short Link 앞부분은 `BASE_URL` 환경 변수를 따르고, 없으면 `http://localhost:8080`이다
- [x] SQLite 저장소가 서버 시작 시 없으면 자동으로 준비된다
- [x] 첫 화면: 입력칸, "줄이기" 버튼, 결과(Short Link + "복사" 버튼)
- [ ] 복사 버튼이 Short Link를 클립보드에 넣는다 (버튼은 `navigator.clipboard.writeText`를 오류 없이 호출했지만, 확인용 브라우저가 클립보드 읽기를 막아 실제 내용은 확인하지 못했다. 실제 브라우저에서 붙여넣기로 확인 필요)
- [x] `GET /{Link Number}`가 302와 `Location: <Original URL>`을 돌려준다
- [x] 경계 1 테스트: 임시 SQLite로 Shorten → Follow 왕복, 번호가 1, 2, 3 차례로 발급됨
- [x] 경계 2 테스트: 실제 서버에 Shorten 서버 함수 호출 후 `GET /{번호}`가 302와 올바른 `Location`
- [x] 직접 실행해서 줄이기 → 복사 → 브라우저로 Short Link 열기까지 확인했다 (복사는 위 항목 참고)

## Comments

- 구현: `src/shortener.rs`(Shortener, Rejection), `src/lib.rs`(`shorten` 서버 함수 `POST /api/shorten` 본문 `{"original_url": ...}`, `GET /{link_number}`). 저장소 파일은 `DATABASE_PATH`(기본 `short-url.db`, git 제외).
- 확인: `dx serve`에서 curl로 줄이기 → `GET /1`이 302와 Location, 서버 재시작 뒤에도 `/1`, `/2`가 그대로 이동하고 다음 번호 3이 발급됨. 브라우저 화면에서 줄이기 → 결과 표시 → Short Link 열기로 Original URL 도착.
