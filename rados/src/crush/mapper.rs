// CRUSH rule execution engine
// Reference: ~/dev/ceph/src/crush/mapper.c

use crate::crush::bucket::{bucket_choose, bucket_perm_choose};
use crate::crush::error::Result;
use crate::crush::hash::crush_hash32_2;
use crate::crush::types::{CrushMap, RuleOp};
use crate::denc::constants::crush::{FIXED_POINT_MASK, FIXED_POINT_ONE};

/// Sentinel value for "no item selected" in INDEP mode
/// Matches CRUSH_ITEM_NONE in Ceph (crush.h)
pub(crate) const CRUSH_ITEM_NONE: i32 = 0x7fffffff;
/// Internal sentinel: slot not yet resolved (distinct from CRUSH_ITEM_NONE which is definitive).
const CRUSH_ITEM_UNDEF: i32 = 0x7ffffffe;

/// Calculate the number of replicas/items to select based on rule step argument
///
/// - arg == 0: use result_max
/// - arg > 0: use arg directly
/// - arg < 0: use result_max + arg (i.e., result_max - |arg|)
#[inline]
fn calculate_numrep(step_arg: i32, result_max: usize) -> usize {
    if step_arg == 0 {
        result_max
    } else if step_arg > 0 {
        step_arg as usize
    } else {
        (result_max as i32 + step_arg).max(0) as usize
    }
}

/// Determine the type of an item (0 for device, bucket_type for buckets)
/// Returns None if item is an invalid bucket
#[inline]
fn get_item_type(map: &CrushMap, item: i32) -> Option<i32> {
    if item >= 0 {
        Some(0) // Device
    } else {
        map.get_bucket(item).ok().map(|b| b.bucket_type)
    }
}

/// Check if an OSD is "out" (failed, fully offloaded)
fn is_out(weight: &[u32], item: i32, x: u32) -> bool {
    if item < 0 || item as usize >= weight.len() {
        return true;
    }

    let w = weight[item as usize];

    // Weight >= FIXED_POINT_ONE (1.0 in 16.16 fixed point) means fully in
    if w >= FIXED_POINT_ONE {
        return false;
    }

    // Weight == 0 means fully out
    if w == 0 {
        return true;
    }

    // Probabilistic: use hash to determine if item is in or out
    // This allows gradual weight changes
    let hash = crush_hash32_2(x, item as u32);
    (hash & FIXED_POINT_MASK) >= w
}

