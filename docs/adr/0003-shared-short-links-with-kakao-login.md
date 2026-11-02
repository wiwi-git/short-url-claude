# 로그인은 카카오로, Short Link는 User끼리 공유한다

줄이기와 목록 보기를 카카오 로그인한 User만 쓸 수 있게 바꿨다. "같은 Original URL이면 같은 Short Link" 정책은 그대로 둔다. 그래서 Short Link는 한 사람의 것이 아니다. 각 User는 My Links에 Short Link를 담고, Remove는 자기 My Links에서 빼는 것이다. 그 Short Link를 담은 User가 더는 없을 때만 Short Link가 지워진다. Follow는 로그인 없이 된다. 이미 퍼진 Short Link를 받은 사람은 로그인하지 않으니까.

Short Link가 지워질 수 있게 되면서 "가장 큰 Link Number + 1"로 번호를 매기면 지운 번호가 다른 Original URL에 다시 붙는다. 그러면 이미 퍼진 Short Link가 엉뚱한 곳으로 간다. 그래서 지금까지 발급한 가장 큰 번호를 따로 기록하고, 그다음 번호를 발급한다 (ADR 0001의 "번호를 비우지 않는다"는 지운 경우를 빼면 그대로다).

로그인 상태는 서버가 Postgres에 세션을 30일 동안 저장하고, 브라우저에는 HttpOnly 쿠키만 둔다. 로그아웃은 이 서비스에서만 하고, 카카오 계정은 로그아웃시키지 않는다. 카카오에서는 회원번호와 닉네임만 받는다.

로그인 전에 만든 Short Link는 주인이 없어서, 이 변경을 배포할 때 모두 지우고 번호를 1부터 다시 시작한다. 그때까지 만든 Short Link는 개발 중 테스트용이라 퍼진 것이 없다.

## Considered Options

- **같은 Original URL이라도 User마다 다른 Short Link**: 주인이 분명해 삭제가 단순하지만, 기존 정책을 깨고 같은 URL의 번호가 늘어난다.
- **주인 없는 기존 Short Link를 남겨 두기**: 동작은 하지만 누구의 목록에도 안 보여 관리할 수 없다.
- **서명한 쿠키만으로 세션 유지**: DB가 필요 없지만 로그아웃해도 쿠키를 서버에서 무효로 만들 수 없다.

## Consequences

Remove해도 다른 User가 담고 있으면 Short Link는 계속 동작한다. 서비스 운영에 카카오 앱 설정과 `KAKAO_REST_API_KEY`, `KAKAO_CLIENT_SECRET` 환경 변수가 필요하다.
