# 06: 첫 화면 아래에 모든 Short Link 목록

**What to build:** 누구나 첫 화면(`/`)에 들어오면 줄이기 입력칸 아래에서 지금까지 만들어진 모든 Short Link를 최신순으로 본다. 목록에는 Short Link만 보이고(Original URL, 만든 시각은 없음), 누르면 Follow된다. 줄이기에 성공하면 새 Short Link가 목록 맨 위에 바로 나타난다. 명세: `.scratch/short-url-mvp/spec.md`

**Blocked by:** None (01–05 resolved)

**Status:** ready-for-agent

- [ ] Shortener가 Short Link를 최신순(가장 큰 Link Number부터)으로, 한 페이지 50개씩 돌려준다. 다음 페이지가 있는지도 알 수 있다
- [ ] 첫 화면의 줄이기 아래에 Short Link 목록이 보이고, 각 Short Link는 그 주소로 가는 링크다
- [ ] 50개를 넘으면 "이전"/"다음"으로 페이지를 넘긴다 (별도 페이지 주소는 만들지 않는다)
- [ ] 줄이기에 성공하면 목록이 다시 불러와져 새 Short Link가 맨 위에 보인다 (같은 Original URL이라 기존 Short Link를 받은 경우 목록은 그대로)
- [ ] Short Link가 하나도 없으면 목록 자리에 빈 상태 문구를 보여 준다
- [ ] 경계 1 테스트: 최신순, 50개 경계에서 페이지 나눔과 다음 페이지 여부, 빈 목록
- [ ] 경계 2 테스트: 목록 서버 함수가 Short Link를 최신순으로 돌려줌
- [ ] 직접 실행해서 줄이기 → 목록 맨 위에 나타남 → 목록의 Short Link 클릭으로 이동을 확인했다
