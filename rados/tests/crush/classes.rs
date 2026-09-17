use bytes::Bytes;
use rados::crush::{
    BucketData, CrushError, CrushMap, RuleOp, RuleType, crush_do_rule_with_choose_args,
};

const QUINCY: &[u8] = include_bytes!("reference/device-class-quincy.crushmap");
const TENTACLE: &[u8] = include_bytes!("reference/device-class-tentacle.crushmap");
const VECTORS: &str = include_str!("reference/device-class-vectors.txt");

fn decode(bytes: &'static [u8]) -> CrushMap {
    CrushMap::decode(&mut Bytes::from_static(bytes)).unwrap()
}

fn assert_straw_bucket(
    map: &CrushMap,
    id: i32,
    bucket_type: i32,
    weight: u32,
    items: &[i32],
    weights: &[u32],
) {
    let bucket = map.get_bucket(id).unwrap();
    assert_eq!(
        (bucket.id, bucket.bucket_type, bucket.weight, bucket.size),
        (id, bucket_type, weight, items.len() as u32)
    );
    assert_eq!(bucket.items, items);
    assert!(
        matches!(&bucket.data, BucketData::Straw { item_weights, .. } if item_weights == weights)
    );
}

fn c_vectors() -> impl Iterator<Item = (u32, u32, Vec<i32>)> {
    VECTORS.lines().map(|line| {
        let fields: Vec<_> = line.split_whitespace().collect();
        let rule = fields[2].parse().unwrap();
        let x = fields[4].parse().unwrap();
        let items = line
            .split_once('[')
            .unwrap()
            .1
            .trim_end_matches(']')
            .split(',')
            .filter(|item| !item.is_empty())
            .map(|item| item.trim().parse().unwrap())
            .collect();
        (rule, x, items)
    })
}

#[test]
fn device_class_fixture_keeps_all_shadows_and_class_rules() {
    // Upstream: v17.2.7/src/test/cli/crushtool/device-class.t::locally assigned compiled-map consumer state (cmd-02)
    // Source: https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/device-class.t#L2
    // Upstream: v20.2.4/src/test/cli/crushtool/device-class.t::locally assigned compiled-map consumer state (cmd-02)
    // Source: https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/device-class.t#L2
    for bytes in [QUINCY, TENTACLE] {
        let map = decode(bytes);
        assert_eq!(map.class_name.get(&0).map(String::as_str), Some("ssd"));
        assert_eq!(map.class_name.get(&1).map(String::as_str), Some("hdd"));
        assert_eq!(map.get_class_id("ssd"), Some(0));
        assert_eq!(map.get_class_id("hdd"), Some(1));
        assert_eq!(map.class_map.len(), 13);
        for (id, class) in [
            (0, "ssd"),
            (1, "ssd"),
            (2, "hdd"),
            (-6, "ssd"),
            (-7, "ssd"),
            (-8, "ssd"),
            (-9, "ssd"),
            (-10, "ssd"),
            (-11, "hdd"),
            (-12, "hdd"),
            (-13, "hdd"),
            (-14, "hdd"),
            (-15, "hdd"),
        ] {
            assert_eq!(map.get_device_class(id), Some(class));
        }
        assert_eq!(
            map.class_bucket,
            [
                (-1, [(0, -6), (1, -11)]),
                (-2, [(0, -7), (1, -12)]),
                (-3, [(0, -9), (1, -14)]),
                (-4, [(0, -10), (1, -15)]),
                (-5, [(0, -8), (1, -13)]),
            ]
            .into_iter()
            .map(|(bucket, classes)| (bucket, classes.into_iter().collect()))
            .collect()
        );
        for (id, name) in [
            (-6, "host0~ssd"),
            (-7, "host1~ssd"),
            (-8, "host2~ssd"),
            (-9, "rack0~ssd"),
            (-10, "root~ssd"),
            (-11, "host0~hdd"),
            (-12, "host1~hdd"),
            (-13, "host2~hdd"),
            (-14, "rack0~hdd"),
            (-15, "root~hdd"),
        ] {
            assert_eq!(map.names.get(&id).map(String::as_str), Some(name));
        }
        for (id, bucket_type, weight, items, weights) in [
            (-6, 1, 65536, &[0][..], &[65536][..]),
            (-7, 1, 65536, &[1][..], &[65536][..]),
            (-8, 1, 0, &[][..], &[][..]),
            (-9, 2, 131072, &[-6, -7, -8][..], &[65536, 65536, 0][..]),
            (-10, 3, 131072, &[-9][..], &[131072][..]),
            (-11, 1, 0, &[][..], &[][..]),
            (-12, 1, 0, &[][..], &[][..]),
            (-13, 1, 65536, &[2][..], &[65536][..]),
            (-14, 2, 65536, &[-11, -12, -13][..], &[0, 0, 65536][..]),
            (-15, 3, 65536, &[-14][..], &[65536][..]),
            (-4, 3, 262144, &[-3][..], &[262144][..]),
        ] {
            assert_straw_bucket(&map, id, bucket_type, weight, items, weights);
        }
        for (rule_id, take) in [(1, -10), (2, -15), (3, -4)] {
            let rule = map.get_rule(rule_id).unwrap();
            assert_eq!(rule.rule_type, RuleType::Replicated);
            assert_eq!(rule.steps.len(), 3);
            assert_eq!((rule.steps[0].op, rule.steps[0].arg1), (RuleOp::Take, take));
            assert_eq!(
                (rule.steps[1].op, rule.steps[1].arg1, rule.steps[1].arg2),
                (RuleOp::ChooseLeafFirstN, 0, 2)
            );
            assert_eq!(rule.steps[2].op, RuleOp::Emit);
        }
    }
}

#[test]
fn device_class_shadow_lookup_matches_split_id_class_consumer_behavior() {
    // Upstream: v17.2.7/src/test/crush/CrushWrapper.cc::CrushWrapperTest.split_id_class
    // Source: https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/crush/CrushWrapper.cc#L1197
    // Upstream: v20.2.4/src/test/crush/CrushWrapper.cc::CrushWrapperTest.split_id_class
    // Source: https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/CrushWrapper.cc#L1194
    for bytes in [QUINCY, TENTACLE] {
        let map = decode(bytes);
        assert_eq!(map.split_id_class(-10).unwrap(), (-4, Some(0)));
        assert_eq!(map.split_id_class(-15).unwrap(), (-4, Some(1)));
        assert_eq!(map.split_id_class(-4).unwrap(), (-4, None));
        assert!(matches!(
            map.split_id_class(-16),
            Err(CrushError::ItemNotFound(-16))
        ));
    }
}

#[test]
fn device_class_rules_match_pinned_c_vectors_and_membership() {
    // Rust contract; local C differential against original device-class.crush,
    // rules 1/2, x=0..19 and num-rep=3. Vectors come from both pinned releases.
    // Sources: https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/device-class.crush#L1
    // and https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/device-class.crush#L1
    for bytes in [QUINCY, TENTACLE] {
        let map = decode(bytes);
        let mut rows = 0;
        for (rule, x, expected) in c_vectors() {
            let mut actual = Vec::new();
            crush_do_rule_with_choose_args(&map, rule, x, &mut actual, 3, &[65536; 3], -1).unwrap();
            assert_eq!(actual, expected, "rule={rule} x={x}");
            let class = if rule == 1 { "ssd" } else { "hdd" };
            assert!(actual.iter().all(|&id| map.device_has_class(id, class)));
            rows += 1;
        }
        assert_eq!(rows, 40);
    }
}
