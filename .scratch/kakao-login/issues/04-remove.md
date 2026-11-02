# 04: Remove: 내 링크에서 빼기

**What to build:** 내 목록의 각 Short Link 옆에 "삭제" 버튼이 있고, 누르면 내 My Links에서 바로 빠진다. 같은 Short Link를 다른 User가 담고 있으면 그 Short Link는 계속 Follow된다. 아무도 담고 있지 않게 되면 Short Link가 지워져 Follow하면 "없는 링크예요"(404)가 되고, 그 번호는 다시 쓰지 않는다. 명세: `.scratch/kakao-login/spec.md`

**Blocked by:** 01, 03

**Status:** resolved

- [x] Remove가 내 My Links에서 Short Link를 뺀다. 내 목록에 없는 번호면 아무 일도 하지 않는다(남의 링크를 뺄 수 없음)
- [x] 다른 User가 담고 있으면 Follow 유지, 아무도 없으면 Short Link가 지워지고 Follow는 404
- [x] 지운 뒤 새 Original URL은 지운 번호를 받지 않는다. 지운 Original URL을 다시 Shorten하면 새 번호로 내 목록에 담긴다
- [x] Remove 서버 함수는 세션이 없으면 401
- [x] 목록 각 항목에 "삭제" 버튼, 누르면 목록을 다시 불러온다
- [x] 경계 1 테스트: 공유 중 Remove, 마지막 Remove, 번호 재사용 없음, 남의 번호 Remove
- [x] 경계 2 테스트: Remove 서버 함수 뒤 목록에서 사라짐, 401, 지워진 Short Link의 Follow 404
- [x] 화면에서 삭제를 눌러 목록에서 사라지고 Follow가 404가 되는 것을 직접 확인했다

## Comments

- 구현 완료. `cargo test` 전체 통과(Shortener 34, HTTP 15). 로컬 `dx serve`에서 로그인 화면, 로그인 후 화면(DB에 만든 테스트 세션), 줄이기 → `/1`부터 발급, 삭제 → 목록에서 빠지고 `/2` 404, 로그아웃 → 로그인 화면, `/api/stats` 확인.
