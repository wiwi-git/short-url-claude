# 06: 첫 화면 아래에 모든 Short Link 목록

**What to build:** 누구나 첫 화면(`/`)에 들어오면 줄이기 입력칸 아래에서 지금까지 만들어진 모든 Short Link를 최신순으로 본다. 목록에는 Short Link만 보이고(Original URL, 만든 시각은 없음), 누르면 Follow된다. 줄이기에 성공하면 새 Short Link가 목록 맨 위에 바로 나타난다. 명세: `.scratch/short-url-mvp/spec.md`

**Blocked by:** None (01–05 resolved)

**Status:** resolved

- [x] Shortener가 Short Link를 최신순(가장 큰 Link Number부터)으로, 한 페이지 50개씩 돌려준다. 다음 페이지가 있는지도 알 수 있다
- [x] 첫 화면의 줄이기 아래에 Short Link 목록이 보이고, 각 Short Link는 그 주소로 가는 링크다
- [x] 50개를 넘으면 "이전"/"다음"으로 페이지를 넘긴다 (별도 페이지 주소는 만들지 않는다)
- [x] 줄이기에 성공하면 목록이 다시 불러와져 새 Short Link가 맨 위에 보인다 (같은 Original URL이라 기존 Short Link를 받은 경우 목록은 그대로)
- [x] Short Link가 하나도 없으면 목록 자리에 빈 상태 문구를 보여 준다
- [x] 경계 1 테스트: 최신순, 50개 경계에서 페이지 나눔과 다음 페이지 여부, 빈 목록
- [x] 경계 2 테스트: 목록 서버 함수가 Short Link를 최신순으로 돌려줌
- [x] 직접 실행해서 줄이기 → 목록 맨 위에 나타남 → 목록의 Short Link 클릭으로 이동을 확인했다

## Comments

- 구현: `Shortener::short_links(page)`(0부터 세는 페이지)가 `ShortLinkPage { short_links, has_next }`를 돌려준다. 51개를 읽어 다음 페이지 여부를 알고, 범위를 넘는 페이지는 빈 페이지다. 서버 함수는 `POST /api/short-links`(`{"page":N}`), 저장소 오류는 503.
- 화면: `ShortLinkList` 컴포넌트가 `use_server_future`로 목록을 읽어 SSR에도 그려진다(`SuspenseBoundary` 안). 줄이기에 성공하면 첫 페이지로 돌아가 다시 불러온다(같은 Original URL이면 내용이 같아 목록은 그대로). "이전"/"다음"은 넘길 페이지가 있을 때만 보이고 끝에서는 비활성화된다. 문구는 "단축 링크 목록", 빈 상태는 "아직 만든 단축 링크가 없어요".
- 테스트: `tests/shortener.rs`에 빈 목록, 최신순(다시 줄인 URL은 중복 없음), 50/51개 경계와 범위 밖 페이지. `tests/short_link_list_http.rs`에 서버 함수 1개.
- 확인: `dx serve`에서 줄이기 두 번 → `/2`, `/1` 순, 같은 URL 다시 줄이기 → 목록 그대로. curl로 52개까지 만든 뒤 "다음" → `/2`, `/1`(주소 `/` 그대로), 2페이지에서 줄이기 → 1페이지로 돌아와 `/53`이 맨 위. 목록의 `/53` 클릭 → `https://example.com/newest`로 이동.
