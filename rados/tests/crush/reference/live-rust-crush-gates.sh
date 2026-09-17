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
    echo "usage: $0 {quincy|tentacle} [all|class-pool|reweight|mon-classes|erasure]" >&2
    exit 2
    ;;
esac

root=$(git rev-parse --show-toplevel)
preflight="$root/rados/tests/crush/reference/live-mon-classes-preflight.sh"
command -v docker >/dev/null || { echo 'missing prerequisite: docker' >&2; exit 1; }
test -f "$preflight" || { echo "missing prerequisite: $preflight" >&2; exit 1; }
docker image inspect "$image" >/dev/null || { echo "missing prerequisite: $image" >&2; exit 1; }

name=''
pid=''
work=''
cleanup() {
  test -z "$name" || docker stop --timeout 5 "$name" >/dev/null 2>&1 || true
  test -z "$pid" || wait "$pid" 2>/dev/null || true
  test -z "$work" || rm -rf "$work"
  name=''
  pid=''
  work=''
}
trap cleanup EXIT

run_gate() {
  local test_name=$1 osds=$2 create_rbd=$3 config
  name="rados-live-crush-${1}-${test_name}-$$"
  work=$(mktemp -d "${TMPDIR:-/tmp}/rados-live-crush.XXXXXX")

  cat > "$work/ceph.conf" <<'EOF'
[global]
mon host = [v2:127.0.0.1:7131,v1:127.0.0.1:7130]
auth cluster required = none
auth service required = none
auth client required = none
EOF

  docker run --rm --name "$name" --network host --user ceph -e CRUSH_PREFLIGHT_HOLD=300 \
  -e CRUSH_PREFLIGHT_OSDS="$osds" -e CRUSH_PREFLIGHT_RBD="$create_rbd" \
  -v "$preflight:/preflight.sh:ro" "$image" bash /preflight.sh >"$work/cluster.log" 2>&1 &
  pid=$!
  for _ in $(seq 1 150); do
    grep -q 'LIVE PREFLIGHT PASS' "$work/cluster.log" && break
    kill -0 "$pid" 2>/dev/null || { cat "$work/cluster.log" >&2; cleanup; return 1; }
    sleep 1
  done
  grep -q 'LIVE PREFLIGHT PASS' "$work/cluster.log" || { cat "$work/cluster.log" >&2; cleanup; return 1; }
  config=$(docker exec --user ceph "$name" sh -c 'find /tmp -path "/tmp/rados-mon-classes.*/ceph.conf" -print -quit')
  test -n "$config" || { echo 'missing fixture ceph.conf' >&2; cat "$work/cluster.log" >&2; cleanup; return 1; }

  if ! CEPH_CONF="$work/ceph.conf" CEPH_LIVE_CONTAINER="$name" CEPH_LIVE_CONFIG="$config" \
    cargo test -p rados --test osdclient_live_crush_gates --offline -- \
    --ignored --exact "$test_name" --test-threads=1 --nocapture; then
    cleanup
    return 1
  fi
  cleanup
}

case "${2:-all}" in
  all)
    run_gate test_original_crush_class_and_pool_rule_gates 3 1
    run_gate test_reweight_vs_classes_consumes_live_pre_and_post_maps 3 0
    run_gate test_mon_classes_writes_before_lifecycle_mutations 3 1
    run_gate test_crush_rule_create_erasure_consumes_default_and_explicit_rules 1 0
    ;;
  class-pool) run_gate test_original_crush_class_and_pool_rule_gates 3 1 ;;
  reweight) run_gate test_reweight_vs_classes_consumes_live_pre_and_post_maps 3 0 ;;
  mon-classes) run_gate test_mon_classes_writes_before_lifecycle_mutations 3 1 ;;
  erasure) run_gate test_crush_rule_create_erasure_consumes_default_and_explicit_rules 1 0 ;;
  *)
    echo "usage: $0 {quincy|tentacle} [all|class-pool|reweight|mon-classes|erasure]" >&2
    exit 2
    ;;
esac
