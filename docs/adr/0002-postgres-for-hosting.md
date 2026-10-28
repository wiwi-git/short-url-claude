# 저장소는 SQLite가 아니라 Postgres

Render 무료 플랜에 배포하기로 했는데, 무료 웹 서비스는 파일 시스템이 휘발성이라(15분 유휴로 잠들거나 재배포할 때마다 초기화) SQLite 파일에 둔 Short Link가 사라지고, 영구 디스크는 유료 플랜에서만 붙일 수 있다. 그래서 저장소를 Postgres로 바꾸고, 로컬 개발과 테스트도 Postgres 하나로 통일한다(SQLite와 두 벌로 유지하지 않는다).

## Considered Options

- **Render 유료 플랜 + 영구 디스크에 SQLite**: 코드 변경이 없지만 매달 비용이 든다.
- **SQLite와 Postgres 둘 다 지원**: 로컬은 간편하지만 저장소 코드가 두 벌이 되고, 번호 발급의 동시성 동작이 달라질 수 있다.

## Consequences

- 무료 Render Postgres는 만든 지 30일 뒤 만료되어 모든 Short Link가 사라진다. 시험 단계로 받아들였고, 계속 운영하려면 유료 DB나 만료 없는 Postgres로 `DATABASE_URL`만 바꿔 옮긴다.
- Postgres 시퀀스는 충돌하거나 롤백된 INSERT에서도 값을 소모하므로, Link Number를 빈틈없이 차례로 발급하려면(ADR 0001) 시퀀스에 기대지 않고 발급을 직렬화해야 한다.
- 로컬에서 `cargo test`와 `dx serve`를 돌리려면 Postgres(예: Docker)가 떠 있어야 한다.
