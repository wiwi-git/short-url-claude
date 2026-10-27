# 03: 없는 Short Link는 404

**What to build:** Short Link를 받은 사람이 발급된 적 없는 번호나 올바른 Link Number가 아닌 주소를 열면, "없는 링크"라고 알려 주고 첫 화면으로 가는 링크가 있는 404 페이지를 본다. 명세: `.scratch/short-url-mvp/spec.md`

**Blocked by:** 02 (Shorten과 Follow 기본 흐름)

**Status:** ready-for-agent

- [ ] Follow가 다음 경로 조각에 "없음"을 돌려준다: 발급되지 않은 번호, `0`, 앞자리 0(`007`), 숫자 아님(`abc`), 정수 범위를 넘는 아주 큰 숫자
- [ ] 위 경우 서버가 404 상태 코드와 404 페이지를 돌려준다
- [ ] 404 페이지에 첫 화면으로 가는 링크가 있다
- [ ] 경계 1 테스트: 위 경로 조각 각각에 대해 Follow가 "없음"
- [ ] 경계 2 테스트: 없는 번호와 잘못된 경로가 404
