# Render 배포

short-url은 Render 무료 웹 서비스 하나(저장소 루트의 `Dockerfile`로 빌드)와 무료 Render Postgres 하나로 배포한다. 둘 다 저장소 루트의 `render.yaml`(Blueprint)에 정의되어 있어, 대시보드에서 Blueprint로 한 번에 만든다. 저장소를 Postgres로 둔 이유는 ADR 0002.

## 처음 한 번 설정

1. Render 대시보드에서 **New → Blueprint**를 고른다.
2. GitHub 계정을 연결하고 `wiwi-git/short-url-claude` 저장소, `main` 브랜치를 고른다. Render가 `render.yaml`을 읽어 아래 두 리소스를 보여 준다.
   - `short-url`: 웹 서비스(Docker, Free, Singapore)
   - `short-url-db`: Postgres(Free, Singapore)
3. **Apply**(또는 Deploy Blueprint)를 누른다. 환경 변수는 따로 넣을 게 없다.
   - `DATABASE_URL`: `short-url-db`의 내부 연결 주소가 자동으로 들어간다.
   - `BASE_URL`: 서비스 주소(`https://short-url-xxxx.onrender.com`)가 자동으로 들어간다.
   - `PORT`: Render가 넣어 준다(기본 10000). 서버는 이 값을 그대로 따른다.
4. 첫 빌드는 Rust와 wasm을 처음부터 컴파일해서 10분 이상 걸릴 수 있다. 끝나면 서비스 페이지의 `onrender.com` 주소를 열어 줄이기 → Short Link 열기를 해 본다.

이후에는 `main`에 push할 때마다 자동으로 다시 배포된다.

## 무료 플랜에서 알아 둘 것

- **잠들기**: 15분 동안 요청이 없으면 서버가 잠든다. 다음 요청 때 깨어나는 데 1분 정도 걸리고, 그동안 Render 로딩 화면이 보인다. Short Link도 처음 열 때 이만큼 늦을 수 있다.
- **DB 30일 만료**: 무료 Postgres는 만든 지 30일 뒤 만료되어 모든 Short Link가 사라진다. 계속 쓰려면 그 전에 유료 DB로 올리거나, 다른 Postgres로 옮기고 `DATABASE_URL`만 바꾼다.
- **월 사용량 한도**: 무료 웹 서비스는 한 달 실행 시간과 대역폭에 한도가 있다. 대시보드 Billing에서 확인한다.
- **커스텀 도메인**: 붙이면 `render.yaml`의 `BASE_URL` 항목을 지우고 대시보드에서 `BASE_URL=https://<도메인>`을 직접 넣는다.

## 로컬에서 이미지 확인

Postgres를 띄운 뒤(명령은 `AGENTS.md`), 이미지를 Render와 같은 조건으로 실행한다.

```bash
docker build -t short-url:local .
docker run --rm -p 10000:10000 -e PORT=10000 -e BASE_URL=http://localhost:10000 -e DATABASE_URL=postgres://postgres:postgres@host.docker.internal:5432/short_url short-url:local
```

## 남은 결정

- 요청 속도 제한 등 남용 방지 (명세에서 배포 때 정하기로 미룬 항목)
- 30일 뒤 DB를 어디로 옮길지
