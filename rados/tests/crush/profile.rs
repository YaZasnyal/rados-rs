use bytes::Bytes;
use rados::crush::{CrushMap, mapper::crush_do_rule_with_choose_profile};

const QUINCY: &[u8] = include_bytes!("reference/show-choose-tries-quincy.crushmap");
const TENTACLE: &[u8] = include_bytes!("reference/show-choose-tries-tentacle.crushmap");

fn map(bytes: &'static [u8]) -> CrushMap {
    CrushMap::decode(&mut Bytes::from_static(bytes)).unwrap()
}

fn weights(map: &CrushMap) -> Vec<u32> {
    vec![1 << 16; map.max_devices as usize]
}

fn run(map: &CrushMap, rule: u32, replicas: usize, profile: &mut rados::crush::ChooseProfile) {
    crush_do_rule_with_choose_profile(
        map,
        rule,
        1,
        &mut Vec::new(),
        replicas,
        &weights(map),
        profile,
    )
    .unwrap();
}

fn expected_profile(bins: &[(usize, u32)]) -> Vec<u32> {
    let mut profile = vec![0; 50];
    for &(bin, count) in bins {
        profile[bin] = count;
    }
    profile
}

// Upstream: v17.2.7/v20.2.4 src/test/cli/crushtool/show-choose-tries.t::cmd-03/cmd-05.
// Sources: https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/show-choose-tries.t#L3
// and https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/show-choose-tries.t#L3
#[test]
fn retry_profiles_match_the_two_fresh_crushtool_commands() {
    for bytes in [QUINCY, TENTACLE] {
        let firstn = map(bytes);
        let mut firstn_profile = firstn.start_choose_profile();
        run(&firstn, 0, 2, &mut firstn_profile);
        assert_eq!(
            firstn_profile.get_choose_profile(),
            Some(expected_profile(&[(0, 1), (1, 1)]).as_slice())
        );
        firstn_profile.stop_choose_profile();
        assert_eq!(firstn_profile.get_choose_profile(), None);

        let indep = map(bytes);
        let mut indep_profile = indep.start_choose_profile();
        run(&indep, 1, 1, &mut indep_profile);
        assert_eq!(
            indep_profile.get_choose_profile(),
            Some(expected_profile(&[(1, 1)]).as_slice())
        );
        indep_profile.stop_choose_profile();
        assert_eq!(indep_profile.get_choose_profile(), None);
    }
}

// Upstream: v17.2.7/v20.2.4 src/crush/CrushWrapper.h::start_choose_profile/get_choose_profile/stop_choose_profile.
// Sources: https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/crush/CrushWrapper.h#L1334
// and https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/crush/CrushWrapper.h#L1390
#[test]
fn retry_profile_accumulates_until_start_resets_or_stop_discards() {
    let map = map(QUINCY);
    let mut profile = map.start_choose_profile();
    run(&map, 0, 2, &mut profile);
    run(&map, 0, 2, &mut profile);
    assert_eq!(
        profile.get_choose_profile(),
        Some(expected_profile(&[(0, 2), (1, 2)]).as_slice())
    );

    profile.start_choose_profile(map.choose_total_tries);
    assert_eq!(
        profile.get_choose_profile(),
        Some(expected_profile(&[]).as_slice())
    );
    run(&map, 1, 1, &mut profile);
    assert_eq!(
        profile.get_choose_profile(),
        Some(expected_profile(&[(1, 1)]).as_slice())
    );

    profile.stop_choose_profile();
    assert_eq!(profile.get_choose_profile(), None);
}
