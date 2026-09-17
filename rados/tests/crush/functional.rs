//! Functional ports of Ceph's CRUSH mapper tests.

use std::collections::{HashMap, HashSet};

use rados::crush::{
    BucketAlgorithm, BucketData, CrushBucket, CrushMap, CrushRule, CrushRuleStep, RuleOp, RuleType,
    mapper::crush_do_rule,
};

const NONE: i32 = 0x7fff_ffff;
const WEIGHT: u32 = 0x1_0000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Normal,
    Msr,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Family {
    Indep,
    FirstN,
}

fn bucket(id: i32, bucket_type: i32, items: Vec<i32>, item_weight: u32) -> CrushBucket {
    let size = items.len() as u32;
    // build_*_map explicitly creates a STRAW root. insert_item creates the
    // intermediate buckets with the optimal default algorithm, STRAW2.
    let alg = if bucket_type == 5 {
        BucketAlgorithm::Straw
    } else {
        BucketAlgorithm::Straw2
    };
    let item_weights = vec![item_weight; items.len()];
    CrushBucket {
        id,
        bucket_type,
        alg,
        hash: 0,
        weight: item_weight * size,
        size,
        data: if alg == BucketAlgorithm::Straw {
            BucketData::Straw {
                item_weights,
                straws: vec![WEIGHT; items.len()],
            }
        } else {
            BucketData::Straw2 { item_weights }
        },
        items,
    }
}

fn store_bucket(map: &mut CrushMap, bucket: CrushBucket) {
    let index = (-1 - bucket.id) as usize;
    if map.buckets.len() <= index {
        map.buckets.resize(index + 1, None);
    }
    map.buckets[index] = Some(bucket);
}

fn build_map(mode: Mode, family: Family, racks: usize, hosts: usize, osds: usize) -> CrushMap {
    let mut map = CrushMap::new();
    map.max_devices = (racks * hosts * osds) as i32;
    map.choose_local_tries = 0;
    map.choose_local_fallback_tries = 0;
    map.choose_total_tries = 50;
    map.chooseleaf_descend_once = 1;
    map.chooseleaf_vary_r = 1;
    map.chooseleaf_stable = 1;
    map.allowed_bucket_algs = (1 << 1) | (1 << 2) | (1 << 4) | (1 << 5);

    let mut next_bucket = -2;
    let mut next_osd = 0;
    let mut rack_ids = Vec::with_capacity(racks);
    for _ in 0..racks {
        let mut host_ids = Vec::with_capacity(hosts);
        let mut rack_id = None;
        for host in 0..hosts {
            let host_id = next_bucket;
            next_bucket -= 1;
            host_ids.push(host_id);
            if host == 0 {
                rack_id = Some(next_bucket);
                next_bucket -= 1;
            }
            let devices: Vec<_> = (next_osd..next_osd + osds as i32).collect();
            next_osd += osds as i32;
            store_bucket(&mut map, bucket(host_id, 1, devices, WEIGHT));
        }
        let rack_id = rack_id.unwrap();
        rack_ids.push(rack_id);
        store_bucket(&mut map, bucket(rack_id, 3, host_ids, WEIGHT * osds as u32));
    }
    store_bucket(
        &mut map,
        bucket(-1, 5, rack_ids, WEIGHT * (hosts * osds) as u32),
    );
    map.max_buckets = map.buckets.len() as i32;

    let (rule_type, steps) = match (mode, family) {
        (Mode::Msr, Family::Indep) => (
            RuleType::MsrIndep,
            vec![
                step(RuleOp::Take, -1, 0),
                step(RuleOp::ChooseMsr, 0, 1),
                step(RuleOp::ChooseMsr, 1, 0),
                step(RuleOp::Emit, 0, 0),
            ],
        ),
        (Mode::Msr, Family::FirstN) => (
            RuleType::MsrFirstN,
            vec![
                step(RuleOp::Take, -1, 0),
                step(RuleOp::ChooseMsr, 0, 1),
                step(RuleOp::ChooseMsr, 1, 0),
                step(RuleOp::Emit, 0, 0),
            ],
        ),
        (Mode::Normal, Family::Indep) => (
            RuleType::Erasure,
            vec![
                step(RuleOp::SetChooseLeafTries, 10, 0),
                step(RuleOp::Take, -1, 0),
                step(RuleOp::ChooseLeafIndep, 0, 1),
                step(RuleOp::Emit, 0, 0),
            ],
        ),
        (Mode::Normal, Family::FirstN) => (
            RuleType::Erasure,
            vec![
                step(RuleOp::SetChooseLeafTries, 0, 0),
                step(RuleOp::Take, -1, 0),
                step(RuleOp::ChooseLeafFirstN, 0, 1),
                step(RuleOp::Emit, 0, 0),
            ],
        ),
    };
    map.rules = vec![Some(CrushRule {
        rule_id: 0,
        rule_type,
        steps,
    })];
    map.max_rules = 1;
    map
}

