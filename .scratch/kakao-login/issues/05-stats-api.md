# 05: 통계 API

**What to build:** 서비스 주인이 로그인 없이 `GET /api/stats`를 열면 `{"users": N, "short_links": M}`을 받는다. users는 한 번이라도 카카오로 로그인한 User 수, short_links는 지금 Follow되는 Short Link 수(지운 것 제외)다. 화면은 만들지 않는다. 명세: `.scratch/kakao-login/spec.md`

**Blocked by:** 02

**Status:** resolved

- [x] Shortener가 User 수와 살아 있는 Short Link 수를 돌려준다
- [x] `GET /api/stats`가 쿠키 없이 200 JSON. 저장소 오류는 503
- [x] 경계 1 테스트: 빈 DB 0/0, User와 Short Link를 만든 뒤 수
- [x] 경계 2 테스트: 쿠키 없이 JSON 응답
- [x] 배포 문서에 통계 API 주소를 적는다

## Comments

- 구현 완료. `cargo test` 전체 통과(Shortener 34, HTTP 15). 로컬 `dx serve`에서 로그인 화면, 로그인 후 화면(DB에 만든 테스트 세션), 줄이기 → `/1`부터 발급, 삭제 → 목록에서 빠지고 `/2` 404, 로그아웃 → 로그인 화면, `/api/stats` 확인.
