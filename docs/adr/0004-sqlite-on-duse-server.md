---
Status: accepted (ADR 0002를 대체한다)
---

# 자체 서버(duse)로 옮기며 저장소를 SQLite로

Render 무료 Postgres가 30일 만료를 앞두고 있어, 배포를 OCI 무료 범위의 duse 서버(이미 방송 프로젝트가 도는 ARM VM)로 옮긴다. 이 서버는 디스크가 사라지지 않으므로 ADR 0002가 Postgres를 고른 이유(휘발성 파일 시스템)가 없어졌다. 2 OCPU / 12GB를 모든 서비스가 나눠 쓰는 서버라, 조금이라도 자원을 덜 쓰는 쪽을 택해 별도 DB 컨테이너 없이 앱 안에서 도는 SQLite로 돌아간다. 저장소는 SQLite 하나뿐이고, 로컬 개발과 테스트도 SQLite로 통일한다(두 벌로 유지하지 않는다는 ADR 0002의 원칙은 이어 간다).

## Considered Options

- **같은 서버의 compose에 Postgres 컨테이너**: 코드 변경이 없지만 50~150MB 메모리를 상시 쓴다.
- **외부 무료 Postgres(Neon 등)**: 서버 자원은 안 쓰지만 콜드 스타트가 있고, 또 다른 무료 정책에 기댄다.

## Consequences

- Render 데이터는 옮기지 않았다. 새 서버에서 Link Number는 1부터 다시 시작하고, 예전 `onrender.com` Short Link는 열리지 않는다.
- 백업은 서버가 직접 챙긴다(DB 파일을 매일 SQLite 백업 명령으로 복사, 7일 보관). 백업이 같은 VM에 있으므로 VM을 잃으면 함께 잃는다.
- Link Number의 빈틈없는 발급(ADR 0001)은 SQLite의 쓰기 잠금으로 직렬화한다.
- 배포는 git push로 자동으로 되지 않는다. Mac에서 이미지를 빌드해 서버로 보낸다(CI 레지스트리나 서버 접속 키를 외부에 맡기지 않으려는 선택).
