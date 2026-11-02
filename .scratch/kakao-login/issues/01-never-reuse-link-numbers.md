# 01: 번호를 다시 쓰지 않게 하고 기존 Short Link 정리

**What to build:** Shorten은 지금처럼 다음 차례 Link Number를 발급하지만, 이제 "지금까지 발급한 가장 큰 번호 + 1"을 쓴다. 그래서 나중에 Short Link가 지워져도 그 번호가 다른 Original URL에 다시 붙지 않는다(ADR 0003). 이 버전이 처음 뜰 때 한 번만, 로그인 기능 전에 만든 주인 없는 Short Link를 모두 지우고 번호를 1부터 다시 시작한다. 명세: `.scratch/kakao-login/spec.md`

**Blocked by:** None (can start immediately)

**Status:** resolved

- [x] 번호 발급이 "발급한 가장 큰 번호"를 따로 기록한 값 + 1을 쓰고, 기존 advisory lock 트랜잭션 안에서 갱신한다
- [x] 새 DB에서 첫 Shorten은 1번, 같은 Original URL은 같은 번호(기존 정책 그대로)
- [x] 예전 스키마에 Short Link가 있는 DB에서 시작하면 모두 지워지고 다음 Shorten이 1번을 받는다
- [x] 두 번째로 시작할 때는 아무것도 지우지 않는다
- [x] 기존 테스트가 모두 통과한다
- [x] 경계 1 테스트: 정리 동작과 정리 후 번호, 재시작 시 보존

## Comments

- 구현 완료. `cargo test` 전체 통과(Shortener 34, HTTP 15). 로컬 `dx serve`에서 로그인 화면, 로그인 후 화면(DB에 만든 테스트 세션), 줄이기 → `/1`부터 발급, 삭제 → 목록에서 빠지고 `/2` 404, 로그아웃 → 로그인 화면, `/api/stats` 확인.