/// Execute a CRUSH rule to map a PG to OSDs
///
/// # Arguments
/// * `map` - The CRUSH map
/// * `rule_id` - Rule ID to execute
/// * `x` - Input value (typically PG hash)
/// * `result` - Output vector for selected OSDs
/// * `result_max` - Maximum number of results to return
/// * `weights` - OSD weights (from OSDMap)
///
/// # Returns
/// Ok(()) on success, Err on failure
pub fn crush_do_rule(
    map: &CrushMap,
    rule_id: u32,
    x: u32,
    result: &mut Vec<i32>,
    result_max: usize,
    weights: &[u32],
) -> Result<()> {
    let rule = map.get_rule(rule_id)?;

    result.clear();

    let mut work: Vec<i32> = Vec::with_capacity(result_max);
    let mut scratch: Vec<i32> = Vec::with_capacity(result_max);

    // C++ mapper.c: "the original choose_total_tries value was off by one
    // (it counted 'retries' and not 'tries'). add one."
    let mut choose_tries = map.choose_total_tries + 1;
    let mut chooseleaf_vary_r = map.chooseleaf_vary_r;
    let mut chooseleaf_stable = map.chooseleaf_stable;
    let mut msr_descents = map.msr_descents;
    let mut msr_collision_tries = map.msr_collision_tries;

    for step in &rule.steps {
        match step.op {
            RuleOp::Take => {
                work.clear();
                work.push(step.arg1);
            }

            RuleOp::ChooseFirstN | RuleOp::ChooseLeafFirstN => {
                let recurse_to_leaf = step.op == RuleOp::ChooseLeafFirstN;
                let numrep = calculate_numrep(step.arg1, result_max);
                let item_type = step.arg2;

                let recurse_tries = if map.chooseleaf_descend_once != 0 {
                    1
                } else {
                    choose_tries
                };
                scratch.clear();
                let mut domains = vec![CRUSH_ITEM_NONE; result_max];
                let mut leaves = vec![CRUSH_ITEM_NONE; result_max];
                for &item in &work {
                    if item >= 0 || numrep == 0 {
                        continue;
                    }
                    let remaining = result_max - scratch.len();
                    let count = crush_choose_firstn(
                        map,
                        item,
                        x,
                        numrep,
                        item_type,
                        &mut domains[..remaining],
                        0,
                        weights,
                        choose_tries,
                        recurse_tries,
                        chooseleaf_vary_r,
                        chooseleaf_stable,
                        recurse_to_leaf.then_some(&mut leaves[..remaining]),
                        0,
                    )?;
                    let selected = if recurse_to_leaf { &leaves } else { &domains };
                    scratch.extend_from_slice(&selected[..count]);
                }
                std::mem::swap(&mut work, &mut scratch);
            }

            RuleOp::ChooseIndep | RuleOp::ChooseLeafIndep => {
                let recurse_to_leaf = step.op == RuleOp::ChooseLeafIndep;
                let numrep = calculate_numrep(step.arg1, result_max);
                let item_type = step.arg2;

                scratch.clear();
                let mut indep_out = vec![CRUSH_ITEM_NONE; numrep];
                for &item in &work {
                    indep_out.fill(CRUSH_ITEM_NONE);
                    crush_choose_indep(
                        map,
                        item,
                        x,
                        numrep,
                        item_type,
                        &mut indep_out,
                        weights,
                        choose_tries,
                        recurse_to_leaf,
                        0, // top-level call: parent_r = 0
                    )?;
                    scratch.extend_from_slice(&indep_out);
                }
                std::mem::swap(&mut work, &mut scratch);
            }

            RuleOp::ChooseMsr => {
                let numrep = calculate_numrep(step.arg1, result_max);
                let item_type = step.arg2;

                scratch.clear();
                let mut msr_out = vec![CRUSH_ITEM_NONE; numrep];
                for &item in &work {
                    msr_out.fill(CRUSH_ITEM_NONE);
                    crush_choose_msr(
                        map,
                        item,
                        x,
                        numrep,
                        item_type,
                        &mut msr_out,
                        weights,
                        msr_descents,
                        msr_collision_tries,
                        false,
                    )?;
                    scratch.extend(msr_out.iter().copied().filter(|&v| v != CRUSH_ITEM_NONE));
                }
                std::mem::swap(&mut work, &mut scratch);
            }

            RuleOp::Emit => {
                for &item in &work {
                    if result.len() < result_max {
                        result.push(item);
                    }
                }
            }

            RuleOp::SetChooseTries => choose_tries = step.arg1 as u32,
            RuleOp::SetChooseLeafVaryR => chooseleaf_vary_r = step.arg1 as u8,
            RuleOp::SetChooseLeafStable => chooseleaf_stable = step.arg1 as u8,
            RuleOp::SetMsrDescents => msr_descents = step.arg1 as u32,
            RuleOp::SetMsrCollisionTries => msr_collision_tries = step.arg1 as u32,

            RuleOp::SetChooseLeafTries
            | RuleOp::SetChooseLocalTries
            | RuleOp::SetChooseLocalFallbackTries
            | RuleOp::Noop => {}
        }
    }

    Ok(())
}

