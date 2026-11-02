# Spec: 카카오 로그인과 My Links

Status: resolved

## Problem Statement

지금은 첫 화면에 누가 만들었든 모든 Short Link가 보인다. 서비스 주인은 각 User가 자기가 Shorten한 Short Link만 모아 보고, 필요 없는 것은 스스로 정리할 수 있기를 바란다. 그러려면 누가 Shorten했는지 알아야 하므로 로그인이 필요하다. 다만 이미 퍼진 Short Link를 받은 사람은 로그인 없이 Follow할 수 있어야 하고, "같은 Original URL이면 같은 Short Link", 차례로 늘어나는 Link Number, 입력 검사 같은 기존 Shorten 정책은 그대로여야 한다. 서비스 주인은 가입한 User 수와 저장된 Short Link 수를 따로 화면 없이 확인하고 싶다.

## Solution

`/`는 로그인 상태에 따라 두 화면 중 하나를 보여 준다.
- 로그인 전: 가운데에 사이트 이름과 "카카오 로그인" 버튼만 있다. 줄이기 폼과 목록은 보이지 않는다.
- 로그인 후: 기존 화면(제목, 줄이기 폼, 결과, 목록)에 오른쪽 위로 닉네임과 로그아웃 버튼이 붙는다. 목록은 내 My Links만 보여 주고, 각 Short Link 옆에 Remove 버튼이 있다.

카카오 로그인은 카카오 인가 코드 방식(서버가 REST API 키와 Client Secret으로 토큰을 받고 회원번호·닉네임을 조회)으로 한다. 로그인하면 30일짜리 세션이 생기고, 로그아웃하면 세션이 지워지고 로그인 화면으로 돌아간다. Short Link는 User끼리 공유된다(ADR 0003): 같은 Original URL을 Shorten한 User 모두의 My Links에 같은 Short Link가 담긴다. Remove는 내 My Links에서 빼는 것이고, 아무 User도 담고 있지 않게 되면 Short Link가 지워져 404가 된다. 지운 Link Number는 다시 쓰지 않는다. `GET /api/stats`는 로그인 없이 User 수와 Short Link 수를 돌려준다. 배포할 때 로그인 전에 만든(주인 없는) Short Link는 모두 지우고 번호를 1부터 다시 시작한다.

## User Stories

