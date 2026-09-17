# CRUSH test status

This page records what has been ported and what remains. Unmodified upstream
maps and transcripts are documented in
[fixtures/README.md](fixtures/README.md). Local C reference runners, generated
outputs and regeneration instructions are in [reference/README.md](reference/README.md).

## Reference releases

- **Quincy v17.2.7**: `b12291d110049b2f35e32e0de30d70e9a4c060d2` — required baseline.
- **Tentacle v20.2.4**: `7f793731f1b39eb4f465e960113d2363c311b964` — additional target; includes MSR.

Run the ported suite without Ceph tools or a cluster:

```sh
cargo test -p rados --test crush --offline
```

To check a newer Ceph release, pin its source commit and tools, regenerate the
reference data with the scripts in `reference/`, then run this offline suite.
Keep a new release-specific fixture only when its bytes or expected placement
differs; otherwise verify and reuse the existing one.

## Ported tests

**98 tests pass, none ignored.** These establish the scenarios below, not
complete CRUSH or end-to-end client compatibility.

| Family | Ceph coverage retained | Rust tests | Status |
| --- | --- | --- | --- |
| Golden mappings, both releases | 12 crushtool scenarios; 87,040 exact ordered vectors, lengths, NONE slots and result-size histograms | [golden.rs](golden.rs) | Passing |
| Rule retry settings, both releases | Three `set-choose.t` profiles, six rules, replicas 2/3, seeds 0..1023: 36,864 vectors and result-size histograms | [golden.rs](golden.rs) `set_choose_*` | Passing with both compiled map encodings |
| Mixed FIRSTN/INDEP, both releases | `test-map-firstn-indep.t` rules 0/1, seed 1, replicas 1..10: ten exact bad vectors and ten successful lengths/no-NONE checks | [golden.rs](golden.rs) `firstn_indep_compiled` | Passing with both compiled map encodings |
| FIRSTN/INDEP, Tentacle | Seven scenarios per family, each in NORMAL and MSR mode: 28 variants | [functional.rs](functional.rs) | Passing |
| Dedicated MSR topologies, Tentacle | Four scenarios: host/OSD failure, truncated fanout, EC 8+6 and two roots | [functional.rs](functional.rs) | Passing; 3,007 ordered vectors also compared with the C mapper |
| Weight distribution, both releases | Both `crush_weights.sh` assertions, each over seeds 1..1,000,000 with the original thresholds | [functional.rs](functional.rs) | Passing; device counts also compared with both C mappers |
| Quincy INDEP, mapping-only type adaptation | `indep_toosmall`, `indep_basic`, `indep_out_alt`, `indep_out_contig`, `indep_out_progressive`: original domains, holes, uniqueness and progressive movement | [functional.rs](functional.rs) existing NORMAL tests | Coverage: Ported; Readiness: Ready now; Verification: Passing. Raw type 123 equals Erasure type 3 for all 508 ordered C vectors; decoding raw 123 remains unported. |
| Legacy STRAW / STRAW2 weights, Quincy | `straw_zero` (10,000), `straw_same` (100,000), `straw2_reweight` (1,000,000) | [weights.rs](weights.rs) | Coverage: Ported; Readiness: Ready now; Verification: Passing. C-built STRAW lengths, output digests and unseeded C RNG realization are recorded in the reference runner. |
| Insufficient mappings, both releases | `bad-mappings.t` rules 0/1, seed 1, ten replicas: exact FIRSTN short result and INDEP NONE slots | [golden.rs](golden.rs) `bad_mappings_compiled`; [functional.rs](functional.rs) `bad_mappings` | Passing with both compiled map encodings and the earlier direct setup; same two upstream cases |
| Retry-profile observation, both releases | `show-choose-tries.t` rule 0 FIRSTN/count 2 and rule 1 INDEP/count 1: both full 50-bin profiles, distinct fresh commands | [profile.rs](profile.rs) | Passing with both compiled encodings; caller-owned batch profile accumulates, explicit start resets, and stop discards |
| Device classes and reclassification, both releases | decoded class/shadow lifecycle, ten successful `reclassify.t` pairs, and pinned C placement results | [classes.rs](classes.rs) | Coverage: Ported; Readiness: Ready now; Verification: Passing. Producer/editor APIs remain outside client scope. |
| Legacy Uniform/List/TREE, both releases | `add-item-in-tree.t` final TREE map, all three rules, and 352 pinned-C ordered vectors. Uniform/List are minimal local C differential contracts for complete, collision, retry, and NONE-hole partial-availability cases. | [legacy_buckets.rs](legacy_buckets.rs) | Coverage: Ported; Readiness: Ready now; Verification: Passing. TREE `num_nodes` decodes as its C wire `u8`; compiler/editor APIs remain outside client scope. |
| Retained weight/topology consumers, both releases | `adjust_item_weight`, `adjust_subtree_weight`, `reweight.t`, `reweight_multiple.t`, `test_crushdiff.sh`, and `TEST_crush_bucket`; 5,536 exact pinned-C ordered rows | [weight_topology.rs](weight_topology.rs) | Coverage: Ported; Readiness: Ready now; Verification: Passing. The emitted shell maps come from a self-cleaning pinned one-MON/three-OSD capture; compiler stderr remains producer-only. |
| Additional mapper regressions | Eight local tests, including 12,600 vectors generated from pinned C mappers | [regressions.rs](regressions.rs) | Passing; additional coverage, not upstream test ports |

