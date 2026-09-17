//! Replay Ceph's committed crushtool mappings through the public Rust API.
//! Fixture provenance, checksums and adaptations: fixtures/README.md.

use bytes::Bytes;
use rados::crush::{CrushMap, mapper::crush_do_rule};
use std::collections::BTreeMap;
use std::ops::RangeInclusive;

fn decode(data: &'static [u8]) -> CrushMap {
    let mut bytes = Bytes::from_static(data);
    let map = CrushMap::decode(&mut bytes).expect("decode original Ceph fixture");
    assert!(bytes.is_empty(), "unconsumed CRUSH map bytes");
    map
}

fn check_mappings(
    map: CrushMap,
    transcript: &str,
    rule: u32,
    replicas: RangeInclusive<usize>,
    out_devices: &[usize],
) {
    let mut weights = vec![0x10000; map.max_devices as usize];
    for &device in out_devices {
        weights[device] = 0;
    }
    let mut fixture_sizes = BTreeMap::<(usize, usize), usize>::new();
    let mut expected_sizes = BTreeMap::new();
    let mut mappings = Vec::new();
    // Validate the entire transcript before the first mapper assertion, so
    // an early placement failure cannot hide missing seeds or statistics.
    for (line_number, line) in transcript.lines().enumerate() {
        let line = line.trim();
        if let Some(mapping) = line.strip_prefix("CRUSH rule ") {
            let (input, output) = mapping.split_once(" [").expect("mapping vector");
            let (source_rule, seed) = input.split_once(" x ").expect("mapping input");
            assert_eq!(source_rule.parse::<u32>().unwrap(), rule);
            let x = seed.parse::<u32>().unwrap();
            assert_eq!(x, (mappings.len() % 1024) as u32, "missing/reordered seed");
            let numrep = replicas.start() + mappings.len() / 1024;
            assert!(replicas.contains(&numrep), "extra mapping record");
            let items = output.strip_suffix(']').expect("complete mapping vector");
            let expected: Vec<i32> = if items.is_empty() {
                Vec::new()
            } else {
                items.split(',').map(|item| item.parse().unwrap()).collect()
            };
            *fixture_sizes.entry((numrep, expected.len())).or_default() += 1;
            mappings.push((line_number + 1, x, numrep, expected));
        } else if let Some((prefix, statistic)) = line.split_once(" num_rep ") {
            assert!(prefix.starts_with(&format!("rule {rule} (")));
            let (numrep, statistic) = statistic.split_once(" result size == ").unwrap();
            let (size, count) = statistic.split_once(":\\t").unwrap();
            let count = count.strip_suffix("/1024 (esc)").unwrap();
            let key = (
                numrep.parse::<usize>().unwrap(),
                size.parse::<usize>().unwrap(),
            );
            assert!(
                expected_sizes
                    .insert(key, count.parse::<usize>().unwrap())
                    .is_none()
            );
        } else {
            // CLI-only command, range banner and optional map-modified advisory.
            assert!(
                line.starts_with("$ crushtool ")
                    || line
                        == format!(
                            "rule {rule} ({name}), x = 0..1023, numrep = {min}..{max}",
                            name = map.rule_names.get(&rule).unwrap(),
                            min = replicas.start(),
                            max = replicas.end()
                        )
                    || line
                        == "crushtool successfully built or modified map.  Use '-o <file>' to write it out.",
                "unrecognized fixture line {}: {line}",
                line_number + 1
            );
        }
    }
    assert_eq!(
        mappings.len(),
        replicas.count() * 1024,
        "incomplete fixture"
    );
    assert_eq!(
        fixture_sizes, expected_sizes,
        "inconsistent fixture statistics"
    );

    let mut actual = Vec::new();
    let mut actual_sizes = BTreeMap::<(usize, usize), usize>::new();
    for (line, x, numrep, expected) in mappings {
        crush_do_rule(&map, rule, x, &mut actual, numrep, &weights).unwrap_or_else(|error| {
            panic!("line {line}, rule {rule}, x {x}, replicas {numrep}: {error}")
        });
        assert_eq!(
            actual, expected,
            "line {line}, rule {rule}, x {x}, replicas {numrep}"
        );
        *actual_sizes.entry((numrep, actual.len())).or_default() += 1;
    }
    assert_eq!(actual_sizes, expected_sizes, "result-size histograms");
}

