# 04: 잘못된 입력 거절

**What to build:** 방문자가 줄일 수 없는 입력으로 "줄이기"를 누르면, 무엇이 잘못됐는지 알려 주는 한국어 오류 문구를 보고, 입력칸의 내용은 그대로 남아 고칠 수 있다. 명세: `.scratch/short-url-mvp/spec.md`

**Blocked by:** 02 (Shorten과 Follow 기본 흐름)

**Status:** resolved

- [x] Shorten이 다음을 각각 다른 거절 사유로 돌려준다: 빈 입력(공백만 있는 입력 포함), URL 형식 아님, http/https가 아닌 스킴, 2048자 초과, 이 서비스 자기 주소(`BASE_URL`과 호스트가 같음)
- [x] 거절된 입력은 저장되지 않고 Link Number를 소모하지 않는다
- [x] 화면이 거절 사유마다 다른 한국어 오류 문구를 보여 주고, 입력칸 내용을 유지한다
- [x] 성공하면 오류 문구가 사라지고 새 결과가 이전 결과를 대체한다
- [x] 경계 1 테스트: 거절 사유 각각, 그리고 거절 뒤 다음 성공 Shorten의 번호가 건너뛰지 않음
- [x] 경계 2 테스트: Shorten 서버 함수가 거절 사유를 돌려줌

## Comments

- 구현: `src/shortener.rs`의 Shorten이 저장 전에 입력을 검사한다. 순서는 빈 입력(`trim` 뒤 빔) → 2048자 초과(`MAX_INPUT_CHARS`, 정리 전 문자 수) → `url` 크레이트로 절대 URL 파싱 실패 → `http`/`https` 아님 → 자기 주소. 자기 주소는 파서가 낸 (소문자 호스트, 기본 포트가 아닌 포트) 쌍을 `BASE_URL`의 것과 비교하므로 대소문자와 기본 포트(`:80`/`:443`)를 무시하고, 스킴은 보지 않는다. `Rejection`에 `Empty`, `NotAUrl`, `UnsupportedScheme`, `TooLong`, `SelfAddress`를 더했고, 한국어 문구는 `Display`에 있다. 서버 함수는 입력 거절을 400, `StorageUnavailable`을 503으로 돌려준다. 화면은 바꾸지 않았다(입력 signal은 그대로 남고 결과는 매번 대체된다).
- 저장은 아직 원래 입력 그대로다. URL 정리와 같은 URL 중복 처리는 티켓 05.
- 테스트: `tests/shortener.rs`(경계 1, 사유별 + 거절 뒤 번호가 건너뛰지 않음), `tests/rejected_input_http.rs`(경계 2, `ftp://`가 400 + `{"data":{"Rejected":"UnsupportedScheme"}}` + 문구).
- 확인: `dx serve`(`BASE_URL=http://127.0.0.1:8094`)에 curl로 빈 입력, 공백, `not a url`, `ftp://`, 2049자, 자기 주소(대문자 스킴 포함)를 보내 각각 400과 사유가 오고, 이어서 성공한 Shorten이 `/1`을 받음. 브라우저에서 `ftp://example.com/a`로 줄이기 → 스킴 오류 문구가 보이고 입력칸 유지, 이어서 올바른 주소로 성공하자 오류 문구가 사라짐.
