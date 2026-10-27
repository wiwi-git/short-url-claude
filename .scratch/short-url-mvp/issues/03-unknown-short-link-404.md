# 03: 없는 Short Link는 404

**What to build:** Short Link를 받은 사람이 발급된 적 없는 번호나 올바른 Link Number가 아닌 주소를 열면, "없는 링크"라고 알려 주고 첫 화면으로 가는 링크가 있는 404 페이지를 본다. 명세: `.scratch/short-url-mvp/spec.md`

**Blocked by:** 02 (Shorten과 Follow 기본 흐름)

**Status:** resolved

- [x] Follow가 다음 경로 조각에 "없음"을 돌려준다: 발급되지 않은 번호, `0`, 앞자리 0(`007`), 숫자 아님(`abc`), 정수 범위를 넘는 아주 큰 숫자
- [x] 위 경우 서버가 404 상태 코드와 404 페이지를 돌려준다
- [x] 404 페이지에 첫 화면으로 가는 링크가 있다
- [x] 경계 1 테스트: 위 경로 조각 각각에 대해 Follow가 "없음"
- [x] 경계 2 테스트: 없는 번호와 잘못된 경로가 404

## Comments

- 구현: `src/shortener.rs`의 Follow가 1~9로 시작하는 10진 숫자만 Link Number로 읽는다(앞자리 0, `+`/`-` 부호, 빈 조각, i64 범위 초과는 "없음"). `src/lib.rs`의 `follow` 핸들러가 "없음"이면 404와 정적 HTML 404 페이지(`NOT_FOUND_PAGE`, "없는 링크예요" + `href="/"` 링크)를 돌려준다. 404 페이지는 Dioxus 화면이 아니라 서버 상수 HTML이다.
- 테스트: `tests/shortener.rs`(경계 1, 없는 번호와 잘못된 조각 목록), `tests/unknown_short_link_http.rs`(경계 2, `/1` 미발급·`/007`·`/abc`가 404 + HTML 페이지).
- 확인: `dx serve`에서 Shorten 뒤 `/1`은 302, `/2`·`/0`·`/007`·`/abc`·아주 큰 숫자는 404 `text/html`. 브라우저에서 404 페이지를 보고 첫 화면 링크로 `/`에 도착.
