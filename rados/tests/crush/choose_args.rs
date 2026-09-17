// Upstream: v17.2.7/src/test/crush/CrushWrapper.cc::CrushWrapperTest.choose_args_compat
// Source: https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/crush/CrushWrapper.cc#L1014
// Tentacle has the same setup and assertions at
// https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/CrushWrapper.cc#L1011.

use bytes::Bytes;
use rados::crush::mapper::crush_do_rule;
use rados::crush::{
    BucketAlgorithm, BucketData, CrushBucket, CrushChooseArg, CrushMap, CrushRule, CrushRuleStep,
    RuleOp, RuleType,
};
use rados::crush::{PgId, crush_do_rule_with_choose_args, pg_to_osds};

fn decode_fixture(bytes: &'static [u8]) -> CrushMap {
    CrushMap::decode(&mut Bytes::from_static(bytes)).unwrap()
}

fn map_with_default_choose_arg() -> CrushMap {
    let mut map = CrushMap::new();
    map.max_devices = 2;
    map.max_buckets = 1;
    map.buckets = vec![Some(CrushBucket {
        id: -1,
        bucket_type: 1,
        alg: BucketAlgorithm::Straw2,
        hash: 0,
        weight: 2 << 16,
        size: 2,
        items: vec![0, 1],
        data: BucketData::Straw2 {
            item_weights: vec![1 << 16, 1 << 16],
        },
    })];
    map.rules = vec![Some(CrushRule {
        rule_id: 0,
        rule_type: RuleType::Replicated,
        steps: vec![
            CrushRuleStep {
                op: RuleOp::Take,
                arg1: -1,
                arg2: 0,
            },
            CrushRuleStep {
                op: RuleOp::ChooseFirstN,
                arg1: 1,
                arg2: 0,
            },
            CrushRuleStep {
                op: RuleOp::Emit,
                arg1: 0,
                arg2: 0,
            },
        ],
    })];
    map.choose_args.insert(
        -1,
        vec![Some(CrushChooseArg {
            weight_set: vec![vec![1 << 16, 0]],
            ids: vec![],
        })],
    );
    map
}

#[test]
fn default_choose_args_override_straw2_weights() {
    let map = map_with_default_choose_arg();
    for x in 0..100 {
        let mut result = Vec::new();
        crush_do_rule(&map, 0, x, &mut result, 1, &[1 << 16, 1 << 16]).unwrap();
        assert_eq!(result, vec![0], "x={x}");
    }
}

#[test]
fn choose_args_compat_keeps_alternate_weights_and_legacy_folds_them() {
    // Upstream: v17.2.7/src/test/crush/CrushWrapper.cc::CrushWrapperTest.choose_args_compat
    // Source: https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/crush/CrushWrapper.cc#L1014
    for bytes in [
        include_bytes!("reference/choose-args-compat-quincy.crushmap").as_slice(),
        include_bytes!("reference/choose-args-compat-tentacle.crushmap").as_slice(),
    ] {
        let map = decode_fixture(bytes);
        let bucket = map.get_bucket(-1).unwrap();
        assert_eq!(bucket.items, vec![1]);
        assert!(matches!(
            &bucket.data,
            BucketData::Straw2 { item_weights } if item_weights == &[12 << 16]
        ));
        let arg = map.choose_args.get(&-1).unwrap()[0].as_ref().unwrap();
        assert_eq!(arg.weight_set, vec![vec![666 << 16]]);
        assert!(arg.ids.is_empty());
    }
    for bytes in [
        include_bytes!("reference/choose-args-compat-quincy-legacy.crushmap").as_slice(),
        include_bytes!("reference/choose-args-compat-tentacle-legacy.crushmap").as_slice(),
    ] {
        let map = decode_fixture(bytes);
        assert!(map.choose_args.is_empty());
        assert!(matches!(
            &map.get_bucket(-1).unwrap().data,
            BucketData::Straw2 { item_weights } if item_weights == &[666 << 16]
        ));
    }
}

