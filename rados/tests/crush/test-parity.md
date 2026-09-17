# CRUSH test status

This page records what has been ported and what remains. Test preparation,
execution history and audit notes live in [`.notes/crush`](../../../.notes/crush).
Unmodified upstream maps and transcripts are documented in
[fixtures/README.md](fixtures/README.md). Local C reference runners, generated
outputs and regeneration instructions are in [reference/README.md](reference/README.md).

## Reference releases

- **Quincy v17.2.7**: `b12291d110049b2f35e32e0de30d70e9a4c060d2` — required baseline.
- **Tentacle v20.2.4**: `7f793731f1b39eb4f465e960113d2363c311b964` — additional target; includes MSR.

Run the ported suite without Ceph tools or a cluster:

```sh
cargo test -p rados --test crush --offline
```

## Ported tests

**59 tests pass, none ignored.** These establish the scenarios below, not
complete CRUSH or end-to-end client compatibility.

| Family | Ceph coverage retained | Rust tests | Status |
| --- | --- | --- | --- |
| Golden mappings, both releases | 12 crushtool scenarios; 87,040 exact ordered vectors, lengths, NONE slots and result-size histograms | [golden.rs](golden.rs) | Passing |
| Rule retry settings, both releases | Three `set-choose.t` profiles, six rules, replicas 2/3, seeds 0..1023: 36,864 vectors and result-size histograms | [golden.rs](golden.rs) `set_choose_*` | Passing with both compiled map encodings |
| Mixed FIRSTN/INDEP, both releases | `test-map-firstn-indep.t` rules 0/1, seed 1, replicas 1..10: ten exact bad vectors and ten successful lengths/no-NONE checks | [golden.rs](golden.rs) `firstn_indep_compiled` | Passing with both compiled map encodings |
| FIRSTN/INDEP, Tentacle | Seven scenarios per family, each in NORMAL and MSR mode: 28 variants | [functional.rs](functional.rs) | Passing |
| Dedicated MSR topologies, Tentacle | Four scenarios: host/OSD failure, truncated fanout, EC 8+6 and two roots | [functional.rs](functional.rs) | Passing; 3,007 ordered vectors also compared with the C mapper |
| Weight distribution, both releases | Both `crush_weights.sh` assertions, each over seeds 1..1,000,000 with the original thresholds | [functional.rs](functional.rs) | Passing; device counts also compared with both C mappers |
| Insufficient mappings, both releases | `bad-mappings.t` rules 0/1, seed 1, ten replicas: exact FIRSTN short result and INDEP NONE slots | [golden.rs](golden.rs) `bad_mappings_compiled`; [functional.rs](functional.rs) `bad_mappings` | Passing with both compiled map encodings and the earlier direct setup; same two upstream cases |
| Additional mapper regressions | Seven local tests, including 11,200 vectors generated from pinned C mappers | [regressions.rs](regressions.rs) | Passing; additional coverage, not upstream test ports |

The golden scenarios are `bobtail_tunables`, `firefly_tunables`,
`hammer_tunables`, `indep`, `jewel_tunables`, `legacy_tunables`,
`tries_vs_retries`, and `vary_r_0` through `vary_r_4`. Their upstream inputs
are identical across the two releases and are counted once.

Both functional families cover `toosmall`, `basic`, `single_out_first`,
`single_out_last`, `out_alt`, `out_contig`, and `out_progressive`. Tests retain
upstream seed ranges, topology, failure transitions, holes, uniqueness,
positional stability and movement bounds. Every test has a pinned source link.

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
fanout, and safe rejection of negative MSR fanout. The C oracle supplies
expectations; negative fanout is a separate Rust validation contract.

The seven mapping commands in `set-choose.t::cmd-02/cmd-03/cmd-04`,
`test-map-firstn-indep.t::cmd-02/cmd-03` and `bad-mappings.t::cmd-02/cmd-03`
(locally assigned command IDs) have Coverage: Ported for client decode and
placement, Readiness: Ready now, Verification: Passing. Each test has pinned
source links. Unmodified inputs/transcripts are in [fixtures](fixtures/README.md);
six generated binaries and their commands, versions and hashes are in
[reference](reference/README.md#compiled-cli-maps). Tentacle adds eight MSR
tunable bytes; both binary variants are exercised, consuming all bytes.

`set-choose` retains all-in, OSDs 0/1/3/4 out, and OSDs 0/3/5/7 out with
OSD 4 at 0.5 and OSD 6 at 0.1. The latter becomes integer weight 6553,
matching crushtool. The seed sequence, rule sequence, sample count and histogram
consistency are validated before placement assertions. `firstn/indep` retains
the original duplicate rack1 entry and both mixed rules. Its transcript reports
only bad mappings; successful calls are checked for length and absence of NONE,
without inventing exact expected vectors for them.

All five new Rust tests passed before production changes; none were needed.
The preparation script independently reproduced every complete command output
with both pinned tools. Native text compilation, temporary-file cleanup and
console formatting remain outside the Rust client API (overall CLI Coverage:
Partial; disposition review: Pending). No sample or mapping assertion is
excluded. The original direct-assembly bad-mappings test and C-builder audit
remain supplementary checks of the same two upstream scenarios.

## Remaining work

This is the continuation order. “Not ported” does not mean unsupported;
“Missing support” names a known implementation gap.

| Order | Tests / behavior | Status and prerequisite |
| --- | --- | --- |
| 3 | Remaining Quincy mapper cases, STRAW zero/perturbed weights, STRAW2 reweight | Not ported; capture original setup and RNG outcome; explicitly resolve Quincy test-only rule type 123 |
| 3 | Zero/nonpositive rule retry settings | Add local C-reference regressions; local retry overrides and `SetChooseTries=0` currently differ from Ceph |
| 4 | Choose arguments: positional weights/IDs and legacy encoding fallback | Missing support: decoder discards choose arguments; import `choose_args_compat` and CLI fixtures after retaining/selecting them |
| 4 | Device-class shadow mapping, hierarchy/location queries, retry counters | Incomplete coverage; import class maps and implement the missing query/observation APIs |
| 5 | OSDMap raw/up/acting sets, primary/affinity, EC positions and map transitions | Not ported as a complete reference suite; capture original OSDMap setup/deltas |
| 5 | Original client I/O and pool scenarios on both releases | Not run; requires matching clusters |

Uniform, List and Tree do not yet have complete reference mapping coverage.
The old corpus integration tests can return successfully without their
external inputs; they do not count as passing compatibility checks.

CRUSH editor/compiler/formatter and monitor/balancer tests are not native
client API ports. Their resulting maps and placement effects still need
client coverage. Detailed per-test dispositions and source references are
preserved in the [upstream inventory](../../../.notes/crush/test-parity-history.md#mapper-unit-tests).
