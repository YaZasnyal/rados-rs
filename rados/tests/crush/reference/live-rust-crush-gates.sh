#!/usr/bin/env bash
set -euo pipefail

case "${1:-}" in
  quincy)
    image='quay.io/ceph/ceph@sha256:a70ccb2d8a0e814aa1c009e7541289b99bf039521727b6f44611499e9ca3fada'
    ;;
  tentacle)
    image='quay.io/ceph/ceph@sha256:6e6bc7b28fa1b334108a3646af5533dfb50db508efdf5b358eb7dd0dd37a48aa'
    ;;
  *)
    echo "usage: $0 {quincy|tentacle}" >&2
    exit 2
    ;;
esac

root=$(git rev-parse --show-toplevel)
preflight="$root/rados/tests/crush/reference/live-mon-classes-preflight.sh"
command -v docker >/dev/null || { echo 'missing prerequisite: docker' >&2; exit 1; }
test -f "$preflight" || { echo "missing prerequisite: $preflight" >&2; exit 1; }
docker image inspect "$image" >/dev/null || { echo "missing prerequisite: $image" >&2; exit 1; }

name="rados-live-crush-${1}-$$"
work=$(mktemp -d "${TMPDIR:-/tmp}/rados-live-crush.XXXXXX")
trap 'docker stop --time 5 "$name" >/dev/null 2>&1 || true; wait "$pid" 2>/dev/null || true; rm -rf "$work"' EXIT

cat > "$work/ceph.conf" <<'EOF'
[global]
mon host = [v2:127.0.0.1:7131,v1:127.0.0.1:7130]
auth cluster required = none
auth service required = none
auth client required = none
EOF

docker run --rm --name "$name" --network host --user ceph -e CRUSH_PREFLIGHT_HOLD=300 \
  -v "$preflight:/preflight.sh:ro" "$image" bash /preflight.sh >"$work/cluster.log" 2>&1 &
pid=$!
for _ in $(seq 1 150); do
  grep -q 'LIVE PREFLIGHT PASS' "$work/cluster.log" && break
  kill -0 "$pid" 2>/dev/null || { cat "$work/cluster.log" >&2; exit 1; }
  sleep 1
done
grep -q 'LIVE PREFLIGHT PASS' "$work/cluster.log" || { cat "$work/cluster.log" >&2; exit 1; }
config=$(docker exec --user ceph "$name" sh -c 'find /tmp -path "/tmp/rados-mon-classes.*/ceph.conf" -print -quit')
test -n "$config" || { echo 'missing fixture ceph.conf' >&2; cat "$work/cluster.log" >&2; exit 1; }

CEPH_CONF="$work/ceph.conf" CEPH_LIVE_CONTAINER="$name" CEPH_LIVE_CONFIG="$config" \
  cargo test -p rados --test osdclient_live_crush_gates --offline -- \
  --ignored --exact test_original_crush_class_and_pool_rule_gates --test-threads=1 --nocapture