/// Choose failure domains with FIRSTN, optionally selecting a leaf in each.
/// `out` retains domains for collision checks; `leaves` retains the OSDs.
/// The returned position includes any prefix supplied by a recursive call.
/// Reference: src/crush/mapper.c::crush_choose_firstn, v17.2.7 and v20.2.4.
#[allow(clippy::too_many_arguments)]
fn crush_choose_firstn(
    map: &CrushMap,
    bucket_id: i32,
    x: u32,
    numrep: usize,
    item_type: i32,
    out: &mut [i32],
    mut outpos: usize,
    weights: &[u32],
    tries: u32,
    recurse_tries: u32,
    vary_r: u8,
    stable: u8,
    mut leaves: Option<&mut [i32]>,
    parent_r: u32,
) -> Result<usize> {
    let _span = tracing::debug_span!(
        "crush_choose_firstn",
        bucket_id,
        x,
        numrep,
        item_type,
        parent_r
    )
    .entered();
    tracing::debug!(
        outpos,
        capacity = out.len(),
        tries,
        recurse_tries,
        local_retries = map.choose_local_tries,
        local_fallback_retries = map.choose_local_fallback_tries,
        vary_r,
        stable,
        chooseleaf = leaves.is_some(),
        "Starting FIRSTN selection"
    );
    let bucket = map.get_bucket(bucket_id)?;
    let first_rep = if stable != 0 { 0 } else { outpos };
    'replicas: for rep in first_rep..numrep {
        if outpos == out.len() {
            tracing::trace!(rep, outpos, "Output capacity reached");
            break;
        }
        let mut current_bucket = bucket;
        let mut total_failures = 0u32;
        let mut local_failures = 0u32;
        loop {
            // Retries always change r. vary_r only controls the recursive seed.
            let r = (rep as u32)
                .wrapping_add(parent_r)
                .wrapping_add(total_failures);
            let mut collide = false;
            if current_bucket.size != 0 {
                let item = if map.choose_local_fallback_tries > 0
                    && local_failures >= current_bucket.size / 2
                    && local_failures > map.choose_local_fallback_tries
                {
                    tracing::trace!(
                        rep,
                        r,
                        current_bucket_id = current_bucket.id,
                        local_failures,
                        "Using permutation fallback"
                    );
                    bucket_perm_choose(current_bucket, x, r)
                } else {
                    bucket_choose(current_bucket, x, r)
                };
                tracing::trace!(
                    rep,
                    r,
                    current_bucket_id = current_bucket.id,
                    item,
                    "Selected candidate"
                );
                if item >= map.max_devices {
                    tracing::trace!(
                        rep,
                        item,
                        max_devices = map.max_devices,
                        "Skipping replica: invalid device"
                    );
                    continue 'replicas;
                }
                let Some(selected_type) = get_item_type(map, item) else {
                    tracing::trace!(rep, item, "Skipping replica: invalid bucket");
                    continue 'replicas;
                };
                if selected_type != item_type {
                    if item >= 0 {
                        tracing::trace!(
                            rep,
                            item,
                            selected_type,
                            "Skipping replica: wrong item type"
                        );
                        continue 'replicas;
                    }
                    tracing::trace!(
                        rep,
                        from = current_bucket.id,
                        to = item,
                        selected_type,
                        "Descending through hierarchy"
                    );
                    current_bucket = map.get_bucket(item)?;
                    continue;
                }

                collide = out[..outpos].contains(&item);
                if collide {
                    tracing::trace!(rep, item, outpos, "Rejecting candidate: collision");
                }
                let mut reject = false;
                if !collide && let Some(leaf_out) = leaves.as_deref_mut() {
                    if item < 0 {
                        let sub_r = if vary_r == 0 {
                            0
                        } else {
                            r.checked_shr(u32::from(vary_r) - 1).unwrap_or(0)
                        };
                        tracing::trace!(
                            rep,
                            item,
                            outpos,
                            sub_r,
                            recurse_tries,
                            "Selecting leaf in failure domain"
                        );
                        let end = crush_choose_firstn(
                            map,
                            item,
                            x,
                            if stable != 0 { 1 } else { outpos + 1 },
                            0,
                            leaf_out,
                            outpos,
                            weights,
                            recurse_tries,
                            0,
                            vary_r,
                            stable,
                            None,
                            sub_r,
                        )?;
                        reject = end == outpos;
                        if reject {
                            tracing::trace!(rep, item, "Rejecting candidate: no eligible leaf");
                        }
                    } else {
                        leaf_out[outpos] = item;
                    }
                }
                if !reject && !collide && item >= 0 {
                    reject = is_out(weights, item, x);
                    if reject {
                        tracing::trace!(rep, item, "Rejecting candidate: device is out");
                    }
                }
                if !reject && !collide {
                    tracing::trace!(
                        bucket_id,
                        x,
                        rep,
                        item,
                        total_failures,
                        "CRUSH FIRSTN selected item"
                    );
                    out[outpos] = item;
                    outpos += 1;
                    continue 'replicas;
                }
            } else {
                tracing::trace!(
                    rep,
                    current_bucket_id = current_bucket.id,
                    "Rejecting empty bucket"
                );
            }

            total_failures += 1;
            local_failures += 1;
            if (collide && local_failures <= map.choose_local_tries)
                || (map.choose_local_fallback_tries > 0
                    && local_failures <= current_bucket.size + map.choose_local_fallback_tries)
            {
                // A local retry stays in the bucket where the collision occurred.
                tracing::trace!(
                    rep,
                    current_bucket_id = current_bucket.id,
                    total_failures,
                    local_failures,
                    "Retrying current bucket"
                );
                continue;
            }
            if total_failures >= tries {
                tracing::debug!(
                    rep,
                    outpos,
                    total_failures,
                    "Skipping replica: retry budget exhausted"
                );
                continue 'replicas;
            }
            // A descent retry starts again at the original failure domain.
            tracing::trace!(rep, total_failures, "Restarting descent");
            current_bucket = bucket;
            local_failures = 0;
        }
    }
    tracing::debug!(outpos, "Finished FIRSTN selection");
    Ok(outpos)
}