// Upstream (cmd-01 is locally assigned):
// v17.2.7/src/test/cli/crushtool/test-map-bobtail-tunables.t::cmd-01
// https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/test-map-bobtail-tunables.t#L1
// v20.2.4/src/test/cli/crushtool/test-map-bobtail-tunables.t::cmd-01
// https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/test-map-bobtail-tunables.t#L1
#[test]
fn bobtail_tunables() {
    let mut map = decode(include_bytes!("fixtures/test-map-a.crushmap"));
    map.choose_local_tries = 0;
    map.choose_local_fallback_tries = 0;
    map.choose_total_tries = 50;
    map.chooseleaf_descend_once = 1;
    check_mappings(
        map,
        include_str!("fixtures/test-map-bobtail-tunables.t"),
        0,
        1..=10,
        &[],
    );
}

// Upstream (cmd-01 is locally assigned):
// v17.2.7/src/test/cli/crushtool/test-map-firefly-tunables.t::cmd-01
// https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/test-map-firefly-tunables.t#L1
// v20.2.4/src/test/cli/crushtool/test-map-firefly-tunables.t::cmd-01
// https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/test-map-firefly-tunables.t#L1
#[test]
fn firefly_tunables() {
    let mut map = decode(include_bytes!("fixtures/test-map-vary-r.crushmap"));
    map.choose_local_tries = 0;
    map.choose_local_fallback_tries = 0;
    map.choose_total_tries = 50;
    map.chooseleaf_descend_once = 1;
    map.chooseleaf_vary_r = 1;
    check_mappings(
        map,
        include_str!("fixtures/test-map-firefly-tunables.t"),
        0,
        1..=10,
        &[12, 20, 30],
    );
}

// Upstream (cmd-01 is locally assigned):
// v17.2.7/src/test/cli/crushtool/test-map-hammer-tunables.t::cmd-01
// https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/test-map-hammer-tunables.t#L1
// v20.2.4/src/test/cli/crushtool/test-map-hammer-tunables.t::cmd-01
// https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/test-map-hammer-tunables.t#L1
#[test]
fn hammer_tunables() {
    let map = decode(include_bytes!("fixtures/test-map-hammer-tunables.crushmap"));
    check_mappings(
        map,
        include_str!("fixtures/test-map-hammer-tunables.t"),
        0,
        1..=10,
        &[12, 20, 30],
    );
}

// Upstream (cmd-01 is locally assigned):
// v17.2.7/src/test/cli/crushtool/test-map-indep.t::cmd-01
// https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/test-map-indep.t#L1
// v20.2.4/src/test/cli/crushtool/test-map-indep.t::cmd-01
// https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/test-map-indep.t#L1
#[test]
fn indep() {
    let mut map = decode(include_bytes!("fixtures/test-map-indep.crushmap"));
    map.choose_local_tries = 0;
    map.choose_local_fallback_tries = 0;
    map.choose_total_tries = 50;
    map.chooseleaf_descend_once = 2;
    check_mappings(
        map,
        include_str!("fixtures/test-map-indep.t"),
        1,
        1..=10,
        &[],
    );
}

// Upstream (cmd-01 is locally assigned):
// v17.2.7/src/test/cli/crushtool/test-map-jewel-tunables.t::cmd-01
// https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/test-map-jewel-tunables.t#L1
// v20.2.4/src/test/cli/crushtool/test-map-jewel-tunables.t::cmd-01
// https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/test-map-jewel-tunables.t#L1
#[test]
fn jewel_tunables() {
    let map = decode(include_bytes!("fixtures/test-map-jewel-tunables.crushmap"));
    check_mappings(
        map,
        include_str!("fixtures/test-map-jewel-tunables.t"),
        0,
        1..=10,
        &[12, 20, 30],
    );
}

// Upstream (cmd-01 is locally assigned):
// v17.2.7/src/test/cli/crushtool/test-map-legacy-tunables.t::cmd-01
// https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/test-map-legacy-tunables.t#L1
// v20.2.4/src/test/cli/crushtool/test-map-legacy-tunables.t::cmd-01
// https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/test-map-legacy-tunables.t#L1
#[test]
fn legacy_tunables() {
    let map = decode(include_bytes!("fixtures/test-map-a.crushmap"));
    check_mappings(
        map,
        include_str!("fixtures/test-map-legacy-tunables.t"),
        0,
        1..=10,
        &[],
    );
}

