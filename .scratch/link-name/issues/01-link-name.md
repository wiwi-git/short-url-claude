# 01: My Links에 Link Name과 Original URL 보여 주기

**What to build:** My Links 각 줄에 Link Name(없으면 Original URL)과 Short Link를 두 줄로 보여 주고, User가 Link Name을 고칠 수 있게 한다. Link Name은 User마다 따로(My Links 항목에) 갖고, Shorten할 때 Original URL의 도메인으로 채운다. 외부 사이트에는 접속하지 않는다. 용어: `GLOSSARY.md`의 Link Name.

**Blocked by:** None (can start immediately)

**Status:** resolved

## 정한 것 (grilling 2026-11-21)

- 기본 Link Name = Original URL의 호스트. 앞의 `www.`만 떼고, 다른 서브도메인은 둔다. 포트는 뺀다. IDN(`xn--`)은 유니코드로 보여 준다. IP 주소는 그대로.
- Shorten할 때 저장한다. 이미 My Links에 있으면 Link Name을 건드리지 않는다.
- 이 버전이 처음 뜰 때 기존 My Links 항목을 도메인으로 채운다(한 번만).
- 고치기: 앞뒤 공백을 떼고 최대 100자. 빈칸이면 "이름 없음"이 되고 화면은 Original URL을 보여 준다. 내 My Links에 없는 번호면 아무 일도 하지 않는다.
- 화면: 첫 줄 Link Name(또는 Original URL), 한 줄을 넘으면 뒤를 …로 자르고, 누르면 Original URL로 가며, 툴팁으로 전체를 보여 준다. 둘째 줄은 작은 글씨의 Short Link. 오른쪽에 "이름 수정", "삭제". "이름 수정"을 누르면 그 줄이 입력칸과 저장/취소로 바뀐다.

## 할 일

- [x] 목록 항목에 Original URL과 Link Name이 담긴다
- [x] Shorten이 도메인 규칙대로 기본 Link Name을 저장하고, 다시 Shorten해도 고친 이름을 지킨다
- [x] 같은 Short Link라도 User마다 Link Name이 따로다
- [x] Link Name 고치기(공백 정리, 빈칸 → 없음, 100자 초과 거절, 남의 링크는 무시)
- [x] 예전 스키마 DB를 열면 기존 항목이 도메인으로 채워지고, 다시 열 때는 손대지 않는다
- [x] 고치기 서버 함수(로그인 필요)
- [x] 목록 화면 두 줄 구성과 이름 수정 UI

## Comments

- 구현 완료(2026-11-21). `cargo test` 전체 통과(Shortener 39, HTTP 포함), wasm 빌드, clippy, fmt 확인. 예전 스키마 DB로 `dx serve`를 띄워 기존 항목이 도메인으로 채워지는 것, 새 Shorten의 기본 이름, 이름 고치기와 지우기(→ Original URL 표시), 긴 이름의 … 자르기, 이름 수정 입력칸을 화면에서 확인했다.
- 참고: 이름을 지운 한글 도메인 링크는 저장된 Original URL을 그대로 보여 주므로 `https://xn--...` 꼴로 보인다.

