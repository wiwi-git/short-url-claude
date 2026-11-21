# duse 서버 배포

short-url은 duse 서버(OCI 무료 범위의 ARM VM, 방송 프로젝트와 함께 씀)에서 컨테이너 하나로 돈다. 주소는 `https://s.duse.dev`이다. 저장소는 컨테이너 밖 서버 디스크에 둔 SQLite 파일이다(ADR 0004).

서버 공통 규칙(접속, 포트, Caddy, 비용, 비밀값 다루는 법)은 `AGENTS.md`의 "배포 서버"가 가리키는 문서를 따른다. 서버 주소와 비밀값은 이 저장소에 적지 않는다.

## 구성

| 항목 | 값 |
| --- | --- |
| 서버 폴더 | `/opt/short-url` (`compose.yaml`, `.env`, `data/`, `backups/`) |
| 포트 | `127.0.0.1:8190` → 컨테이너 `8080`. 밖에서는 Caddy의 `s.duse.dev` 블록으로만 닿는다 |
| 저장소 | `/opt/short-url/data/short-url.db` (컨테이너 안 `/data/short-url.db`) |
| 비밀값 | `/opt/short-url/.env`(권한 600)의 `KAKAO_REST_API_KEY`, `KAKAO_CLIENT_SECRET` |
| 백업 | `short-url-backup.timer`가 매일 `/opt/short-url/backups/short-url-<날짜>.db`를 만들고 7일치만 남긴다 |
| 상태 확인 | `https://s.duse.dev/api/stats`, 홈 카드 `https://duse.dev` |

`deploy/` 폴더의 파일이 서버 파일의 원본이다. `compose.yaml`은 배포할 때마다 서버로 복사되고, 백업 systemd 파일은 처음 한 번 설정할 때 복사한다.

## 배포

Mac에서 다음을 실행한다.

```bash
scripts/deploy.sh
```

스크립트는 `cargo test` → `docker build --platform linux/arm64` → `docker save | ssh duse docker load` → `docker compose up -d` → `/api/stats` 응답 확인 순서로 진행하고, 어느 단계든 실패하면 멈춘다. git push로는 배포되지 않는다. Mac과 서버가 둘 다 arm64라 Mac에서 만든 이미지가 서버에서 그대로 돈다. 첫 빌드는 Rust와 wasm을 처음부터 컴파일해서 10분 이상 걸릴 수 있다.

필요한 것: Docker Desktop이 켜져 있을 것, `~/.ssh/config`에 서버 별칭 `duse`가 있을 것(다른 이름이면 `DUSE_SSH=<별칭> scripts/deploy.sh`).

## 처음 한 번 설정

서버에서(`ssh duse`):

1. 폴더를 만든다: `sudo mkdir -p /opt/short-url/data /opt/short-url/backups && sudo chown -R ubuntu:ubuntu /opt/short-url`
2. 카카오 키를 넣는다. 사용자가 Mac 터미널에서 직접 실행한다(값이 화면, 대화, git에 남지 않는다).
   ```bash
   ssh -t duse 'umask 077; read -rsp "KAKAO_REST_API_KEY: " a; echo; read -rsp "KAKAO_CLIENT_SECRET: " b; echo; printf "KAKAO_REST_API_KEY=%s\nKAKAO_CLIENT_SECRET=%s\n" "$a" "$b" > /opt/short-url/.env'
   ```
3. 백업을 켠다: `sudo apt install -y sqlite3`, `deploy/short-url-backup.{service,timer}`를 `/etc/systemd/system/`에 복사한 뒤 `sudo systemctl daemon-reload && sudo systemctl enable --now short-url-backup.timer`
4. Caddy에 블록을 더한다(먼저 Caddyfile을 백업): `s.duse.dev { reverse_proxy 127.0.0.1:8190 }`
5. 홈 카드에 등록한다: `/etc/duse/services.json`에 `compose_dir` `/opt/short-url`, `compose_files` `["compose.yaml"]`, `health_url` `http://127.0.0.1:8190/api/stats`인 항목을 더한다.

Mac에서는 `scripts/deploy.sh`를 실행한다. 켜기, 끄기, 상태 보기는 `~/.zshrc`의 `surl-on`, `surl-off`, `surl-status`로 한다.

## 백업에서 되살리기

```bash
ssh duse 'cd /opt/short-url && docker compose stop && cp backups/short-url-<날짜>.db data/short-url.db && rm -f data/short-url.db-wal data/short-url.db-shm && docker compose start'
```

백업은 같은 VM 안에 있으므로 VM을 잃으면 함께 잃는다. 중요한 시점에는 Mac으로 받아 둔다: `scp duse:/opt/short-url/backups/short-url-<날짜>.db .`

## 카카오 로그인 설정

줄이기와 My Links는 카카오로 로그인해야 쓴다(ADR 0003). 서버는 `KAKAO_REST_API_KEY`나 `KAKAO_CLIENT_SECRET`이 없으면 시작하지 않는다.

1. [카카오 디벨로퍼스](https://developers.kakao.com)에서 앱을 만든다.
2. **카카오 로그인**을 켠다(상태 ON).
3. **Redirect URI**에 두 주소를 등록한다. 끝에 `/`를 붙이지 않고 스킴까지 똑같이 쓴다.
   - `https://s.duse.dev/auth/kakao/callback`
   - `http://localhost:8080/auth/kakao/callback` (로컬 개발용)
4. **동의 항목**에서 닉네임(프로필 정보)을 켠다. 이메일, 프로필 사진은 받지 않는다.
5. **앱 키**에서 REST API 키를 복사해 `KAKAO_REST_API_KEY`로 쓴다.
6. REST API 키의 **클라이언트 시크릿** 코드를 만들고 활성화 상태를 **사용함**으로 둔 뒤 `KAKAO_CLIENT_SECRET`으로 쓴다. 이 값은 서버만 알아야 하므로 저장소나 채팅에 남기지 않는다.

## 통계 API

로그인 없이 `GET https://s.duse.dev/api/stats`를 열면 `{"users": 한 번이라도 로그인한 User 수, "short_links": 지금 Follow되는 Short Link 수}`를 돌려준다.

## 로컬에서 이미지 확인

```bash
docker build -t short-url:local .
docker run --rm -p 10000:8080 -v "$PWD/.local-data:/data" -e BASE_URL=http://localhost:10000 -e DATABASE_URL=sqlite:///data/short-url.db -e KAKAO_REST_API_KEY -e KAKAO_CLIENT_SECRET short-url:local
```

## 남은 결정

- 요청 속도 제한 등 남용 방지 (명세에서 배포 때 정하기로 미룬 항목)
