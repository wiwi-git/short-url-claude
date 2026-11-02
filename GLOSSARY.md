# short-url

긴 URL을 짧은 번호 주소로 바꿔 주고, 그 주소로 들어온 사람을 원래 URL로 보내 주는 서비스. 카카오 계정으로 로그인한 User만 줄이기와 My Links를 쓰고, Follow는 누구나 할 수 있다.

## Language

**Original URL** (원래 URL):
사용자가 줄여 달라고 넣은 `http`/`https` 주소.
_Avoid_: long URL, target, destination

**Link Number** (링크 번호):
Original URL 하나에 발급되는 1부터 시작하는 정수. 같은 Original URL에는 늘 같은 Link Number가 대응한다. 한 번 발급한 번호는 그 Short Link가 지워져도 다시 쓰지 않는다.
_Avoid_: code, slug, ID, key

**Short Link** (단축 링크):
서비스 주소 뒤에 Link Number를 붙인 주소 (예: `example.com/42`). Original URL과 Link Number의 짝 자체를 가리킬 때도 이 말을 쓴다.
_Avoid_: short URL, alias, shortcut

**Shorten** (줄이기):
User가 Original URL을 받아 Short Link를 돌려받는 일. 처음 보는 Original URL이면 새 Link Number를 발급하고, 본 적 있으면 기존 것을 돌려준다. 어느 쪽이든 그 Short Link는 그 User의 My Links에 담긴다.
_Avoid_: create, register, generate

**Follow** (이동하기):
Short Link로 들어온 사람을 Original URL로 보내는 일.
_Avoid_: resolve, redirect, visit

**User** (사용자):
카카오 계정으로 한 번이라도 로그인한 사람. 카카오 회원번호로 구분하고, 닉네임은 로그인할 때마다 카카오의 최신 값으로 바꾼다.
_Avoid_: member, account, owner

**My Links** (내 링크):
한 User가 Shorten한 Short Link 모음. 같은 Short Link가 여러 User의 My Links에 함께 담길 수 있다.
_Avoid_: owned links, my list

**Remove** (내 링크에서 빼기):
User가 자기 My Links에서 Short Link를 빼는 일. 그 Short Link를 담은 User가 더는 없으면 Short Link도 지워지고 Follow하면 404가 된다.
_Avoid_: delete (Short Link 자체를 지우는 것처럼 들린다)