/// Choose N items using INDEP (independent) algorithm
///
/// Unlike FIRSTN, each position independently selects an item.
/// If a position fails to find a valid item, it gets CRUSH_ITEM_NONE
/// rather than shifting subsequent items. This is critical for
/// erasure-coded pools where each chunk has a fixed position.
///
/// Reference: Ceph src/crush/mapper.c crush_choose_indep()
#[allow(clippy::too_many_arguments)]
fn crush_choose_indep(
    map: &CrushMap,
    bucket_id: i32,
    x: u32,
    numrep: usize,
    item_type: i32,
    out: &mut [i32],
    weights: &[u32],
    tries: u32,
    recurse_to_leaf: bool,
    parent_r: i32,
) -> Result<()> {
    tracing::debug!(
        "crush_choose_indep: bucket_id={}, numrep={}, item_type={}, recurse_to_leaf={}",
        bucket_id,
        numrep,
        item_type,
        recurse_to_leaf
    );

    if bucket_id >= 0 {
        if item_type == 0 && !is_out(weights, bucket_id, x) && !out.is_empty() {
            out[0] = bucket_id;
        }
        return Ok(());
    }

    let bucket = map.get_bucket(bucket_id)?;
    tracing::debug!(
        "Got bucket: id={}, type={}, size={}, items={:?}",
        bucket.id,
        bucket.bucket_type,
        bucket.size,
        bucket.items
    );

    // UNDEF distinguishes "slot not yet filled" from definitive NONE (not retried).
    for slot in out.iter_mut().take(numrep) {
        *slot = CRUSH_ITEM_UNDEF;
    }

    for ftotal in 0..tries {
        let mut all_done = true;

        for rep in 0..numrep {
            if out[rep] != CRUSH_ITEM_UNDEF {
                continue;
            }

            all_done = false;

            let mut current_bucket = bucket;

            // r = rep + parent_r + numrep * ftotal
            // Matches C++ crush_choose_indep (non-uniform bucket path).
            let r = (rep as u32)
                .wrapping_add(parent_r as u32)
                .wrapping_add((numrep as u32).wrapping_mul(ftotal));

            let mut item = CRUSH_ITEM_NONE;
            let mut item_found = false;

            loop {
                let candidate = bucket_choose(current_bucket, x, r);

                tracing::debug!(
                    "Selected item {} from bucket {} (rep={}, try={})",
                    candidate,
                    current_bucket.id,
                    rep,
                    ftotal
                );

                let Some(itemtype) = get_item_type(map, candidate) else {
                    tracing::debug!("Invalid bucket {}", candidate);
                    out[rep] = CRUSH_ITEM_NONE;
                    break;
                };

                tracing::debug!(
                    "Item {} has type {}, looking for type {}",
                    candidate,
                    itemtype,
                    item_type
                );

                if itemtype != item_type {
                    if candidate >= 0 {
                        tracing::debug!("Device {} has wrong type", candidate);
                        out[rep] = CRUSH_ITEM_NONE;
                        break;
                    }
                    current_bucket = map.get_bucket(candidate)?;
                    tracing::debug!("Descending into bucket {}", candidate);
                    continue;
                }

                let collision = out
                    .iter()
                    .enumerate()
                    .take(numrep)
                    .any(|(j, &val)| j != rep && val == candidate);
                if collision {
                    tracing::debug!("Item {} collides with another position", candidate);
                    break;
                }

                if candidate >= 0 && is_out(weights, candidate, x) {
                    tracing::debug!("Device {} is out", candidate);
                    break;
                }

                if recurse_to_leaf && candidate < 0 {
                    let mut leaf_out = [CRUSH_ITEM_UNDEF; 1];
                    crush_choose_indep(
                        map,
                        candidate,
                        x,
                        1,
                        0, // Type 0 = device
                        &mut leaf_out,
                        weights,
                        tries,
                        true,
                        r as i32,
                    )?;

                    if leaf_out[0] == CRUSH_ITEM_NONE || leaf_out[0] == CRUSH_ITEM_UNDEF {
                        tracing::debug!("Failed to find leaf in bucket {}", candidate);
                        break;
                    }

                    let leaf_collision = out
                        .iter()
                        .enumerate()
                        .take(numrep)
                        .any(|(j, &val)| j != rep && val == leaf_out[0]);
                    if leaf_collision {
                        tracing::debug!("Leaf item {} collides with another position", leaf_out[0]);
                        break;
                    }

                    item = leaf_out[0];
                    item_found = true;
                    break;
                }

                tracing::debug!("Found valid item {}", candidate);
                item = candidate;
                item_found = true;
                break;
            }

            if item_found {
                out[rep] = item;
                tracing::trace!("crush_choose_indep: rep={}, item={} SELECTED", rep, item);
            }
        }

        if all_done {
            break;
        }
    }

    for slot in out.iter_mut().take(numrep) {
        if *slot == CRUSH_ITEM_UNDEF {
            *slot = CRUSH_ITEM_NONE;
        }
    }

    Ok(())
}

