# 05: 같은 Original URL이면 같은 Short Link

**What to build:** 방문자가 이미 줄인 적 있는 Original URL을 넣으면, 표기만 다르더라도(공백, 대소문자, 기본 포트) 새 번호 대신 기존 Short Link를 받는다. 동시에 같은 URL이 들어와도 Short Link는 하나만 생긴다. 명세: `.scratch/short-url-mvp/spec.md`

**Blocked by:** 02 (Shorten과 Follow 기본 흐름)

**Status:** ready-for-agent

- [ ] Original URL을 정리한 뒤 저장, 비교한다: 앞뒤 공백 제거, 스킴과 호스트 소문자, 기본 포트(`:80`, `:443`) 제거
- [ ] 경로, 쿼리, 프래그먼트, 끝의 `/`는 건드리지 않는다 (`/a`와 `/a/`는 다른 Original URL)
- [ ] 같은 Original URL을 다시 Shorten하면 기존 Link Number를 돌려주고 새 번호를 소모하지 않는다
- [ ] 같은 Original URL을 동시에 Shorten해도 Short Link가 하나만 생긴다
- [ ] Follow는 정리된 Original URL로 보낸다
- [ ] 경계 1 테스트: 표기 차이 사례들이 같은 번호, `/a`와 `/a/`가 다른 번호, 동시 Shorten 결과가 하나
