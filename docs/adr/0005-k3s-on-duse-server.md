---
Status: accepted
---

# duse 서버에서 compose 대신 K3s로 돌리기

같은 서버의 방송 프로젝트가 운영을 docker compose에서 K3s로 옮긴다(Traefik 없이 설치하고 HTTPS는 호스트 Caddy가 계속 맡는다). 서버 하나에서 두 가지 실행 방식을 함께 관리하지 않도록 short-url도 K3s로 옮기고, 서버에서 Docker를 지운다(2026-11-21). 앱은 Pod 하나(`replicas: 1`, `Recreate`)로 돈다. SQLite 파일 하나에 쓰므로 여러 개로 늘리지 않는다.

- **네트워크**: `hostNetwork`로 `127.0.0.1:8190`에만 연다. ServiceLB(LoadBalancer)나 NodePort는 `0.0.0.0`에 열려 서버 iptables를 우회하고 OCI 보안 목록에만 기대게 된다. compose 때의 "127.0.0.1에만 연다" 약속을 그대로 지키고, Caddyfile과 상태 카드의 `health_url`도 바꾸지 않는다.
- **저장소**: PVC가 아니라 `hostPath`로 compose 때와 같은 `/opt/short-url/data`를 쓴다. 데이터 이전이 없고, 백업 timer와 되살리기 절차가 같은 경로로 동작한다.
- **이미지**: 레지스트리 없이 Mac에서 빌드해 `k3s ctr images import`로 넣는다(`imagePullPolicy: Never`). ADR 0004의 "서버 접속 키나 레지스트리를 외부에 맡기지 않는다"를 이어 간다.
- **비밀값**: 서버의 `/opt/short-url/.env`에서 배포 때마다 Secret `short-url-kakao`를 만든다. 저장소에는 넣지 않는다.

## 고려한 다른 안

- **short-url만 compose에 남기기**: 바꿀 것이 없지만 서버에 Docker와 K3s 두 런타임의 상태 확인·켜기·끄기 방법이 섞인다.
- **local-path PVC**: K3s다운 방식이지만 데이터가 `/var/lib/rancher/k3s/storage/<임의 이름>`으로 옮겨 가서 백업 경로와 되살리기 절차를 다시 써야 한다.
