# CRUSH case dispositions

This ledger records the current client-facing compatibility decision for the
pinned Ceph releases: Quincy v17.2.7
[`b12291d`](https://github.com/ceph/ceph/tree/b12291d110049b2f35e32e0de30d70e9a4c060d2)
and Tentacle v20.2.4
[`7f7937`](https://github.com/ceph/ceph/tree/7f793731f1b39eb4f465e960113d2363c311b964).
Each retained result is checked by Cargo against a pinned-C map or vector;
generators and hashes are in this directory and `../fixtures/`.

## Retained and adapted client observations

| Cases | Lost observation | Reason / replacement | Remaining risk | Revisit | Decision |
| --- | --- | --- | --- | --- | --- |
| Golden maps, `set-choose`, `bad-mappings`, `firstn-indep`, `show-choose-tries` | None | Compiled source maps, exact ordered rows, NONE slots, lengths, and both 50-bin profiles in `golden.rs` and `profile.rs` | Unselected CLI rendering | A public tool API | Retained: passing offline |
| Quincy INDEP `toosmall`, `basic`, `out_alt`, `out_contig`, `out_progressive` | Raw rule-type-123 decoding | Pinned C compares type 123 with Erasure type 3 for 508 ordered mappings; `functional.rs` uses the supported Erasure type | Wire decoder still rejects 123 | Decoder support is added | Adapted: mapping parity only |
| `straw_zero`, `straw_same`, `straw2_reweight`, `crush_weights.sh` | None | Original counts/acceptance assertions plus unmodified C builder lengths and mapping digests in `weights.rs` | Host libc determines the captured unseeded C RNG realization | Regenerating on a different reference platform | Retained: passing offline |
| `straw2_stddev` | No asserted client observation; upstream only prints diagnostics | No acceptance threshold exists | Diagnostic drift is unmeasured | Upstream supplies a criterion | Excluded: no acceptance criterion |
| `choose_args_compat`, `choose-args.t`, and all 16 `crush-choose-args.sh` states | CLI compilation/dump text | Decoded canonical/compat state, positional IDs and weights, malformed-byte rejection, and pinned-C placements in `choose_args.rs` | Producer formatting | A tool API | Retained: passing offline |
| `get_immediate_parent`, `check_item_loc`, `bucket_types`, `distance`, `location.t` | Editor setup commands | Source-shaped and captured maps exercise the read-only queries in `hierarchy.rs` | Editor return values | Map editing is exposed | Adapted: client queries retained |
| Device-class, `reclassify.t`, `TEST_mon_classes` maps | Class create/rename/remove responses | Decoded shadows/rules/metadata and filtered C placements in `classes.rs`; original write is live-tested below | Monitor lifecycle responses | Typed monitor management API | Adapted: map effects retained |
| Uniform, List, TREE and `add-item-in-tree.t` | Bucket editor responses | Captured TREE map and 352 exact C vectors; local C contracts cover Uniform/List in `legacy_buckets.rs` | Editor-only behavior | Editor API | Adapted: decoder and mapper retained |
| `adjust_item_weight`, `adjust_subtree_weight`, `reweight*.t`, `test_crushdiff.sh`, `TEST_crush_bucket` | Editor/compiler stderr and return counts | Captured before/after maps, decoded weights, movement, and 5,536 C rows in `weight_topology.rs` | Text compiler behavior | Tool API | Adapted: resulting map effects retained |
| Local retry/MSR mapper regressions | No upstream case name | Pinned unmodified C mapper provides exact vectors for retry guards and malformed MSR behavior in `regressions.rs` | Cases outside the selected matrix | A new upstream regression | Retained: supplemental |
| `OSDMapTest.MapPG`, `PrimaryAffinity`, feature masks, node flags, upmap/pg_temp transitions | Producer incremental construction | Source-shaped incrementals and pinned-C placement digests exercise public OSDMap queries | Uncaptured monitor producer paths | More source fixture coverage is needed | Retained: passing library tests |
| `CleanPGUpmapPrimaries` | The source's keyed cleaner count sequence | The 3-host/64-PG map and emitted keyed deltas are unavailable | Primary-removal sentinel replay | Cleaner count behavior | Captured map and deltas | Deferred: producer prerequisite |
| `BUG_38897` | Capacity balancer result for one-element/full overrides | The custom 12-OSD map and post-balancer delta are unavailable | None; the generic override test is not a replacement | Incorrect capacity response | Captured map and delta | Deferred: producer prerequisite |
| `BUG_42052` | Cleanup of conflicting full and item overrides | The keyed old-upmap deltas and fixed rule map are unavailable | None | Incorrect conflict cleanup | Captured map and deltas | Deferred: producer prerequisite |
| `BUG_42485` | Cross-device-class/rack item removal | The 60-OSD topology, chosen pairs, and keyed deltas are unavailable | None | Topology-sensitive removal | Captured topology and deltas | Deferred: producer prerequisite |
| `BUG_43124` | EC-safe cross-rack remap retention | The 200-OSD EC topology and decision delta are unavailable | None | Cleaner drops a safe remap | Captured topology and delta | Deferred: producer prerequisite |
| `BUG_51842` | Parameterized-rule cleanup across size changes | The parameter instance, topology, and keyed upmap delta are unavailable | None | Parameter/size cleanup divergence | Captured instance and deltas | Deferred: producer prerequisite |
| `osdmaptool/{create-racks,test-map-pgs,crush,create-print}` | CLI rendering and randomized map creation | Pinned maps, empty PG0 result, and every 500-OSD workload count are checked in `osdclient_osdmap_test.rs` | CLI-only regressions | Tool API | Adapted: decode and placement retained |
| v19 external OSDMap corpus | Reproducible fixture ownership | Explicit ignored gates fail on absent, invalid, or malformed selected corpus | No corpus breadth in normal CI | `RADOS_OSDMAP_CORPUS_DIR` and incremental corpus supplied | Deferred: supplemental gate |
| `TEST_classes`, `TEST_set_device_class`, `PoolCreateWithCrushRule` | Admin command formatting | Exact ordered `up` sets, source-order Rust writes, application enable, explicit rule 0, and deletion in the live gate | Requires Docker and a local host-capable runtime | CI provides the pinned runtime | Retained: passed on both pins |

## Closure of the client-observation gaps

| Case | Lost client observation | Reason | Replacement | Remaining risk | Revisit | Decision |
| --- | --- | --- | --- | --- | --- | --- |
| `TEST_reweight_vs_classes` | Canonical and shadow host reweight | Class commands are producer operations | Fresh live map before/after, exact `65536 -> 131072`, 16 C/Rust rows each | Management response behavior | Management API | Retained live |
| `TEST_mon_classes` | Original rbd write and resulting lifecycle map effects | Create/rename/remove replies are producer-only | Exact initial `[1,2,0]`, Rust `ABCDEF\n` write before mutations; captured state maps and vectors | Lifecycle replies | Management API | Retained live/offline |
| QA choose-argument transitions | Before/after positional state and placement | CLI emits maps, this crate consumes them | All 16 decoded maps and 160 pinned-C placement rows | CLI text | Tool API | Retained offline |
| `adjust_item_weight` / `adjust_subtree_weight` | Aggregate and selected-location weights | Mutation itself is producer work | Captured maps, exact decoded aggregates, C rows | Editor return counts | Editor API | Retained offline |
| `reweight.t`, `reweight_multiple.t`, `test_crushdiff.sh` | Shared topology map/movement effects | CLI produces maps | Pinned before/after maps and movement vectors | CLI reporting | Tool API | Retained offline |
| `check-invalid-map.t` | Reject source-shaped non-CRUSH input | CLI diagnostic text is producer-only | Decoder rejection of checked-in hosts text | Exact CLI message | Tool API | Retained boundary check |
| `TEST_crush_bucket` | Added STRAW bucket wire shape | Shell/compiler output is producer-only | Fresh pinned capture, decoded bucket fields, C vectors | Compiler stderr | Tool API | Retained offline |
| `OSDMapTest.Features` | Feature masks derived from pool/rule | Map construction is producer work | Public `get_features_for_release` source expectations, including stretch mode | Automatic release selection | Map provenance can select it | Retained library query |
| `OSDMapTest.get_osd_crush_node_flags` | Root flag transitions and missing ID | Incremental construction is producer work | Source-shaped `0 -> 123 -> 456 -> 0` incrementals | Other hierarchy patterns | New source case | Retained library query |
| `osdmaptool` rack and 500-OSD cases | Exact PG0 and large-topology placement | Tool setup is producer work | Captured maps, empty PG0 and all 8,000 PGs | Other tool modes | New pinned fixture | Retained offline |
| `TEST_crush_rule_create_erasure` | Generated implicit/explicit rule consumption | Rule creation response/XML/profile logs are producer-only | Fresh live maps, complete five-step tuple, 16 C/Rust rows per map | Producer output formatting | Management API | Retained live |

## Technical decisions and reproduction

- `get_features_for_release` is a public read-only query, including stretch
  behavior, because feature derivation is client-visible. Its cost is a small
  explicit release selector until map provenance can choose semantics.
- Optimized-EC `pg_temp` derives primary from the stored primary-first vector;
  an all-NONE vector has primary `-1`. This preserves Ceph routing at the cost
  of retaining the source's order conversion.
- The OSDMap placement oracle uses the source-shaped `build_simple` fixture and
  pool-adjusted seed, rather than an older standalone `x=0` map. This costs a
  larger fixture but tests the actual contract.
- A missing pool yields an empty `PgPlacement` with primaries `-1`; raw
  `pg_to_osds` still errors. This matches the higher-level Ceph query without
  weakening the raw API.
- MOSDOp v9 and its OpenTelemetry prefix require every bit of
  `MASK_SERVER_SQUID`; the full-mask check keeps Quincy on v8 and prevents its
  writes from timing out.
- Each live observation starts a fresh one-MON fixture and verifies its
  container is removed. This costs startup time and keeps pool/rule IDs and
  class state from leaking between source cases.
- The `TEST_mon_classes` Rust write occurs immediately after the source's
  `[1,2,0]` observation and before lifecycle mutations; order is part of the
  compatibility assertion.

### Live-source provenance

The live gate ports [Quincy `TEST_classes` and
`TEST_set_device_class`](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/qa/standalone/crush/crush-classes.sh#L88-L164),
the [Tentacle equivalent](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/qa/standalone/crush/crush-classes.sh#L88-L164),
[Quincy `PoolCreateWithCrushRule`](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/librados/pool.cc#L124-L135),
the [Tentacle equivalent](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/librados/pool.cc#L125-L136),
and the [Quincy](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/qa/standalone/mon/osd-crush.sh#L84-L121)
and [Tentacle](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/qa/standalone/mon/osd-crush.sh#L84-L121)
erasure-rule cases. `live-rust-crush-gates.sh` verifies the official image's
`ceph --version`, starts a non-root one-MON fixture with the required OSD
count, runs the exact ignored Cargo test, and verifies container removal.

Run offline checks with `cargo test -p rados --test crush --offline`.
Regenerate reference fixtures using the commands in [README.md](README.md);
run live checks with `live-rust-crush-gates.sh {quincy|tentacle} all`. The
README pins the exact source commits, official image digests, tools, hashes,
and network-disabled generation commands.
