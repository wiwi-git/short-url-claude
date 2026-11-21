#!/usr/bin/env bash
# short-url을 duse 서버(s.duse.dev)에 배포한다. 자세한 내용은 docs/deploy/duse.md.
# 테스트 → arm64 이미지 빌드 → 서버로 전송(docker load) → compose up → 응답 확인.
# 서버는 ~/.ssh/config의 별칭으로 찾는다(기본 `duse`, DUSE_SSH로 바꿀 수 있다).
set -euo pipefail

host="${DUSE_SSH:-duse}"
dir=/opt/short-url
url=https://s.duse.dev

cd "$(dirname "$0")/.."

echo "== 테스트"
cargo test

echo "== 이미지 빌드 (linux/arm64)"
docker build --platform linux/arm64 -t short-url:latest .

echo "== 서버 준비 확인"
ssh "$host" "test -f $dir/.env" || {
  echo "서버에 $dir/.env가 없어요. docs/deploy/duse.md의 '처음 한 번 설정'을 먼저 해 주세요." >&2
  exit 1
}
scp -q deploy/compose.yaml "$host:$dir/compose.yaml"

echo "== 이미지 전송"
docker save short-url:latest | gzip | ssh "$host" 'gunzip | docker load'

echo "== 컨테이너 시작"
ssh "$host" "cd $dir && docker compose up -d && docker image prune -f >/dev/null"

echo "== 응답 확인"
for _ in $(seq 1 30); do
  if stats=$(curl -fsS "$url/api/stats"); then
    echo "배포 완료: $url  $stats"
    exit 0
  fi
  sleep 2
done
echo "$url/api/stats가 응답하지 않아요. 서버에서 'cd $dir && docker compose logs app'을 확인해 주세요." >&2
exit 1
