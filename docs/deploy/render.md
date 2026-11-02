# Render 배포

short-url은 Render 무료 웹 서비스 하나(저장소 루트의 `Dockerfile`로 빌드)와 무료 Render Postgres 하나로 배포한다. 둘 다 저장소 루트의 `render.yaml`(Blueprint)에 정의되어 있어, 대시보드에서 Blueprint로 한 번에 만든다. 저장소를 Postgres로 둔 이유는 ADR 0002.

## 처음 한 번 설정

1. Render 대시보드에서 **New → Blueprint**를 고른다.
2. GitHub 계정을 연결하고 `wiwi-git/short-url-claude` 저장소, `main` 브랜치를 고른다. Render가 `render.yaml`을 읽어 아래 두 리소스를 보여 준다.
   - `short-url`: 웹 서비스(Docker, Free, Singapore)
   - `short-url-db`: Postgres(Free, Singapore)
3. **Apply**(또는 Deploy Blueprint)를 누른다. 카카오 키 두 개(`KAKAO_REST_API_KEY`, `KAKAO_CLIENT_SECRET`)를 묻는 칸에 아래 "카카오 로그인 설정"에서 받은 값을 넣는다. 나머지는 자동으로 들어간다.
   - `DATABASE_URL`: `short-url-db`의 내부 연결 주소가 자동으로 들어간다.
   - `BASE_URL`: 서비스 주소(`https://short-url-xxxx.onrender.com`)가 자동으로 들어간다.
   - `PORT`: Render가 넣어 준다(기본 10000). 서버는 이 값을 그대로 따른다.
4. 첫 빌드는 Rust와 wasm을 처음부터 컴파일해서 10분 이상 걸릴 수 있다. 끝나면 서비스 페이지의 `onrender.com` 주소를 열어 카카오 로그인 → 줄이기 → Short Link 열기를 해 본다.

이후에는 `main`에 push할 때마다 자동으로 다시 배포된다.

## 카카오 로그인 설정

줄이기와 My Links는 카카오로 로그인해야 쓴다(ADR 0003). 서버는 `KAKAO_REST_API_KEY`나 `KAKAO_CLIENT_SECRET`이 없으면 시작하지 않는다.

1. [카카오 디벨로퍼스](https://developers.kakao.com)에서 앱을 만든다.
2. **카카오 로그인**을 켠다(상태 ON).
3. **Redirect URI**에 두 주소를 등록한다. 끝에 `/`를 붙이지 않고 스킴까지 똑같이 쓴다.
   - `https://<서비스 주소>.onrender.com/auth/kakao/callback`
   - `http://localhost:8080/auth/kakao/callback` (로컬 개발용)
4. **동의 항목**에서 닉네임(프로필 정보)을 켠다. 이메일, 프로필 사진은 받지 않는다.
5. **앱 키**에서 REST API 키를 복사해 `KAKAO_REST_API_KEY`로 쓴다.
6. REST API 키의 **클라이언트 시크릿** 코드를 만들고 활성화 상태를 **사용함**으로 둔 뒤 `KAKAO_CLIENT_SECRET`으로 쓴다. 이 값은 서버만 알아야 하므로 저장소나 채팅에 남기지 않는다.

이미 Blueprint로 만든 서비스라면, 이 기능이 들어간 커밋을 push하기 **전에** 서비스의 **Environment**에서 두 키를 먼저 추가한다. 키 없이 배포되면 서버가 시작하지 못해 배포가 실패한다.

## 통계 API

로그인 없이 `GET https://<서비스 주소>/api/stats`를 열면 `{"users": 한 번이라도 로그인한 User 수, "short_links": 지금 Follow되는 Short Link 수}`를 돌려준다.

## 로그인 기능 배포 때 한 번 일어나는 일

로그인 기능이 들어간 버전이 처음 뜰 때, 그 전에 만든(주인 없는) Short Link를 모두 지우고 Link Number를 1부터 다시 시작한다. 두 번째로 뜰 때부터는 아무것도 지우지 않는다(ADR 0003).

## 무료 플랜에서 알아 둘 것

- **잠들기**: 15분 동안 요청이 없으면 서버가 잠든다. 다음 요청 때 깨어나는 데 1분 정도 걸리고, 그동안 Render 로딩 화면이 보인다. Short Link도 처음 열 때 이만큼 늦을 수 있다.
- **DB 30일 만료**: 무료 Postgres는 만든 지 30일 뒤 만료되어 모든 Short Link와 User, 로그인 상태가 사라진다. 계속 쓰려면 그 전에 유료 DB로 올리거나, 다른 Postgres로 옮기고 `DATABASE_URL`만 바꾼다.
- **월 사용량 한도**: 무료 웹 서비스는 한 달 실행 시간과 대역폭에 한도가 있다. 대시보드 Billing에서 확인한다.
- **커스텀 도메인**: 붙이면 `render.yaml`의 `BASE_URL` 항목을 지우고 대시보드에서 `BASE_URL=https://<도메인>`을 직접 넣는다.

## 로컬에서 이미지 확인

Postgres를 띄운 뒤(명령은 `AGENTS.md`), 이미지를 Render와 같은 조건으로 실행한다.

```bash
docker build -t short-url:local .
docker run --rm -p 10000:10000 -e PORT=10000 -e BASE_URL=http://localhost:10000 -e DATABASE_URL=postgres://postgres:postgres@host.docker.internal:5432/short_url -e KAKAO_REST_API_KEY -e KAKAO_CLIENT_SECRET short-url:local
```

## 남은 결정

- 요청 속도 제한 등 남용 방지 (명세에서 배포 때 정하기로 미룬 항목)
- 30일 뒤 DB를 어디로 옮길지
