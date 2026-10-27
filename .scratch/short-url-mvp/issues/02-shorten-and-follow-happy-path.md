# 02: Shorten과 Follow 기본 흐름 (tracer bullet)

**What to build:** 방문자가 첫 화면에서 올바른 Original URL을 넣고 "줄이기"를 누르면 `BASE_URL/1`, `/2`, ... 형태의 Short Link와 "복사" 버튼을 받는다. 그 Short Link를 열면 302로 Original URL에 도착한다. Short Link는 SQLite 파일에 저장되어 서버를 재시작해도 남는다. 명세의 Shortener 모듈(Shorten, Follow 두 인터페이스)과 두 테스트 경계를 처음으로 세우는 티켓이다. 명세: `.scratch/short-url-mvp/spec.md`

**Blocked by:** 01 (개발 환경과 빈 Dioxus fullstack 앱)

**Status:** ready-for-agent

- [ ] Shortener 모듈이 Shorten(입력 문자열 → Short Link)과 Follow(경로 조각 → Original URL 또는 없음)를 공개한다
- [ ] Link Number는 1부터 차례로 발급된다 (ADR 0001)
- [ ] Short Link 앞부분은 `BASE_URL` 환경 변수를 따르고, 없으면 `http://localhost:8080`이다
- [ ] SQLite 저장소가 서버 시작 시 없으면 자동으로 준비된다
- [ ] 첫 화면: 입력칸, "줄이기" 버튼, 결과(Short Link + "복사" 버튼). 복사 버튼이 Short Link를 클립보드에 넣는다
- [ ] `GET /{Link Number}`가 302와 `Location: <Original URL>`을 돌려준다
- [ ] 경계 1 테스트: 임시 SQLite로 Shorten → Follow 왕복, 번호가 1, 2, 3 차례로 발급됨
- [ ] 경계 2 테스트: 실제 서버에 Shorten 서버 함수 호출 후 `GET /{번호}`가 302와 올바른 `Location`
- [ ] 직접 실행해서 줄이기 → 복사 → 브라우저로 Short Link 열기까지 확인했다
