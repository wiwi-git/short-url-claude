# 빌드 단계: 클라이언트 wasm과 서버 바이너리를 `dx bundle`로 함께 만든다.
FROM rust:1-trixie AS builder

RUN rustup target add wasm32-unknown-unknown
# dx 버전은 Cargo.toml의 dioxus 크레이트 버전(0.7.x)과 맞춘다.
RUN curl -L --proto '=https' --tlsv1.2 -sSf \
        https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash \
    && cargo binstall dioxus-cli@0.7.10 --no-confirm

WORKDIR /app
COPY . .
RUN dx bundle --web --release --out-dir /app/bundle

# 실행 단계: 서버 바이너리와 그 옆의 public/만 담는다.
FROM debian:trixie-slim

WORKDIR /app
COPY --from=builder /app/bundle/server /app/server
COPY --from=builder /app/bundle/public /app/public

# Dioxus 서버는 IP·PORT 환경 변수로 주소를 정한다. 기본 IP가 127.0.0.1이라 컨테이너 밖에서 닿도록 0.0.0.0으로 연다.
# PORT는 Railway가 넣어 준다. BASE_URL과 DATABASE_PATH는 배포 환경에서 설정한다.
ENV IP=0.0.0.0 \
    PORT=8080 \
    DATABASE_PATH=/data/short-url.db

RUN mkdir -p /data
EXPOSE 8080
CMD ["/app/server"]