1. 방문자로서, 로그인하지 않고 `/`에 들어가면 사이트 이름과 카카오 로그인 버튼만 보고 싶다. 무엇을 해야 하는지 바로 알도록.
2. 방문자로서, 로그인하지 않으면 줄이기 폼과 목록이 보이지 않기를 바란다. 로그인한 User만 쓰는 서비스이니까.
3. 방문자로서, 카카오 로그인 버튼을 누르면 카카오 로그인 화면으로 가고 싶다. 따로 회원가입하지 않고 카카오 계정으로 들어가도록.
4. 방문자로서, 카카오에서 동의하고 나면 `/`로 돌아와 바로 로그인된 화면을 보고 싶다.
5. 방문자로서, 카카오 로그인 화면에서 취소하거나 동의하지 않으면 로그인 화면으로 돌아오고 싶다. 깨진 화면을 보지 않도록.
6. User로서, 처음 로그인하면 따로 가입 절차 없이 User가 되고 싶다.
7. User로서, 오른쪽 위에서 내 카카오 닉네임을 보고 싶다. 내 계정으로 로그인했는지 알 수 있도록.
8. User로서, 카카오에서 닉네임을 바꾸면 다음 로그인 때 바뀐 닉네임이 보이기를 바란다.
9. User로서, 브라우저를 닫았다 열어도 30일 동안은 로그인 상태가 유지되기를 바란다.
10. User로서, 30일이 지나면 다시 로그인하라는 화면을 보고 싶다. 오래된 로그인이 계속 남지 않도록.
11. User로서, 로그아웃 버튼을 누르면 로그인 화면으로 돌아가고 싶다.
12. User로서, 로그아웃하면 같은 브라우저에서 뒤로 가기나 새로 고침을 해도 내 목록이 보이지 않기를 바란다.
13. User로서, 로그아웃해도 카카오 계정 자체는 로그아웃되지 않기를 바란다. 다른 카카오 서비스에 영향이 없도록.
14. User로서, 로그인 후에도 지금처럼 Original URL을 넣고 Shorten하고 싶다.
15. User로서, Shorten 결과와 복사 버튼은 지금과 똑같이 쓰고 싶다.
16. User로서, 잘못된 URL을 넣으면 지금과 같은 안내 문구를 보고 싶다.
17. User로서, 처음 보는 Original URL을 Shorten하면 다음 차례 Link Number를 받고 싶다.
18. User로서, 다른 User가 이미 Shorten한 Original URL을 Shorten하면 같은 Short Link를 받고, 그 Short Link가 내 My Links에도 담기기를 바란다.
19. User로서, 내가 이미 담고 있는 Original URL을 다시 Shorten하면 같은 Short Link를 받고, 내 목록에 중복으로 생기지 않기를 바란다.
20. User로서, 첫 화면 목록에서 내가 Shorten한 Short Link만 보고 싶다. 다른 User의 링크가 섞이지 않도록.
21. User로서, 내 목록은 내가 가장 최근에 담은 Short Link부터 보고 싶다.
22. User로서, 내 목록이 50개를 넘으면 "이전"/"다음"으로 넘겨 보고 싶다.
23. User로서, 아직 아무것도 Shorten하지 않았으면 빈 상태 문구를 보고 싶다.
24. User로서, Shorten에 성공하면 그 Short Link가 내 목록 맨 위에 바로 보이기를 바란다.
25. User로서, 목록의 각 Short Link 옆에 있는 버튼으로 내 목록에서 빼고 싶다. 더는 필요 없는 링크를 정리하도록.
26. User로서, Remove하면 그 Short Link가 내 목록에서 바로 사라지기를 바란다.
27. User로서, 내가 Remove해도 같은 Short Link를 담은 다른 User의 목록과 Follow는 그대로이기를 바란다. 다른 사람의 링크를 내가 망가뜨리지 않도록.
28. User로서, 아무도 담지 않은 Short Link를 Remove하면 그 Short Link가 더는 Follow되지 않기를 바란다. 정말로 정리되도록.
29. User로서, 지워진 Link Number가 나중에 다른 Original URL에 다시 붙지 않기를 바란다. 예전에 퍼진 Short Link가 엉뚱한 곳으로 가지 않도록.
30. User로서, Remove한 Original URL을 다시 Shorten하면 다시 내 목록에 담기기를 바란다(아직 살아 있으면 같은 번호, 지워졌으면 새 번호).
31. User로서, 다른 User의 Short Link는 Remove할 수 없기를 바란다.
32. Short Link를 받은 사람으로서, 로그인하지 않아도 Short Link를 열면 Original URL로 가고 싶다.
33. Short Link를 받은 사람으로서, 지워진 Short Link를 열면 지금과 같은 "없는 링크예요" 화면을 보고 싶다.
34. 서비스 주인으로서, `GET /api/stats`로 지금까지 로그인한 User 수를 보고 싶다.
35. 서비스 주인으로서, 같은 응답에서 지금 Follow되는 Short Link 수를 보고 싶다.
36. 서비스 주인으로서, 통계 API는 로그인 없이 브라우저나 curl로 바로 열어 보고 싶다.
37. 서비스 주인으로서, 로그인 기능 전에 만든 주인 없는 Short Link는 배포 때 사라지고 번호가 1부터 다시 시작되기를 바란다.
38. 서비스 주인으로서, 카카오 키는 환경 변수로만 넣고 저장소에는 남기지 않고 싶다.
39. 서비스 주인으로서, 카카오 키가 빠진 채로 서버가 뜨면 바로 알 수 있기를 바란다.
40. 서비스 주인으로서, 로컬 개발에서도 카카오 로그인을 직접 해 볼 수 있기를 바란다.
41. 로그인하지 않은 사람으로서, 서버 함수(줄이기, 목록, Remove)를 직접 호출하면 거절당하기를 바란다. 화면을 숨기는 것만으로 막지 않도록.

## Implementation Decisions