fn step(op: RuleOp, arg1: i32, arg2: i32) -> CrushRuleStep {
    CrushRuleStep { op, arg1, arg2 }
}

fn place(map: &CrushMap, x: u32, count: usize, weights: &[u32]) -> Vec<i32> {
    let mut out = Vec::new();
    crush_do_rule(map, 0, x, &mut out, count, weights).unwrap();
    out
}

fn assert_no_duplicates(out: &[i32], context: &str) {
    let selected: Vec<_> = out.iter().copied().filter(|&item| item != NONE).collect();
    assert_eq!(
        selected.iter().copied().collect::<HashSet<_>>().len(),
        selected.len(),
        "{context}: {out:?}"
    );
}

fn toosmall(mode: Mode, family: Family) {
    let map = build_map(mode, family, 1, 3, 1);
    let weights = vec![WEIGHT; map.max_devices as usize];
    for x in 0..100 {
        let out = place(&map, x, 5, &weights);
        assert_no_duplicates(&out, &format!("{mode:?} {family:?} x={x}"));
        match family {
            Family::Indep => assert_eq!(out.iter().filter(|&&item| item == NONE).count(), 2),
            Family::FirstN => {
                assert_eq!(out.len(), 3);
                assert!(!out.contains(&NONE));
            }
        }
    }
}

fn basic(mode: Mode, family: Family) {
    let map = build_map(mode, family, 3, 3, 3);
    let weights = vec![WEIGHT; map.max_devices as usize];
    let count = if family == Family::Indep { 5 } else { 3 };
    for x in 0..100 {
        let out = place(&map, x, count, &weights);
        assert_eq!(out.len(), count, "{mode:?} {family:?} x={x}: {out:?}");
        assert!(!out.contains(&NONE));
        assert_no_duplicates(&out, &format!("{mode:?} {family:?} x={x}"));
    }
}

fn single_out_first(mode: Mode, family: Family) {
    let map = build_map(mode, family, 3, 3, 3);
    let count = if family == Family::Indep { 5 } else { 3 };
    for x in 0..1000 {
        let mut weights = vec![WEIGHT; map.max_devices as usize];
        let before = place(&map, x, count, &weights);
        assert_eq!(before.len(), count);
        assert!(!before.contains(&NONE));
        assert_no_duplicates(&before, &format!("before {mode:?} {family:?} x={x}"));
        weights[before[0] as usize] = 0;
        let after = place(&map, x, count, &weights);
        assert_eq!(after.len(), count, "{mode:?} {family:?} x={x}: {after:?}");
        assert_no_duplicates(&after, &format!("after {mode:?} {family:?} x={x}"));
        assert!(!after.contains(&before[0]));
        if family == Family::Indep {
            assert_ne!(after[0], NONE);
            assert_eq!(&after[1..], &before[1..]);
        } else if mode == Mode::Msr {
            assert_eq!(&after[..2], &before[1..]);
        }
    }
}