The golden scenarios are `bobtail_tunables`, `firefly_tunables`,
`hammer_tunables`, `indep`, `jewel_tunables`, `legacy_tunables`,
`tries_vs_retries`, and `vary_r_0` through `vary_r_4`. Their upstream inputs
are identical across the two releases and are counted once.

Both functional families cover `toosmall`, `basic`, `single_out_first`,
`single_out_last`, `out_alt`, `out_contig`, and `out_progressive`. Tests retain
upstream seed ranges, topology, failure transitions, holes, uniqueness,
positional stability and movement bounds. Every test has a pinned source link.

The five pre-existing NORMAL INDEP tests also port the Quincy names listed in
the table. Quincy assigns the test-only raw rule type 123, which the decoder
intentionally does not accept. `reference-check.c` constructs the exact map
with raw 123 and type 3, compares every ordered output in the five complete
domains (including 108 progressive failure transitions), then verifies matching
per-case streaming digests in both pinned releases. The Rust tests therefore
use the existing `RuleType::Erasure` only as this mapping adaptation; they do
not exercise or claim raw-123 decoding.

`weights.rs` retains each upstream assertion and every original sample count.
For legacy STRAW, the pinned unmodified `builder.c` with `straw_calc_version=1`
provides the ordered lengths. The C audit streams every ordered Rust/C output
through a fixed word-FNV-1a digest: initialize `1469598103934665603`, then for
each output XOR/multiply its length and each item as u32 by `1099511628211`.
The published C/Rust digest pairs are `straw_zero` `12722a47fde289ef`,
`straw_same` `32c9040dd1425108` (12 differences), and `straw2_reweight`
`65e72f17e5a64b0b`. The latter preserves the reference process/libc result
`rand()%10 == 7`, so item 1 changes to `65536 / 10 * 7 == 45871`.