- **Shortener 모듈을 넓힌다.** 저장소 하나(Postgres)에 User, My Links, 세션, 통계를 함께 둔다. 새 인터페이스(이름은 구현에서 정함):
  - `sign_in(kakao_id, nickname) -> User`: 회원번호로 찾거나 만들고, 닉네임을 갱신한다.
  - `shorten(user, input) -> Result<ShortLink, ...>`: 기존 검사·번호 정책 그대로, 결과를 그 User의 My Links에 담는다(이미 담겨 있으면 그대로).
  - `my_links(user, page) -> ShortLinkPage`: 담은 순서의 최신순, 50개씩, `has_next` 포함.
  - `remove(user, link_number)`: 내 My Links에서 뺀다. 담은 User가 없어지면 Short Link를 지운다. 내 목록에 없는 번호면 아무 일도 하지 않는다.
  - `follow(link_number)`: 지금과 같다(로그인 불필요).
  - 세션: `start_session(user) -> token`, `user_for_session(token) -> Option<User>`(만료된 세션은 없는 것으로), `end_session(token)`.
  - `stats() -> { users, short_links }`.
- **스키마**
  - `users(id, kakao_id UNIQUE, nickname, created_at)`.
  - `short_links(link_number, original_url UNIQUE)`는 그대로.
  - `my_links(user_id, link_number, added_at, PRIMARY KEY(user_id, link_number))`: 같은 Short Link를 여러 User가 담을 수 있다. 목록 순서는 `added_at`(같으면 link_number) 내림차순.
  - `sessions(token PRIMARY KEY, user_id, expires_at)`: 토큰은 추측할 수 없는 무작위 값, 만료는 만든 때부터 30일.
  - 번호 발급 상태: 지금까지 발급한 가장 큰 Link Number를 한 행짜리 표에 둔다. Shorten은 기존과 같은 advisory lock 트랜잭션 안에서 이 값 + 1을 쓰고 갱신한다. Short Link가 지워져도 이 값은 줄지 않는다(ADR 0003).
- **기존 데이터 정리**: 시작할 때 스키마를 만드는 같은 lock 안에서, 번호 발급 상태 표가 처음 만들어지는 경우에 한해 기존 `short_links` 행을 모두 지우고 발급 상태를 0에서 시작한다. 두 번째 시작부터는 아무것도 지우지 않는다.
- **카카오 경계**: 서버는 "인가 코드 → (카카오 회원번호, 닉네임)"을 돌려주는 카카오 클라이언트를 주입받는다. 실제 구현은 카카오 토큰 발급(`kauth.kakao.com/oauth/token`, REST API 키 + Client Secret + redirect URI)과 사용자 정보 조회(`kapi.kakao.com/v2/user/me`)를 한다. 테스트는 가짜를 꽂는다. 카카오 토큰은 저장하지 않는다.
- **라우트** (`build_router`에 덧붙임)
  - `GET /auth/kakao/login`: 무작위 `state`를 짧게 사는 HttpOnly 쿠키에 넣고, 카카오 인가 주소(`client_id`, `redirect_uri = BASE_URL/auth/kakao/callback`, `response_type=code`, `state`)로 302.
  - `GET /auth/kakao/callback?code&state`: `state`가 쿠키와 다르거나, `code`가 없거나(취소·거절), 카카오 호출이 실패하면 세션 없이 `/`로 302. 성공하면 `sign_in` → 세션 쿠키를 심고 `/`로 302.
  - `POST /auth/logout`: 세션을 지우고 쿠키를 만료시킨 뒤 `/`로 302(로그아웃 버튼은 폼 POST).
  - `GET /api/stats`: 로그인 없이 `{"users": N, "short_links": M}` JSON.
  - `GET /{link_number}`: 지금과 같음.
- **세션 쿠키**: 이름 하나, `HttpOnly`, `SameSite=Lax`, `Path=/`, `Max-Age` 30일, `BASE_URL`이 https면 `Secure`.
- **서버 함수**: 줄이기, 내 목록, Remove 모두 요청의 세션 쿠키로 User를 찾는다. 없거나 만료면 401. 새 서버 함수: Remove(`link_number`), 그리고 화면용 "지금 로그인한 User"(닉네임 또는 없음).
- **화면**: `App`은 SSR에서 현재 User를 읽어 로그인 화면 또는 기존 화면을 고른다. 로그인 화면은 제목과 "카카오 로그인" 링크 버튼만 가운데. 로그인 화면의 header 오른쪽 위에 닉네임과 로그아웃 폼 버튼. 목록 제목은 "내 단축 링크", 각 항목에 Remove 버튼("삭제"), 누르면 목록을 다시 불러온다. 기존 색과 레이아웃(STYLE)을 따른다.
- **설정**: `KAKAO_REST_API_KEY`, `KAKAO_CLIENT_SECRET` 환경 변수. 없으면 서버가 시작할 때 실패한다. 로컬은 `.env`나 셸 환경으로 넣는다. `render.yaml`에는 두 값을 `sync: false`(대시보드에서 입력)로 추가한다. 서버에서 카카오로 HTTPS 요청을 보내는 HTTP 클라이언트(rustls)를 `server` 기능에만 추가한다.
- **문서**: 카카오 디벨로퍼스 설정 순서(앱 만들기, 카카오 로그인 켜기, Redirect URI 두 개, 닉네임 동의 항목, REST API 키·Client Secret)를 배포 문서에 덧붙인다. AGENTS.md 개발 서버 항목에 카카오 환경 변수를 적는다.

