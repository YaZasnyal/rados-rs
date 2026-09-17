//! Original live CRUSH class and pool-rule integration gates.
//!
//! Run through `rados/tests/crush/reference/live-rust-crush-gates.sh`; it
//! starts an isolated pinned Ceph cluster and supplies `CEPH_CONF` plus
//! `CEPH_LIVE_CONTAINER` for the selected ignored test.

use std::process::Command;
use std::time::Duration;

use bytes::Bytes;

mod common;
use common::build_test_client;

fn admin(script: &str) {
    let container = std::env::var("CEPH_LIVE_CONTAINER")
        .expect("CEPH_LIVE_CONTAINER is required; use live-rust-crush-gates.sh");
    let config = std::env::var("CEPH_LIVE_CONFIG")
        .expect("CEPH_LIVE_CONFIG is required; use live-rust-crush-gates.sh");
    let script = format!("export CEPH_CONF={config}; {script}");
    let output = Command::new("docker")
        .args(["exec", &container, "bash", "-lc", &script])
        .output()
        .expect("run pinned Ceph admin command");
    assert!(
        output.status.success(),
        "admin command failed:\n{script}\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

async fn write(client: &rados::Client, object: &str) {
    let ioctx = client.open_pool("rbd").await.expect("open rbd");
    tokio::time::timeout(
        Duration::from_secs(45),
        ioctx.write_full(object, Bytes::from_static(b"ABCDEF\n")),
    )
    .await
    .expect("write_full deadline")
    .expect("write_full");
}

// Upstream: v17.2.7/qa/standalone/crush/crush-classes.sh::TEST_classes and
// TEST_set_device_class; identical at v20.2.4.
// Upstream: v17.2.7/src/test/librados/pool.cc::LibRadosPools.PoolCreateWithCrushRule;
// identical at v20.2.4.
#[tokio::test]
#[ignore]
async fn test_original_crush_class_and_pool_rule_gates() {
    common::init_tracing();
    let client = build_test_client().await.expect("build_test_client");

    admin(
        r#"
        up() {
          actual=$(ceph osd map rbd "$1" -f json | python3 -c 'import json,sys; print(" ".join(map(str, json.load(sys.stdin)["up"])))')
          test "$actual" = "$2"
        }
        up SOMETHING '1 2 0'
        "#,
    );
    write(&client, "SOMETHING").await;

    admin(
        r#"
        ceph osd getcrushmap > /tmp/task9a-map
        crushtool -d /tmp/task9a-map -o /tmp/task9a-map.txt
        sed -i -e '/device 0 osd.0/s/$/ class ssd/' -e '/step take default/s/$/ class ssd/' /tmp/task9a-map.txt
        crushtool -c /tmp/task9a-map.txt -o /tmp/task9a-map-new
        ceph osd setcrushmap -i /tmp/task9a-map-new
        "#,
    );
    admin(
        r#"
        up() {
          actual=$(ceph osd map rbd SOMETHING_ELSE -f json | python3 -c 'import json,sys; print(" ".join(map(str, json.load(sys.stdin)["up"])))')
          test "$actual" = '0'
        }
        for delay in 2 4 8 16 32 64 128 256; do
          up && exit 0
          sleep "$delay"
          ceph osd dump
          ceph pg dump
        done
        exit 1
        "#,
    );
    write(&client, "SOMETHING_ELSE").await;

    admin(
        r#"
        ceph osd crush dump | grep -q '~ssd'
        ceph osd crush set-device-class ssd osd.0
        ceph osd crush class ls-osd ssd | grep -qx '0'
        ceph osd crush set-device-class ssd osd.1
        ceph osd crush class ls-osd ssd | grep -qx '1'
        ceph osd crush set-device-class ssd 0 1
        "#,
    );
    admin(
        r#"
        up() {
          actual=$(ceph osd map rbd SOMETHING_ELSE -f json | python3 -c 'import json,sys; print(" ".join(map(str, json.load(sys.stdin)["up"])))')
          test "$actual" = '0 1'
        }
        for delay in 2 4 8 16 32 64 128 256; do
          up && exit 0
          sleep "$delay"
          ceph osd crush dump
          ceph osd dump
          ceph pg dump
        done
        exit 1
        "#,
    );

    let osd = client.osd_client();
    let guard = format!("task9a-guard-{}", rand::random::<u32>());
    let explicit = format!("task9a-rule-{}", rand::random::<u32>());
    osd.create_pool(&guard, None)
        .await
        .expect("create default-rule guard pool");
    let _ = client.open_pool(&guard).await.expect("open guard pool");
    let application = client
        .mon_client()
        .invoke(
            vec![format!(
                r#"{{"prefix":"osd pool application enable","pool":"{guard}","app":"rados"}}"#
            )],
            Bytes::new(),
        )
        .await
        .expect("enable rados application");
    assert_eq!(application.retval, 0, "enable rados application");
    osd.create_pool(&explicit, Some(0))
        .await
        .expect("create explicit-rule pool");
    osd.delete_pool(&explicit, true)
        .await
        .expect("delete explicit-rule pool");
    osd.delete_pool(&guard, true)
        .await
        .expect("delete guard pool");
}