/// Choose N items using MSR (Main Search Rule) algorithm
///
/// MSR is an alternative selection algorithm for multi-way replication.
/// It uses a different hash function and collision handling strategy.
///
/// Reference: ~/dev/ceph/src/crush/mapper.c (crush_choose_msr)
#[allow(clippy::too_many_arguments)]
fn crush_choose_msr(
    map: &CrushMap,
    bucket_id: i32,
    x: u32,
    numrep: usize,
    item_type: i32,
    out: &mut [i32],
    weights: &[u32],
    descents: u32,
    collision_tries: u32,
    recurse_to_leaf: bool,
) -> Result<()> {
    tracing::debug!(
        "crush_choose_msr: bucket_id={}, numrep={}, item_type={}, descents={}, collision_tries={}",
        bucket_id,
        numrep,
        item_type,
        descents,
        collision_tries
    );

    if bucket_id >= 0 {
        if item_type == 0 && !is_out(weights, bucket_id, x) {
            out[0] = bucket_id;
        }
        return Ok(());
    }

    let bucket = map.get_bucket(bucket_id)?;

    let mut chosen_items: Vec<i32> = Vec::with_capacity(numrep);
    let mut collisions = vec![0u32; numrep];

    for rep in 0..numrep {
        let mut descent = 0;

        'retry: loop {
            if descent >= descents {
                break 'retry;
            }
            descent += 1;

            let r = rep as u32 + descent * numrep as u32 + collisions[rep];
            let hash = crush_hash32_2(x + r, bucket_id as u32);
            let item = bucket_choose(bucket, hash, r);

            let item_type_match = match get_item_type(map, item) {
                Some(t) => t == item_type || item_type == 0,
                None => false,
            };

            if item < 0 && (recurse_to_leaf || !item_type_match) {
                let mut sub_out = [CRUSH_ITEM_NONE; 1];
                crush_choose_msr(
                    map,
                    item,
                    x,
                    1,
                    item_type,
                    &mut sub_out,
                    weights,
                    descents,
                    collision_tries,
                    recurse_to_leaf,
                )?;

                if sub_out[0] == CRUSH_ITEM_NONE {
                    continue 'retry;
                }

                if chosen_items.contains(&sub_out[0]) {
                    collisions[rep] += 1;
                    if collisions[rep] < collision_tries {
                        continue 'retry;
                    }
                    break 'retry;
                }

                out[rep] = sub_out[0];
                chosen_items.push(sub_out[0]);
                break 'retry;
            }

            if item >= 0 && is_out(weights, item, x) {
                continue 'retry;
            }

            if item_type_match {
                if chosen_items.contains(&item) {
                    collisions[rep] += 1;
                    if collisions[rep] < collision_tries {
                        continue 'retry;
                    }
                    break 'retry;
                }
                out[rep] = item;
                chosen_items.push(item);
                break 'retry;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crush::types::{BucketAlgorithm, BucketData, CrushBucket, CrushRule, CrushRuleStep};

    #[test]
    fn test_is_out() {
        let weights = vec![0x10000, 0x8000, 0, 0x20000];

        // Fully in (weight >= 0x10000)
        assert!(!is_out(&weights, 0, 123));
        assert!(!is_out(&weights, 3, 123));

        // Fully out (weight == 0)
        assert!(is_out(&weights, 2, 123));

        // Out of bounds
        assert!(is_out(&weights, 10, 123));
        assert!(is_out(&weights, -1, 123));
    }

    #[test]
    fn test_crush_do_rule_simple() {
        // Create a simple CRUSH map with one bucket and two devices
        let mut map = CrushMap::new();
        map.max_devices = 2;
        map.max_buckets = 1;

        // Create a bucket with two devices
        let bucket = CrushBucket {
            id: -1,
            bucket_type: 1, // Type 1 = host
            alg: BucketAlgorithm::Straw2,
            hash: 0, // CRUSH_HASH_RJENKINS1
            weight: 0x20000,
            size: 2,
            items: vec![0, 1],
            data: BucketData::Straw2 {
                item_weights: vec![0x10000, 0x10000],
            },
        };

        map.buckets = vec![Some(bucket)];

        // Create a simple rule: TAKE -1, CHOOSELEAF_FIRSTN 1 type 0, EMIT
        let rule = CrushRule {
            rule_id: 0,
            rule_type: crate::crush::types::RuleType::Replicated,
            steps: vec![
                CrushRuleStep {
                    op: RuleOp::Take,
                    arg1: -1,
                    arg2: 0,
                },
                CrushRuleStep {
                    op: RuleOp::ChooseLeafFirstN,
                    arg1: 1,
                    arg2: 0, // Type 0 = device
                },
                CrushRuleStep {
                    op: RuleOp::Emit,
                    arg1: 0,
                    arg2: 0,
                },
            ],
        };

        map.rules = vec![Some(rule)];

        // Execute the rule
        let mut result = Vec::new();
        let weights = vec![0x10000, 0x10000]; // Both devices fully in

        let res = crush_do_rule(&map, 0, 123, &mut result, 1, &weights);
        assert!(res.is_ok());
        assert_eq!(result.len(), 1);
        assert!(result[0] == 0 || result[0] == 1);
    }

    #[test]
    fn test_crush_choose_firstn() {
        let mut map = CrushMap::new();
        map.max_devices = 3;
        map.max_buckets = 1;

        let bucket = CrushBucket {
            id: -1,
            bucket_type: 1,
            alg: BucketAlgorithm::Straw2,
            hash: 0, // CRUSH_HASH_RJENKINS1
            weight: 0x30000,
            size: 3,
            items: vec![0, 1, 2],
            data: BucketData::Straw2 {
                item_weights: vec![0x10000, 0x10000, 0x10000],
            },
        };

        map.buckets = vec![Some(bucket)];

        let mut out = vec![CRUSH_ITEM_NONE; 2];
        let weights = vec![0x10000, 0x10000, 0x10000];
        let count = crush_choose_firstn(
            &map, -1, 123, 2, 0, &mut out, 0, &weights, 50, 50, 0, 0, None, 0,
        )
        .unwrap();
        out.truncate(count);

        assert_eq!(out.len(), 2);
        assert_ne!(out[0], out[1]);
    }

    #[test]
    fn test_crush_choose_indep() {
        let mut map = CrushMap::new();
        map.max_devices = 4;
        map.max_buckets = 1;

        let bucket = CrushBucket {
            id: -1,
            bucket_type: 1,
            alg: BucketAlgorithm::Straw2,
            hash: 0,
            weight: 0x40000,
            size: 4,
            items: vec![0, 1, 2, 3],
            data: BucketData::Straw2 {
                item_weights: vec![0x10000, 0x10000, 0x10000, 0x10000],
            },
        };

        map.buckets = vec![Some(bucket)];

        let mut out = vec![CRUSH_ITEM_NONE; 3];
        let weights = vec![0x10000, 0x10000, 0x10000, 0x10000];

        let res = crush_choose_indep(&map, -1, 123, 3, 0, &mut out, &weights, 50, false, 0);

        assert!(res.is_ok());
        // All positions should be filled (enough devices available)
        for &item in &out {
            assert_ne!(item, CRUSH_ITEM_NONE, "all positions should be filled");
            assert!((0..4).contains(&item), "item should be a valid device");
        }
        // All items should be distinct
        assert_ne!(out[0], out[1]);
        assert_ne!(out[0], out[2]);
        assert_ne!(out[1], out[2]);
    }

    #[test]
    fn test_crush_choose_indep_stable_positions() {
        // INDEP key property: removing a device only affects its position,
        // not other positions. Verify determinism across multiple runs.
        let mut map = CrushMap::new();
        map.max_devices = 5;
        map.max_buckets = 1;

        let bucket = CrushBucket {
            id: -1,
            bucket_type: 1,
            alg: BucketAlgorithm::Straw2,
            hash: 0,
            weight: 0x50000,
            size: 5,
            items: vec![0, 1, 2, 3, 4],
            data: BucketData::Straw2 {
                item_weights: vec![0x10000, 0x10000, 0x10000, 0x10000, 0x10000],
            },
        };

        map.buckets = vec![Some(bucket)];

        let weights = vec![0x10000, 0x10000, 0x10000, 0x10000, 0x10000];

        // Run the same input twice - should produce identical results
        let mut out1 = vec![CRUSH_ITEM_NONE; 3];
        let mut out2 = vec![CRUSH_ITEM_NONE; 3];
        crush_choose_indep(&map, -1, 42, 3, 0, &mut out1, &weights, 50, false, 0).unwrap();
        crush_choose_indep(&map, -1, 42, 3, 0, &mut out2, &weights, 50, false, 0).unwrap();
        assert_eq!(out1, out2, "same input should produce same output");
    }

    #[test]
    fn test_crush_choose_indep_with_out_device() {
        // When a device is "out", INDEP should put CRUSH_ITEM_NONE in that
        // position rather than shifting other items
        let mut map = CrushMap::new();
        map.max_devices = 3;
        map.max_buckets = 1;

        let bucket = CrushBucket {
            id: -1,
            bucket_type: 1,
            alg: BucketAlgorithm::Straw2,
            hash: 0,
            weight: 0x30000,
            size: 3,
            items: vec![0, 1, 2],
            data: BucketData::Straw2 {
                item_weights: vec![0x10000, 0x10000, 0x10000],
            },
        };

        map.buckets = vec![Some(bucket)];

        // All devices in
        let weights_all_in = vec![0x10000, 0x10000, 0x10000];
        let mut out_all = vec![CRUSH_ITEM_NONE; 3];
        crush_choose_indep(
            &map,
            -1,
            100,
            3,
            0,
            &mut out_all,
            &weights_all_in,
            50,
            false,
            0,
        )
        .unwrap();

        // All positions should be filled when all devices are in
        for &item in &out_all {
            assert_ne!(item, CRUSH_ITEM_NONE);
        }
    }

    #[test]
    fn test_crush_do_rule_indep() {
        // Test ChooseIndep via crush_do_rule
        let mut map = CrushMap::new();
        map.max_devices = 4;
        map.max_buckets = 1;

        let bucket = CrushBucket {
            id: -1,
            bucket_type: 1,
            alg: BucketAlgorithm::Straw2,
            hash: 0,
            weight: 0x40000,
            size: 4,
            items: vec![0, 1, 2, 3],
            data: BucketData::Straw2 {
                item_weights: vec![0x10000, 0x10000, 0x10000, 0x10000],
            },
        };

        map.buckets = vec![Some(bucket)];

        // Erasure rule: TAKE -1, CHOOSE_INDEP 3 type 0, EMIT
        let rule = CrushRule {
            rule_id: 0,
            rule_type: crate::crush::types::RuleType::Erasure,
            steps: vec![
                CrushRuleStep {
                    op: RuleOp::Take,
                    arg1: -1,
                    arg2: 0,
                },
                CrushRuleStep {
                    op: RuleOp::ChooseIndep,
                    arg1: 3,
                    arg2: 0,
                },
                CrushRuleStep {
                    op: RuleOp::Emit,
                    arg1: 0,
                    arg2: 0,
                },
            ],
        };

        map.rules = vec![Some(rule)];

        let mut result = Vec::new();
        let weights = vec![0x10000, 0x10000, 0x10000, 0x10000];

        let res = crush_do_rule(&map, 0, 123, &mut result, 3, &weights);
        assert!(res.is_ok());
        assert_eq!(result.len(), 3);

        // All results should be valid devices or CRUSH_ITEM_NONE
        let valid_count = result
            .iter()
            .filter(|&&item| item != CRUSH_ITEM_NONE)
            .count();
        assert_eq!(
            valid_count, 3,
            "should find 3 valid devices with 4 available"
        );

        // Valid items should be distinct
        let mut valid_items: Vec<i32> = result
            .iter()
            .copied()
            .filter(|&item| item != CRUSH_ITEM_NONE)
            .collect();
        valid_items.sort();
        valid_items.dedup();
        assert_eq!(valid_items.len(), 3, "valid items should be distinct");
    }

    #[test]
    fn test_crush_do_rule_chooseleaf_indep() {
        // Test ChooseLeafIndep via crush_do_rule
        let mut map = CrushMap::new();
        map.max_devices = 4;
        map.max_buckets = 1;

        let bucket = CrushBucket {
            id: -1,
            bucket_type: 1,
            alg: BucketAlgorithm::Straw2,
            hash: 0,
            weight: 0x40000,
            size: 4,
            items: vec![0, 1, 2, 3],
            data: BucketData::Straw2 {
                item_weights: vec![0x10000, 0x10000, 0x10000, 0x10000],
            },
        };

        map.buckets = vec![Some(bucket)];

        // Erasure rule: TAKE -1, CHOOSELEAF_INDEP 3 type 0, EMIT
        let rule = CrushRule {
            rule_id: 0,
            rule_type: crate::crush::types::RuleType::Erasure,
            steps: vec![
                CrushRuleStep {
                    op: RuleOp::Take,
                    arg1: -1,
                    arg2: 0,
                },
                CrushRuleStep {
                    op: RuleOp::ChooseLeafIndep,
                    arg1: 3,
                    arg2: 0, // Type 0 = device
                },
                CrushRuleStep {
                    op: RuleOp::Emit,
                    arg1: 0,
                    arg2: 0,
                },
            ],
        };

        map.rules = vec![Some(rule)];

        let mut result = Vec::new();
        let weights = vec![0x10000, 0x10000, 0x10000, 0x10000];

        let res = crush_do_rule(&map, 0, 123, &mut result, 3, &weights);
        assert!(res.is_ok());
        assert_eq!(result.len(), 3);

        // All results should be valid devices
        let valid_count = result
            .iter()
            .filter(|&&item| item != CRUSH_ITEM_NONE)
            .count();
        assert_eq!(
            valid_count, 3,
            "should find 3 valid devices with 4 available"
        );
    }
}