## Testing Decisions

- 좋은 테스트는 바깥에서 보이는 동작만 확인한다: 공개 메서드의 결과, HTTP 상태·헤더·본문, Follow 여부. 표 구조나 SQL은 확인하지 않는다.
- **경계 1: Shortener (진짜 Postgres, 테스트마다 새 DB)**. 선례: 기존 shortener 테스트.
  - sign_in이 처음엔 User를 만들고, 다시 하면 같은 User에 닉네임만 바뀜.
  - 두 User가 같은 Original URL을 Shorten하면 같은 번호, 둘의 my_links에 모두 보임. 같은 User가 다시 Shorten해도 중복 없음.
  - my_links는 그 User 것만, 최신순, 50/51 경계, 빈 목록.
  - remove: 다른 User가 담고 있으면 Follow 유지, 아무도 없으면 Follow가 없음. 지운 뒤 새 Original URL은 지운 번호를 받지 않음. 남의 번호 remove는 아무 일 없음.
  - 세션: 시작 → User 조회, end 뒤 없음, 만료된 세션 없음(만료 시각을 주입하거나 시계를 넘길 수 있게).
  - stats: User 수와 살아 있는 Short Link 수(지운 것 제외).
  - 기존 데이터 정리: 예전 스키마에 Short Link가 있는 DB에서 시작하면 모두 지워지고 다음 번호가 1. 두 번째 시작에서는 지우지 않음.
- **경계 2: 라우터 HTTP (`oneshot`, 가짜 카카오)**. 선례: 기존 `*_http.rs` 테스트와 공용 setup.
  - 로그인 전 `/`: 제목과 로그인 링크만, 줄이기 폼과 목록 없음.
  - `/auth/kakao/login`: 카카오 인가 주소로 302, `state` 쿠키를 심음.
  - callback: 올바른 state+code → 세션 쿠키와 `/`로 302, 그 쿠키로 `/`를 열면 닉네임·로그아웃·내 목록. state 불일치나 code 없음 → 세션 없이 `/`.
  - 로그아웃: 쿠키 만료 + `/`로 302, 그 뒤 예전 쿠키로 `/`는 로그인 화면.
  - 세션 없이 줄이기·목록·Remove 서버 함수 → 401.
  - Remove 서버 함수 뒤 목록에서 사라짐.
  - Follow는 쿠키 없이 302. `/api/stats`는 쿠키 없이 JSON.
- 진짜 카카오와의 통신은 자동 테스트하지 않고, 키를 넣고 로컬과 Render에서 직접 로그인해 확인한다.

## Out of Scope

- 카카오 외 로그인(구글, 이메일 등)과 여러 로그인 방식 연결.
- 카카오 계정 로그아웃, 연결 끊기(회원 탈퇴), 카카오 연결 끊기 웹훅.
- 이메일, 프로필 사진 등 회원번호·닉네임 외 정보.
- 통계 화면, 관리자 권한, 통계 API 인증.
- Short Link 수정, Original URL 보기, 클릭 수 집계.
- 로그인 없이 Shorten하기.
- 지운 Short Link 되살리기.

## Further Notes

- 결정 근거: ADR 0001(차례 번호), ADR 0002(Postgres), ADR 0003(카카오 로그인과 Short Link 공유). 용어는 GLOSSARY의 User, My Links, Remove를 쓴다.
- 카카오 앱 설정과 키 입력은 서비스 주인이 직접 한다. Redirect URI는 `https://<Render 주소>/auth/kakao/callback`과 `http://localhost:8080/auth/kakao/callback` 두 개.
- Render 무료 Postgres는 30일 뒤 만료되므로(ADR 0002) User와 My Links도 그때 함께 사라진다.
