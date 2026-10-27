# 05: 같은 Original URL이면 같은 Short Link

**What to build:** 방문자가 이미 줄인 적 있는 Original URL을 넣으면, 표기만 다르더라도(공백, 대소문자, 기본 포트) 새 번호 대신 기존 Short Link를 받는다. 동시에 같은 URL이 들어와도 Short Link는 하나만 생긴다. 명세: `.scratch/short-url-mvp/spec.md`

**Blocked by:** 02 (Shorten과 Follow 기본 흐름)

**Status:** resolved

- [x] Original URL을 정리한 뒤 저장, 비교한다: 앞뒤 공백 제거, 스킴과 호스트 소문자, 기본 포트(`:80`, `:443`) 제거
- [x] 경로, 쿼리, 프래그먼트, 끝의 `/`는 건드리지 않는다 (`/a`와 `/a/`는 다른 Original URL)
- [x] 같은 Original URL을 다시 Shorten하면 기존 Link Number를 돌려주고 새 번호를 소모하지 않는다
- [x] 같은 Original URL을 동시에 Shorten해도 Short Link가 하나만 생긴다
- [x] Follow는 정리된 Original URL로 보낸다
- [x] 경계 1 테스트: 표기 차이 사례들이 같은 번호, `/a`와 `/a/`가 다른 번호, 동시 Shorten 결과가 하나

## Comments

- 구현: `src/shortener.rs`의 `check()`가 정리한 Original URL을 돌려주고(`normalize()`), Shorten은 그것을 저장, 비교한다. 검증은 여전히 `url` 크레이트 파싱으로 하지만, 저장 형태는 파서 직렬화를 쓰지 않고 입력 문자열에서 스킴과 호스트만 소문자로, 기본 포트(파서가 `None`으로 읽는 포트, 빈 `:` 포함)만 뺀다. 파서 직렬화는 빈 경로에 `/`를 붙이고 퍼센트 인코딩, IDN 변환도 하기 때문이다. 사용자 정보, 경로, 쿼리, 프래그먼트는 입력 그대로.
- 선택: `https://example.com`과 `https://example.com/`는 명세의 "끝의 `/`는 그대로 둔다"를 따라 다른 Original URL로 둔다(`/a`와 `/a/`와 같은 규칙). `~`와 `%7E`도 다르다. `scheme://` 꼴이 아니거나(`http:example.com`) 역슬래시, 제어 문자가 있는 드문 입력만 파서 직렬화 형태로 저장한다.
- 동시성: 유일 제약에 걸린 INSERT도 AUTOINCREMENT 번호를 소모함을 테스트로 확인했다(16개 연결 동시 Shorten 뒤 다음 번호가 17). 그래서 조회와 `INSERT ... ON CONFLICT DO NOTHING`, 재조회를 `BEGIN IMMEDIATE` 트랜잭션 하나로 묶어 번호가 건너뛰지 않게 했다. 테스트는 같은 DB 파일에 스레드마다 따로 `Shortener`(연결)를 열고 Barrier로 동시에 Shorten한다.
- 테스트: `tests/shortener.rs`에 다시 Shorten, 표기 차이, 경로 등 그대로, Follow가 정리된 URL로, 동시 Shorten 5개 추가.
- 확인: `dx serve`(`BASE_URL=http://127.0.0.1:8095`)에 curl로 `  HTTPS://Example.COM:443/Path?Q=A#F `, 소문자, 대문자 호스트 변형이 모두 `/1`, `/Path/`는 `/2`, `HTTP://Example.com:80/x`와 `http://example.com/x`가 모두 `/3`. `/1`은 302로 `https://example.com/Path?Q=A#F`.
