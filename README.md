# 숫자형 단축 URL 서비스

## 배포 URL

- 서비스 URL: https://s.duse.dev
- 테스트 단축 URL: https://s.duse.dev/1
- 테스트 원본 URL: https://wiwi-pe.tistory.com/318

## 배포

duse 서버(OCI 무료 범위의 ARM VM)에 컨테이너 하나로 올라가 있고, 저장소는 서버 디스크의 SQLite 파일이다(ADR 0004). git push로는 배포되지 않는다. Mac에서 아래 명령 하나로 배포한다.

```bash
scripts/deploy.sh
```

1. `cargo test`를 돌리고, 실패하면 멈춘다.
2. `docker build --platform linux/arm64`로 이미지를 만든다(Mac과 서버가 둘 다 arm64).
3. `docker save | ssh duse docker load`로 레지스트리 없이 서버에 이미지를 넣는다.
4. 서버의 `/opt/short-url`에서 `docker compose up -d`로 컨테이너를 바꾼다.
5. `https://s.duse.dev/api/stats`가 응답하는지 확인한다.

필요한 것: Docker Desktop이 켜져 있을 것, `~/.ssh/config`에 서버 별칭 `duse`가 있을 것. 켜기·끄기·상태는 `surl-on`, `surl-off`, `surl-status`(`~/.zshrc`)로 한다.

처음 한 번 하는 서버 설정(카카오 키, 백업 timer, Caddy), 백업에서 되살리기, 카카오 로그인 설정은 [docs/deploy/duse.md](docs/deploy/duse.md)에 있다.