#[test]
fn choose_args_cli_fixture_keeps_empty_index_and_signed_hash_ids() {
    // Upstream: v17.2.7/src/test/cli/crushtool/choose-args.t::cmd-01
    // Source: https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/choose-args.t#L1
    for bytes in [
        include_bytes!("reference/choose-args-quincy.crushmap").as_slice(),
        include_bytes!("reference/choose-args-tentacle.crushmap").as_slice(),
    ] {
        let map = decode_fixture(bytes);
        assert!(map.choose_args.contains_key(&1));
        assert!(map.choose_args[&1].iter().all(Option::is_none));
        assert!(!map.choose_args.contains_key(&0));
        assert_eq!(
            map.choose_args[&2][2].as_ref().unwrap().ids,
            vec![-20, 30, -25]
        );
        let arg = map.choose_args[&3][2].as_ref().unwrap();
        assert_eq!(
            arg.weight_set,
            vec![
                vec![1 << 16, 2 << 16, 5 << 16],
                vec![3 << 16, 2 << 16, 5 << 16]
            ]
        );
        assert_eq!(arg.ids, vec![-20, -30, -25]);
        assert_eq!(map.choose_args[&5][0].as_ref().unwrap().ids, vec![-450]);
    }
}

#[test]
fn selected_empty_choose_arg_does_not_fall_back_to_default() {
    // Local C-mapper contract: an existing empty index is distinct from an
    // absent index; see CrushWrapper::choose_args_get_with_fallback.
    let mut map = map_with_default_choose_arg();
    map.choose_args.insert(1, vec![None]);
    let mut default = Vec::new();
    let mut empty = Vec::new();
    let mut absent = Vec::new();
    crush_do_rule_with_choose_args(&map, 0, 2, &mut default, 1, &[1 << 16; 2], -1).unwrap();
    crush_do_rule_with_choose_args(&map, 0, 2, &mut empty, 1, &[1 << 16; 2], 1).unwrap();
    crush_do_rule_with_choose_args(&map, 0, 2, &mut absent, 1, &[1 << 16; 2], 99).unwrap();
    assert_eq!(default, vec![0]);
    assert_eq!(empty, vec![1]);
    assert_eq!(absent, default);

    assert_eq!(
        pg_to_osds(&map, PgId::new(1, 2), 64, 0, &[1 << 16; 2], 1, false).unwrap(),
        empty
    );
}

#[test]
fn replacement_ids_change_straw2_hashes_but_not_returned_items() {
    // Local C-mapper contract; replacement IDs are hash inputs, never topology
    // references. The returned value must remain a canonical bucket item.
    let mut map = map_with_default_choose_arg();
    map.choose_args.insert(
        2,
        vec![Some(CrushChooseArg {
            weight_set: vec![vec![1 << 16, 1 << 16]],
            ids: vec![-450, 30],
        })],
    );
    let mut base = Vec::new();
    let mut alternate = Vec::new();
    let mut changed = false;
    for x in 0..100 {
        crush_do_rule_with_choose_args(&map, 0, x, &mut base, 1, &[1 << 16; 2], 1).unwrap();
        crush_do_rule_with_choose_args(&map, 0, x, &mut alternate, 1, &[1 << 16; 2], 2).unwrap();
        assert!(matches!(alternate.as_slice(), [0 | 1]));
        changed |= base != alternate;
    }
    assert!(changed);
}

#[test]
fn decoder_rejects_text_and_truncated_choose_arg_data() {
    // Rust contract; no upstream analogue found. The related upstream CLI case
    // is src/test/cli/crushtool/check-invalid-map.t::cmd-01.
    assert!(
        CrushMap::decode(&mut Bytes::from_static(include_bytes!(
            "fixtures/choose-args/hosts.txt"
        )))
        .is_err()
    );
    let bytes = include_bytes!("reference/choose-args-quincy.crushmap");
    assert!(CrushMap::decode(&mut Bytes::from_static(&bytes[..bytes.len() - 1])).is_err());
}