fn single_out_last(mode: Mode, family: Family) {
    let map = build_map(mode, family, 3, 3, 3);
    let count = if family == Family::Indep { 5 } else { 3 };
    for x in 0..1000 {
        let mut weights = vec![WEIGHT; map.max_devices as usize];
        let before = place(&map, x, count, &weights);
        assert_eq!(before.len(), count);
        assert!(!before.contains(&NONE));
        assert_no_duplicates(&before, &format!("before {mode:?} {family:?} x={x}"));
        let last = count - 1;
        weights[before[last] as usize] = 0;
        let after = place(&map, x, count, &weights);
        assert_eq!(after.len(), count, "{mode:?} {family:?} x={x}: {after:?}");
        assert_no_duplicates(&after, &format!("after {mode:?} {family:?} x={x}"));
        assert!(!after.contains(&before[last]));
        if family == Family::Indep {
            assert_ne!(after[last], NONE);
        }
        assert_eq!(&after[..last], &before[..last]);
    }
}

fn out_alt(mode: Mode, family: Family) {
    let mut map = build_map(mode, family, 3, 3, 3);
    if family == Family::Indep {
        map.choose_total_tries = 100;
    } else if mode == Mode::Normal {
        map.choose_total_tries = 500;
    }
    let mut weights = vec![WEIGHT; map.max_devices as usize];
    for osd in (0..26).step_by(2) {
        weights[osd] = 0;
    }
    for x in 0..100 {
        let out = place(&map, x, 9, &weights);
        assert_eq!(out.len(), 9, "{mode:?} {family:?} x={x}: {out:?}");
        assert!(!out.contains(&NONE));
        assert_no_duplicates(&out, &format!("{mode:?} {family:?} x={x}"));
    }
}

fn out_contig(mode: Mode, family: Family) {
    let mut map = build_map(mode, family, 3, 3, 3);
    if family == Family::Indep {
        map.choose_total_tries = 100;
    } else if mode == Mode::Normal {
        map.choose_total_tries = 500;
    }
    let mut weights = vec![WEIGHT; map.max_devices as usize];
    weights[..9].fill(0);
    for x in 0..100 {
        let out = place(&map, x, 7, &weights);
        assert_no_duplicates(&out, &format!("{mode:?} {family:?} x={x}"));
        match family {
            Family::Indep => {
                assert_eq!(out.len(), 7);
                assert_eq!(out.iter().filter(|&&item| item == NONE).count(), 1);
            }
            Family::FirstN => assert_eq!(out.len(), 6),
        }
    }
}

fn out_progressive(mode: Mode, family: Family) {
    let mut map = build_map(mode, family, 3, 3, 3);
    if family == Family::Indep {
        map.choose_total_tries = 100;
    } else if mode == Mode::Normal {
        map.choose_total_tries = 500;
    }
    for x in 1..5 {
        let mut weights = vec![WEIGHT; map.max_devices as usize];
        let mut previous = Vec::new();
        let mut positions = HashMap::new();
        for osd in 0..weights.len() {
            let out = place(&map, x, 7, &weights);
            assert_no_duplicates(&out, &format!("{mode:?} {family:?} x={x} out={osd}"));
            if osd > 0 {
                let changed = if family == Family::Indep {
                    out.iter()
                        .enumerate()
                        .filter(|&(index, item)| previous.get(index) != Some(item))
                        .count()
                } else {
                    let previous: HashSet<_> = previous.iter().copied().collect();
                    out.iter().filter(|item| !previous.contains(item)).count()
                };
                assert!(changed <= 3, "{mode:?} {family:?} x={x} out={osd}: {out:?}");
                if family == Family::Indep {
                    let moved = out
                        .iter()
                        .enumerate()
                        .filter(|(_, item)| **item != NONE)
                        .filter(|&(index, item)| {
                            positions.get(item).is_some_and(|&old| old != index)
                        })
                        .count();
                    assert!(moved <= 1, "{mode:?} x={x} out={osd}: {out:?}");
                }
            }
            weights[osd] = 0;
            positions = out
                .iter()
                .enumerate()
                .filter(|(_, item)| **item != NONE)
                .map(|(index, &item)| (item, index))
                .collect();
            previous = out;
        }
    }
}

macro_rules! variants {
    ($normal:ident, $msr:ident, $scenario:ident, $family:expr) => {
        #[test]
        fn $normal() {
            $scenario(Mode::Normal, $family);
        }

        #[test]
        fn $msr() {
            $scenario(Mode::Msr, $family);
        }
    };
}