The three weight tests have identical setup and assertions in both pinned
releases: [`straw_zero` Quincy](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/crush/crush.cc#L268) /
[Tentacle](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/crush.cc#L709),
[`straw_same` Quincy](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/crush/crush.cc#L322) /
[Tentacle](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/crush.cc#L763), and
[`straw2_reweight` Quincy](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/crush/crush.cc#L533) /
[Tentacle](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/crush.cc#L974).
The source audit found no setup or assertion difference. The successful C audit
retains and compares the exact legacy STRAW lengths emitted by both builders
with the arrays in the executing Rust maps.

`CRUSHTest.straw2_stddev` remains Coverage: Not ported; Readiness: Ready now;
Verification: Not run. It only prints diagnostics and supplies no acceptance
assertion; revisit if upstream defines a pass criterion.

The dedicated MSR ports are `msr_4_host_2_choose_rule`, `msr_2_host_2_osd`,
`msr_5_host_8_6_ec_choose`, and `msr_multi_root`. The first three retain seed 0;
multi-root retains seeds 0..999 and both failure transitions. Multi-root also
checks every OSD's root and all hosts used by each failure domain, extending
upstream's check of the first OSD in each group. Setup is assembled directly;
no Rust CRUSH editor is required. These MSR tests do not exist in Quincy.

The distribution ports retain STRAW2 weights `[10,10,10,10,1]`, legacy map
tunables, and `bc scale=5` arithmetic. They check the original one-replica
proportionality and three-replica deviation criteria; distribution alone does
not establish exact placement parity.

Local regressions cover chained INDEP, explicit leaf retries, conventional
rules containing CHOOSE_MSR, malformed MSR blocks, EMIT reset, truncated MSR
fanout, rule retry overrides (positive, zero, negative and repeated values),
and safe rejection of negative MSR fanout. The C oracle supplies expected
values; negative fanout is a separate Rust validation contract.

The nine mapping commands in `set-choose.t::cmd-02/cmd-03/cmd-04`,
`test-map-firstn-indep.t::cmd-02/cmd-03`, `bad-mappings.t::cmd-02/cmd-03`,
and `show-choose-tries.t::cmd-03/cmd-05` (locally assigned command IDs) have
Coverage: Ported for client decode and placement, Readiness: Ready now,
Verification: Passing. Each test has pinned source links. Unmodified
inputs/transcripts are in [fixtures](fixtures/README.md); eight generated
binaries and their commands, versions and hashes are in
[reference](reference/README.md#compiled-cli-maps). Tentacle adds eight MSR
tunable bytes; both binary variants are exercised, consuming all bytes.

`set-choose` retains all-in, OSDs 0/1/3/4 out, and OSDs 0/3/5/7 out with
OSD 4 at 0.5 and OSD 6 at 0.1. The latter becomes integer weight 6553,
matching crushtool. The seed sequence, rule sequence, sample count and histogram
consistency are validated before placement assertions. `firstn/indep` retains
the original duplicate rack1 entry and both mixed rules. Its transcript reports
only bad mappings; successful calls are checked for length and absence of NONE,
without inventing exact expected vectors for them.

The new retry-profile tests recorded an initial behavioral failure with every
visible bin zero, then passed after the counter plumbing. The preparation
script independently reproduced every complete command output with both pinned
tools. Profile collection is caller-owned: ordinary placement does not allocate
or collect counter data. FIRSTN records only accepted replicas; INDEP records
once after each retry loop, including recursive calls. Ceph stores 51 counters
for `choose_total_tries=50` but exposes exactly 50, which the snapshot keeps.
The earlier five new Rust tests passed before production changes; none were needed.
Native text compilation, temporary-file cleanup and console formatting remain
outside the Rust client API (overall CLI Coverage: Partial; disposition review:
Pending). No sample or mapping assertion is excluded. The original
direct-assembly bad-mappings test and C-builder audit remain supplementary
checks of the same two upstream scenarios.

## Remaining work

This is the continuation order. “Not ported” does not mean unsupported;
“Missing support” names a known implementation gap.

| Order | Tests / behavior | Status and prerequisite |
| --- | --- | --- |
| 3 | Remaining Quincy mapper cases, STRAW zero/perturbed weights, STRAW2 reweight | Passing; five cases reuse their existing NORMAL ports after the raw-123/type-3 proof, and three assertion-bearing weight cases retain original setup/RNG outcome |
| 3 | Zero/nonpositive rule retry settings | Passing; local C-reference cases retain positive, zero, negative and repeated override semantics for conventional FIRSTN and recursive chooseleaf |
| 4 | Choose arguments: positional weights/IDs and legacy encoding fallback | Coverage: Ported; Readiness: Ready now; Verification: Passing — 2,100 pinned C vectors cover direct FIRSTN/INDEP, recursive CHOOSELEAF, chained steps, and Tentacle MSR FIRSTN/INDEP with unavailable-device retries. All 16 published QA transition states have decoded canonical/compat fields and 160 ordered C/Rust placement rows; update/no-update pre/remove retain both weight-set positions and IDs at real index 0 without default fallback. CLI indexes 1–6 retain all declared arguments and every bucket's type, size, canonical weights and items. Malformed source-byte mutations assert the bucket/weight/ID guards. Producer APIs remain outside this retained client-observation scope. |
| 4 | Device-class reclassification mappings | Coverage: Ported; Readiness: Ready now; Verification: Passing. Ten successful original pairs retain every rule's complete before/after C/Rust workload (replicas 1..10, x=0..1023); `gabe` remains a producer-only failure disposition. |
| 5 | OSDMap raw/up/acting sets, primary/affinity, EC positions and map transitions | Not ported as a complete reference suite; capture original OSDMap setup/deltas |
| 5 | Original client I/O and pool scenarios on both releases | Not run; requires matching clusters |

The old corpus integration tests can return successfully without their
external inputs; they do not count as passing compatibility checks.

## Reclassification and monitor classes — 2026-09-17

Coverage: Ported. Readiness: Ready now. Verification: Passing. Ten successful
`reclassify.t` pairs retain both pins, every original rule, 10,240 ordered
mappings per rule on each before/after side, and their original nonzero
movements. `gabe` is explicitly the producer/editor failure. Captured
`TEST_mon_classes` states cover removal, retained `asdf`, moved `abc`, and
`class_1` to `class_2` rename metadata plus C/Rust filtered placement. The
original `SOMETHING` write remains task9 scope.

CRUSH editor/compiler/formatter and monitor/balancer tests are not native
client API ports. Their resulting maps and placement effects still need
client coverage. The table above tracks the retained client-visible behavior;
source and regeneration details stay beside each fixture in `fixtures/` and
`reference/`.
