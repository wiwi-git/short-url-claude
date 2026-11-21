#!/usr/bin/env bash
# short-url을 duse 서버(s.duse.dev)의 K3s에 배포한다. 자세한 내용은 docs/deploy/duse.md.
# 테스트 → arm64 이미지 빌드 → 서버 확인 → 이미지 전송(k3s ctr import) → 매니페스트·비밀 적용 → 재시작 → 응답 확인.
# 서버는 ~/.ssh/config의 별칭으로 찾는다(기본 `duse`, DUSE_SSH로 바꿀 수 있다). kubectl은 서버에서 `sudo k3s kubectl`로 돈다.
set -euo pipefail

host="${DUSE_SSH:-duse}"
dir=/opt/short-url
ns=short-url
url=https://s.duse.dev
kubectl="sudo k3s kubectl"

cd "$(dirname "$0")/.."

echo "== 테스트"
cargo test

echo "== 이미지 빌드 (linux/arm64)"
# provenance 증명 매니페스트를 빼야 `k3s ctr images import`가 빠진 blob 없이 받는다.
docker build --platform linux/arm64 --provenance=false -t short-url:latest .

echo "== 서버 준비 확인"
# 한 줄에 한 문제씩 적고, 하나라도 있으면 멈춘다.
problems=$(ssh "$host" bash -s <<EOF
command -v k3s >/dev/null || echo "K3s가 설치되어 있지 않아요."
test -f $dir/.env || echo "$dir/.env(카카오 키)가 없어요."
test -d $dir/data || echo "$dir/data 폴더가 없어요."
EOF
)
if [[ -n "$problems" ]]; then
  echo "$problems" >&2
  echo "docs/deploy/duse.md의 '처음 한 번 설정'을 먼저 해 주세요." >&2
  exit 1
fi

echo "== 매니페스트 전송"
ssh "$host" "rm -rf $dir/k8s && mkdir -p $dir/k8s"
scp -q deploy/k8s/*.yaml "$host:$dir/k8s/"

echo "== 이미지 전송 (k3s containerd)"
docker save short-url:latest | gzip | ssh "$host" 'gunzip | sudo k3s ctr images import -'

echo "== 적용"
# 비밀값은 서버의 .env에서 바로 만든다. 값은 서버 밖으로 나가거나 화면에 찍히지 않는다.
ssh "$host" "set -e
$kubectl apply -f $dir/k8s/namespace.yaml
sudo cat $dir/.env | $kubectl -n $ns create secret generic short-url-kakao --from-env-file=/dev/stdin --dry-run=client -o yaml | $kubectl apply -f -
$kubectl apply -k $dir/k8s
# 태그(latest)가 같아도 새로 넣은 이미지로 Pod를 다시 띄운다.
$kubectl -n $ns rollout restart deployment/short-url
$kubectl -n $ns rollout status deployment/short-url --timeout=180s"

echo "== 응답 확인"
for _ in $(seq 1 30); do
  if stats=$(curl -fsS "$url/api/stats"); then
    echo "배포 완료: $url  $stats"
    exit 0
  fi
  sleep 2
done
echo "$url/api/stats가 응답하지 않아요. 서버에서 '$kubectl -n $ns logs deploy/short-url'을 확인해 주세요." >&2
exit 1
