# short-url

긴 URL을 짧은 번호 주소로 바꿔 주고, 그 주소로 들어온 사람을 원래 URL로 보내 주는 서비스. 로그인 없이 누구나 쓴다.

## Language

**Original URL** (원래 URL):
사용자가 줄여 달라고 넣은 `http`/`https` 주소.
_Avoid_: long URL, target, destination

**Link Number** (링크 번호):
Original URL 하나에 발급되는 1부터 시작하는 정수. 같은 Original URL에는 늘 같은 Link Number가 대응한다.
_Avoid_: code, slug, ID, key

**Short Link** (단축 링크):
서비스 주소 뒤에 Link Number를 붙인 주소 (예: `example.com/42`). Original URL과 Link Number의 짝 자체를 가리킬 때도 이 말을 쓴다.
_Avoid_: short URL, alias, shortcut

**Shorten** (줄이기):
Original URL을 받아 Short Link를 돌려주는 일. 처음 보는 Original URL이면 새 Link Number를 발급하고, 본 적 있으면 기존 것을 돌려준다.
_Avoid_: create, register, generate

**Follow** (이동하기):
Short Link로 들어온 사람을 Original URL로 보내는 일.
_Avoid_: resolve, redirect, visit