// Upstream (cmd-01 is locally assigned):
// v17.2.7/src/test/cli/crushtool/test-map-tries-vs-retries.t::cmd-01
// https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/test-map-tries-vs-retries.t#L1
// v20.2.4/src/test/cli/crushtool/test-map-tries-vs-retries.t::cmd-01
// https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/test-map-tries-vs-retries.t#L1
#[test]
fn tries_vs_retries() {
    let map = decode(include_bytes!(
        "fixtures/test-map-tries-vs-retries.crushmap"
    ));
    check_mappings(
        map,
        include_str!("fixtures/test-map-tries-vs-retries.t"),
        0,
        1..=10,
        &[0, 8],
    );
}

// Upstream (cmd-01 is locally assigned):
// v17.2.7/src/test/cli/crushtool/test-map-vary-r-0.t::cmd-01
// https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/test-map-vary-r-0.t#L1
// v20.2.4/src/test/cli/crushtool/test-map-vary-r-0.t::cmd-01
// https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/test-map-vary-r-0.t#L1
#[test]
fn vary_r_0() {
    let mut map = decode(include_bytes!("fixtures/test-map-vary-r.crushmap"));
    map.chooseleaf_vary_r = 0;
    check_mappings(
        map,
        include_str!("fixtures/test-map-vary-r-0.t"),
        3,
        2..=4,
        &[0, 4, 9],
    );
}

// Upstream (cmd-01 is locally assigned):
// v17.2.7/src/test/cli/crushtool/test-map-vary-r-1.t::cmd-01
// https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/test-map-vary-r-1.t#L1
// v20.2.4/src/test/cli/crushtool/test-map-vary-r-1.t::cmd-01
// https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/test-map-vary-r-1.t#L1
#[test]
fn vary_r_1() {
    let mut map = decode(include_bytes!("fixtures/test-map-vary-r.crushmap"));
    map.chooseleaf_vary_r = 1;
    check_mappings(
        map,
        include_str!("fixtures/test-map-vary-r-1.t"),
        3,
        2..=4,
        &[0, 4, 9],
    );
}

// Upstream (cmd-01 is locally assigned):
// v17.2.7/src/test/cli/crushtool/test-map-vary-r-2.t::cmd-01
// https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/test-map-vary-r-2.t#L1
// v20.2.4/src/test/cli/crushtool/test-map-vary-r-2.t::cmd-01
// https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/test-map-vary-r-2.t#L1
#[test]
fn vary_r_2() {
    let mut map = decode(include_bytes!("fixtures/test-map-vary-r.crushmap"));
    map.chooseleaf_vary_r = 2;
    check_mappings(
        map,
        include_str!("fixtures/test-map-vary-r-2.t"),
        3,
        2..=4,
        &[0, 4, 9],
    );
}

// Upstream (cmd-01 is locally assigned):
// v17.2.7/src/test/cli/crushtool/test-map-vary-r-3.t::cmd-01
// https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/test-map-vary-r-3.t#L1
// v20.2.4/src/test/cli/crushtool/test-map-vary-r-3.t::cmd-01
// https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/test-map-vary-r-3.t#L1
#[test]
fn vary_r_3() {
    let mut map = decode(include_bytes!("fixtures/test-map-vary-r.crushmap"));
    map.chooseleaf_vary_r = 3;
    check_mappings(
        map,
        include_str!("fixtures/test-map-vary-r-3.t"),
        3,
        2..=4,
        &[0, 4, 9],
    );
}

// Upstream (cmd-01 is locally assigned):
// v17.2.7/src/test/cli/crushtool/test-map-vary-r-4.t::cmd-01
// https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/test-map-vary-r-4.t#L1
// v20.2.4/src/test/cli/crushtool/test-map-vary-r-4.t::cmd-01
// https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/test-map-vary-r-4.t#L1
#[test]
fn vary_r_4() {
    let mut map = decode(include_bytes!("fixtures/test-map-vary-r.crushmap"));
    map.chooseleaf_vary_r = 4;
    check_mappings(
        map,
        include_str!("fixtures/test-map-vary-r-4.t"),
        3,
        2..=4,
        &[0, 4, 9],
    );
}