// Upstream: v20.2.4/src/test/crush/crush.cc::IndepTest.toosmall (NORMAL, MSR)
// Source: https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/crush.cc#L152
variants!(
    indep_toosmall_normal,
    indep_toosmall_msr,
    toosmall,
    Family::Indep
);

// Upstream: v20.2.4/src/test/crush/crush.cc::IndepTest.basic (NORMAL, MSR)
// Source: https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/crush.cc#L171
variants!(indep_basic_normal, indep_basic_msr, basic, Family::Indep);

// Upstream: v20.2.4/src/test/crush/crush.cc::IndepTest.single_out_first (NORMAL, MSR)
// Source: https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/crush.cc#L190
variants!(
    indep_single_out_first_normal,
    indep_single_out_first_msr,
    single_out_first,
    Family::Indep
);

// Upstream: v20.2.4/src/test/crush/crush.cc::IndepTest.single_out_last (NORMAL, MSR)
// Source: https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/crush.cc#L230
variants!(
    indep_single_out_last_normal,
    indep_single_out_last_msr,
    single_out_last,
    Family::Indep
);

// Upstream: v20.2.4/src/test/crush/crush.cc::IndepTest.out_alt (NORMAL, MSR)
// Source: https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/crush.cc#L271
variants!(
    indep_out_alt_normal,
    indep_out_alt_msr,
    out_alt,
    Family::Indep
);

// Upstream: v20.2.4/src/test/crush/crush.cc::IndepTest.out_contig (NORMAL, MSR)
// Source: https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/crush.cc#L297
variants!(
    indep_out_contig_normal,
    indep_out_contig_msr,
    out_contig,
    Family::Indep
);

// Upstream: v20.2.4/src/test/crush/crush.cc::IndepTest.out_progressive (NORMAL, MSR)
// Source: https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/crush.cc#L322
variants!(
    indep_out_progressive_normal,
    indep_out_progressive_msr,
    out_progressive,
    Family::Indep
);

// Upstream: v20.2.4/src/test/crush/crush.cc::FirstnTest.basic (NORMAL, MSR)
// Source: https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/crush.cc#L485
variants!(firstn_basic_normal, firstn_basic_msr, basic, Family::FirstN);

// Upstream: v20.2.4/src/test/crush/crush.cc::FirstnTest.toosmall (NORMAL, MSR)
// Source: https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/crush.cc#L502
variants!(
    firstn_toosmall_normal,
    firstn_toosmall_msr,
    toosmall,
    Family::FirstN
);

// Upstream: v20.2.4/src/test/crush/crush.cc::FirstnTest.single_out_first (NORMAL, MSR)
// Source: https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/crush.cc#L519
variants!(
    firstn_single_out_first_normal,
    firstn_single_out_first_msr,
    single_out_first,
    Family::FirstN
);

// Upstream: v20.2.4/src/test/crush/crush.cc::FirstnTest.single_out_last (NORMAL, MSR)
// Source: https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/crush.cc#L560
variants!(
    firstn_single_out_last_normal,
    firstn_single_out_last_msr,
    single_out_last,
    Family::FirstN
);

// Upstream: v20.2.4/src/test/crush/crush.cc::FirstnTest.out_alt (NORMAL, MSR)
// Source: https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/crush.cc#L598
variants!(
    firstn_out_alt_normal,
    firstn_out_alt_msr,
    out_alt,
    Family::FirstN
);

// Upstream: v20.2.4/src/test/crush/crush.cc::FirstnTest.out_contig (NORMAL, MSR)
// Source: https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/crush.cc#L621
variants!(
    firstn_out_contig_normal,
    firstn_out_contig_msr,
    out_contig,
    Family::FirstN
);

// Upstream: v20.2.4/src/test/crush/crush.cc::FirstnTest.out_progressive (NORMAL, MSR)
// Source: https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/crush.cc#L644
variants!(
    firstn_out_progressive_normal,
    firstn_out_progressive_msr,
    out_progressive,
    Family::FirstN
);
