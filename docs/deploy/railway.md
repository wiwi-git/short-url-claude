# Railway 배포

short-url은 Railway 서비스 하나로 통째로 배포한다. 저장소 루트의 `Dockerfile`로 빌드하고(`railway.json`이 지정), SQLite 파일은 Railway Volume에 둔다.

## 처음 한 번 설정

1. **서비스 만들기**: Railway에서 New Project → Deploy from GitHub repo → `wiwi-git/short-url-claude`를 고른다. `railway.json` 덕분에 Dockerfile 빌드로 잡힌다.
2. **Volume 붙이기**: 서비스에 Volume을 추가하고 마운트 경로를 `/data`로 한다. SQLite 파일(`/data/short-url.db`)이 여기 저장되어 재배포해도 Short Link가 남는다.
3. **도메인 만들기**: Settings → Networking → Generate Domain. 포트를 묻으면 `8080`을 넣는다(아래 `PORT`와 같은 값).
4. **환경 변수**(Variables):

   | 변수 | 값 | 비고 |
   |---|---|---|
   | `BASE_URL` | `https://<만든 도메인>` | 필수. Short Link 앞부분이자 자기 주소 판단 기준. 잘못된 값이면 서버가 시작하지 않는다 |
   | `PORT` | `8080` | Dockerfile 기본값과 같다. 3번 도메인의 포트와 맞춘다 |
   | `DATABASE_PATH` | `/data/short-url.db` | Dockerfile 기본값. Volume 경로를 바꿨을 때만 고친다 |

   `IP=0.0.0.0`은 Dockerfile에 들어 있어 따로 넣지 않는다.
5. **배포 확인**: 배포가 끝나면 `https://<도메인>/`을 열어 줄이기 → Short Link 열기를 해 본다. `railway.json`의 헬스체크가 `/`를 확인한다.

이후에는 `main`에 push할 때마다 자동으로 다시 배포된다.

## 로컬에서 이미지 확인

```bash
docker build -t short-url:local .
docker run --rm -p 8080:8080 -e BASE_URL=http://localhost:8080 -v short-url-data:/data short-url:local
```

`http://localhost:8080`에서 줄이기와 이동하기를 확인한다. `short-url-data` 볼륨에 저장되므로 컨테이너를 다시 띄워도 Short Link가 남는다.

## 남은 결정

- 요청 속도 제한 등 남용 방지 (명세에서 배포 때 정하기로 미룬 항목)
- 커스텀 도메인, Cloudflare를 DNS/CDN으로 앞에 둘지
