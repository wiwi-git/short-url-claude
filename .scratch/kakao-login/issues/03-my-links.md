# 03: My Links: 내가 줄인 Short Link만 보기

**What to build:** 로그인한 User가 Shorten하면 그 Short Link가 User의 My Links에 담기고, 첫 화면 목록은 "내 단축 링크"만 담은 순서의 최신순으로 50개씩 보여 준다. 다른 User가 이미 Shorten한 Original URL을 Shorten하면 같은 Short Link를 받고 내 목록에도 담긴다. 명세: `.scratch/kakao-login/spec.md`

**Blocked by:** 02

**Status:** resolved

- [x] Shorten이 User를 받아 결과를 그 User의 My Links에 담는다. 이미 담겨 있으면 중복 없이 그대로
- [x] 두 User가 같은 Original URL을 Shorten하면 같은 번호, 둘의 My Links에 모두 보임
- [x] 목록은 그 User 것만, 담은 순서 최신순, 50개씩, 다음 페이지 여부
- [x] 화면의 목록 제목은 "내 단축 링크", 빈 상태 문구 유지, Shorten 성공 후 목록 다시 불러오기 유지
- [x] 경계 1 테스트: 공유, 중복 없음, 다른 User 것 안 보임, 50/51 경계, 빈 목록
- [x] 경계 2 테스트: 로그인한 쿠키로 Shorten 후 목록 서버 함수가 내 Short Link만 돌려줌
- [x] 두 카카오 계정(또는 가짜 User 두 명)으로 같은 URL을 줄여 각자 목록에 보이는 것을 직접 확인했다

## Comments

- 구현 완료. `cargo test` 전체 통과(Shortener 34, HTTP 15). 로컬 `dx serve`에서 로그인 화면, 로그인 후 화면(DB에 만든 테스트 세션), 줄이기 → `/1`부터 발급, 삭제 → 목록에서 빠지고 `/2` 404, 로그아웃 → 로그인 화면, `/api/stats` 확인.
- 여러 User의 공유는 경계 1·2 테스트로 확인했다. 진짜 카카오 계정은 하나로만 확인했다.
