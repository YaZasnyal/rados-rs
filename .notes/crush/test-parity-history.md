> Historical inventory and execution notes, moved from the public status document.
> Snapshot at d1f0dc8; historical statuses and line references are not current.
> Current coverage: [CRUSH test status](../../rados/tests/crush/test-parity.md).

# CRUSH test parity inventory

Inventory date: 2026-09-17. Rust baseline: `9388052` on `main`, before the
`docs/crush-test-parity` branch (documentation first, then Stage 1 tests). Policy:
[Testing requirements](../../.claude/TESTING.md).

**All 47 CRUSH integration tests pass**: 12 golden scenarios, 28 Tentacle
functional variants and 7 local regression tests. The original goldens check
87,040 ordered mappings and result-size histograms shared by the pinned
releases; the regressions add 11,200 vectors generated from the C mapper.
The original failing run and the fixes are recorded in
[Stage 1](#stage-1-execution), [Stage 2](#stage-2-mapper-fixes),
[Stage 3](#stage-3-tentacle-functional-ports), and
[Stage 4](#stage-4-functional-mapper-fixes), and
[Stage 5](#stage-5-source-audit-regressions).
This verifies these fixtures, not every possible CRUSH input or the full
placement pipeline.

Navigation: [Stage 5 audit regressions](#stage-5-source-audit-regressions) · [Stage 4 functional fixes](#stage-4-functional-mapper-fixes) · [Stage 3 functional ports](#stage-3-tentacle-functional-ports) · [Stage 2 fixes](#stage-2-mapper-fixes) · [Stage 1 execution](#stage-1-execution) · [Current Rust coverage](#current-rust-coverage) ·
[Prerequisites](#prerequisites-and-disposition-reasons) ·
[Mapper](#mapper-unit-tests) · [Wrapper](#crushwrapper-unit-tests) ·
[CLI](#crushtool-cases) · [Fixtures](#fixture-and-helper-catalogue) ·
[Shell](#crush-shell-and-standalone-tests) ·
[Placement](#adjacent-placement-and-management-tests) ·
[Porting order](#recommended-porting-order-and-acceptance).

## CRUSH validation requirements

- Inventory all bucket algorithms, FIRSTN/INDEP, chooseleaf, MSR, retries,
  zero/partial weights, reweighting, tunables, choose arguments, device
  classes, hierarchy/locality, and placement overrides where relevant to
  the reference release and client. Do not narrow coverage to the easiest
  supported algorithm.
- Preserve exact ordered OSD vectors, lengths, holes (`CRUSH_ITEM_NONE`),
  failure domains, and changes after devices become unavailable. For the
  placement pipeline, also compare raw/up/acting sets, primary, and EC shard
  positions; a set-membership check loses required semantics.
- Reuse crushtool maps and expected vectors. Preserve seeds, sample counts,
  and acceptance criteria in statistical tests; golden vectors complement
  those tests rather than replacing them. Printing-only diagnostics are not
  assertion-based compatibility checks.
- Preserve both expectations when releases differ and document which
  version or feature selects the behavior. Unsupported behavior remains a
  documented compatibility gap, not a passing substitute algorithm.

## References and coverage boundary

| Release | Immutable commit | Role |
| --- | --- | --- |
| v17.2.7 (Quincy) | `b12291d110049b2f35e32e0de30d70e9a4c060d2` | Required baseline |
| v20.2.4 (Tentacle) | `7f793731f1b39eb4f465e960113d2363c311b964` | Additional target |

The complete primary inventory covers every `TEST_F`/`TEST_P` definition in
`src/test/crush/crush.cc` and `CrushWrapper.cc`, all 37 `.t` files in
`src/test/cli/crushtool`, and `src/test/crush/crush_weights.sh`, in both
releases. The CLI command catalogue below retains setup and cleanup commands
as well as assertions so no scenario disappears inside a file-level label.

The adjacent inventory covers named tests in the CRUSH standalone suites,
the CRUSH monitor suite and bucket script, and relevant OSDMap tests. It also
records the boundaries found in librados/neorados, other OSD tests, and QA.
It is not an inventory of every Ceph subsystem that happens to use CRUSH.
Direct CRUSH tests are complete within the paths above; a broader OSDMap,
protocol, or live-client parity claim requires its own inventory.

### Status conventions

Unless a row explicitly says otherwise, every upstream entry inherits:
**Coverage: Not ported; Rust counterpart: None; Verification: Not run;
Review: Pending.** Existing Rust tests with overlapping concepts are listed
separately and are not credited as upstream ports. An upstream diagnostic
without mapping assertions is explicitly identified.

- **Ready now**: a faithful test can be written using current Rust entry
  points and the available upstream inputs/assertions. It may fail; readiness
  is not a compatibility result.
- **Blocked**: the named prerequisite is missing. The action column explains
  what makes the test executable without substituting a different scenario.
- **Outside client scope**: the original assertions exercise an editor,
  monitor, balancer, or CLI presentation API rather than client placement.
  This is a proposed disposition, not an approved deletion. Revisit it if that
  API enters scope. Any useful client decode/mapping subset is retained
  explicitly; it does not make the entire upstream test a port.

Rows containing both releases retain both source identities. Counts below
are release-specific source cases, not a count of distinct Rust tests.
Parameterized `IndepTest` and `FirstnTest` cases each have **NORMAL and MSR**
variants; both must be preserved.

### Counts at this baseline

| Primary unit suite | Source definitions | Expanded variants | Ready now | Blocked | Outside client scope | Verified ports |
| --- | --- | --- | --- | --- | --- | --- |
| v17.2.7: mapper + wrapper | 33 | 33 | 1 | 14 | 18 | 0 |
| v20.2.4: mapper + wrapper | 46 | 60 | 33 | 9 | 18 | 28 |

Each Ready-now unit total includes one `straw2_stddev` diagnostic without a distribution assertion. Stages 3 and 4 port and verify 28 Tentacle variants; other source definitions have Coverage: Not ported. Expanded variants do not represent additional upstream definitions.

| Other catalogue | v17.2.7 | v20.2.4 | Counting unit |
| --- | --- | --- | --- |
| crushtool | 37 files / 215 commands | 37 files / 215 commands | Includes setup/cleanup and 48 unindented output-csv commands; 167 commands per release use standard Cram indentation. |
| Ready binary-map goldens | 12 files | 12 files | 87,040 mapping vectors per release; files/inputs shared when bytes match. |
| Named standalone/bucket functions | 17 | 18 | Includes management cases and retained client subsets. |
| Weight-distribution assertions | 2 | 2 | Both ready now; one million seeds per assertion. |
| Adjacent OSDMap definitions | 18 | 30 | Listed individually; BUG_51842 has four variants per release. |

For CLI readiness, 12 complete mapping files are Ready now; invalid-map decoding is an additional ready **subset**. Other files carry explicit F/A/L/D prerequisites or E exclusions below. These heterogeneous totals are intentionally not added into one percentage.

## Stage 1 execution

Entry point: [rados/tests/crush.rs](../../rados/tests/crush.rs), with golden tests
in [crush/golden.rs](../../rados/tests/crush/golden.rs). Run without Ceph tools,
a Ceph checkout, environment variables or a cluster:

```sh
cargo test -p rados --test crush --offline
```

The 18 original inputs (6 binary maps and 12 Cram transcripts) are checked in
with [provenance, SHA256 and licenses](../../rados/tests/crush/fixtures/README.md).
Each file is byte-identical across the pinned releases. Tests preserve all
87,040 ordered vectors, requested replica counts, seeds 0..1023, command
weight/tunable overrides and result-size histograms. The helper validates every transcript
record and its histogram before calling the mapper. Each scenario stops at
its first mapping mismatch, reporting source line, rule, seed and requested replicas.
Passing means every mapping and histogram was checked; a failed scenario
must not be counted as having verified its later vectors.

Coverage of each complete Cram scenario is **Partial**: all client
mapping/statistics assertions are ported in the 12 entries below. CLI output
formatting and the map-modified advisory are excluded because this crate has no crushtool CLI. Original text remains
in the fixtures for review; revisit if a compatible CLI is implemented.
There are no ignored tests or substituted expectations. Review: Pending.
No generated oracle, upstream executable or live-cluster check was run.

Stage 1 (`478f061`) deliberately changed only tests, fixtures and this
inventory. All 12 tests were left active and failing, with implementation
fixes deferred to the separately authorized Stage 2 below.

Stage 1 functional runs on `8dbcbb8` plus the new tests (macOS,
default features, no Ceph runtime dependencies):
**0 passed, 12 failed, 0 ignored**. All six maps decoded successfully and
consumed their input. First differences (`expected -> actual`):

| Rust scenario | Rule / seed / replicas | First mismatch |
| --- | --- | --- |
| `bobtail_tunables`, `legacy_tunables` | 0 / 788 / 1 | `[226] -> [472]` |
| `indep` | 1 / 788 / 1 | `[226] -> [472]` |
| `firefly_tunables`, `hammer_tunables`, `jewel_tunables` | 0 / 155 / 1 | `[75] -> [114]` |
| `tries_vs_retries` | 0 / 0 / 1 | `[7] -> []` |
| `vary_r_0` | 3 / 0 / 2 | `[94,85] -> [94]` |
| `vary_r_1` | 3 / 0 / 2 | `[94,6] -> [94,114]` |
| `vary_r_2` | 3 / 0 / 2 | `[94,45] -> [94,114]` |
| `vary_r_3`, `vary_r_4` | 3 / 0 / 2 | `[94,85] -> [94,114]` |

Additional validation on the final Stage 1 tree:

- `cargo test -p rados --lib crush:: --offline`: 26 passed, 1 ignored.
- `cargo fmt --all -- --check`: passed.
- `cargo clippy -p rados --test crush --offline -- -D warnings`: blocked by
  existing unused `OSD_STAT_INTERFACES_NUM_FIELDS` and
  `OSD_STAT_INTERFACES_SIZE` in `rados/src/osdclient/pgmap_types.rs`.
- `cargo clippy -p rados --test crush --offline -- -D warnings -A dead-code`:
  passed; this allows only that pre-existing lint category for this check.
- All 18 fixture files compared byte for byte against both pinned Git
  commits; SHA256 values, 87,040 mapping records, source references, local
  document links and upstream license copies verified.

## Stage 2 mapper fixes

Base: `478f061`; environment: macOS, default Rust features, offline Cargo.
The initial `cargo test -p rados --test crush --offline` reproduced all 12
failures before edits. No golden test, map or expected transcript was changed.

Corrections in [bucket.rs](../../rados/src/crush/bucket.rs) and
[mapper.rs](../../rados/src/crush/mapper.rs):

- STRAW keeps the first item on equal draws, matching Ceph. This alone made
  all 10,240 INDEP vectors pass; the other 11 scenarios still failed.
- FIRSTN always advances the retry input. Local retries retain the current
  bucket; descent retries return to the initial bucket. Exhaustive fallback
  uses Ceph's permutation selection, also shared with uniform buckets.
- Chooseleaf keeps selected failure domains separately from selected OSDs,
  checks collisions at the correct level, and honors `chooseleaf_descend_once`.
- Recursive selection uses the output position, `chooseleaf_vary_r` shifted
  parent input and `chooseleaf_stable` replica start as in Ceph. Intermediate
  FIRSTN results respect the caller's output capacity.

Algorithm references (the relevant functions match between these releases):
[v17.2.7 mapper.c](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/crush/mapper.c#L51),
[v20.2.4 mapper.c](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/crush/mapper.c#L54).

Validation:

- `cargo test -p rados --test crush --offline`: **12 passed**, 0 failed,
  0 ignored; all **87,040 vectors** and all result-size histograms checked.
- `cargo test --workspace --lib --offline`: **456 passed**, 0 failed,
  1 ignored in `rados`; the macros crate has 0 unit tests. The sandbox first
  blocked two existing loopback TCP tests; rerunning outside the sandbox
  passed. No live Ceph cluster was used.
- `cargo fmt --all -- --check` and `git diff --check`: passed.
- `cargo clippy --workspace --all-targets --all-features --offline -- --no-deps -D warnings`:
  blocked by the same two pre-existing unused constants documented in Stage 1.
- `cargo clippy -p rados --lib --test crush --offline -- -D warnings -A dead-code`:
  passed. No repository lint configuration was weakened.

Scope limits: these maps contain STRAW/STRAW2 buckets; they do not establish
complete uniform/list/tree coverage. Permutation selection currently rebuilds
its local work array on each call; cache it only if profiling warrants it.
The INDEP scenario selects device type directly; recursive INDEP, MSR, choose
arguments and per-rule retry-setting opcodes still need their listed tests
and fixes. No upstream executable differential or live-cluster gate was run.

## Stage 3 Tentacle functional ports

Base: `81d34c8`. This test-only stage adds
[`functional.rs`](../../rados/tests/crush/functional.rs), covering all seven `IndepTest` and seven
`FirstnTest` scenarios from Tentacle with separate NORMAL and MSR executions:
**28 ported variants**. Coverage: Ported; Readiness: Ready now; Review:
Pending. Verification is recorded per variant below.

The setup now reproduces `build_indep_map` and `build_firstn_map`: optimal
tunables, a STRAW root, STRAW2 racks and hosts, equal fixed-point weights,
the original rule steps and the bucket allocation order produced by
`CrushWrapper::insert_item`. The Rust adaptation constructs that map directly
because this client has no CRUSH editor. Assertions retain the original seed
ranges, requested result sizes, holes, uniqueness, positional stability,
movement bounds and retry overrides. Upstream supplies behavioral assertions,
not exact output vectors, for these cases.

Audit correction: Stage 3 initially used STRAW for every bucket and omitted
the explicit non-NONE replacement assertions in two INDEP scenarios.
Stage 5 corrects both issues. The historical Stage 3/4 results below describe
the original test setup; only Stage 5 verifies the corrected port.

`cargo test -p rados --test crush --offline --no-fail-fast` completed all 40
tests: the existing 12 goldens and 13 functional variants passed; 15 functional
variants failed. No mapper code or expectation was changed.

| Scenario | NORMAL | MSR |
| --- | --- | --- |
| `IndepTest.toosmall` | Passing | Failing: result holes/width lost |
| `IndepTest.basic` | Passing | Failing: requested width 5 becomes 3 |
| `IndepTest.single_out_first` | Failing: later position changes | Failing: requested width 5 becomes 3 |
| `IndepTest.single_out_last` | Failing: earlier position changes | Failing: requested width 5 becomes 3 |
| `IndepTest.out_alt` | Passing | Failing: requested width 9 becomes 3 |
| `IndepTest.out_contig` | Failing: required hole is removed | Failing: requested width 7 becomes 2 |
| `IndepTest.out_progressive` | Failing: movement bound exceeded | Failing: movement bound exceeded after result collapse |
| `FirstnTest.basic` | Passing | Passing |
| `FirstnTest.toosmall` | Passing | Failing: expected 3 results, got 1 |
| `FirstnTest.single_out_first` | Passing | Failing: suffix does not shift as required |
| `FirstnTest.single_out_last` | Passing | Passing |
| `FirstnTest.out_alt` | Passing | Failing: requested width 9 becomes 3 |
| `FirstnTest.out_contig` | Passing | Failing: expected 6 results, got 2 |
| `FirstnTest.out_progressive` | Passing | Passing |

These failures are the baseline for a separate mapper-fix stage. In
particular, the MSR rule currently executes each `ChooseMsr` step against the
final result width and filters holes before `Emit`; the failing tests show the
observable consequences without prescribing the fix.

## Stage 4 functional mapper fixes

Base: `f0717e5`. The 28 Stage 3 tests and their expectations are unchanged.
The implementation changes are intentionally left uncommitted for review.

- INDEP now keeps failure-domain and leaf results in separate positional
  arrays, retains the original replica index during recursive leaf selection,
  uses Ceph's uniform-bucket retry stride, and applies the rule's separate
  chooseleaf retry budget.
- MSR rules now execute complete `TAKE -> CHOOSE_MSR* -> EMIT` blocks. Each
  choose level retains its collision workspace; selection uses the same
  stride ranges, retry value, local collision attempts, failed-descent undo,
  and FIRSTN-versus-INDEP emission order as Tentacle
  `crush_msr_do_rule`/`crush_msr_choose`.

Validation on the uncommitted review tree:

- `cargo test -p rados --test crush --offline --no-fail-fast`: **40 passed**,
  0 failed, 0 ignored. This includes all 28 functional variants and the 12
  unchanged golden scenarios.
- `cargo test -p rados --lib crush:: --offline`: **26 passed**, 0 failed,
  1 pre-existing corpus test ignored.
- `cargo test --workspace --lib --offline`: **456 passed**, 0 failed,
  1 pre-existing corpus test ignored; the macros crate has no unit tests.

Coverage remains limited to the imported scenarios. The four dedicated MSR
topologies, multi-root MSR, unequal fanout and malformed-rule behavior remain
listed below and are not established by these results.

## Stage 5 source-audit regressions

Base: `f0717e5` plus the uncommitted Stage 4 changes. All changes remain
uncommitted for review. This stage resolves the eight source-audit findings;
it does not claim complete mapper parity.

Corrections in [mapper.rs](../../rados/src/crush/mapper.rs):

- Conventional INDEP skips devices and unresolved work positions before
  invoking the bucket chooser, matching the FIRSTN executor's guard.
- Conventional rules skip CHOOSE_MSR instead of panicking. FIRSTN honors
  SetChooseLeafTries before falling back to chooseleaf_descend_once.
- EMIT clears the working set. An invalid MSR block clears the entire
  returned vector, including output from an earlier valid block. The device
  TAKE followed by CHOOSE_MSR case also returns an empty vector.
- MSR rejects negative fanouts before casting or allocating the undo vector,
  returning `CrushError::InvalidMsrFanout`. This is a Rust safety contract:
  the C mapper does not safely validate these values. Internal workspace
  checks return typed errors in place of the Stage 4 `expect` calls.
- [functional.rs](../../rados/tests/crush/functional.rs) uses STRAW only for the explicitly created
  root and STRAW2 for hosts/racks, as `insert_item` does with optimal tunables.
  Both single-OSD INDEP cases now assert that the replacement is not NONE.
  The corrected 28 variants passed before any Stage 5 mapper changes.

Setup references:
[build_indep_map](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/crush.cc#L71),
[insert_item](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/crush/CrushWrapper.cc#L1165),
[default bucket algorithm](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/crush/CrushWrapper.h#L376).

The seven new tests in [regressions.rs](../../rados/tests/crush/regressions.rs) are local differential
cases, not named upstream ports. Each has an adjacent source reference.
Their common status is **Coverage: Ported (local scenarios); Readiness:
Ready now; Verification: Passing; Review: Pending**. They do not increase
the upstream-port totals in the inventory.

| Rust test | Variants / asserted behavior | Before fixes |
| --- | --- | --- |
| `chained_indep_skips_unresolved_domains` | Three requested hosts, two available; subsequent INDEP over surviving hosts; all ordered vectors including holes. | Failing: InvalidBucketId(NONE). |
| `firstn_honors_explicit_leaf_retries` | Explicit leaf tries 10, map total retries 0, descend_once 1; all out masks. | Failing: mask 1, x=1 expected [1], got []. |
| `conventional_rules_skip_choose_msr` | Replicated and Erasure; unknown opcode leaves the working root unchanged. | Failing: panic. |
| `malformed_msr_blocks_discard_output` | MSR FIRSTN/INDEP; conventional CHOOSE, missing TAKE/EMIT, invalid second block, device TAKE with CHOOSE. | Failing: prior output or NONE vector retained. |
| `emit_clears_working_set` | TAKE device, EMIT, EMIT; output [0] exactly once. | Failing: [0,0]. |
| `msr_truncated_fanout_matches_ceph_for_every_out_mask` | MSR FIRSTN/INDEP; fanout 2 hosts x 2 OSDs, result limit 3, all 16 out masks and 100 seeds. | Already passing: 3,200 vectors. |
| `msr_rejects_negative_fanout` | Rust contract; both MSR types, -1 and i32::MIN, outer and nested CHOOSE_MSR. | Failing: capacity-overflow panic. |

Reference search, in both pinned releases:

```sh
git -C ../ceph grep -n -E 'SetChooseLeafTries|set_chooseleaf_tries|CHOOSE_MSR|choosemsr|CRUSH_ITEM_NONE' <commit> -- src/test/crush src/test/cli/crushtool src/test/osd src/test/librados src/test/neorados qa/workunits/crush
```

The search found the existing functional tests and CLI retry settings, but
no named counterpart for these exact malformed-rule or chained-INDEP
regressions, and no safe-negative-MSR-fanout test. Expected results therefore
come from executing unmodified pinned C sources. The negative-fanout test
is labeled `Rust contract; no upstream analogue found`.

[Fixture provenance and the tested regeneration command](../../rados/tests/crush/reference/README.md#generated-mapper-regressions)
include the C input generator, source revisions, compiler, SHA256 and all
inputs. No Ceph tools or checkout are needed to run the Rust tests.
The C comparison established identical output for 6,400 conventional-rule
vectors in Quincy/Tentacle. Another 4,800 stored vectors cover Tentacle MSR;
the four additional malformed MSR inputs returned empty for all 6,400
seed/mask combinations. Rust tests compare full ordered vectors and validate
the stored fixture's complete record sequence and lengths.

Validation on macOS, default Rust features:

- `cargo test -p rados --test crush regressions:: --offline`: before fixes,
  **1 passed / 6 failed**; afterwards **7 passed**, 0 ignored.
- `cargo test -p rados --test crush --offline`: **47 passed**, 0 ignored.
- `cargo test --workspace --lib --offline`: **456 passed**, 1 pre-existing
  corpus test ignored. Two existing loopback TCP tests needed execution
  outside the sandbox; their sandbox failures were PermissionDenied.
- The README regeneration command rebuilt both C oracles and reproduced
  the checked-in fixture and the cross-release comparisons.
- `cargo fmt --all -- --check`, `git diff --check`, fixture SHA256 checks
  and local documentation-link checks passed.
- `cargo clippy --workspace --all-targets --all-features --offline -- --no-deps -D warnings`
  remains blocked by the two existing unused `OSD_STAT_INTERFACES_*`
  constants in `osdclient/pgmap_types.rs`.
- `cargo clippy -p rados --lib --test crush --offline -- -D warnings -A dead-code`
  passed. No repository lint settings were changed.

Remaining scope: the four dedicated upstream MSR topologies and multi-root
tests are still not ported. These tests do not establish complete unequal
fanout, bucket-algorithm, choose-argument, per-rule local-retry override, or
malformed-map coverage. No live-cluster placement check was run.

## Current Rust coverage

`cargo test -p rados --lib crush:: --offline` was run at the baseline: **26 passed, 0 failed, 1 ignored, 430 filtered out**. No Ceph gtest, crushtool differential run, or live-cluster gate was run.

| Existing Rust test | What it establishes / gap |
| --- | --- |
| [rados/src/crush/bucket.rs::test_straw2_choose](../../rados/src/crush/bucket.rs#L265) | One equal-weight bucket; valid range and repeatability only. |
| [rados/src/crush/bucket.rs::test_uniform_choose](../../rados/src/crush/bucket.rs#L292) | Valid range only; no complete uniform-bucket Ceph mapping oracle. |
| [rados/src/crush/bucket.rs::test_bucket_choose](../../rados/src/crush/bucket.rs#L311) | Valid device only. |
| [rados/src/crush/bucket.rs::test_crush_ln](../../rados/src/crush/bucket.rs#L330) | Two-point monotonicity, not bit-for-bit integer logarithm parity. |
| [rados/src/crush/crush_ln_table.rs::test_rh_lh_table_approximate](../../rados/src/crush/crush_ln_table.rs#L547) | Approximate table sanity with broad tolerance. |
| [rados/src/crush/crush_ln_table.rs::test_table_sizes](../../rados/src/crush/crush_ln_table.rs#L563) | Table lengths only. |
| [rados/src/crush/crush_ln_table.rs::test_table_values_are_from_ceph](../../rados/src/crush/crush_ln_table.rs#L573) | Selected constants; no complete mapper comparison. |
| [rados/src/crush/crush_ln_table.rs::test_tables_are_monotonic](../../rados/src/crush/crush_ln_table.rs#L598) | Table shape only. |
| [rados/src/crush/decode.rs::test_decode_crushmap_corpus](../../rados/src/crush/decode.rs#L357) | Ignored; hard-coded external corpus path; cannot validate CI decode parity. |
| [rados/src/crush/decode.rs::test_device_class_methods](../../rados/src/crush/decode.rs#L396) | Synthetic class-name/ID lookups, not decoded shadow-tree mapping. |
| [rados/src/crush/hash.rs::test_crush_hash32_2](../../rados/src/crush/hash.rs#L237) | One exact hash vector (`10, 2 -> 1838530675`); no pinned upstream test attribution. |
| [rados/src/crush/hash.rs::test_ceph_str_hash_rjenkins](../../rados/src/crush/hash.rs#L248) | Nonzero/determinism/different-input checks; no Ceph expected string hashes. |
| [rados/src/crush/mapper.rs::test_is_out](../../rados/src/crush/mapper.rs#L745) | Fully in/out and invalid IDs; no partial-weight oracle. |
| [rados/src/crush/mapper.rs::test_crush_do_rule_simple](../../rados/src/crush/mapper.rs#L761) | One flat bucket, one replica; valid device only. |
| [rados/src/crush/mapper.rs::test_crush_choose_firstn](../../rados/src/crush/mapper.rs#L819) | Now asserts two distinct items after the helper signature change; this local unit scenario has no Ceph expected vector. |
| [rados/src/crush/mapper.rs::test_crush_choose_indep](../../rados/src/crush/mapper.rs#L852) | Flat bucket, three filled/distinct positions; no Ceph vector. |
| [rados/src/crush/mapper.rs::test_crush_choose_indep_stable_positions](../../rados/src/crush/mapper.rs#L890) | Repeats the same call; never changes availability. |
| [rados/src/crush/mapper.rs::test_crush_choose_indep_with_out_device](../../rados/src/crush/mapper.rs#L923) | All weights remain fully in; no device is actually marked out. |
| [rados/src/crush/mapper.rs::test_crush_do_rule_indep](../../rados/src/crush/mapper.rs#L969) | Count and uniqueness in a flat map; no ordered Ceph result. |
| [rados/src/crush/mapper.rs::test_crush_do_rule_chooseleaf_indep](../../rados/src/crush/mapper.rs#L1044) | Flat device-level selection; no multi-level failure-domain assertion. |
| [rados/src/crush/placement.rs::test_object_locator](../../rados/src/crush/placement.rs#L411) | Rust locator construction. |
| [rados/src/crush/placement.rs::test_pg_id](../../rados/src/crush/placement.rs#L427) | Rust PG identity construction. |
| [rados/src/crush/placement.rs::test_object_to_pg](../../rados/src/crush/placement.rs#L435) | Deterministic synthetic object hashing; no upstream oracle. |
| [rados/src/crush/placement.rs::test_object_to_pg_with_namespace](../../rados/src/crush/placement.rs#L452) | Only asserts equal pool IDs; does not assert a namespace-dependent hash or PG seed. |
| [rados/src/crush/placement.rs::test_pg_to_osds](../../rados/src/crush/placement.rs#L465) | Synthetic map, count/range checks. |
| [rados/src/crush/placement.rs::test_object_to_osds](../../rados/src/crush/placement.rs#L531) | Synthetic end-to-end placement/repeatability. |
| [rados/src/crush/placement.rs::test_pg_distribution](../../rados/src/crush/placement.rs#L596) | Distribution sanity; does not prove ordered placement parity. |

Related checks are not included in the 27-test total:

- [OSDMap/CRUSH integration](../../rados/tests/osdclient_osdmap_crush_integration_test.rs)
  returns successfully when its external corpus is absent. Even with input,
  the final assertion only requires some OSDMaps to decode, not a nonzero
  number of decoded CRUSH maps or matching mappings. Not run in this audit.
- [Object placement integration](../../rados/tests/osdclient_object_placement_test.rs)
  has external-corpus skip paths. It is not an upstream mapping oracle.
  Not run in this audit.
- [Ordering example](../../rados/examples/crush_test_crush_ordering.rs) reads
  `/tmp/crushmap` and prints failure without failing the process. It is not a
  Cargo assertion test and its input is not checked in. Not run.
- [OSDMap unit tests](../../rados/src/osdclient/osdmap.rs) cover primary affinity,
  temp/upmap overrides and EC shard transformations with local scenarios.
  They are useful overlap, not the full upstream OSDMap scenarios below;
  they were not run by the `crush::` filter.

Implementation evidence requiring stronger tests:
[Uniform selection](../../rados/src/crush/bucket.rs#L139) now shares permutation
selection with FIRSTN fallback but still lacks a full uniform mapping oracle; [decode](../../rados/src/crush/decode.rs#L123) discards choose arguments;
[rule execution](../../rados/src/crush/mapper.rs#L79) ignores three retry-setting
opcodes and removes holes after `ChooseMsr`. These are inspected code paths,
not failures reproduced by a differential test in this audit.

## Prerequisites and disposition reasons

| Code | Missing capability / scope boundary | Concrete next action and risk |
| --- | --- | --- |
| F | Reference-generated binary fixture | Build or obtain `crushtool`/a small fixture producer at **each pinned commit**; run the original setup, preserve the original map, check in bytes and SHA256/provenance. No `crushtool` was found on PATH or at `../ceph/build/bin/crushtool`. Existing upstream binary maps do not have this blocker. Without the fixture, complex hierarchy/legacy straw preparation can accidentally change the regression. |
| Q | Quincy test-only rule type `123` | The five Quincy INDEP tests construct type `123`, which Rust's closed `RuleType` cannot represent. Preserve it with an appropriate raw-rule representation, or explicitly review a mapping-only adaptation to `Erasure`: Quincy `crush_do_rule` selects classic behavior from opcodes, not this tag. Do not silently rewrite encoded input or count that as raw-type decode coverage. |
| L | Hierarchy/location query API | Add the exact missing parent/location/distance query, preserving ordered type levels, missing items and multipath minima. Current `CrushMap` stores names/buckets but lacks these methods. Until then locality semantics remain untested. |
| A | Choose-argument state and selection | Retain decoded per-index/per-bucket IDs and positional weight sets, expose the relevant selection index to mapping, and import both feature-enabled and legacy-fallback encoded fixtures. Current decode discards this information; successful ordinary straw2 selection is not replacement coverage. |
| R | Randomized C reference setup | Capture the reference runner's `rand()` state/result and the changed weight for `straw2_reweight`; upstream specifies no explicit seed. Preserve one million inputs and the original arithmetic. A separately documented exhaustive test of all ten `rand() % 10` outcomes may supplement the original; do not silently replace its setup with Rust RNG. |
| D | Additional read-only map queries | Implement the named metadata query, not a test-local reimplementation; use decoded or faithfully assembled original setup. Fields alone do not test a missing production query. |
| P | Full OSDMap reference setup / interface | Capture the original full/incremental maps or transcribe their complete pool/state/CRUSH setup, then expose any missing mapping result queried by that case. Test raw/up/acting/primary/EC semantics together. A standalone CRUSH vector does not cover this pipeline. |
| C | Reference live environment | Provide the matching Ceph MON/MGR/OSDs and client tools, replay the original sequence, capture maps and expected client results. A captured offline subset can then run hermetically; it does not validate server mutations or live I/O. No live gate has been run. |
| E | Editor/formatter API outside the native client | Preserve the named case as outside client scope, with review pending. No full Rust replacement exists. Revisit if a CRUSH editor, compiler, exporter, or compatible CLI is added; captured maps may still supply decode/placement tests. |
| M | Monitor/balancer behavior outside the native client | The client consumes the resulting maps/overrides; it does not generate balancing decisions or maintain monitor state. Retain consumption assertions separately. Revisit full porting if those producer APIs become part of this project; missing producer tests do not excuse client mapping gaps. |

Constructing an equivalent **setup** directly in Rust is allowed when all
bucket IDs, item order, weights, straws, rule steps and tunables are known from
upstream. It must not generate expected results from the Rust mapper. For
the regular Tentacle FIRSTN/INDEP hierarchy, equal nonzero straw weights have
straw length `0x10000`; preserve the original mixed hierarchy and Jewel
optimal tunables instead of using `CrushMap::new()` defaults. Builder setup
assertions become fixture validation; they do not claim a Rust map-editor port.

## Mapper unit tests

Each source link includes the original setup via the same file. Tentacle `IndepTest`/`FirstnTest` helpers and instantiations must be ported with both NORMAL and MSR rule variants. Quincy has no MSR tests. The four shared STRAW/STRAW2 test bodies retain the same assertions across releases.

| Original test | Variants | Assertions / required input | Readiness |
| --- | --- | --- | --- |
| [v17.2.7/src/test/crush/crush.cc::CRUSHTest.indep_toosmall](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/crush/crush.cc#L115) | Single | Hierarchy 1 rack / 3 hosts / 1 OSD each; x=0..99, request 5. INDEP: exactly two holes and no duplicate devices. FIRSTN: size 3, no holes/duplicates. | Blocked: Q |
| [v17.2.7/src/test/crush/crush.cc::CRUSHTest.indep_basic](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/crush/crush.cc#L134) | Single | Hierarchy 3 racks / 3 hosts / 3 OSDs each; x=0..99. INDEP requests 5 and checks no holes/duplicates; FIRSTN requests 3 and also asserts length 3. | Blocked: Q |
| [v17.2.7/src/test/crush/crush.cc::CRUSHTest.indep_out_alt](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/crush/crush.cc#L153) | Single | Mark OSD IDs 0,2,...,24 out in the 27-OSD hierarchy; x=0..99, request 9. INDEP total tries=100; classic FIRSTN=500. All requested positions are filled and unique. | Blocked: Q |
| [v17.2.7/src/test/crush/crush.cc::CRUSHTest.indep_out_contig](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/crush/crush.cc#L179) | Single | Mark IDs 0..8 out; x=0..99, request 7. INDEP total tries=100: one hole/no duplicates. FIRSTN classic tries=500: length 6/no duplicates. | Blocked: Q |
| [v17.2.7/src/test/crush/crush.cc::CRUSHTest.indep_out_progressive](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/crush/crush.cc#L205) | Single | x=1..4; progressively mark IDs 0..26 out, request 7. INDEP: duplicates=0, moved<=1, changed<=3. FIRSTN: duplicates=0, newly selected items<=3. Preserve retry settings and every transition. | Blocked: Q |
| [v17.2.7/src/test/crush/crush.cc::CRUSHTest.straw_zero](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/crush/crush.cc#L268) | Single | Two STRAW roots with [4,3,2,1,0] versus [4,3,2,1] fixed-point weights; x=0..9999, one replica; equal selected device and size 1. Needs exact Ceph-generated straw lengths (F). | Blocked: F |
| [v17.2.7/src/test/crush/crush.cc::CRUSHTest.straw_same](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/crush/crush.cc#L322) | Single | Two ten-item STRAW roots: paired stair weights versus odd-index +100 perturbations; 100,000 mappings, size 1 each, differing fraction <0.001. Preserve both straw preparations (F). | Blocked: F |
| [v17.2.7/src/test/crush/crush.cc::CRUSHTest.straw2_stddev](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/crush/crush.cc#L516) | Single | Diagnostic: 15 weights with multipliers 1,1.25,1.5,1.75; one million x values per multiplier. Prints standard deviation but has no distribution acceptance assertion. Ready to reproduce; not strong parity evidence. | Ready now |
| [v17.2.7/src/test/crush/crush.cc::CRUSHTest.straw2_reweight](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/crush/crush.cc#L533) | Single | Uses the first 15 entries of the 17-element weight initializer; item 1 becomes weight/10*(rand()%10). One million mappings; movement must only be to/from item 1 and both results have length 1. Preserve RNG realization (R); do not drop the last high-weight item actually used. | Blocked: R |
| [v20.2.4/src/test/crush/crush.cc::IndepTest.toosmall](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/crush.cc#L152) | NORMAL; MSR | Hierarchy 1 rack / 3 hosts / 1 OSD each; x=0..99, request 5. INDEP: exactly two holes and no duplicate devices. FIRSTN: size 3, no holes/duplicates. | Ready now |
| [v20.2.4/src/test/crush/crush.cc::IndepTest.basic](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/crush.cc#L171) | NORMAL; MSR | Hierarchy 3 racks / 3 hosts / 3 OSDs each; x=0..99. INDEP requests 5 and checks no holes/duplicates; FIRSTN requests 3 and also asserts length 3. | Ready now |
| [v20.2.4/src/test/crush/crush.cc::IndepTest.single_out_first](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/crush.cc#L190) | NORMAL; MSR | x=0..999; mark the first mapped OSD out. INDEP replaces that position and preserves every other position. FIRSTN removes it with length/uniqueness checks; only MSR requires the original suffix to shift left. | Ready now |
| [v20.2.4/src/test/crush/crush.cc::IndepTest.single_out_last](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/crush.cc#L230) | NORMAL; MSR | x=0..999; mark the last mapped OSD out, preserve all earlier positions and replace the last without duplicates. | Ready now |
| [v20.2.4/src/test/crush/crush.cc::IndepTest.out_alt](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/crush.cc#L271) | NORMAL; MSR | Mark OSD IDs 0,2,...,24 out in the 27-OSD hierarchy; x=0..99, request 9. INDEP total tries=100; classic FIRSTN=500. All requested positions are filled and unique. | Ready now |
| [v20.2.4/src/test/crush/crush.cc::IndepTest.out_contig](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/crush.cc#L297) | NORMAL; MSR | Mark IDs 0..8 out; x=0..99, request 7. INDEP total tries=100: one hole/no duplicates. FIRSTN classic tries=500: length 6/no duplicates. | Ready now |
| [v20.2.4/src/test/crush/crush.cc::IndepTest.out_progressive](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/crush.cc#L322) | NORMAL; MSR | x=1..4; progressively mark IDs 0..26 out, request 7. INDEP: duplicates=0, moved<=1, changed<=3. FIRSTN: duplicates=0, newly selected items<=3. Preserve retry settings and every transition. | Ready now |
| [v20.2.4/src/test/crush/crush.cc::FirstnTest.basic](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/crush.cc#L485) | NORMAL; MSR | Hierarchy 3 racks / 3 hosts / 3 OSDs each; x=0..99. INDEP requests 5 and checks no holes/duplicates; FIRSTN requests 3 and also asserts length 3. | Ready now |
| [v20.2.4/src/test/crush/crush.cc::FirstnTest.toosmall](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/crush.cc#L502) | NORMAL; MSR | Hierarchy 1 rack / 3 hosts / 1 OSD each; x=0..99, request 5. INDEP: exactly two holes and no duplicate devices. FIRSTN: size 3, no holes/duplicates. | Ready now |
| [v20.2.4/src/test/crush/crush.cc::FirstnTest.single_out_first](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/crush.cc#L519) | NORMAL; MSR | x=0..999; mark the first mapped OSD out. INDEP replaces that position and preserves every other position. FIRSTN removes it with length/uniqueness checks; only MSR requires the original suffix to shift left. | Ready now |
| [v20.2.4/src/test/crush/crush.cc::FirstnTest.single_out_last](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/crush.cc#L560) | NORMAL; MSR | x=0..999; mark the last mapped OSD out, preserve all earlier positions and replace the last without duplicates. | Ready now |
| [v20.2.4/src/test/crush/crush.cc::FirstnTest.out_alt](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/crush.cc#L598) | NORMAL; MSR | Mark OSD IDs 0,2,...,24 out in the 27-OSD hierarchy; x=0..99, request 9. INDEP total tries=100; classic FIRSTN=500. All requested positions are filled and unique. | Ready now |
| [v20.2.4/src/test/crush/crush.cc::FirstnTest.out_contig](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/crush.cc#L621) | NORMAL; MSR | Mark IDs 0..8 out; x=0..99, request 7. INDEP total tries=100: one hole/no duplicates. FIRSTN classic tries=500: length 6/no duplicates. | Ready now |
| [v20.2.4/src/test/crush/crush.cc::FirstnTest.out_progressive](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/crush.cc#L644) | NORMAL; MSR | x=1..4; progressively mark IDs 0..26 out, request 7. INDEP: duplicates=0, moved<=1, changed<=3. FIRSTN: duplicates=0, newly selected items<=3. Preserve retry settings and every transition. | Ready now |
| [v20.2.4/src/test/crush/crush.cc::CRUSHTest.straw_zero](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/crush.cc#L709) | Single | Two STRAW roots with [4,3,2,1,0] versus [4,3,2,1] fixed-point weights; x=0..9999, one replica; equal selected device and size 1. Needs exact Ceph-generated straw lengths (F). | Blocked: F |
| [v20.2.4/src/test/crush/crush.cc::CRUSHTest.straw_same](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/crush.cc#L763) | Single | Two ten-item STRAW roots: paired stair weights versus odd-index +100 perturbations; 100,000 mappings, size 1 each, differing fraction <0.001. Preserve both straw preparations (F). | Blocked: F |
| [v20.2.4/src/test/crush/crush.cc::CRUSHTest.straw2_stddev](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/crush.cc#L957) | Single | Diagnostic: 15 weights with multipliers 1,1.25,1.5,1.75; one million x values per multiplier. Prints standard deviation but has no distribution acceptance assertion. Ready to reproduce; not strong parity evidence. | Ready now |
| [v20.2.4/src/test/crush/crush.cc::CRUSHTest.straw2_reweight](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/crush.cc#L974) | Single | Uses the first 15 entries of the 17-element weight initializer; item 1 becomes weight/10*(rand()%10). One million mappings; movement must only be to/from item 1 and both results have length 1. Preserve RNG realization (R); do not drop the last high-weight item actually used. | Blocked: R |
| [v20.2.4/src/test/crush/crush.cc::CRUSHTest.msr_4_host_2_choose_rule](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/crush.cc#L1277) | Single | STRAW2, 4 hosts x 3 OSDs, seed 0; select 3 hosts x 1 OSD. Losing first selected host or OSD preserves mapped count. Preserve original count assertions; add exact vectors separately. | Ready now |
| [v20.2.4/src/test/crush/crush.cc::CRUSHTest.msr_2_host_2_osd](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/crush.cc#L1325) | Single | STRAW2, 3 hosts x 2 OSDs, seed 0; select 2 hosts x 2 OSDs, result limit 3. Losing first selected host remaps positions 0..1 to a new host and preserves position 2. | Ready now |
| [v20.2.4/src/test/crush/crush.cc::CRUSHTest.msr_5_host_8_6_ec_choose](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/crush.cc#L1371) | Single | STRAW2, 5 hosts x 4 OSDs, seed 0; select 4 hosts x 4 OSDs, result limit 14. Losing first selected host remaps positions 0..3 to a new host, preserving 4..13. | Ready now |
| [v20.2.4/src/test/crush/crush.cc::CRUSHTest.msr_multi_root](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/crush.cc#L1417) | Single | Two roots (ssd,hdd), each 4 hosts x 3 OSDs; x=0..999; 8 outputs in 2-host x 2-OSD groups per root. Remove selected OSD positions 1,5 and hosts at positions 2,6 separately; preserve unaffected positions and mapped count. Upstream validates every OSD ID but looks up host/root via out[start] for each group; stronger per-item domain checks must be documented as additional assertions. | Ready now |

## CrushWrapper unit tests

All 24 definitions exist in both releases. The only source change is formatter ownership in `dump_rules`; its assertions are unchanged. Editor exclusions below are per-test proposals with Review: Pending.

| Original test (both releases) | Behavior and remaining client coverage | Readiness / action |
| --- | --- | --- |
| [v17.2.7/src/test/crush/CrushWrapper.cc::CrushWrapperTest.get_immediate_parent](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/crush/CrushWrapper.cc#L55)<br>[v20.2.4/src/test/crush/CrushWrapper.cc::CrushWrapperTest.get_immediate_parent](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/CrushWrapper.cc#L55) | Missing item -> ENOENT; after insertion returns (root,default). Add parent query; static fixture can replace editor setup. | Blocked: L |
| [v17.2.7/src/test/crush/CrushWrapper.cc::CrushWrapperTest.move_bucket](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/crush/CrushWrapper.cc#L89)<br>[v20.2.4/src/test/crush/CrushWrapper.cc::CrushWrapperTest.move_bucket](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/CrushWrapper.cc#L89) | Reject device ID and nonexistent bucket; moving host root0->root1 changes parent. Needs a bucket editor, not client mapping; parent-query subset is retained above. | Outside client scope: E |
| [v17.2.7/src/test/crush/CrushWrapper.cc::CrushWrapperTest.swap_bucket](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/crush/CrushWrapper.cc#L147)<br>[v20.2.4/src/test/crush/CrushWrapper.cc::CrushWrapperTest.swap_bucket](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/CrushWrapper.cc#L147) | Reject parent/child swap; swap names, items and weights while preserving parent references. Requires editor semantics; no replacement coverage. | Outside client scope: E |
| [v17.2.7/src/test/crush/CrushWrapper.cc::CrushWrapperTest.rename_bucket_or_item](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/crush/CrushWrapper.cc#L213)<br>[v20.2.4/src/test/crush/CrushWrapper.cc::CrushWrapperTest.rename_bucket_or_item](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/CrushWrapper.cc#L213) | Invalid/conflicting/missing names, repeated rename, bucket-vs-device errors, stable IDs. Revisit with editor name APIs. | Outside client scope: E |
| [v17.2.7/src/test/crush/CrushWrapper.cc::CrushWrapperTest.check_item_loc](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/crush/CrushWrapper.cc#L280)<br>[v20.2.4/src/test/crush/CrushWrapper.cc::CrushWrapperTest.check_item_loc](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/CrushWrapper.cc#L280) | Empty/missing/wrong-name/wrong-type location fails; valid location returns exact weight 1.0. Add production location-membership/weight query. | Blocked: L |
| [v17.2.7/src/test/crush/CrushWrapper.cc::CrushWrapperTest.update_item](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/crush/CrushWrapper.cc#L345)<br>[v20.2.4/src/test/crush/CrushWrapper.cc::CrushWrapperTest.update_item](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/CrushWrapper.cc#L345) | Invalid names, no-op update, name/weight update and relocation, with exact change counts and membership. Requires map editor. | Outside client scope: E |
| [v17.2.7/src/test/crush/CrushWrapper.cc::CrushWrapperTest.adjust_item_weight](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/crush/CrushWrapper.cc#L439)<br>[v20.2.4/src/test/crush/CrushWrapper.cc::CrushWrapperTest.adjust_item_weight](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/CrushWrapper.cc#L439) | Device appears under host0 and fake: change everywhere versus only one location; return counts and exact weights. Capture resulting maps for client weights; that subset is not a full editor test. | Outside client scope: E |
| [v17.2.7/src/test/crush/CrushWrapper.cc::CrushWrapperTest.adjust_subtree_weight](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/crush/CrushWrapper.cc#L558)<br>[v20.2.4/src/test/crush/CrushWrapper.cc::CrushWrapperTest.adjust_subtree_weight](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/CrushWrapper.cc#L558) | Change two descendant weights; host and root totals update exactly. Client consumes totals; producing/recalculating them requires editor API. | Outside client scope: E |
| [v17.2.7/src/test/crush/CrushWrapper.cc::CrushWrapperTest.insert_item](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/crush/CrushWrapper.cc#L651)<br>[v20.2.4/src/test/crush/CrushWrapper.cc::CrushWrapperTest.insert_item](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/CrushWrapper.cc#L651) | Invalid/duplicate names, implicit bucket creation, wrong types, empty location, duplicate ancestry and cycle rejection. Client decoder boundary tests are additional coverage, not this builder test. | Outside client scope: E |
| [v17.2.7/src/test/crush/CrushWrapper.cc::CrushWrapperTest.remove_item](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/crush/CrushWrapper.cc#L785)<br>[v20.2.4/src/test/crush/CrushWrapper.cc::CrushWrapperTest.remove_item](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/CrushWrapper.cc#L785) | Remove item 6 from 12-device host; location membership becomes false. Requires unlink/remove editor operation. | Outside client scope: E |
| [v17.2.7/src/test/crush/CrushWrapper.cc::CrushWrapperTest.item_bucket_names](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/crush/CrushWrapper.cc#L827)<br>[v20.2.4/src/test/crush/CrushWrapper.cc::CrushWrapperTest.item_bucket_names](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/CrushWrapper.cc#L827) | Reject control-character name; set/get/existence round trip for ID 123. Name mutation/validation absent; decoded name retention needs separate fixture checks. | Outside client scope: E |
| [v17.2.7/src/test/crush/CrushWrapper.cc::CrushWrapperTest.bucket_types](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/crush/CrushWrapper.cc#L839)<br>[v20.2.4/src/test/crush/CrushWrapper.cc::CrushWrapperTest.bucket_types](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/CrushWrapper.cc#L839) | Type ID 123/name NAME; count and both lookup directions. Add production type lookup queries; direct HashMap assertions alone would not port the accessor API. | Blocked: D |
| [v17.2.7/src/test/crush/CrushWrapper.cc::CrushWrapperTest.is_valid_crush_name](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/crush/CrushWrapper.cc#L849)<br>[v20.2.4/src/test/crush/CrushWrapper.cc::CrushWrapperTest.is_valid_crush_name](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/CrushWrapper.cc#L849) | Allowed alphanumeric/-/_ name, empty and control-character rejection. This is the editor name validator; reconsider if user-supplied CRUSH names are accepted. | Outside client scope: E |
| [v17.2.7/src/test/crush/CrushWrapper.cc::CrushWrapperTest.is_valid_crush_loc](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/crush/CrushWrapper.cc#L855)<br>[v20.2.4/src/test/crush/CrushWrapper.cc::CrushWrapperTest.is_valid_crush_loc](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/CrushWrapper.cc#L855) | Empty and valid location accepted; invalid key/value rejected. Revisit with a location-input API; distance/membership tests remain in scope. | Outside client scope: E |
| [v17.2.7/src/test/crush/CrushWrapper.cc::CrushWrapperTest.dump_rules](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/crush/CrushWrapper.cc#L872)<br>[v20.2.4/src/test/crush/CrushWrapper.cc::CrushWrapperTest.dump_rules](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/CrushWrapper.cc#L872) | Empty JSON, XML rule identity/take item, and normalized per-OSD weight map {0:1.0}. Formatter is outside scope; preserve the weight-query subset as blocked D if exposed to clients. | Outside client scope: E |
| [v17.2.7/src/test/crush/CrushWrapper.cc::CrushWrapperTest.distance](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/crush/CrushWrapper.cc#L945)<br>[v20.2.4/src/test/crush/CrushWrapper.cc::CrushWrapperTest.distance](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/CrushWrapper.cc#L942) | Ordered host/rack/root location for OSD 3; distances [3,3,2,1], missing ID ENOENT, multipath host minima. Add full-location and common-ancestor APIs; do not flatten multipath inputs. | Blocked: L |
| [v17.2.7/src/test/crush/CrushWrapper.cc::CrushWrapperTest.choose_args_compat](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/crush/CrushWrapper.cc#L1014)<br>[v20.2.4/src/test/crush/CrushWrapper.cc::CrushWrapperTest.choose_args_compat](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/CrushWrapper.cc#L1011) | Two wire feature masks: retained choose-args weight 666 with canonical weight 12, versus no choose-args and fallback bucket weight 666. Capture both encodings and retain chosen state; Rust does not encode CRUSH maps. | Blocked: A + F |
| [v17.2.7/src/test/crush/CrushWrapper.cc::CrushWrapperTest.remove_root](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/crush/CrushWrapper.cc#L1087)<br>[v20.2.4/src/test/crush/CrushWrapper.cc::CrushWrapperTest.remove_root](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/CrushWrapper.cc#L1084) | Remove default root and both descendant rack names. Requires recursive editor deletion. | Outside client scope: E |
| [v17.2.7/src/test/crush/CrushWrapper.cc::CrushWrapperTest.trim_roots_with_class](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/crush/CrushWrapper.cc#L1119)<br>[v20.2.4/src/test/crush/CrushWrapper.cc::CrushWrapperTest.trim_roots_with_class](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/CrushWrapper.cc#L1116) | Create default~ssd clone, then trim it while retaining default. Client must decode shadow trees; clone deletion is an editor operation. | Outside client scope: E |
| [v17.2.7/src/test/crush/CrushWrapper.cc::CrushWrapperTest.device_class_clone](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/crush/CrushWrapper.cc#L1150)<br>[v20.2.4/src/test/crush/CrushWrapper.cc::CrushWrapperTest.device_class_clone](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/CrushWrapper.cc#L1147) | Clone contains only class members, weight 1 vs root 2; repeated clone reuses ID; invalid bucket/class errors. Import class-filtered mapping fixtures rather than claim a clone API port. | Outside client scope: E |
| [v17.2.7/src/test/crush/CrushWrapper.cc::CrushWrapperTest.split_id_class](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/crush/CrushWrapper.cc#L1197)<br>[v20.2.4/src/test/crush/CrushWrapper.cc::CrushWrapperTest.split_id_class](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/CrushWrapper.cc#L1194) | Shadow bucket resolves to original bucket/class; ordinary bucket yields class -1. Add reverse shadow lookup; setup can use a captured class map. | Blocked: D |
| [v17.2.7/src/test/crush/CrushWrapper.cc::CrushWrapperTest.populate_classes](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/crush/CrushWrapper.cc#L1229)<br>[v20.2.4/src/test/crush/CrushWrapper.cc::CrushWrapperTest.populate_classes](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/CrushWrapper.cc#L1226) | Create shadow names; repeat population preserves class_bucket mapping. Client consumption remains required; generation requires class-tree editor. | Outside client scope: E |
| [v17.2.7/src/test/crush/CrushWrapper.cc::CrushWrapperTest.remove_class_name](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/crush/CrushWrapper.cc#L1253)<br>[v20.2.4/src/test/crush/CrushWrapper.cc::CrushWrapperTest.remove_class_name](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/CrushWrapper.cc#L1250) | Missing/create/remove/repeated-remove return values. Requires class editor; existing Rust class lookup tests are not this test. | Outside client scope: E |
| [v17.2.7/src/test/crush/CrushWrapper.cc::CrushWrapperTest.try_remap_rule](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/crush/CrushWrapper.cc#L1263)<br>[v20.2.4/src/test/crush/CrushWrapper.cc::CrushWrapperTest.try_remap_rule](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/CrushWrapper.cc#L1260) | Balancer replacement with underfull/more_underfull, duplicate avoidance, choose-device/chooseleaf/multilevel/short orig vectors and exact expected outputs. Client applies received upmaps; it does not generate these remaps. | Outside client scope: M |

## crushtool cases

All 37 files exist in both releases; only `choose-args.t` differs, adding
MSR-related JSON fields in Tentacle. All expected vectors in the `test-map-*`
files are identical between the two pinned releases. Preserve both provenance
records even when one fixture can be shared.

`cmd-NN` below is a **locally assigned identifier**, numbered by `$` command
order within its file (including setup/cleanup; not an upstream test name).
The source columns link every command block separately for each release.
Coverage/Rust/Verification defaults above apply to all these entries. A file
readiness statement applies to its behavioral commands; setup and cleanup
commands are dependencies, not additional assertion tests. For mixed cases,
the client subset and the CLI/editor exclusion are explicitly distinguished.

### add-bucket.t

**Readiness:** Outside client scope: E. Add/move buckets and device, then exact decompiled tree; input simple.template. Client subset: decoded final topology, blocked F until captured.

| Local case / original command | Source blocks |
| --- | --- |
| `cmd-01`: ``crushtool -i "$TESTDIR/simple.template" --add-bucket host0 host --loc cluster cluster0 -o map0 > /dev/null`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/add-bucket.t#L1 "v17.2.7/src/test/cli/crushtool/add-bucket.t::cmd-01") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/add-bucket.t#L1 "v20.2.4/src/test/cli/crushtool/add-bucket.t::cmd-01") |
| `cmd-02`: ``crushtool -i map0 --add-bucket host1 host -o map1 > /dev/null`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/add-bucket.t#L2 "v17.2.7/src/test/cli/crushtool/add-bucket.t::cmd-02") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/add-bucket.t#L2 "v20.2.4/src/test/cli/crushtool/add-bucket.t::cmd-02") |
| `cmd-03`: ``crushtool -i map1 --move host1 --loc cluster cluster0 -o map2 > /dev/null`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/add-bucket.t#L3 "v17.2.7/src/test/cli/crushtool/add-bucket.t::cmd-03") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/add-bucket.t#L3 "v20.2.4/src/test/cli/crushtool/add-bucket.t::cmd-03") |
| `cmd-04`: ``crushtool -i map2 --add-item 1 1.0 device1 --loc cluster cluster0 -o map3 > /dev/null`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/add-bucket.t#L4 "v17.2.7/src/test/cli/crushtool/add-bucket.t::cmd-04") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/add-bucket.t#L4 "v20.2.4/src/test/cli/crushtool/add-bucket.t::cmd-04") |
| `cmd-05`: ``crushtool -i map3 --move device1 --loc host host0 -o map4 > /dev/null`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/add-bucket.t#L5 "v17.2.7/src/test/cli/crushtool/add-bucket.t::cmd-05") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/add-bucket.t#L5 "v20.2.4/src/test/cli/crushtool/add-bucket.t::cmd-05") |
| `cmd-06`: ``crushtool -d map4`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/add-bucket.t#L6 "v17.2.7/src/test/cli/crushtool/add-bucket.t::cmd-06") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/add-bucket.t#L6 "v20.2.4/src/test/cli/crushtool/add-bucket.t::cmd-06") |

### add-item-in-tree.t

**Readiness:** Outside client scope: E. Add eight devices to TREE bucket, compare tree.template.final. Client TREE selection must be covered separately; editor insertion is not required.

| Local case / original command | Source blocks |
| --- | --- |
| `cmd-01`: ``crushtool -i "$TESTDIR/tree.template" --add-item 0 1.0 device0 --loc host host0 --loc cluster cluster0 -o one > /dev/null`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/add-item-in-tree.t#L1 "v17.2.7/src/test/cli/crushtool/add-item-in-tree.t::cmd-01") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/add-item-in-tree.t#L1 "v20.2.4/src/test/cli/crushtool/add-item-in-tree.t::cmd-01") |
| `cmd-02`: ``crushtool -i one   --add-item 1 1.0 device1 --loc host host0 --loc cluster cluster0 -o two   > /dev/null`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/add-item-in-tree.t#L2 "v17.2.7/src/test/cli/crushtool/add-item-in-tree.t::cmd-02") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/add-item-in-tree.t#L2 "v20.2.4/src/test/cli/crushtool/add-item-in-tree.t::cmd-02") |
| `cmd-03`: ``crushtool -i two   --add-item 2 1.0 device2 --loc host host0 --loc cluster cluster0 -o tree  > /dev/null`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/add-item-in-tree.t#L3 "v17.2.7/src/test/cli/crushtool/add-item-in-tree.t::cmd-03") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/add-item-in-tree.t#L3 "v20.2.4/src/test/cli/crushtool/add-item-in-tree.t::cmd-03") |
| `cmd-04`: ``crushtool -i tree  --add-item 3 1.0 device3 --loc host host0 --loc cluster cluster0 -o four  > /dev/null`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/add-item-in-tree.t#L4 "v17.2.7/src/test/cli/crushtool/add-item-in-tree.t::cmd-04") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/add-item-in-tree.t#L4 "v20.2.4/src/test/cli/crushtool/add-item-in-tree.t::cmd-04") |
| `cmd-05`: ``crushtool -i four  --add-item 4 1.0 device4 --loc host host0 --loc cluster cluster0 -o five  > /dev/null`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/add-item-in-tree.t#L5 "v17.2.7/src/test/cli/crushtool/add-item-in-tree.t::cmd-05") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/add-item-in-tree.t#L5 "v20.2.4/src/test/cli/crushtool/add-item-in-tree.t::cmd-05") |
| `cmd-06`: ``crushtool -i five  --add-item 5 1.0 device5 --loc host host0 --loc cluster cluster0 -o six   > /dev/null`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/add-item-in-tree.t#L6 "v17.2.7/src/test/cli/crushtool/add-item-in-tree.t::cmd-06") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/add-item-in-tree.t#L6 "v20.2.4/src/test/cli/crushtool/add-item-in-tree.t::cmd-06") |
| `cmd-07`: ``crushtool -i six   --add-item 6 1.0 device6 --loc host host0 --loc cluster cluster0 -o seven > /dev/null`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/add-item-in-tree.t#L7 "v17.2.7/src/test/cli/crushtool/add-item-in-tree.t::cmd-07") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/add-item-in-tree.t#L7 "v20.2.4/src/test/cli/crushtool/add-item-in-tree.t::cmd-07") |
| `cmd-08`: ``crushtool -i seven --add-item 7 1.0 device7 --loc host host0 --loc cluster cluster0 -o eight > /dev/null`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/add-item-in-tree.t#L8 "v17.2.7/src/test/cli/crushtool/add-item-in-tree.t::cmd-08") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/add-item-in-tree.t#L8 "v20.2.4/src/test/cli/crushtool/add-item-in-tree.t::cmd-08") |
| `cmd-09`: ``crushtool -d eight -o final`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/add-item-in-tree.t#L9 "v17.2.7/src/test/cli/crushtool/add-item-in-tree.t::cmd-09") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/add-item-in-tree.t#L9 "v20.2.4/src/test/cli/crushtool/add-item-in-tree.t::cmd-09") |
| `cmd-10`: ``diff final "$TESTDIR/tree.template.final"`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/add-item-in-tree.t#L10 "v17.2.7/src/test/cli/crushtool/add-item-in-tree.t::cmd-10") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/add-item-in-tree.t#L10 "v20.2.4/src/test/cli/crushtool/add-item-in-tree.t::cmd-10") |

### add-item.t

**Readiness:** Outside client scope: E. Add devices/rule, remove rule, reject duplicate insertion, remove/update/move item, repeat update, show location; exact template.two/four/five output. Location subset blocked L+F.

| Local case / original command | Source blocks |
| --- | --- |
| `cmd-01`: ``crushtool -i "$TESTDIR/simple.template" --add-item 0 1.0 device0 --loc host host0 --loc cluster cluster0 -o one > /dev/null`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/add-item.t#L1 "v17.2.7/src/test/cli/crushtool/add-item.t::cmd-01") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/add-item.t#L1 "v20.2.4/src/test/cli/crushtool/add-item.t::cmd-01") |
| `cmd-02`: ``crushtool -i one --add-item 1 1.0 device1 --loc host host0 --loc cluster cluster0 -o two > /dev/null`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/add-item.t#L2 "v17.2.7/src/test/cli/crushtool/add-item.t::cmd-02") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/add-item.t#L2 "v20.2.4/src/test/cli/crushtool/add-item.t::cmd-02") |
| `cmd-03`: ``crushtool -i two --create-simple-rule simple-rule cluster0 host firstn -o two > /dev/null`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/add-item.t#L3 "v17.2.7/src/test/cli/crushtool/add-item.t::cmd-03") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/add-item.t#L3 "v20.2.4/src/test/cli/crushtool/add-item.t::cmd-03") |
| `cmd-04`: ``crushtool -d two`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/add-item.t#L4 "v17.2.7/src/test/cli/crushtool/add-item.t::cmd-04") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/add-item.t#L4 "v20.2.4/src/test/cli/crushtool/add-item.t::cmd-04") |
| `cmd-05`: ``crushtool -i two --remove-rule simple-rule -o two > /dev/null`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/add-item.t#L64 "v17.2.7/src/test/cli/crushtool/add-item.t::cmd-05") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/add-item.t#L64 "v20.2.4/src/test/cli/crushtool/add-item.t::cmd-05") |
| `cmd-06`: ``crushtool -d two`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/add-item.t#L65 "v17.2.7/src/test/cli/crushtool/add-item.t::cmd-06") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/add-item.t#L65 "v20.2.4/src/test/cli/crushtool/add-item.t::cmd-06") |
| `cmd-07`: ``crushtool -d two -o final`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/add-item.t#L118 "v17.2.7/src/test/cli/crushtool/add-item.t::cmd-07") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/add-item.t#L118 "v20.2.4/src/test/cli/crushtool/add-item.t::cmd-07") |
| `cmd-08`: ``diff final "$TESTDIR/simple.template.two"`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/add-item.t#L119 "v17.2.7/src/test/cli/crushtool/add-item.t::cmd-08") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/add-item.t#L119 "v20.2.4/src/test/cli/crushtool/add-item.t::cmd-08") |
| `cmd-09`: ``crushtool -i two --add-item 1 1.0 device1 --loc host host0 --loc cluster cluster0 -o three 2>/dev/null >/dev/null \|\| echo FAIL`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/add-item.t#L120 "v17.2.7/src/test/cli/crushtool/add-item.t::cmd-09") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/add-item.t#L120 "v20.2.4/src/test/cli/crushtool/add-item.t::cmd-09") |
| `cmd-10`: ``crushtool -i two --remove-item device1 -o four > /dev/null`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/add-item.t#L122 "v17.2.7/src/test/cli/crushtool/add-item.t::cmd-10") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/add-item.t#L122 "v20.2.4/src/test/cli/crushtool/add-item.t::cmd-10") |
| `cmd-11`: ``crushtool -d four -o final`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/add-item.t#L123 "v17.2.7/src/test/cli/crushtool/add-item.t::cmd-11") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/add-item.t#L123 "v20.2.4/src/test/cli/crushtool/add-item.t::cmd-11") |
| `cmd-12`: ``diff final "$TESTDIR/simple.template.four"`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/add-item.t#L124 "v17.2.7/src/test/cli/crushtool/add-item.t::cmd-12") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/add-item.t#L124 "v20.2.4/src/test/cli/crushtool/add-item.t::cmd-12") |
| `cmd-13`: ``crushtool -i two --update-item 1 2.0 osd1 --loc host host1 --loc cluster cluster0 -o five > /dev/null`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/add-item.t#L125 "v17.2.7/src/test/cli/crushtool/add-item.t::cmd-13") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/add-item.t#L125 "v20.2.4/src/test/cli/crushtool/add-item.t::cmd-13") |
| `cmd-14`: ``crushtool -d five -o final`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/add-item.t#L126 "v17.2.7/src/test/cli/crushtool/add-item.t::cmd-14") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/add-item.t#L126 "v20.2.4/src/test/cli/crushtool/add-item.t::cmd-14") |
| `cmd-15`: ``diff final "$TESTDIR/simple.template.five"`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/add-item.t#L127 "v17.2.7/src/test/cli/crushtool/add-item.t::cmd-15") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/add-item.t#L127 "v20.2.4/src/test/cli/crushtool/add-item.t::cmd-15") |
| `cmd-16`: ``crushtool -i five --update-item 1 2.0 osd1 --loc host host1 --loc cluster cluster0 -o six > /dev/null`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/add-item.t#L128 "v17.2.7/src/test/cli/crushtool/add-item.t::cmd-16") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/add-item.t#L128 "v20.2.4/src/test/cli/crushtool/add-item.t::cmd-16") |
| `cmd-17`: ``crushtool -i five --show-location 1`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/add-item.t#L129 "v17.2.7/src/test/cli/crushtool/add-item.t::cmd-17") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/add-item.t#L129 "v20.2.4/src/test/cli/crushtool/add-item.t::cmd-17") |
| `cmd-18`: ``crushtool -d six -o final`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/add-item.t#L132 "v17.2.7/src/test/cli/crushtool/add-item.t::cmd-18") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/add-item.t#L132 "v20.2.4/src/test/cli/crushtool/add-item.t::cmd-18") |
| `cmd-19`: ``diff final "$TESTDIR/simple.template.five"`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/add-item.t#L133 "v17.2.7/src/test/cli/crushtool/add-item.t::cmd-19") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/add-item.t#L133 "v20.2.4/src/test/cli/crushtool/add-item.t::cmd-19") |

### adjust-item-weight.t

**Readiness:** Outside client scope: E. Same device in two hosts; change only one location and compare exact .adj.two/.adj.three maps. Client post-update decode subset blocked F.

| Local case / original command | Source blocks |
| --- | --- |
| `cmd-01`: ``crushtool -i "$TESTDIR/simple.template" --add-item 0 1.0 device0 --loc host host0 --loc cluster cluster0 -o one > /dev/null`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/adjust-item-weight.t#L1 "v17.2.7/src/test/cli/crushtool/adjust-item-weight.t::cmd-01") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/adjust-item-weight.t#L1 "v20.2.4/src/test/cli/crushtool/adjust-item-weight.t::cmd-01") |
| `cmd-02`: ``crushtool -i one --add-item 0 2.0 device0 --loc host fake --loc cluster cluster0 -o two > /dev/null`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/adjust-item-weight.t#L7 "v17.2.7/src/test/cli/crushtool/adjust-item-weight.t::cmd-02") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/adjust-item-weight.t#L7 "v20.2.4/src/test/cli/crushtool/adjust-item-weight.t::cmd-02") |
| `cmd-03`: ``crushtool -d two -o final`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/adjust-item-weight.t#L8 "v17.2.7/src/test/cli/crushtool/adjust-item-weight.t::cmd-03") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/adjust-item-weight.t#L8 "v20.2.4/src/test/cli/crushtool/adjust-item-weight.t::cmd-03") |
| `cmd-04`: ``diff final "$TESTDIR/simple.template.adj.two"`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/adjust-item-weight.t#L9 "v17.2.7/src/test/cli/crushtool/adjust-item-weight.t::cmd-04") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/adjust-item-weight.t#L9 "v20.2.4/src/test/cli/crushtool/adjust-item-weight.t::cmd-04") |
| `cmd-05`: ``crushtool -i two --update-item 0 3.0 device0 --loc host host0 --loc cluster cluster0 -o three > /dev/null`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/adjust-item-weight.t#L15 "v17.2.7/src/test/cli/crushtool/adjust-item-weight.t::cmd-05") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/adjust-item-weight.t#L15 "v20.2.4/src/test/cli/crushtool/adjust-item-weight.t::cmd-05") |
| `cmd-06`: ``crushtool -d three -o final`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/adjust-item-weight.t#L16 "v17.2.7/src/test/cli/crushtool/adjust-item-weight.t::cmd-06") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/adjust-item-weight.t#L16 "v20.2.4/src/test/cli/crushtool/adjust-item-weight.t::cmd-06") |
| `cmd-07`: ``diff final "$TESTDIR/simple.template.adj.three"`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/adjust-item-weight.t#L17 "v17.2.7/src/test/cli/crushtool/adjust-item-weight.t::cmd-07") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/adjust-item-weight.t#L17 "v20.2.4/src/test/cli/crushtool/adjust-item-weight.t::cmd-07") |

### arg-order-checks.t

**Readiness:** Blocked: F (mapping subset); E (CLI ordering). Preserve command order and 25-device STRAW setup; x=1..100, replicas 1..10, straw_calc_version 0 and 1; exact utilization and result-size distributions. Capture both post-reweight maps. Argument processing and printed trees are outside client scope.

| Local case / original command | Source blocks |
| --- | --- |
| `cmd-01`: ``crushtool -d "$TESTDIR/simple.template" --set-straw-calc-version 1 \| head -2`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/arg-order-checks.t#L2 "v17.2.7/src/test/cli/crushtool/arg-order-checks.t::cmd-01") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/arg-order-checks.t#L2 "v20.2.4/src/test/cli/crushtool/arg-order-checks.t::cmd-01") |
| `cmd-02`: ``map="$TESTDIR/foo"`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/arg-order-checks.t#L6 "v17.2.7/src/test/cli/crushtool/arg-order-checks.t::cmd-02") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/arg-order-checks.t#L6 "v20.2.4/src/test/cli/crushtool/arg-order-checks.t::cmd-02") |
| `cmd-03`: ``crushtool --outfn "$map" --build --set-chooseleaf-vary-r 0 --set-chooseleaf-stable 0 --num_osds 25 node straw 5 rack straw 1 root straw 0 --reweight-item osd.2 99 -o "$map" --tree`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/arg-order-checks.t#L7 "v17.2.7/src/test/cli/crushtool/arg-order-checks.t::cmd-03") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/arg-order-checks.t#L7 "v20.2.4/src/test/cli/crushtool/arg-order-checks.t::cmd-03") |
| `cmd-04`: ``crushtool -d "$map"`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/arg-order-checks.t#L46 "v17.2.7/src/test/cli/crushtool/arg-order-checks.t::cmd-04") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/arg-order-checks.t#L46 "v20.2.4/src/test/cli/crushtool/arg-order-checks.t::cmd-04") |
| `cmd-05`: ``crushtool -i "$map" --set-straw-calc-version 0 --reweight --test --show-utilization --max-x 100 --min-x 1 --min-rep 1 --max-rep 10`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/arg-order-checks.t#L202 "v17.2.7/src/test/cli/crushtool/arg-order-checks.t::cmd-05") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/arg-order-checks.t#L202 "v20.2.4/src/test/cli/crushtool/arg-order-checks.t::cmd-05") |
| `cmd-06`: ``crushtool -i "$map" --set-straw-calc-version 1 --reweight --test --show-utilization --max-x 100 --min-x 1 --min-rep 1 --max-rep 10`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/arg-order-checks.t#L467 "v17.2.7/src/test/cli/crushtool/arg-order-checks.t::cmd-06") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/arg-order-checks.t#L467 "v20.2.4/src/test/cli/crushtool/arg-order-checks.t::cmd-06") |

### bad-mappings.t

**Readiness:** Blocked: F. Compile original bad-mappings.crushmap.txt; seed 1, rule 0/1, 10 replicas: FIRSTN returns [4,0,2,3,1], INDEP [4,0,2,1,3] plus five holes. Preserve order and sentinel 2147483647.

| Local case / original command | Source blocks |
| --- | --- |
| `cmd-01`: ``crushtool -c "$TESTDIR/bad-mappings.crushmap.txt" -o "$TESTDIR/bad-mappings.crushmap"`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/bad-mappings.t#L1 "v17.2.7/src/test/cli/crushtool/bad-mappings.t::cmd-01") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/bad-mappings.t#L1 "v20.2.4/src/test/cli/crushtool/bad-mappings.t::cmd-01") |
| `cmd-02`: ``crushtool -i "$TESTDIR/bad-mappings.crushmap" --test --show-bad-mappings --rule 0 --x 1 --num-rep 10`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/bad-mappings.t#L2 "v17.2.7/src/test/cli/crushtool/bad-mappings.t::cmd-02") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/bad-mappings.t#L2 "v20.2.4/src/test/cli/crushtool/bad-mappings.t::cmd-02") |
| `cmd-03`: ``crushtool -i "$TESTDIR/bad-mappings.crushmap" --test --show-bad-mappings --rule 1 --x 1 --num-rep 10`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/bad-mappings.t#L4 "v17.2.7/src/test/cli/crushtool/bad-mappings.t::cmd-03") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/bad-mappings.t#L4 "v20.2.4/src/test/cli/crushtool/bad-mappings.t::cmd-03") |
| `cmd-04`: ``rm -f "$TESTDIR/bad-mappings.crushmap"`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/bad-mappings.t#L6 "v17.2.7/src/test/cli/crushtool/bad-mappings.t::cmd-04") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/bad-mappings.t#L6 "v20.2.4/src/test/cli/crushtool/bad-mappings.t::cmd-04") |

### build.t

**Readiness:** Outside client scope: E. Build tree, quiet mode, multiple-root warning, default rule/tunables and malformed layer argument rejection. Client subset requires captured generated maps (F).

| Local case / original command | Source blocks |
| --- | --- |
| `cmd-01`: ``map="$TESTDIR/build.crushmap"`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/build.t#L1 "v17.2.7/src/test/cli/crushtool/build.t::cmd-01") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/build.t#L1 "v20.2.4/src/test/cli/crushtool/build.t::cmd-01") |
| `cmd-02`: ``crushtool --outfn "$map" --build --num_osds 5 node straw 2 rack straw 1 root straw 0`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/build.t#L6 "v17.2.7/src/test/cli/crushtool/build.t::cmd-02") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/build.t#L6 "v20.2.4/src/test/cli/crushtool/build.t::cmd-02") |
| `cmd-03`: ``CEPH_ARGS="--debug-crush 0" crushtool --outfn "$map" --build --num_osds 5 node straw 2 rack straw 1 root straw 0`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/build.t#L11 "v17.2.7/src/test/cli/crushtool/build.t::cmd-03") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/build.t#L11 "v20.2.4/src/test/cli/crushtool/build.t::cmd-03") |
| `cmd-04`: ``crushtool --outfn "$map" --build --num_osds 5 node straw 2 rack straw 1`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/build.t#L16 "v17.2.7/src/test/cli/crushtool/build.t::cmd-04") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/build.t#L16 "v20.2.4/src/test/cli/crushtool/build.t::cmd-04") |
| `cmd-05`: ``CEPH_ARGS="--debug-crush 0" crushtool --outfn "$map" --set-straw-calc-version 0 --build --num_osds 1 root straw 0 --set-chooseleaf-stable 0`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/build.t#L26 "v17.2.7/src/test/cli/crushtool/build.t::cmd-05") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/build.t#L26 "v20.2.4/src/test/cli/crushtool/build.t::cmd-05") |
| `cmd-06`: ``crushtool -o "$map.txt" -d "$map"`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/build.t#L27 "v17.2.7/src/test/cli/crushtool/build.t::cmd-06") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/build.t#L27 "v20.2.4/src/test/cli/crushtool/build.t::cmd-06") |
| `cmd-07`: ``cat "$map.txt"`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/build.t#L28 "v17.2.7/src/test/cli/crushtool/build.t::cmd-07") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/build.t#L28 "v20.2.4/src/test/cli/crushtool/build.t::cmd-07") |
| `cmd-08`: ``rm "$map" "$map.txt"`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/build.t#L63 "v17.2.7/src/test/cli/crushtool/build.t::cmd-08") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/build.t#L63 "v20.2.4/src/test/cli/crushtool/build.t::cmd-08") |
| `cmd-09`: ``crushtool --outfn "$map" --debug-crush 0 --build --num_osds 5 node straw 0`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/build.t#L68 "v17.2.7/src/test/cli/crushtool/build.t::cmd-09") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/build.t#L68 "v20.2.4/src/test/cli/crushtool/build.t::cmd-09") |

### check-invalid-map.t

**Readiness:** Ready now (decode subset); E (CLI text). Reject non-CRUSH bytes read from /etc/hosts. Use a checked-in representative hosts-text payload for hermetic Rust decode rejection; document this input adaptation. CLI error string/exit status is not a Rust library API.

| Local case / original command | Source blocks |
| --- | --- |
| `cmd-01`: ``crushtool -d /etc/hosts`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/check-invalid-map.t#L1 "v17.2.7/src/test/cli/crushtool/check-invalid-map.t::cmd-01") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/check-invalid-map.t#L1 "v20.2.4/src/test/cli/crushtool/check-invalid-map.t::cmd-01") |

### check-names.empty.t

**Readiness:** Outside client scope: E. Compile/check map with unknown type name item#0 and exit 1. Compiler/map-edit validation is outside scope; malformed decoded-map handling needs its own contract.

| Local case / original command | Source blocks |
| --- | --- |
| `cmd-01`: ``crushtool -c "$TESTDIR/check-names.empty.crushmap.txt" -o "$TESTDIR/check-names.empty.crushmap"`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/check-names.empty.t#L1 "v17.2.7/src/test/cli/crushtool/check-names.empty.t::cmd-01") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/check-names.empty.t#L1 "v20.2.4/src/test/cli/crushtool/check-names.empty.t::cmd-01") |
| `cmd-02`: ``crushtool -i "$TESTDIR/check-names.empty.crushmap" --check 0`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/check-names.empty.t#L2 "v17.2.7/src/test/cli/crushtool/check-names.empty.t::cmd-02") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/check-names.empty.t#L2 "v20.2.4/src/test/cli/crushtool/check-names.empty.t::cmd-02") |
| `cmd-03`: ``rm -f "$TESTDIR/check-names.empty.crushmap"`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/check-names.empty.t#L5 "v17.2.7/src/test/cli/crushtool/check-names.empty.t::cmd-03") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/check-names.empty.t#L5 "v20.2.4/src/test/cli/crushtool/check-names.empty.t::cmd-03") |

### check-names.max-id.t

**Readiness:** Outside client scope: E. Validation at max ID 2: IDs 0/1 pass, adding 2 fails, unrestricted check passes. Client decode limits are separate, not a port of --check.

| Local case / original command | Source blocks |
| --- | --- |
| `cmd-01`: ``crushtool -i "$TESTDIR/simple.template" --add-item 0 1.0 device0 --loc host host0 --loc cluster cluster0 -o check-names.crushmap > /dev/null`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/check-names.max-id.t#L1 "v17.2.7/src/test/cli/crushtool/check-names.max-id.t::cmd-01") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/check-names.max-id.t#L1 "v20.2.4/src/test/cli/crushtool/check-names.max-id.t::cmd-01") |
| `cmd-02`: ``crushtool -i check-names.crushmap       --add-item 1 1.0 device1 --loc host host0 --loc cluster cluster0 -o check-names.crushmap > /dev/null`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/check-names.max-id.t#L2 "v17.2.7/src/test/cli/crushtool/check-names.max-id.t::cmd-02") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/check-names.max-id.t#L2 "v20.2.4/src/test/cli/crushtool/check-names.max-id.t::cmd-02") |
| `cmd-03`: ``crushtool -i check-names.crushmap --check 2`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/check-names.max-id.t#L3 "v17.2.7/src/test/cli/crushtool/check-names.max-id.t::cmd-03") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/check-names.max-id.t#L3 "v20.2.4/src/test/cli/crushtool/check-names.max-id.t::cmd-03") |
| `cmd-04`: ``crushtool -i check-names.crushmap       --add-item 2 1.0 device2 --loc host host0 --loc cluster cluster0 -o check-names.crushmap > /dev/null`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/check-names.max-id.t#L4 "v17.2.7/src/test/cli/crushtool/check-names.max-id.t::cmd-04") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/check-names.max-id.t#L4 "v20.2.4/src/test/cli/crushtool/check-names.max-id.t::cmd-04") |
| `cmd-05`: ``crushtool -i check-names.crushmap --check 2`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/check-names.max-id.t#L5 "v17.2.7/src/test/cli/crushtool/check-names.max-id.t::cmd-05") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/check-names.max-id.t#L5 "v20.2.4/src/test/cli/crushtool/check-names.max-id.t::cmd-05") |
| `cmd-06`: ``crushtool -i check-names.crushmap --check`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/check-names.max-id.t#L8 "v17.2.7/src/test/cli/crushtool/check-names.max-id.t::cmd-06") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/check-names.max-id.t#L8 "v20.2.4/src/test/cli/crushtool/check-names.max-id.t::cmd-06") |

### choose-args.t

**Readiness:** Blocked: A + F (decode subset); E (compiler/dump). Round-trip choose-args.crush text and binary; validate full JSON state and JSON parseability. Capture binary and typed expected choose args; retain empty sets, positional weights and replacement IDs. Tentacle JSON adds MSR defaults/has_msr_rules.

| Local case / original command | Source blocks |
| --- | --- |
| `cmd-01`: ``cp "$TESTDIR/choose-args.crush" .`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/choose-args.t#L1 "v17.2.7/src/test/cli/crushtool/choose-args.t::cmd-01") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/choose-args.t#L1 "v20.2.4/src/test/cli/crushtool/choose-args.t::cmd-01") |
| `cmd-02`: ``crushtool -c choose-args.crush -o choose-args.compiled`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/choose-args.t#L2 "v17.2.7/src/test/cli/crushtool/choose-args.t::cmd-02") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/choose-args.t#L2 "v20.2.4/src/test/cli/crushtool/choose-args.t::cmd-02") |
| `cmd-03`: ``crushtool -d choose-args.compiled -o choose-args.conf`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/choose-args.t#L3 "v17.2.7/src/test/cli/crushtool/choose-args.t::cmd-03") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/choose-args.t#L3 "v20.2.4/src/test/cli/crushtool/choose-args.t::cmd-03") |
| `cmd-04`: ``crushtool -c choose-args.conf -o choose-args.recompiled`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/choose-args.t#L4 "v17.2.7/src/test/cli/crushtool/choose-args.t::cmd-04") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/choose-args.t#L4 "v20.2.4/src/test/cli/crushtool/choose-args.t::cmd-04") |
| `cmd-05`: ``cmp choose-args.crush choose-args.conf`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/choose-args.t#L5 "v17.2.7/src/test/cli/crushtool/choose-args.t::cmd-05") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/choose-args.t#L5 "v20.2.4/src/test/cli/crushtool/choose-args.t::cmd-05") |
| `cmd-06`: ``cmp choose-args.compiled choose-args.recompiled`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/choose-args.t#L6 "v17.2.7/src/test/cli/crushtool/choose-args.t::cmd-06") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/choose-args.t#L6 "v20.2.4/src/test/cli/crushtool/choose-args.t::cmd-06") |
| `cmd-07`: ``crushtool -c choose-args.conf -o /dev/null --dump`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/choose-args.t#L7 "v17.2.7/src/test/cli/crushtool/choose-args.t::cmd-07") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/choose-args.t#L7 "v20.2.4/src/test/cli/crushtool/choose-args.t::cmd-07") |
| `cmd-08`: ``crushtool -c choose-args.conf -o /dev/null --dump \| jq .for_json_validation`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/choose-args.t#L275 "v17.2.7/src/test/cli/crushtool/choose-args.t::cmd-08") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/choose-args.t#L278 "v20.2.4/src/test/cli/crushtool/choose-args.t::cmd-08") |

### compile-decompile-recompile.t

**Readiness:** Outside client scope: E. need_tree_order.crush text/binary round-trip plus missing-bucket rejection. Client ordered decode subset is blocked F until the original binary is captured.

| Local case / original command | Source blocks |
| --- | --- |
| `cmd-01`: ``cp "$TESTDIR/need_tree_order.crush" .`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/compile-decompile-recompile.t#L1 "v17.2.7/src/test/cli/crushtool/compile-decompile-recompile.t::cmd-01") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/compile-decompile-recompile.t#L1 "v20.2.4/src/test/cli/crushtool/compile-decompile-recompile.t::cmd-01") |
| `cmd-02`: ``crushtool -c need_tree_order.crush -o nto.compiled`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/compile-decompile-recompile.t#L2 "v17.2.7/src/test/cli/crushtool/compile-decompile-recompile.t::cmd-02") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/compile-decompile-recompile.t#L2 "v20.2.4/src/test/cli/crushtool/compile-decompile-recompile.t::cmd-02") |
| `cmd-03`: ``crushtool -d nto.compiled -o nto.conf`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/compile-decompile-recompile.t#L3 "v17.2.7/src/test/cli/crushtool/compile-decompile-recompile.t::cmd-03") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/compile-decompile-recompile.t#L3 "v20.2.4/src/test/cli/crushtool/compile-decompile-recompile.t::cmd-03") |
| `cmd-04`: ``crushtool -c nto.conf -o nto.recompiled`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/compile-decompile-recompile.t#L4 "v17.2.7/src/test/cli/crushtool/compile-decompile-recompile.t::cmd-04") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/compile-decompile-recompile.t#L4 "v20.2.4/src/test/cli/crushtool/compile-decompile-recompile.t::cmd-04") |
| `cmd-05`: ``cmp need_tree_order.crush nto.conf`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/compile-decompile-recompile.t#L10 "v17.2.7/src/test/cli/crushtool/compile-decompile-recompile.t::cmd-05") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/compile-decompile-recompile.t#L10 "v20.2.4/src/test/cli/crushtool/compile-decompile-recompile.t::cmd-05") |
| `cmd-06`: ``cmp nto.compiled nto.recompiled`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/compile-decompile-recompile.t#L11 "v17.2.7/src/test/cli/crushtool/compile-decompile-recompile.t::cmd-06") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/compile-decompile-recompile.t#L11 "v20.2.4/src/test/cli/crushtool/compile-decompile-recompile.t::cmd-06") |
| `cmd-07`: ``crushtool -c "$TESTDIR/missing-bucket.crushmap.txt"`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/compile-decompile-recompile.t#L13 "v17.2.7/src/test/cli/crushtool/compile-decompile-recompile.t::cmd-07") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/compile-decompile-recompile.t#L13 "v20.2.4/src/test/cli/crushtool/compile-decompile-recompile.t::cmd-07") |

### device-class.t

**Readiness:** Blocked: F (decode subset); E (compiler). device-class.crush text/binary round-trip. Capture original binary and validate class IDs, names and shadow roots; this subset does not implement the compiler.

| Local case / original command | Source blocks |
| --- | --- |
| `cmd-01`: ``cp "$TESTDIR/device-class.crush" .`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/device-class.t#L1 "v17.2.7/src/test/cli/crushtool/device-class.t::cmd-01") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/device-class.t#L1 "v20.2.4/src/test/cli/crushtool/device-class.t::cmd-01") |
| `cmd-02`: ``crushtool -c device-class.crush -o device-class.compiled`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/device-class.t#L2 "v17.2.7/src/test/cli/crushtool/device-class.t::cmd-02") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/device-class.t#L2 "v20.2.4/src/test/cli/crushtool/device-class.t::cmd-02") |
| `cmd-03`: ``crushtool -d device-class.compiled -o device-class.conf`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/device-class.t#L3 "v17.2.7/src/test/cli/crushtool/device-class.t::cmd-03") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/device-class.t#L3 "v20.2.4/src/test/cli/crushtool/device-class.t::cmd-03") |
| `cmd-04`: ``crushtool -c device-class.conf -o device-class.recompiled`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/device-class.t#L4 "v17.2.7/src/test/cli/crushtool/device-class.t::cmd-04") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/device-class.t#L4 "v20.2.4/src/test/cli/crushtool/device-class.t::cmd-04") |
| `cmd-05`: ``cmp device-class.crush device-class.conf`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/device-class.t#L5 "v17.2.7/src/test/cli/crushtool/device-class.t::cmd-05") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/device-class.t#L5 "v20.2.4/src/test/cli/crushtool/device-class.t::cmd-05") |
| `cmd-06`: ``cmp device-class.compiled device-class.recompiled`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/device-class.t#L6 "v17.2.7/src/test/cli/crushtool/device-class.t::cmd-06") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/device-class.t#L6 "v20.2.4/src/test/cli/crushtool/device-class.t::cmd-06") |

### empty-default.t

**Readiness:** Outside client scope: E. Compile empty-default.cushmap.txt with deprecated min/max-size warnings. Distinguish accepted empty-map decoding from monitor rejection; no compiler port.

| Local case / original command | Source blocks |
| --- | --- |
| `cmd-01`: ``crushtool -c "$TESTDIR/empty-default.cushmap.txt"`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/empty-default.t#L1 "v17.2.7/src/test/cli/crushtool/empty-default.t::cmd-01") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/empty-default.t#L1 "v20.2.4/src/test/cli/crushtool/empty-default.t::cmd-01") |

### help.t

**Readiness:** Outside client scope: E. Exact --help and --help-output presentation. No replacement in a native library; revisit with a crushtool-compatible CLI.

| Local case / original command | Source blocks |
| --- | --- |
| `cmd-01`: ``crushtool --help`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/help.t#L1 "v17.2.7/src/test/cli/crushtool/help.t::cmd-01") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/help.t#L1 "v20.2.4/src/test/cli/crushtool/help.t::cmd-01") |
| `cmd-02`: ``crushtool --help-output`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/help.t#L131 "v17.2.7/src/test/cli/crushtool/help.t::cmd-02") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/help.t#L131 "v20.2.4/src/test/cli/crushtool/help.t::cmd-02") |

### location.t

**Readiness:** Blocked: L. Existing binary test-map-big-1.crushmap: IDs 44,16 have no printed location; 167,258,87 have exact ordered host/rack/room/root paths. No fixture generation needed; production location query is missing.

| Local case / original command | Source blocks |
| --- | --- |
| `cmd-01`: ``crushtool -i $TESTDIR/test-map-big-1.crushmap --show-location 44`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/location.t#L1 "v17.2.7/src/test/cli/crushtool/location.t::cmd-01") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/location.t#L1 "v20.2.4/src/test/cli/crushtool/location.t::cmd-01") |
| `cmd-02`: ``crushtool -i $TESTDIR/test-map-big-1.crushmap --show-location 16`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/location.t#L2 "v17.2.7/src/test/cli/crushtool/location.t::cmd-02") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/location.t#L2 "v20.2.4/src/test/cli/crushtool/location.t::cmd-02") |
| `cmd-03`: ``crushtool -i $TESTDIR/test-map-big-1.crushmap --show-location 167`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/location.t#L3 "v17.2.7/src/test/cli/crushtool/location.t::cmd-03") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/location.t#L3 "v20.2.4/src/test/cli/crushtool/location.t::cmd-03") |
| `cmd-04`: ``crushtool -i $TESTDIR/test-map-big-1.crushmap --show-location 258`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/location.t#L8 "v17.2.7/src/test/cli/crushtool/location.t::cmd-04") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/location.t#L8 "v20.2.4/src/test/cli/crushtool/location.t::cmd-04") |
| `cmd-05`: ``crushtool -i $TESTDIR/test-map-big-1.crushmap --show-location 87`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/location.t#L12 "v17.2.7/src/test/cli/crushtool/location.t::cmd-05") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/location.t#L12 "v20.2.4/src/test/cli/crushtool/location.t::cmd-05") |

### output-csv.t

**Readiness:** Outside client scope: E. CSV presence, row counts and output-name prefixes for rules data/metadata/rbd; five-devices.crushmap, x=0..9. All 48 $ commands are unindented, unlike other Cram cases: verify upstream execution before treating this file as an executed gate.

| Local case / original command | Source blocks |
| --- | --- |
| `cmd-01`: ``crushtool -i five-devices.crushmap --test --num-rep 1 --min-x 0 --max-x 9 --output-csv`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/output-csv.t#L2 "v17.2.7/src/test/cli/crushtool/output-csv.t::cmd-01") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/output-csv.t#L2 "v20.2.4/src/test/cli/crushtool/output-csv.t::cmd-01") |
| `cmd-02`: ``if [ ! -f data-absolute_weights.csv ]; then echo FAIL; fi`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/output-csv.t#L3 "v17.2.7/src/test/cli/crushtool/output-csv.t::cmd-02") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/output-csv.t#L3 "v20.2.4/src/test/cli/crushtool/output-csv.t::cmd-02") |
| `cmd-03`: ``if [ ! -f data-batch_device_expected_utilization_all.csv ]; then echo FAIL; fi`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/output-csv.t#L4 "v17.2.7/src/test/cli/crushtool/output-csv.t::cmd-03") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/output-csv.t#L4 "v20.2.4/src/test/cli/crushtool/output-csv.t::cmd-03") |
| `cmd-04`: ``if [ ! -f data-batch_device_utilization_all.csv ]; then echo FAIL; fi`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/output-csv.t#L5 "v17.2.7/src/test/cli/crushtool/output-csv.t::cmd-04") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/output-csv.t#L5 "v20.2.4/src/test/cli/crushtool/output-csv.t::cmd-04") |
| `cmd-05`: ``if [ ! -f data-device_utilization_all.csv ]; then echo FAIL; fi`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/output-csv.t#L6 "v17.2.7/src/test/cli/crushtool/output-csv.t::cmd-05") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/output-csv.t#L6 "v20.2.4/src/test/cli/crushtool/output-csv.t::cmd-05") |
| `cmd-06`: ``if [ ! -f data-device_utilization.csv ]; then echo FAIL; fi`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/output-csv.t#L7 "v17.2.7/src/test/cli/crushtool/output-csv.t::cmd-06") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/output-csv.t#L7 "v20.2.4/src/test/cli/crushtool/output-csv.t::cmd-06") |
| `cmd-07`: ``if [ ! -f data-placement_information.csv ]; then echo FAIL; fi`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/output-csv.t#L8 "v17.2.7/src/test/cli/crushtool/output-csv.t::cmd-07") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/output-csv.t#L8 "v20.2.4/src/test/cli/crushtool/output-csv.t::cmd-07") |
| `cmd-08`: ``if [ ! -f data-proportional_weights_all.csv ]; then echo FAIL; fi`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/output-csv.t#L9 "v17.2.7/src/test/cli/crushtool/output-csv.t::cmd-08") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/output-csv.t#L9 "v20.2.4/src/test/cli/crushtool/output-csv.t::cmd-08") |
| `cmd-09`: ``if [ ! -f data-proportional_weights.csv ]; then echo FAIL; fi`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/output-csv.t#L10 "v17.2.7/src/test/cli/crushtool/output-csv.t::cmd-09") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/output-csv.t#L10 "v20.2.4/src/test/cli/crushtool/output-csv.t::cmd-09") |
| `cmd-10`: ``if [ ! -f metadata-absolute_weights.csv ]; then echo FAIL; fi`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/output-csv.t#L11 "v17.2.7/src/test/cli/crushtool/output-csv.t::cmd-10") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/output-csv.t#L11 "v20.2.4/src/test/cli/crushtool/output-csv.t::cmd-10") |
| `cmd-11`: ``if [ ! -f metadata-batch_device_expected_utilization_all.csv ]; then echo FAIL; fi`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/output-csv.t#L12 "v17.2.7/src/test/cli/crushtool/output-csv.t::cmd-11") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/output-csv.t#L12 "v20.2.4/src/test/cli/crushtool/output-csv.t::cmd-11") |
| `cmd-12`: ``if [ ! -f metadata-batch_device_utilization_all.csv ]; then echo FAIL; fi`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/output-csv.t#L13 "v17.2.7/src/test/cli/crushtool/output-csv.t::cmd-12") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/output-csv.t#L13 "v20.2.4/src/test/cli/crushtool/output-csv.t::cmd-12") |
| `cmd-13`: ``if [ ! -f metadata-device_utilization_all.csv ]; then echo FAIL; fi`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/output-csv.t#L14 "v17.2.7/src/test/cli/crushtool/output-csv.t::cmd-13") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/output-csv.t#L14 "v20.2.4/src/test/cli/crushtool/output-csv.t::cmd-13") |
| `cmd-14`: ``if [ ! -f metadata-device_utilization.csv ]; then echo FAIL; fi`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/output-csv.t#L15 "v17.2.7/src/test/cli/crushtool/output-csv.t::cmd-14") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/output-csv.t#L15 "v20.2.4/src/test/cli/crushtool/output-csv.t::cmd-14") |
| `cmd-15`: ``if [ ! -f metadata-placement_information.csv ]; then echo FAIL; fi`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/output-csv.t#L16 "v17.2.7/src/test/cli/crushtool/output-csv.t::cmd-15") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/output-csv.t#L16 "v20.2.4/src/test/cli/crushtool/output-csv.t::cmd-15") |
| `cmd-16`: ``if [ ! -f metadata-proportional_weights_all.csv ]; then echo FAIL; fi`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/output-csv.t#L17 "v17.2.7/src/test/cli/crushtool/output-csv.t::cmd-16") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/output-csv.t#L17 "v20.2.4/src/test/cli/crushtool/output-csv.t::cmd-16") |
| `cmd-17`: ``if [ ! -f metadata-proportional_weights.csv ]; then echo FAIL; fi`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/output-csv.t#L18 "v17.2.7/src/test/cli/crushtool/output-csv.t::cmd-17") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/output-csv.t#L18 "v20.2.4/src/test/cli/crushtool/output-csv.t::cmd-17") |
| `cmd-18`: ``if [ ! -f rbd-absolute_weights.csv ]; then echo FAIL; fi`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/output-csv.t#L19 "v17.2.7/src/test/cli/crushtool/output-csv.t::cmd-18") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/output-csv.t#L19 "v20.2.4/src/test/cli/crushtool/output-csv.t::cmd-18") |
| `cmd-19`: ``if [ ! -f rbd-batch_device_expected_utilization_all.csv ]; then echo FAIL; fi`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/output-csv.t#L20 "v17.2.7/src/test/cli/crushtool/output-csv.t::cmd-19") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/output-csv.t#L20 "v20.2.4/src/test/cli/crushtool/output-csv.t::cmd-19") |
| `cmd-20`: ``if [ ! -f rbd-batch_device_utilization_all.csv ]; then echo FAIL; fi`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/output-csv.t#L21 "v17.2.7/src/test/cli/crushtool/output-csv.t::cmd-20") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/output-csv.t#L21 "v20.2.4/src/test/cli/crushtool/output-csv.t::cmd-20") |
| `cmd-21`: ``if [ ! -f rbd-device_utilization_all.csv ]; then echo FAIL; fi`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/output-csv.t#L22 "v17.2.7/src/test/cli/crushtool/output-csv.t::cmd-21") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/output-csv.t#L22 "v20.2.4/src/test/cli/crushtool/output-csv.t::cmd-21") |
| `cmd-22`: ``if [ ! -f rbd-device_utilization.csv ]; then echo FAIL; fi`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/output-csv.t#L23 "v17.2.7/src/test/cli/crushtool/output-csv.t::cmd-22") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/output-csv.t#L23 "v20.2.4/src/test/cli/crushtool/output-csv.t::cmd-22") |
| `cmd-23`: ``if [ ! -f rbd-placement_information.csv ]; then echo FAIL; fi`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/output-csv.t#L24 "v17.2.7/src/test/cli/crushtool/output-csv.t::cmd-23") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/output-csv.t#L24 "v20.2.4/src/test/cli/crushtool/output-csv.t::cmd-23") |
| `cmd-24`: ``if [ ! -f rbd-proportional_weights_all.csv ]; then echo FAIL; fi`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/output-csv.t#L25 "v17.2.7/src/test/cli/crushtool/output-csv.t::cmd-24") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/output-csv.t#L25 "v20.2.4/src/test/cli/crushtool/output-csv.t::cmd-24") |
| `cmd-25`: ``if [ ! -f rbd-proportional_weights.csv ]; then echo FAIL; fi`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/output-csv.t#L26 "v17.2.7/src/test/cli/crushtool/output-csv.t::cmd-25") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/output-csv.t#L26 "v20.2.4/src/test/cli/crushtool/output-csv.t::cmd-25") |
| `cmd-26`: ``rm data*csv`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/output-csv.t#L27 "v17.2.7/src/test/cli/crushtool/output-csv.t::cmd-26") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/output-csv.t#L27 "v20.2.4/src/test/cli/crushtool/output-csv.t::cmd-26") |
| `cmd-27`: ``rm metadata*csv`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/output-csv.t#L28 "v17.2.7/src/test/cli/crushtool/output-csv.t::cmd-27") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/output-csv.t#L28 "v20.2.4/src/test/cli/crushtool/output-csv.t::cmd-27") |
| `cmd-28`: ``rm rbd*csv`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/output-csv.t#L29 "v17.2.7/src/test/cli/crushtool/output-csv.t::cmd-28") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/output-csv.t#L29 "v20.2.4/src/test/cli/crushtool/output-csv.t::cmd-28") |
| `cmd-29`: ``crushtool -i five-devices.crushmap --test --rule 0 --num-rep 1 --min-x 0 --max-x 9 --output-csv`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/output-csv.t#L31 "v17.2.7/src/test/cli/crushtool/output-csv.t::cmd-29") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/output-csv.t#L31 "v20.2.4/src/test/cli/crushtool/output-csv.t::cmd-29") |
| `cmd-30`: ``if [ $(wc -l data-absolute_weights.csv \| awk '{print $1}') != "5" ]; then echo FAIL; fi`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/output-csv.t#L32 "v17.2.7/src/test/cli/crushtool/output-csv.t::cmd-30") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/output-csv.t#L32 "v20.2.4/src/test/cli/crushtool/output-csv.t::cmd-30") |
| `cmd-31`: ``if [ $(wc -l data-batch_device_expected_utilization_all.csv \| awk '{print $1}') != "5" ]; then echo FAIL; fi`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/output-csv.t#L33 "v17.2.7/src/test/cli/crushtool/output-csv.t::cmd-31") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/output-csv.t#L33 "v20.2.4/src/test/cli/crushtool/output-csv.t::cmd-31") |
| `cmd-32`: ``if [ $(wc -l data-batch_device_utilization_all.csv \| awk '{print $1}') != "5" ]; then echo FAIL; fi`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/output-csv.t#L34 "v17.2.7/src/test/cli/crushtool/output-csv.t::cmd-32") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/output-csv.t#L34 "v20.2.4/src/test/cli/crushtool/output-csv.t::cmd-32") |
| `cmd-33`: ``if [ $(wc -l data-device_utilization_all.csv \| awk '{print $1}') != "5" ]; then echo FAIL; fi`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/output-csv.t#L35 "v17.2.7/src/test/cli/crushtool/output-csv.t::cmd-33") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/output-csv.t#L35 "v20.2.4/src/test/cli/crushtool/output-csv.t::cmd-33") |
| `cmd-34`: ``if [ $(wc -l data-device_utilization.csv \| awk '{print $1}') != "5" ]; then echo FAIL; fi`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/output-csv.t#L36 "v17.2.7/src/test/cli/crushtool/output-csv.t::cmd-34") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/output-csv.t#L36 "v20.2.4/src/test/cli/crushtool/output-csv.t::cmd-34") |
| `cmd-35`: ``if [ $(wc -l data-placement_information.csv \| awk '{print $1}') != "10" ]; then echo FAIL; fi`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/output-csv.t#L37 "v17.2.7/src/test/cli/crushtool/output-csv.t::cmd-35") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/output-csv.t#L37 "v20.2.4/src/test/cli/crushtool/output-csv.t::cmd-35") |
| `cmd-36`: ``if [ $(wc -l data-proportional_weights_all.csv \| awk '{print $1}') != "5" ]; then echo FAIL; fi`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/output-csv.t#L38 "v17.2.7/src/test/cli/crushtool/output-csv.t::cmd-36") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/output-csv.t#L38 "v20.2.4/src/test/cli/crushtool/output-csv.t::cmd-36") |
| `cmd-37`: ``if [ $(wc -l data-proportional_weights.csv \| awk '{print $1}') != "5" ]; then echo FAIL; fi`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/output-csv.t#L39 "v17.2.7/src/test/cli/crushtool/output-csv.t::cmd-37") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/output-csv.t#L39 "v20.2.4/src/test/cli/crushtool/output-csv.t::cmd-37") |
| `cmd-38`: ``rm data*csv`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/output-csv.t#L40 "v17.2.7/src/test/cli/crushtool/output-csv.t::cmd-38") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/output-csv.t#L40 "v20.2.4/src/test/cli/crushtool/output-csv.t::cmd-38") |
| `cmd-39`: ``crushtool -i five-devices.crushmap --test --rule 0 --num-rep 1 --min-x 0 --max-x 9 --output-name "test-tag" --output-csv`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/output-csv.t#L42 "v17.2.7/src/test/cli/crushtool/output-csv.t::cmd-39") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/output-csv.t#L42 "v20.2.4/src/test/cli/crushtool/output-csv.t::cmd-39") |
| `cmd-40`: ``if [ ! -f test-tag-data-absolute_weights.csv ]; then echo FAIL; fi`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/output-csv.t#L43 "v17.2.7/src/test/cli/crushtool/output-csv.t::cmd-40") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/output-csv.t#L43 "v20.2.4/src/test/cli/crushtool/output-csv.t::cmd-40") |
| `cmd-41`: ``if [ ! -f test-tag-data-batch_device_expected_utilization_all.csv ]; then echo FAIL; fi`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/output-csv.t#L44 "v17.2.7/src/test/cli/crushtool/output-csv.t::cmd-41") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/output-csv.t#L44 "v20.2.4/src/test/cli/crushtool/output-csv.t::cmd-41") |
| `cmd-42`: ``if [ ! -f test-tag-data-batch_device_utilization_all.csv ]; then echo FAIL; fi`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/output-csv.t#L45 "v17.2.7/src/test/cli/crushtool/output-csv.t::cmd-42") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/output-csv.t#L45 "v20.2.4/src/test/cli/crushtool/output-csv.t::cmd-42") |
| `cmd-43`: ``if [ ! -f test-tag-data-device_utilization_all.csv ]; then echo FAIL; fi`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/output-csv.t#L46 "v17.2.7/src/test/cli/crushtool/output-csv.t::cmd-43") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/output-csv.t#L46 "v20.2.4/src/test/cli/crushtool/output-csv.t::cmd-43") |
| `cmd-44`: ``if [ ! -f test-tag-data-device_utilization.csv ]; then echo FAIL; fi`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/output-csv.t#L47 "v17.2.7/src/test/cli/crushtool/output-csv.t::cmd-44") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/output-csv.t#L47 "v20.2.4/src/test/cli/crushtool/output-csv.t::cmd-44") |
| `cmd-45`: ``if [ ! -f test-tag-data-placement_information.csv ]; then echo FAIL; fi`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/output-csv.t#L48 "v17.2.7/src/test/cli/crushtool/output-csv.t::cmd-45") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/output-csv.t#L48 "v20.2.4/src/test/cli/crushtool/output-csv.t::cmd-45") |
| `cmd-46`: ``if [ ! -f test-tag-data-proportional_weights_all.csv ]; then echo FAIL; fi`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/output-csv.t#L49 "v17.2.7/src/test/cli/crushtool/output-csv.t::cmd-46") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/output-csv.t#L49 "v20.2.4/src/test/cli/crushtool/output-csv.t::cmd-46") |
| `cmd-47`: ``if [ ! -f test-tag-data-proportional_weights.csv ]; then echo FAIL; fi`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/output-csv.t#L50 "v17.2.7/src/test/cli/crushtool/output-csv.t::cmd-47") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/output-csv.t#L50 "v20.2.4/src/test/cli/crushtool/output-csv.t::cmd-47") |
| `cmd-48`: ``rm test-tag*csv`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/output-csv.t#L51 "v17.2.7/src/test/cli/crushtool/output-csv.t::cmd-48") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/output-csv.t#L51 "v20.2.4/src/test/cli/crushtool/output-csv.t::cmd-48") |

### reclassify.t

**Readiness:** Outside client scope: E; blocked F for mapping subset. Ten source class maps; rewrite roots/buckets/classes then compare mappings for replicas 1..10. Preserve intentionally failing gabe reclassification and gabe2/f mapping comparisons. Capture before/after pairs and check their mappings; do not claim the editor is ported.

| Local case / original command | Source blocks |
| --- | --- |
| `cmd-01`: ``crushtool -i $TESTDIR/crush-classes/a --set-subtree-class default hdd --reclassify --reclassify-bucket %-ssd ssd default --reclassify-bucket ssd ssd default --reclassify-root default hdd -o foo`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/reclassify.t#L1 "v17.2.7/src/test/cli/crushtool/reclassify.t::cmd-01") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/reclassify.t#L1 "v20.2.4/src/test/cli/crushtool/reclassify.t::cmd-01") |
| `cmd-02`: ``crushtool -i $TESTDIR/crush-classes/a --compare foo --min-rep 1 --max-rep 10`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/reclassify.t#L21 "v17.2.7/src/test/cli/crushtool/reclassify.t::cmd-02") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/reclassify.t#L21 "v20.2.4/src/test/cli/crushtool/reclassify.t::cmd-02") |
| `cmd-03`: ``crushtool -i $TESTDIR/crush-classes/d --set-subtree-class default hdd --reclassify --reclassify-bucket %-ssd ssd default --reclassify-bucket ssd ssd default --reclassify-root default hdd -o foo`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/reclassify.t#L26 "v17.2.7/src/test/cli/crushtool/reclassify.t::cmd-03") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/reclassify.t#L26 "v20.2.4/src/test/cli/crushtool/reclassify.t::cmd-03") |
| `cmd-04`: ``crushtool -i $TESTDIR/crush-classes/d --compare foo --min-rep 1 --max-rep 10`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/reclassify.t#L54 "v17.2.7/src/test/cli/crushtool/reclassify.t::cmd-04") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/reclassify.t#L54 "v20.2.4/src/test/cli/crushtool/reclassify.t::cmd-04") |
| `cmd-05`: ``crushtool -i $TESTDIR/crush-classes/e --reclassify --reclassify-bucket ceph-osd-ssd-% ssd default --reclassify-bucket ssd-root ssd default --reclassify-root default hdd -o foo`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/reclassify.t#L59 "v17.2.7/src/test/cli/crushtool/reclassify.t::cmd-05") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/reclassify.t#L59 "v20.2.4/src/test/cli/crushtool/reclassify.t::cmd-05") |
| `cmd-06`: ``crushtool -i $TESTDIR/crush-classes/e --compare foo --min-rep 1 --max-rep 10`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/reclassify.t#L102 "v17.2.7/src/test/cli/crushtool/reclassify.t::cmd-06") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/reclassify.t#L102 "v20.2.4/src/test/cli/crushtool/reclassify.t::cmd-06") |
| `cmd-07`: ``crushtool -i $TESTDIR/crush-classes/c --reclassify --reclassify-bucket %-SSD ssd default --reclassify-bucket ssd ssd default --reclassify-root default hdd -o foo`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/reclassify.t#L108 "v17.2.7/src/test/cli/crushtool/reclassify.t::cmd-07") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/reclassify.t#L108 "v20.2.4/src/test/cli/crushtool/reclassify.t::cmd-07") |
| `cmd-08`: ``crushtool -i $TESTDIR/crush-classes/c --compare foo --min-rep 1 --max-rep 10`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/reclassify.t#L152 "v17.2.7/src/test/cli/crushtool/reclassify.t::cmd-08") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/reclassify.t#L152 "v20.2.4/src/test/cli/crushtool/reclassify.t::cmd-08") |
| `cmd-09`: ``crushtool -i $TESTDIR/crush-classes/beesly --set-subtree-class 0513-R-0060 hdd --set-subtree-class 0513-R-0050 hdd --reclassify --reclassify-root 0513-R-0050 hdd --reclassify-root 0513-R-0060 hdd -o foo`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/reclassify.t#L159 "v17.2.7/src/test/cli/crushtool/reclassify.t::cmd-09") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/reclassify.t#L159 "v20.2.4/src/test/cli/crushtool/reclassify.t::cmd-09") |
| `cmd-10`: ``crushtool -i $TESTDIR/crush-classes/beesly --compare foo --min-rep 1 --max-rep 10`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/reclassify.t#L225 "v17.2.7/src/test/cli/crushtool/reclassify.t::cmd-10") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/reclassify.t#L225 "v20.2.4/src/test/cli/crushtool/reclassify.t::cmd-10") |
| `cmd-11`: ``crushtool -i $TESTDIR/crush-classes/flax --reclassify --reclassify-root default hdd -o foo`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/reclassify.t#L232 "v17.2.7/src/test/cli/crushtool/reclassify.t::cmd-11") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/reclassify.t#L232 "v20.2.4/src/test/cli/crushtool/reclassify.t::cmd-11") |
| `cmd-12`: ``crushtool -i $TESTDIR/crush-classes/flax --compare foo --min-rep 1 --max-rep 10`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/reclassify.t#L241 "v17.2.7/src/test/cli/crushtool/reclassify.t::cmd-12") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/reclassify.t#L241 "v20.2.4/src/test/cli/crushtool/reclassify.t::cmd-12") |
| `cmd-13`: ``crushtool -i $TESTDIR/crush-classes/gabe --reclassify --reclassify-root default hdd -o foo`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/reclassify.t#L245 "v17.2.7/src/test/cli/crushtool/reclassify.t::cmd-13") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/reclassify.t#L245 "v20.2.4/src/test/cli/crushtool/reclassify.t::cmd-13") |
| `cmd-14`: ``crushtool -i $TESTDIR/crush-classes/gabe2 --reclassify --reclassify-root default hdd -o foo`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/reclassify.t#L255 "v17.2.7/src/test/cli/crushtool/reclassify.t::cmd-14") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/reclassify.t#L255 "v20.2.4/src/test/cli/crushtool/reclassify.t::cmd-14") |
| `cmd-15`: ``crushtool -i $TESTDIR/crush-classes/gabe2 --compare foo --min-rep 1 --max-rep 10`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/reclassify.t#L282 "v17.2.7/src/test/cli/crushtool/reclassify.t::cmd-15") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/reclassify.t#L282 "v20.2.4/src/test/cli/crushtool/reclassify.t::cmd-15") |
| `cmd-16`: ``crushtool -i $TESTDIR/crush-classes/b --reclassify --reclassify-bucket %-hdd hdd default --reclassify-bucket %-ssd ssd default --reclassify-bucket ssd ssd default --reclassify-bucket hdd hdd default -o foo`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/reclassify.t#L290 "v17.2.7/src/test/cli/crushtool/reclassify.t::cmd-16") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/reclassify.t#L290 "v20.2.4/src/test/cli/crushtool/reclassify.t::cmd-16") |
| `cmd-17`: ``crushtool -i $TESTDIR/crush-classes/b --compare foo --min-rep 1 --max-rep 10`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/reclassify.t#L407 "v17.2.7/src/test/cli/crushtool/reclassify.t::cmd-17") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/reclassify.t#L407 "v20.2.4/src/test/cli/crushtool/reclassify.t::cmd-17") |
| `cmd-18`: ``crushtool -i $TESTDIR/crush-classes/f --reclassify --reclassify-root default hdd -o foo`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/reclassify.t#L412 "v17.2.7/src/test/cli/crushtool/reclassify.t::cmd-18") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/reclassify.t#L412 "v20.2.4/src/test/cli/crushtool/reclassify.t::cmd-18") |
| `cmd-19`: ``crushtool -i $TESTDIR/crush-classes/f --compare foo --min-rep 1 --max-rep 10`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/reclassify.t#L443 "v17.2.7/src/test/cli/crushtool/reclassify.t::cmd-19") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/reclassify.t#L443 "v20.2.4/src/test/cli/crushtool/reclassify.t::cmd-19") |
| `cmd-20`: ``crushtool -i $TESTDIR/crush-classes/g --reclassify --reclassify-bucket sata-% hdd-sata default --reclassify-bucket sas-% hdd-sas default --reclassify-bucket sas hdd-sas default --reclassify-bucket sata hdd-sata default -o foo`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/reclassify.t#L449 "v17.2.7/src/test/cli/crushtool/reclassify.t::cmd-20") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/reclassify.t#L449 "v20.2.4/src/test/cli/crushtool/reclassify.t::cmd-20") |
| `cmd-21`: ``crushtool -i $TESTDIR/crush-classes/g --compare foo --min-rep 1 --max-rep 10`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/reclassify.t#L559 "v17.2.7/src/test/cli/crushtool/reclassify.t::cmd-21") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/reclassify.t#L559 "v20.2.4/src/test/cli/crushtool/reclassify.t::cmd-21") |

### reweight.t

**Readiness:** Outside client scope: E. multitype.before -> multitype.after: osd0/3/6 weight 2, osd7 weight .5. Preserve algorithm-specific totals; captured output can later test client decode/mapping (F).

| Local case / original command | Source blocks |
| --- | --- |
| `cmd-01`: ``crushtool -c "$TESTDIR/multitype.before" -o mt > /dev/null`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/reweight.t#L1 "v17.2.7/src/test/cli/crushtool/reweight.t::cmd-01") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/reweight.t#L1 "v20.2.4/src/test/cli/crushtool/reweight.t::cmd-01") |
| `cmd-02`: ``crushtool -i mt --reweight-item osd0 2.0 -o mt > /dev/null`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/reweight.t#L2 "v17.2.7/src/test/cli/crushtool/reweight.t::cmd-02") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/reweight.t#L2 "v20.2.4/src/test/cli/crushtool/reweight.t::cmd-02") |
| `cmd-03`: ``crushtool -i mt --reweight-item osd3 2.0 -o mt > /dev/null`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/reweight.t#L3 "v17.2.7/src/test/cli/crushtool/reweight.t::cmd-03") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/reweight.t#L3 "v20.2.4/src/test/cli/crushtool/reweight.t::cmd-03") |
| `cmd-04`: ``crushtool -i mt --reweight-item osd6 2.0 -o mt > /dev/null`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/reweight.t#L4 "v17.2.7/src/test/cli/crushtool/reweight.t::cmd-04") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/reweight.t#L4 "v20.2.4/src/test/cli/crushtool/reweight.t::cmd-04") |
| `cmd-05`: ``crushtool -i mt --reweight-item osd7 .5 -o mt > /dev/null`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/reweight.t#L5 "v17.2.7/src/test/cli/crushtool/reweight.t::cmd-05") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/reweight.t#L5 "v20.2.4/src/test/cli/crushtool/reweight.t::cmd-05") |
| `cmd-06`: ``crushtool -d mt -o final`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/reweight.t#L6 "v17.2.7/src/test/cli/crushtool/reweight.t::cmd-06") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/reweight.t#L6 "v20.2.4/src/test/cli/crushtool/reweight.t::cmd-06") |
| `cmd-07`: ``diff final "$TESTDIR/multitype.after"`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/reweight.t#L7 "v17.2.7/src/test/cli/crushtool/reweight.t::cmd-07") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/reweight.t#L7 "v20.2.4/src/test/cli/crushtool/reweight.t::cmd-07") |
| `cmd-08`: ``rm mt final`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/reweight.t#L8 "v17.2.7/src/test/cli/crushtool/reweight.t::cmd-08") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/reweight.t#L8 "v20.2.4/src/test/cli/crushtool/reweight.t::cmd-08") |

### reweight_multiple.t

**Readiness:** Outside client scope: E. Reweight shared osd1 to 2.5 in simple.template.multitree; exact final map. Client consumption subset blocked F.

| Local case / original command | Source blocks |
| --- | --- |
| `cmd-01`: ``crushtool -c "$TESTDIR/simple.template.multitree" -o mt`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/reweight_multiple.t#L1 "v17.2.7/src/test/cli/crushtool/reweight_multiple.t::cmd-01") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/reweight_multiple.t#L1 "v20.2.4/src/test/cli/crushtool/reweight_multiple.t::cmd-01") |
| `cmd-02`: ``crushtool -i mt --reweight-item osd1 2.5 -o mt`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/reweight_multiple.t#L2 "v17.2.7/src/test/cli/crushtool/reweight_multiple.t::cmd-02") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/reweight_multiple.t#L2 "v20.2.4/src/test/cli/crushtool/reweight_multiple.t::cmd-02") |
| `cmd-03`: ``crushtool -d mt -o mt.txt`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/reweight_multiple.t#L4 "v17.2.7/src/test/cli/crushtool/reweight_multiple.t::cmd-03") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/reweight_multiple.t#L4 "v20.2.4/src/test/cli/crushtool/reweight_multiple.t::cmd-03") |
| `cmd-04`: ``diff mt.txt "$TESTDIR/simple.template.multitree.reweighted"`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/reweight_multiple.t#L5 "v17.2.7/src/test/cli/crushtool/reweight_multiple.t::cmd-04") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/reweight_multiple.t#L5 "v20.2.4/src/test/cli/crushtool/reweight_multiple.t::cmd-04") |

### rules.t

**Readiness:** Outside client scope: E. Create replicated foo and foo-ssd rules, preserve device-class take and decompiled rules. Capture both binaries for client rule execution (F).

| Local case / original command | Source blocks |
| --- | --- |
| `cmd-01`: ``crushtool -c $TESTDIR/rules.txt --create-replicated-rule foo default host -o one > /dev/null`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/rules.t#L1 "v17.2.7/src/test/cli/crushtool/rules.t::cmd-01") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/rules.t#L1 "v20.2.4/src/test/cli/crushtool/rules.t::cmd-01") |
| `cmd-02`: ``crushtool -d one`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/rules.t#L4 "v17.2.7/src/test/cli/crushtool/rules.t::cmd-02") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/rules.t#L4 "v20.2.4/src/test/cli/crushtool/rules.t::cmd-02") |
| `cmd-03`: ``crushtool -c $TESTDIR/rules.txt --create-replicated-rule foo-ssd default host -o two --device-class ssd > /dev/null`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/rules.t#L81 "v17.2.7/src/test/cli/crushtool/rules.t::cmd-03") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/rules.t#L81 "v20.2.4/src/test/cli/crushtool/rules.t::cmd-03") |
| `cmd-04`: ``crushtool -d two`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/rules.t#L84 "v17.2.7/src/test/cli/crushtool/rules.t::cmd-04") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/rules.t#L84 "v20.2.4/src/test/cli/crushtool/rules.t::cmd-04") |

### set-choose.t

**Readiness:** Blocked: F. Original set-choose.crushmap.txt, six rules 0..5, replicas 2 and 3, x=0..1023. Three weight profiles: all-in; IDs 0,1,3,4 out; IDs 0,3,5,7 out with 4=.5 and 6=.1. Preserve every vector/statistic and retry opcode; compiler required for exact legacy bucket setup.

| Local case / original command | Source blocks |
| --- | --- |
| `cmd-01`: ``crushtool -c "$TESTDIR/set-choose.crushmap.txt" -o set-choose.crushmap`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/set-choose.t#L1 "v17.2.7/src/test/cli/crushtool/set-choose.t::cmd-01") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/set-choose.t#L1 "v20.2.4/src/test/cli/crushtool/set-choose.t::cmd-01") |
| `cmd-02`: ``crushtool -i set-choose.crushmap --test --show-mappings --show-statistics --set-straw-calc-version 0 --min-rep 2 --max-rep 3`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/set-choose.t#L2 "v17.2.7/src/test/cli/crushtool/set-choose.t::cmd-02") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/set-choose.t#L2 "v20.2.4/src/test/cli/crushtool/set-choose.t::cmd-02") |
| `cmd-03`: ``crushtool -i set-choose.crushmap --test --show-mappings --show-statistics --weight 0 0 --weight 1 0 --weight 3 0 --weight 4 0 --set-straw-calc-version 0 --min-rep 2 --max-rep 3`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/set-choose.t#L12310 "v17.2.7/src/test/cli/crushtool/set-choose.t::cmd-03") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/set-choose.t#L12310 "v20.2.4/src/test/cli/crushtool/set-choose.t::cmd-03") |
| `cmd-04`: ``crushtool -i set-choose.crushmap --test --show-mappings --show-statistics --weight 0 0 --weight 3 0 --weight 4 .5 --weight 5 0 --weight 6 .1 --weight 7 0 --set-straw-calc-version 0 --min-rep 2 --max-rep 3`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/set-choose.t#L24623 "v17.2.7/src/test/cli/crushtool/set-choose.t::cmd-04") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/set-choose.t#L24623 "v20.2.4/src/test/cli/crushtool/set-choose.t::cmd-04") |

### show-choose-tries.t

**Readiness:** Blocked: F + D. Original show-choose-tries.txt; seed 1; FIRSTN rule 0/2 replicas and INDEP rule 1/1 replica; exact 0..49 retry histogram. Import binary and add test-visible retry instrumentation; final vector alone does not preserve these assertions.

| Local case / original command | Source blocks |
| --- | --- |
| `cmd-01`: ``crushtool -c "$TESTDIR/show-choose-tries.txt" -o "$TESTDIR/show-choose-tries.crushmap"`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/show-choose-tries.t#L1 "v17.2.7/src/test/cli/crushtool/show-choose-tries.t::cmd-01") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/show-choose-tries.t#L1 "v20.2.4/src/test/cli/crushtool/show-choose-tries.t::cmd-01") |
| `cmd-02`: ``FIRSTN_RULESET=0`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/show-choose-tries.t#L2 "v17.2.7/src/test/cli/crushtool/show-choose-tries.t::cmd-02") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/show-choose-tries.t#L2 "v20.2.4/src/test/cli/crushtool/show-choose-tries.t::cmd-02") |
| `cmd-03`: ``crushtool -i "$TESTDIR/show-choose-tries.crushmap" --test --show-choose-tries --rule $FIRSTN_RULESET --x 1 --num-rep 2`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/show-choose-tries.t#L3 "v17.2.7/src/test/cli/crushtool/show-choose-tries.t::cmd-03") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/show-choose-tries.t#L3 "v20.2.4/src/test/cli/crushtool/show-choose-tries.t::cmd-03") |
| `cmd-04`: ``INDEP_RULESET=1`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/show-choose-tries.t#L54 "v17.2.7/src/test/cli/crushtool/show-choose-tries.t::cmd-04") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/show-choose-tries.t#L54 "v20.2.4/src/test/cli/crushtool/show-choose-tries.t::cmd-04") |
| `cmd-05`: ``crushtool -i "$TESTDIR/show-choose-tries.crushmap" --test --show-choose-tries --rule $INDEP_RULESET --x 1 --num-rep 1`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/show-choose-tries.t#L55 "v17.2.7/src/test/cli/crushtool/show-choose-tries.t::cmd-05") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/show-choose-tries.t#L55 "v20.2.4/src/test/cli/crushtool/show-choose-tries.t::cmd-05") |
| `cmd-06`: ``rm -f "$TESTDIR/show-choose-tries.crushmap"`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/show-choose-tries.t#L106 "v17.2.7/src/test/cli/crushtool/show-choose-tries.t::cmd-06") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/show-choose-tries.t#L106 "v20.2.4/src/test/cli/crushtool/show-choose-tries.t::cmd-06") |

### straw2.t

**Readiness:** Outside client scope: E. Compile/decompile straw2.txt and compare ignoring whitespace. This is not a straw2 mapping test. Captured binary/fields can provide client decode subset (F).

| Local case / original command | Source blocks |
| --- | --- |
| `cmd-01`: ``crushtool -c $TESTDIR/straw2.txt -o straw2`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/straw2.t#L1 "v17.2.7/src/test/cli/crushtool/straw2.t::cmd-01") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/straw2.t#L1 "v20.2.4/src/test/cli/crushtool/straw2.t::cmd-01") |
| `cmd-02`: ``crushtool -d straw2 -o straw2.txt.new`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/straw2.t#L2 "v17.2.7/src/test/cli/crushtool/straw2.t::cmd-02") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/straw2.t#L2 "v20.2.4/src/test/cli/crushtool/straw2.t::cmd-02") |
| `cmd-03`: ``diff -b $TESTDIR/straw2.txt straw2.txt.new`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/straw2.t#L3 "v17.2.7/src/test/cli/crushtool/straw2.t::cmd-03") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/straw2.t#L3 "v20.2.4/src/test/cli/crushtool/straw2.t::cmd-03") |
| `cmd-04`: ``rm straw2 straw2.txt.new`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/straw2.t#L4 "v17.2.7/src/test/cli/crushtool/straw2.t::cmd-04") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/straw2.t#L4 "v20.2.4/src/test/cli/crushtool/straw2.t::cmd-04") |

### test-map-bobtail-tunables.t

**Coverage:** Partial (all client mapping/statistics assertions ported; CLI presentation excluded); **Verification:** Passing; **Review:** Pending. Rust: [golden::bobtail_tunables](../../rados/tests/crush/golden.rs). Adaptations: [Stage 1](#stage-1-execution); passing run: [Stage 2](#stage-2-mapper-fixes).

**Readiness:** Ready now. Existing binary input and exact .t vectors/statistics; x=0..1023, replicas 1..10. Preserve the command-specific rule, weight overrides and tunables listed below; no Ceph build required for replay. Historical tunable names do not make these tests out of scope for Quincy.

| Local case / original command | Source blocks |
| --- | --- |
| `cmd-01`: ``crushtool -i "$TESTDIR/test-map-a.crushmap" --test --show-mappings --show-statistics --rule 0 --set-choose-local-tries 0 --set-choose-local-fallback-tries 0 --set-choose-total-tries 50 --set-chooseleaf-descend-once 1 --min-rep 1 --max-rep 10`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/test-map-bobtail-tunables.t#L1 "v17.2.7/src/test/cli/crushtool/test-map-bobtail-tunables.t::cmd-01") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/test-map-bobtail-tunables.t#L1 "v20.2.4/src/test/cli/crushtool/test-map-bobtail-tunables.t::cmd-01") |

### test-map-firefly-tunables.t

**Coverage:** Partial (all client mapping/statistics assertions ported; CLI presentation excluded); **Verification:** Passing; **Review:** Pending. Rust: [golden::firefly_tunables](../../rados/tests/crush/golden.rs). Adaptations: [Stage 1](#stage-1-execution); passing run: [Stage 2](#stage-2-mapper-fixes).

**Readiness:** Ready now. Existing binary input and exact .t vectors/statistics; x=0..1023, replicas 1..10. Preserve the command-specific rule, weight overrides and tunables listed below; no Ceph build required for replay. Historical tunable names do not make these tests out of scope for Quincy.

| Local case / original command | Source blocks |
| --- | --- |
| `cmd-01`: ``crushtool -i "$TESTDIR/test-map-vary-r.crushmap" --test --show-mappings --show-statistics --rule 0 --set-choose-local-tries 0 --set-choose-local-fallback-tries 0 --set-choose-total-tries 50 --set-chooseleaf-descend-once 1 --set-chooseleaf-vary-r 1 --weight 12 0 --weight 20 0 --weight 30 0 --min-rep 1 --max-rep 10`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/test-map-firefly-tunables.t#L1 "v17.2.7/src/test/cli/crushtool/test-map-firefly-tunables.t::cmd-01") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/test-map-firefly-tunables.t#L1 "v20.2.4/src/test/cli/crushtool/test-map-firefly-tunables.t::cmd-01") |

### test-map-firstn-indep.t

**Readiness:** Blocked: F. Original test-map-firstn-indep.txt; seed 1, rules 0/1, replicas 1..10. Rule 0 undersized for 9/10: [93,80,88,87,56,50,53,72]; rule 1 for 3..10: [93,56]. Also retain absence of bad mappings for other sizes; do not replace this complex regression map.

| Local case / original command | Source blocks |
| --- | --- |
| `cmd-01`: ``crushtool -c "$TESTDIR/test-map-firstn-indep.txt" -o "$TESTDIR/test-map-firstn-indep.crushmap"`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/test-map-firstn-indep.t#L1 "v17.2.7/src/test/cli/crushtool/test-map-firstn-indep.t::cmd-01") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/test-map-firstn-indep.t#L1 "v20.2.4/src/test/cli/crushtool/test-map-firstn-indep.t::cmd-01") |
| `cmd-02`: ``crushtool -i "$TESTDIR/test-map-firstn-indep.crushmap" --test --rule 0 --x 1 --show-bad-mappings --min-rep 1 --max-rep 10`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/test-map-firstn-indep.t#L2 "v17.2.7/src/test/cli/crushtool/test-map-firstn-indep.t::cmd-02") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/test-map-firstn-indep.t#L2 "v20.2.4/src/test/cli/crushtool/test-map-firstn-indep.t::cmd-02") |
| `cmd-03`: ``crushtool -i "$TESTDIR/test-map-firstn-indep.crushmap" --test --rule 1 --x 1 --show-bad-mappings --min-rep 1 --max-rep 10`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/test-map-firstn-indep.t#L5 "v17.2.7/src/test/cli/crushtool/test-map-firstn-indep.t::cmd-03") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/test-map-firstn-indep.t#L5 "v20.2.4/src/test/cli/crushtool/test-map-firstn-indep.t::cmd-03") |
| `cmd-04`: ``rm -f "$TESTDIR/test-map-firstn-indep.crushmap"`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/test-map-firstn-indep.t#L14 "v17.2.7/src/test/cli/crushtool/test-map-firstn-indep.t::cmd-04") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/test-map-firstn-indep.t#L14 "v20.2.4/src/test/cli/crushtool/test-map-firstn-indep.t::cmd-04") |

### test-map-hammer-tunables.t

**Coverage:** Partial (all client mapping/statistics assertions ported; CLI presentation excluded); **Verification:** Passing; **Review:** Pending. Rust: [golden::hammer_tunables](../../rados/tests/crush/golden.rs). Adaptations: [Stage 1](#stage-1-execution); passing run: [Stage 2](#stage-2-mapper-fixes).

**Readiness:** Ready now. Existing binary input and exact .t vectors/statistics; x=0..1023, replicas 1..10. Preserve the command-specific rule, weight overrides and tunables listed below; no Ceph build required for replay. Historical tunable names do not make these tests out of scope for Quincy.

| Local case / original command | Source blocks |
| --- | --- |
| `cmd-01`: ``crushtool -i "$TESTDIR/test-map-hammer-tunables.crushmap" --test --show-mappings --show-statistics --rule 0 --weight 12 0 --weight 20 0 --weight 30 0 --min-rep 1 --max-rep 10`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/test-map-hammer-tunables.t#L1 "v17.2.7/src/test/cli/crushtool/test-map-hammer-tunables.t::cmd-01") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/test-map-hammer-tunables.t#L1 "v20.2.4/src/test/cli/crushtool/test-map-hammer-tunables.t::cmd-01") |

### test-map-indep.t

**Coverage:** Partial (all client mapping/statistics assertions ported; CLI presentation excluded); **Verification:** Passing; **Review:** Pending. Rust: [golden::indep](../../rados/tests/crush/golden.rs). Adaptations: [Stage 1](#stage-1-execution); passing run: [Stage 2](#stage-2-mapper-fixes).

**Readiness:** Ready now. Existing binary input and exact .t vectors/statistics; x=0..1023, replicas 1..10. Preserve the command-specific rule, weight overrides and tunables listed below; no Ceph build required for replay. Historical tunable names do not make these tests out of scope for Quincy.

| Local case / original command | Source blocks |
| --- | --- |
| `cmd-01`: ``crushtool -i "$TESTDIR/test-map-indep.crushmap" --test --show-mappings --show-statistics --rule 1 --set-choose-local-tries 0 --set-choose-local-fallback-tries 0 --set-choose-total-tries 50 --set-chooseleaf-descend-once 2 --min-rep 1 --max-rep 10`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/test-map-indep.t#L1 "v17.2.7/src/test/cli/crushtool/test-map-indep.t::cmd-01") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/test-map-indep.t#L1 "v20.2.4/src/test/cli/crushtool/test-map-indep.t::cmd-01") |

### test-map-jewel-tunables.t

**Coverage:** Partial (all client mapping/statistics assertions ported; CLI presentation excluded); **Verification:** Passing; **Review:** Pending. Rust: [golden::jewel_tunables](../../rados/tests/crush/golden.rs). Adaptations: [Stage 1](#stage-1-execution); passing run: [Stage 2](#stage-2-mapper-fixes).

**Readiness:** Ready now. Existing binary input and exact .t vectors/statistics; x=0..1023, replicas 1..10. Preserve the command-specific rule, weight overrides and tunables listed below; no Ceph build required for replay. Historical tunable names do not make these tests out of scope for Quincy.

| Local case / original command | Source blocks |
| --- | --- |
| `cmd-01`: ``crushtool -i "$TESTDIR/test-map-jewel-tunables.crushmap" --test --show-mappings --show-statistics --rule 0 --weight 12 0 --weight 20 0 --weight 30 0 --min-rep 1 --max-rep 10`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/test-map-jewel-tunables.t#L1 "v17.2.7/src/test/cli/crushtool/test-map-jewel-tunables.t::cmd-01") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/test-map-jewel-tunables.t#L1 "v20.2.4/src/test/cli/crushtool/test-map-jewel-tunables.t::cmd-01") |

### test-map-legacy-tunables.t

**Coverage:** Partial (all client mapping/statistics assertions ported; CLI presentation excluded); **Verification:** Passing; **Review:** Pending. Rust: [golden::legacy_tunables](../../rados/tests/crush/golden.rs). Adaptations: [Stage 1](#stage-1-execution); passing run: [Stage 2](#stage-2-mapper-fixes).

**Readiness:** Ready now. Existing binary input and exact .t vectors/statistics; x=0..1023, replicas 1..10. Preserve the command-specific rule, weight overrides and tunables listed below; no Ceph build required for replay. Historical tunable names do not make these tests out of scope for Quincy.

| Local case / original command | Source blocks |
| --- | --- |
| `cmd-01`: ``crushtool -i "$TESTDIR/test-map-a.crushmap" --test --show-mappings --show-statistics --rule 0 --min-rep 1 --max-rep 10`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/test-map-legacy-tunables.t#L1 "v17.2.7/src/test/cli/crushtool/test-map-legacy-tunables.t::cmd-01") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/test-map-legacy-tunables.t#L1 "v20.2.4/src/test/cli/crushtool/test-map-legacy-tunables.t::cmd-01") |

### test-map-tries-vs-retries.t

**Coverage:** Partial (all client mapping/statistics assertions ported; CLI presentation excluded); **Verification:** Passing; **Review:** Pending. Rust: [golden::tries_vs_retries](../../rados/tests/crush/golden.rs). Adaptations: [Stage 1](#stage-1-execution); passing run: [Stage 2](#stage-2-mapper-fixes).

**Readiness:** Ready now. Existing binary input and exact .t vectors/statistics; x=0..1023, replicas 1..10. Preserve the command-specific rule, weight overrides and tunables listed below; no Ceph build required for replay. Historical tunable names do not make these tests out of scope for Quincy.

| Local case / original command | Source blocks |
| --- | --- |
| `cmd-01`: ``crushtool -i "$TESTDIR/test-map-tries-vs-retries.crushmap" --test --show-mappings --show-statistics --weight 0 0 --weight 8 0 --min-rep 1 --max-rep 10`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/test-map-tries-vs-retries.t#L1 "v17.2.7/src/test/cli/crushtool/test-map-tries-vs-retries.t::cmd-01") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/test-map-tries-vs-retries.t#L1 "v20.2.4/src/test/cli/crushtool/test-map-tries-vs-retries.t::cmd-01") |

### test-map-vary-r-0.t

**Coverage:** Partial (all client mapping/statistics assertions ported; CLI presentation excluded); **Verification:** Passing; **Review:** Pending. Rust: [golden::vary_r_0](../../rados/tests/crush/golden.rs). Adaptations: [Stage 1](#stage-1-execution); passing run: [Stage 2](#stage-2-mapper-fixes).

**Readiness:** Ready now. Existing test-map-vary-r.crushmap; rule 3, vary_r=0, OSDs 0/4/9 out, x=0..1023, replicas 2/3/4. Compare all 3072 ordered vectors and result-size counts, including undersized results.

| Local case / original command | Source blocks |
| --- | --- |
| `cmd-01`: ``crushtool -i "$TESTDIR/test-map-vary-r.crushmap" --test --show-mappings --show-statistics --rule 3 --set-chooseleaf-vary-r 0 --weight 0 0 --weight 4 0 --weight 9 0 --min-rep 2 --max-rep 4`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/test-map-vary-r-0.t#L1 "v17.2.7/src/test/cli/crushtool/test-map-vary-r-0.t::cmd-01") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/test-map-vary-r-0.t#L1 "v20.2.4/src/test/cli/crushtool/test-map-vary-r-0.t::cmd-01") |

### test-map-vary-r-1.t

**Coverage:** Partial (all client mapping/statistics assertions ported; CLI presentation excluded); **Verification:** Passing; **Review:** Pending. Rust: [golden::vary_r_1](../../rados/tests/crush/golden.rs). Adaptations: [Stage 1](#stage-1-execution); passing run: [Stage 2](#stage-2-mapper-fixes).

**Readiness:** Ready now. Existing test-map-vary-r.crushmap; rule 3, vary_r=1, OSDs 0/4/9 out, x=0..1023, replicas 2/3/4. Compare all 3072 ordered vectors and result-size counts, including undersized results.

| Local case / original command | Source blocks |
| --- | --- |
| `cmd-01`: ``crushtool -i "$TESTDIR/test-map-vary-r.crushmap" --test --show-mappings --show-statistics --rule 3 --set-chooseleaf-vary-r 1 --weight 0 0 --weight 4 0 --weight 9 0 --min-rep 2 --max-rep 4`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/test-map-vary-r-1.t#L1 "v17.2.7/src/test/cli/crushtool/test-map-vary-r-1.t::cmd-01") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/test-map-vary-r-1.t#L1 "v20.2.4/src/test/cli/crushtool/test-map-vary-r-1.t::cmd-01") |

### test-map-vary-r-2.t

**Coverage:** Partial (all client mapping/statistics assertions ported; CLI presentation excluded); **Verification:** Passing; **Review:** Pending. Rust: [golden::vary_r_2](../../rados/tests/crush/golden.rs). Adaptations: [Stage 1](#stage-1-execution); passing run: [Stage 2](#stage-2-mapper-fixes).

**Readiness:** Ready now. Existing test-map-vary-r.crushmap; rule 3, vary_r=2, OSDs 0/4/9 out, x=0..1023, replicas 2/3/4. Compare all 3072 ordered vectors and result-size counts, including undersized results.

| Local case / original command | Source blocks |
| --- | --- |
| `cmd-01`: ``crushtool -i "$TESTDIR/test-map-vary-r.crushmap" --test --show-mappings --show-statistics --rule 3 --set-chooseleaf-vary-r 2 --weight 0 0 --weight 4 0 --weight 9 0 --min-rep 2 --max-rep 4`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/test-map-vary-r-2.t#L1 "v17.2.7/src/test/cli/crushtool/test-map-vary-r-2.t::cmd-01") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/test-map-vary-r-2.t#L1 "v20.2.4/src/test/cli/crushtool/test-map-vary-r-2.t::cmd-01") |

### test-map-vary-r-3.t

**Coverage:** Partial (all client mapping/statistics assertions ported; CLI presentation excluded); **Verification:** Passing; **Review:** Pending. Rust: [golden::vary_r_3](../../rados/tests/crush/golden.rs). Adaptations: [Stage 1](#stage-1-execution); passing run: [Stage 2](#stage-2-mapper-fixes).

**Readiness:** Ready now. Existing test-map-vary-r.crushmap; rule 3, vary_r=3, OSDs 0/4/9 out, x=0..1023, replicas 2/3/4. Compare all 3072 ordered vectors and result-size counts, including undersized results.

| Local case / original command | Source blocks |
| --- | --- |
| `cmd-01`: ``crushtool -i "$TESTDIR/test-map-vary-r.crushmap" --test --show-mappings --show-statistics --rule 3 --set-chooseleaf-vary-r 3 --weight 0 0 --weight 4 0 --weight 9 0 --min-rep 2 --max-rep 4`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/test-map-vary-r-3.t#L1 "v17.2.7/src/test/cli/crushtool/test-map-vary-r-3.t::cmd-01") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/test-map-vary-r-3.t#L1 "v20.2.4/src/test/cli/crushtool/test-map-vary-r-3.t::cmd-01") |

### test-map-vary-r-4.t

**Coverage:** Partial (all client mapping/statistics assertions ported; CLI presentation excluded); **Verification:** Passing; **Review:** Pending. Rust: [golden::vary_r_4](../../rados/tests/crush/golden.rs). Adaptations: [Stage 1](#stage-1-execution); passing run: [Stage 2](#stage-2-mapper-fixes).

**Readiness:** Ready now. Existing test-map-vary-r.crushmap; rule 3, vary_r=4, OSDs 0/4/9 out, x=0..1023, replicas 2/3/4. Compare all 3072 ordered vectors and result-size counts, including undersized results.

| Local case / original command | Source blocks |
| --- | --- |
| `cmd-01`: ``crushtool -i "$TESTDIR/test-map-vary-r.crushmap" --test --show-mappings --show-statistics --rule 3 --set-chooseleaf-vary-r 4 --weight 0 0 --weight 4 0 --weight 9 0 --min-rep 2 --max-rep 4`` | [v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/test-map-vary-r-4.t#L1 "v17.2.7/src/test/cli/crushtool/test-map-vary-r-4.t::cmd-01") / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/test-map-vary-r-4.t#L1 "v20.2.4/src/test/cli/crushtool/test-map-vary-r-4.t::cmd-01") |

## Fixture and helper catalogue

The following supporting files are present in both pinned releases with
identical bytes (verified by SHA256 comparison). They have **not** been
imported into this repository. Every path is relative to the named Ceph
directory. Binary maps are ready-to-copy inputs; text templates still require
reference compilation where the target is Rust binary decode/mapping.
Copy the original notices and record SHA256 when importing. Hashing source
bytes in this inventory does not constitute a mapping test.

| Upstream supporting file | Kind | SHA256 (both releases) |
| --- | --- | --- |
| `src/test/cli/crushtool/bad-mappings.crushmap.txt` ([v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/bad-mappings.crushmap.txt#L1) / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/bad-mappings.crushmap.txt#L1)) | Text template / expected data | `7769a2304e7a81b864deb9e10eb550b882e12cb3093467f6c612f3a949bcbeeb` |
| `src/test/cli/crushtool/check-names.empty.crushmap.txt` ([v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/check-names.empty.crushmap.txt#L1) / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/check-names.empty.crushmap.txt#L1)) | Text template / expected data | `1118a9cb2c6a5147dcf5749715c3826f2fb0282a005eb3affdd9b39d890de3b3` |
| `src/test/cli/crushtool/check-overlapped-rules.crushmap.txt` ([v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/check-overlapped-rules.crushmap.txt#L1) / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/check-overlapped-rules.crushmap.txt#L1)) | Text template / expected data | `bdbcbb31ad0664bf96703302c8cbea4affaf69215828c89a2c7b79be9f26bfbf` |
| `src/test/cli/crushtool/choose-args.crush` ([v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/choose-args.crush#L1) / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/choose-args.crush#L1)) | Text template / expected data | `fb2a66da99bbaa4e79a260ef6bb1432a55a0ed6475d1ca00594dcc83fd7dc8e5` |
| `src/test/cli/crushtool/crush-classes/a` ([v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/crush-classes/a#L1) / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/crush-classes/a#L1)) | Binary map | `50b1ee4d759d975754e6f09f37c575c09a526d21e55fe1db37d8f44f4cbf4b12` |
| `src/test/cli/crushtool/crush-classes/b` ([v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/crush-classes/b#L1) / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/crush-classes/b#L1)) | Binary map | `ddd3936270d32c9719dc6ed21a5b3a9be5ef5a0f9915d1a050120141c50a4f6c` |
| `src/test/cli/crushtool/crush-classes/beesly` ([v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/crush-classes/beesly#L1) / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/crush-classes/beesly#L1)) | Binary map | `7b212f21ad801b85b39a39ee860b697dcf66d2c72d9058abf655612de5785704` |
| `src/test/cli/crushtool/crush-classes/c` ([v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/crush-classes/c#L1) / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/crush-classes/c#L1)) | Binary map | `153be04a43265237908b1e991a1621b768257e4b94d14bb62efa033b1e2fa272` |
| `src/test/cli/crushtool/crush-classes/d` ([v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/crush-classes/d#L1) / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/crush-classes/d#L1)) | Binary map | `31177d4daaccbaf5c4be0c23c515156983782060f8cc6a3be31be2f6e5578a2e` |
| `src/test/cli/crushtool/crush-classes/e` ([v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/crush-classes/e#L1) / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/crush-classes/e#L1)) | Binary map | `8f7939072391df6eb1b7568431788c45c49b298df05cf313d225d77ab1a68724` |
| `src/test/cli/crushtool/crush-classes/f` ([v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/crush-classes/f#L1) / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/crush-classes/f#L1)) | Binary map | `60990c4e25e5eeb174865244fbe1ca825d59fca248bb4b8723fa8813ad1ff133` |
| `src/test/cli/crushtool/crush-classes/flax` ([v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/crush-classes/flax#L1) / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/crush-classes/flax#L1)) | Binary map | `98efbd93e90886e2707868583960f4fd3546c560544297474b7c8192db9e1440` |
| `src/test/cli/crushtool/crush-classes/g` ([v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/crush-classes/g#L1) / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/crush-classes/g#L1)) | Binary map | `14ff999cfe58f5c4b125e631dae60595f7cdfaf7c1c88bf9e2be5e32dee0ab90` |
| `src/test/cli/crushtool/crush-classes/gabe` ([v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/crush-classes/gabe#L1) / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/crush-classes/gabe#L1)) | Binary map | `6ae4ea974a10d18ab58f1dc299fcd0d97a63701880b8c0e7a24ec97c75a2d162` |
| `src/test/cli/crushtool/crush-classes/gabe2` ([v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/crush-classes/gabe2#L1) / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/crush-classes/gabe2#L1)) | Binary map | `60990c4e25e5eeb174865244fbe1ca825d59fca248bb4b8723fa8813ad1ff133` |
| `src/test/cli/crushtool/device-class.crush` ([v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/device-class.crush#L1) / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/device-class.crush#L1)) | Text template / expected data | `80efc22899ac0c09a7d98199f8b4a2b146b72694faa0129c54184c043b794a25` |
| `src/test/cli/crushtool/empty-default.cushmap.txt` ([v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/empty-default.cushmap.txt#L1) / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/empty-default.cushmap.txt#L1)) | Text template / expected data | `73cd1472115bb70232915df3d6aa3b89c57c16ca88a12cc6df1c2d0054110b1c` |
| `src/test/cli/crushtool/five-devices.crushmap` ([v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/five-devices.crushmap#L1) / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/five-devices.crushmap#L1)) | Binary map | `2ea1b5791e201b784c14c2587e135c54d03c4fb6ef7d9ebd01f83855e24e6b24` |
| `src/test/cli/crushtool/missing-bucket.crushmap.txt` ([v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/missing-bucket.crushmap.txt#L1) / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/missing-bucket.crushmap.txt#L1)) | Text template / expected data | `a0e7cd2f1835b7931bd3deddccfc9d102284d48b6ab8ad02c994c3f0877fe0e9` |
| `src/test/cli/crushtool/multitype.after` ([v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/multitype.after#L1) / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/multitype.after#L1)) | Text template / expected data | `65bb0dab74430497c14d0aec6f7c450aefa587f7d0b2157a03a62d5711b7baf0` |
| `src/test/cli/crushtool/multitype.before` ([v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/multitype.before#L1) / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/multitype.before#L1)) | Text template / expected data | `0cdd4049797af8f536ffbc0088e2be3f06bc6a593dd444a966158720dcd10e20` |
| `src/test/cli/crushtool/need_tree_order.crush` ([v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/need_tree_order.crush#L1) / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/need_tree_order.crush#L1)) | Text template / expected data | `bd5e95597a7d8aaadd1b2f60d1801deea620f571fd3cf04571ce8f8d36a1bdfb` |
| `src/test/cli/crushtool/rules.txt` ([v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/rules.txt#L1) / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/rules.txt#L1)) | Text template / expected data | `f28af99e57e5b54aad337d63f516c49fdbe764eefea94fb49397b7fe2608f726` |
| `src/test/cli/crushtool/set-choose.crushmap.txt` ([v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/set-choose.crushmap.txt#L1) / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/set-choose.crushmap.txt#L1)) | Text template / expected data | `6c29846b7d52c0f575cbdf73d0d98e86b151fe860740a4555e77aacf638b69cc` |
| `src/test/cli/crushtool/show-choose-tries.txt` ([v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/show-choose-tries.txt#L1) / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/show-choose-tries.txt#L1)) | Text template / expected data | `d84f192d7352fd023a4ff8a8960c38dfbf95b0c260f48a6f4f6c0cd358fe884d` |
| `src/test/cli/crushtool/simple.template` ([v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/simple.template#L1) / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/simple.template#L1)) | Binary map | `9760406b194a065275d1366eb7a07ca388a0dd2fa25514495ca8ae0d10e8ca79` |
| `src/test/cli/crushtool/simple.template.adj.one` ([v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/simple.template.adj.one#L1) / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/simple.template.adj.one#L1)) | Text template / expected data | `951bc51df802ba126f7192616d51672f5c3d267a20c0177f4c60b66c277d859d` |
| `src/test/cli/crushtool/simple.template.adj.three` ([v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/simple.template.adj.three#L1) / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/simple.template.adj.three#L1)) | Text template / expected data | `4100fe16e21bdd8e976f7d952be7879c4593a4d35641b040274ff9a9ca8a6987` |
| `src/test/cli/crushtool/simple.template.adj.two` ([v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/simple.template.adj.two#L1) / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/simple.template.adj.two#L1)) | Text template / expected data | `206990d07bda9120edad276dc2cb19e8bc581cf87ae204a7eb4ba07c3e20c9d0` |
| `src/test/cli/crushtool/simple.template.five` ([v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/simple.template.five#L1) / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/simple.template.five#L1)) | Text template / expected data | `063842c5a43a6658f412344bfb69973c1526c69c457637412c09638cd34ffc57` |
| `src/test/cli/crushtool/simple.template.four` ([v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/simple.template.four#L1) / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/simple.template.four#L1)) | Text template / expected data | `951bc51df802ba126f7192616d51672f5c3d267a20c0177f4c60b66c277d859d` |
| `src/test/cli/crushtool/simple.template.multitree` ([v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/simple.template.multitree#L1) / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/simple.template.multitree#L1)) | Text template / expected data | `e822339b487ae559ba24776fd7eb659325e4f54915cf04f6be2d984aba287ed3` |
| `src/test/cli/crushtool/simple.template.multitree.reweighted` ([v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/simple.template.multitree.reweighted#L1) / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/simple.template.multitree.reweighted#L1)) | Text template / expected data | `fe641197e22d776e46800b9ef04eaad063e0b8f940355f9d45d08bcb0c24d9fe` |
| `src/test/cli/crushtool/simple.template.one` ([v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/simple.template.one#L1) / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/simple.template.one#L1)) | Text template / expected data | `5462799bb46cbe9e39282d3554ab1ff7315e01a7ddddb61ac2ff3011d832d444` |
| `src/test/cli/crushtool/simple.template.three` ([v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/simple.template.three#L1) / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/simple.template.three#L1)) | Text template / expected data | `5462799bb46cbe9e39282d3554ab1ff7315e01a7ddddb61ac2ff3011d832d444` |
| `src/test/cli/crushtool/simple.template.two` ([v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/simple.template.two#L1) / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/simple.template.two#L1)) | Text template / expected data | `5462799bb46cbe9e39282d3554ab1ff7315e01a7ddddb61ac2ff3011d832d444` |
| `src/test/cli/crushtool/straw2.txt` ([v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/straw2.txt#L1) / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/straw2.txt#L1)) | Text template / expected data | `515d9a851ec69e682cb243db391469623cde5225e5502d1116e54106b419b49f` |
| `src/test/cli/crushtool/test-map-a.crushmap` ([v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/test-map-a.crushmap#L1) / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/test-map-a.crushmap#L1)) | Binary map | `3eacd04c60fa0143e298c49f6609a5b405fc01fc668f80386da28a8ea40781ca` |
| `src/test/cli/crushtool/test-map-big-1.crushmap` ([v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/test-map-big-1.crushmap#L1) / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/test-map-big-1.crushmap#L1)) | Binary map | `1c53e90b3756abebfad9ba9caa64186ab42af2e11cdb0ce5ef65efc264cb9ad3` |
| `src/test/cli/crushtool/test-map-firstn-indep.txt` ([v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/test-map-firstn-indep.txt#L1) / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/test-map-firstn-indep.txt#L1)) | Text template / expected data | `44f9f719d65e8f70056482e4b3ee2df93fd08207bab2b75e8c2e39bba02d06b6` |
| `src/test/cli/crushtool/test-map-hammer-tunables.crushmap` ([v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/test-map-hammer-tunables.crushmap#L1) / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/test-map-hammer-tunables.crushmap#L1)) | Binary map | `c1bf51b90ccdcc7dccef50a5d92384d444c44d75b7c3b4d915b8b30c03170b57` |
| `src/test/cli/crushtool/test-map-indep.crushmap` ([v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/test-map-indep.crushmap#L1) / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/test-map-indep.crushmap#L1)) | Binary map | `7de3cd6cc1410bd0cf3729ba08c9572f4f6a9c9b696103f938b243e0a6682b9b` |
| `src/test/cli/crushtool/test-map-jewel-tunables.crushmap` ([v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/test-map-jewel-tunables.crushmap#L1) / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/test-map-jewel-tunables.crushmap#L1)) | Binary map | `4a54a318d1e9f33d1ed405edec87366add5e443433e4fcc40206f130fda46880` |
| `src/test/cli/crushtool/test-map-tries-vs-retries.crushmap` ([v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/test-map-tries-vs-retries.crushmap#L1) / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/test-map-tries-vs-retries.crushmap#L1)) | Binary map | `df6669db23b0184704bb7b9b99d3d184f5d1ece01ef1607008365ac6df5c3d15` |
| `src/test/cli/crushtool/test-map-vary-r.crushmap` ([v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/test-map-vary-r.crushmap#L1) / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/test-map-vary-r.crushmap#L1)) | Binary map | `3c8979266fd148244ddd3dd05455f460c2fa411c5d40f4d7d788227716ae2b57` |
| `src/test/cli/crushtool/tree.template` ([v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/tree.template#L1) / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/tree.template#L1)) | Binary map | `0622bacb6b14b47c766bda9cd3a2e63d5f616c8c48cc4dd533361fcee8731b80` |
| `src/test/cli/crushtool/tree.template.final` ([v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/tree.template.final#L1) / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/tree.template.final#L1)) | Text template / expected data | `4a22c67c29bc3d9fc640a1fe7425d93ceac96414ca8320c759e66b5ae55bf1c8` |
| `src/test/crush/crush-choose-args-expected-one-more-0.txt` ([v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/crush/crush-choose-args-expected-one-more-0.txt#L1) / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/crush-choose-args-expected-one-more-0.txt#L1)) | Text template / expected data | `3460aba58ccbff4023e3126b77aca19c9ff97901b3f38a2a58d26072e775ba4e` |
| `src/test/crush/crush-choose-args-expected-one-more-3.txt` ([v17.2.7](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/crush/crush-choose-args-expected-one-more-3.txt#L1) / [v20.2.4](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/crush-choose-args-expected-one-more-3.txt#L1)) | Text template / expected data | `28bc2c4b4029e2ad4474bd62b689f0e7ecc3e6b8fe2c1660ebca00de3728cc57` |

Relevant construction/oracle helpers are `build_indep_map`,
`build_firstn_map`, `get_num_dups`, `calc_straw2_stddev`,
`create_crush_heirarchy` (upstream spelling), `compare_mappings` and
`get_mapping` in `crush.cc`; `CrushWrapper::set_tunables_optimal` and
`crush_calc_straw` determine setup that cannot be replaced with Rust defaults.
The CLI runner is `src/test/run-cli-tests`; the primary CMake registration is
`src/test/crush/CMakeLists.txt`. Inspect these with the same release as the case.

## CRUSH shell and standalone tests

Named functions below include their original helpers and setup. Running
monitor mutations alone does not test this Rust client. For cluster-derived
maps, capture input/output snapshots and replay decode/mapping through Rust;
retain the live I/O stage separately where the original requires it.

| Original function | Scenario / adaptation | Readiness |
| --- | --- | --- |
| [v17.2.7/qa/standalone/crush/crush-choose-args.sh::TEST_choose_args_update](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/qa/standalone/crush/crush-choose-args.sh#L45)<br>[v20.2.4/qa/standalone/crush/crush-choose-args.sh::TEST_choose_args_update](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/qa/standalone/crush/crush-choose-args.sh#L45) | Add then remove weighted OSD; compare full choose-args map against crush-choose-args-expected-one-more-3.txt and original. Server update production is M; decode/weight-set consumption is the client subset. | Blocked: A + F + C |
| [v17.2.7/qa/standalone/crush/crush-choose-args.sh::TEST_no_update_weight_set](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/qa/standalone/crush/crush-choose-args.sh#L100)<br>[v20.2.4/qa/standalone/crush/crush-choose-args.sh::TEST_no_update_weight_set](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/qa/standalone/crush/crush-choose-args.sh#L100) | Disable osd_crush_update_weight_set; add/remove OSD with positional weight sets and replacement IDs; compare expected-one-more-0.txt and original. Preserve option, not just misleading zero-weight comment. | Blocked: A + F + C |
| [v17.2.7/qa/standalone/crush/crush-choose-args.sh::TEST_reweight](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/qa/standalone/crush/crush-choose-args.sh#L162)<br>[v20.2.4/qa/standalone/crush/crush-choose-args.sh::TEST_reweight](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/qa/standalone/crush/crush-choose-args.sh#L162) | Canonical versus compat weights: host totals 6/5,9/5,10/5,10/9 after reweight/add steps. Server edits M; client subset must consume both weight domains. | Blocked: A + F + C |
| [v17.2.7/qa/standalone/crush/crush-choose-args.sh::TEST_move_bucket](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/qa/standalone/crush/crush-choose-args.sh#L194)<br>[v20.2.4/qa/standalone/crush/crush-choose-args.sh::TEST_move_bucket](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/qa/standalone/crush/crush-choose-args.sh#L194) | Move HOST under RACK, reweight compat, move leaves with update-weight-set true/false; preserve canonical/compat totals and zero fallback. Server move itself M. | Blocked: A + F + C |
| [v17.2.7/qa/standalone/crush/crush-classes.sh::TEST_reweight_vs_classes](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/qa/standalone/crush/crush-classes.sh#L60)<br>[v20.2.4/qa/standalone/crush/crush-classes.sh::TEST_reweight_vs_classes](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/qa/standalone/crush/crush-classes.sh#L60) | Regression #48065: canonical and ~ssd shadow weight changes 65536->131072 after set/reweight. Capture both map states; server rebuild M, decoded shadow weights in scope. | Blocked: F + C |
| [v17.2.7/qa/standalone/crush/crush-classes.sh::TEST_classes](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/qa/standalone/crush/crush-classes.sh#L88)<br>[v20.2.4/qa/standalone/crush/crush-classes.sh::TEST_classes](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/qa/standalone/crush/crush-classes.sh#L88) | Original SOMETHING up vector [1,2,0]; class-restricted SOMETHING_ELSE maps to [0], live writes still succeed, ~ssd shadow exists. Requires original object/pool/hash setup, map and I/O gate. | Blocked: F + C |
| [v17.2.7/qa/standalone/crush/crush-classes.sh::TEST_set_device_class](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/qa/standalone/crush/crush-classes.sh#L141)<br>[v20.2.4/qa/standalone/crush/crush-classes.sh::TEST_set_device_class](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/qa/standalone/crush/crush-classes.sh#L141) | Runs TEST_classes, then assigns ssd to OSDs 0 and 1 idempotently; exact up vector becomes [0,1]. Preserve sequence and convergence, then replay both maps in Rust. | Blocked: F + C |
| [v17.2.7/qa/standalone/crush/crush-classes.sh::TEST_mon_classes](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/qa/standalone/crush/crush-classes.sh#L166)<br>[v20.2.4/qa/standalone/crush/crush-classes.sh::TEST_mon_classes](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/qa/standalone/crush/crush-classes.sh#L166) | Class create/rename/remove/busy/idempotence, shadow rebuilding and moves, class rules and weight sets. Monitor commands are not client algorithms; retain resulting map/class filtering and original write checks as separate captured/live cases. | Outside client scope: M; blocked F + C for client subset |
| [v17.2.7/qa/standalone/mon/osd-crush.sh::TEST_crush_rule_create_simple](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/qa/standalone/mon/osd-crush.sh#L37)<br>[v20.2.4/qa/standalone/mon/osd-crush.sh::TEST_crush_rule_create_simple](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/qa/standalone/mon/osd-crush.sh#L37) | Create-simple and duplicate behavior; inspect TAKE/CHOOSE rule XML; capture produced rule if validating Rust execution (F). | Outside client scope: M |
| [v17.2.7/qa/standalone/mon/osd-crush.sh::TEST_crush_rule_dump](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/qa/standalone/mon/osd-crush.sh#L58)<br>[v20.2.4/qa/standalone/mon/osd-crush.sh::TEST_crush_rule_dump](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/qa/standalone/mon/osd-crush.sh#L58) | JSON single/all rule dumps and missing-rule failure. No Rust rule-dump API; revisit with management API. | Outside client scope: M |
| [v17.2.7/qa/standalone/mon/osd-crush.sh::TEST_crush_rule_rm](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/qa/standalone/mon/osd-crush.sh#L73)<br>[v20.2.4/qa/standalone/mon/osd-crush.sh::TEST_crush_rule_rm](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/qa/standalone/mon/osd-crush.sh#L73) | Create/list/remove erasure rule. No client-side rule editor. | Outside client scope: M |
| [v17.2.7/qa/standalone/mon/osd-crush.sh::TEST_crush_rule_create_erasure](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/qa/standalone/mon/osd-crush.sh#L84)<br>[v20.2.4/qa/standalone/mon/osd-crush.sh::TEST_crush_rule_create_erasure](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/qa/standalone/mon/osd-crush.sh#L84) | Implicit/explicit/default-profile creation, duplicate rule, chooseleaf-indep XML and monitor profile log. Client rule execution subset needs captured map (F). | Outside client scope: M |
| [v17.2.7/qa/standalone/mon/osd-crush.sh::TEST_add_rule_failed](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/qa/standalone/mon/osd-crush.sh#L123)<br>[v20.2.4/qa/standalone/mon/osd-crush.sh::TEST_add_rule_failed](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/qa/standalone/mon/osd-crush.sh#L168) | Fill rule IDs through 255; further create-simple returns ENOSPC. Monitor rule allocation, not mapping. | Outside client scope: M |
| [v17.2.7/qa/standalone/mon/osd-crush.sh::TEST_crush_rename_bucket](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/qa/standalone/mon/osd-crush.sh#L153)<br>[v20.2.4/qa/standalone/mon/osd-crush.sh::TEST_crush_rename_bucket](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/qa/standalone/mon/osd-crush.sh#L198) | Rename bucket, idempotence, missing bucket ENOENT. Client decode of resulting names remains a separate fixture check. | Outside client scope: M |
| [v17.2.7/qa/standalone/mon/osd-crush.sh::TEST_crush_ls_node](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/qa/standalone/mon/osd-crush.sh#L168)<br>[v20.2.4/qa/standalone/mon/osd-crush.sh::TEST_crush_ls_node](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/qa/standalone/mon/osd-crush.sh#L213) | List children under existing root, missing-root ENOENT. No matching monitor CLI surface in CRUSH library. | Outside client scope: M |
| [v17.2.7/qa/standalone/mon/osd-crush.sh::TEST_crush_reject_empty](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/qa/standalone/mon/osd-crush.sh#L178)<br>[v20.2.4/qa/standalone/mon/osd-crush.sh::TEST_crush_reject_empty](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/qa/standalone/mon/osd-crush.sh#L223) | Monitor rejects compiled empty map with EINVAL for an existing pool; do not turn this into a blanket binary decoder prohibition. | Outside client scope: M |
| [v20.2.4/qa/standalone/mon/osd-crush.sh::TEST_erasure_pool_crush_rule_rm](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/qa/standalone/mon/osd-crush.sh#L123) | Tentacle-only pool deletion/rule-lifetime sequences, including shared default EC rule. Retain source assertions verbatim if ever ported; do not infer stronger lifetime checks from comments. | Outside client scope: M |
| [v17.2.7/src/test/test_crush_bucket.sh::TEST_crush_bucket](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/test_crush_bucket.sh#L25)<br>[v20.2.4/src/test/test_crush_bucket.sh::TEST_crush_bucket](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/test_crush_bucket.sh#L25) | Start MON/three OSDs, add a STRAW bucket in exported text, require compilation without stderr. Capture compiled input to test Rust decode; compiler stderr is not a library contract. | Outside client scope: E; blocked F + C for decode subset |

### Weight-distribution shell assertions

`src/test/crush/crush_weights.sh` has no named test function. The identifiers below are locally assigned to its two assertion blocks. Both use straw2 weights [10,10,10,10,1], seeds 1..1,000,000 inclusive. The map can be assembled directly with existing Rust types.

| Original block | Assertions / adaptation | Readiness |
| --- | --- | --- |
| [v17.2.7/src/test/crush/crush_weights.sh::three-replica-distribution](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/crush/crush_weights.sh#L44)<br>[v20.2.4/src/test/crush/crush_weights.sh::three-replica-distribution](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/crush_weights.sh#L44) | Three replicas: reject when 10 - count(osd0)/count(osd4) < .75; preserve bc scale=5 semantics, even though the criterion is intentionally not ideal proportionality. | Ready now |
| [v17.2.7/src/test/crush/crush_weights.sh::one-replica-distribution](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/crush/crush_weights.sh#L53)<br>[v20.2.4/src/test/crush/crush_weights.sh::one-replica-distribution](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/crush/crush_weights.sh#L53) | One replica: reject when deviation from weight ratio 10 is outside [-.1,.1]; preserve bc scale=5 arithmetic and full sample count. | Ready now |
## Adjacent placement and management tests

These rows prevent a CRUSH-only result from being mistaken for full
object-to-OSD compatibility. Full `OSDMapTest` setup also creates pools,
applies incremental maps and sets OSD flags. The rows below identify client
assertions and distinguish them from monitor cleanup/balancing assertions.
None is credited as a complete port by the local OSDMap unit tests.

| Original OSDMap test | Scenario / remaining scope | Readiness |
| --- | --- | --- |
| [v17.2.7/src/test/osd/TestOSDMap.cc::OSDMapTest.Features](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/osd/TestOSDMap.cc#L222)<br>[v20.2.4/src/test/osd/TestOSDMap.cc::OSDMapTest.Features](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/osd/TestOSDMap.cc#L463) | Derived feature masks from pool types/CRUSH rules; preserve both releases feature expectations. Client feature negotiation is separate from raw mapping. | Blocked: P |
| [v17.2.7/src/test/osd/TestOSDMap.cc::OSDMapTest.MapPG](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/osd/TestOSDMap.cc#L261)<br>[v20.2.4/src/test/osd/TestOSDMap.cc::OSDMapTest.MapPG](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/osd/TestOSDMap.cc#L502) | Replicated PG 0: mapping overloads agree and up-vector length equals pool size. Reproduce set_up_map and expose up/acting separately if needed. | Blocked: P |
| [v17.2.7/src/test/osd/TestOSDMap.cc::OSDMapTest.MapFunctionsMatch](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/osd/TestOSDMap.cc#L281)<br>[v20.2.4/src/test/osd/TestOSDMap.cc::OSDMapTest.MapFunctionsMatch](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/osd/TestOSDMap.cc#L522) | All pg_to_up_acting_osds/pg_to_acting_osds forms agree on up/acting vectors and acting primary. One Rust method called twice is not equivalent coverage. | Blocked: P |
| [v17.2.7/src/test/osd/TestOSDMap.cc::OSDMapTest.PrimaryIsFirst](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/osd/TestOSDMap.cc#L309)<br>[v20.2.4/src/test/osd/TestOSDMap.cc::OSDMapTest.PrimaryIsFirst](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/osd/TestOSDMap.cc#L550) | For this default replicated setup, up/acting primaries are first elements. Preserve scenario boundaries; do not generalize to overrides or EC. | Blocked: P |
| [v17.2.7/src/test/osd/TestOSDMap.cc::OSDMapTest.PGTempRespected](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/osd/TestOSDMap.cc#L323)<br>[v20.2.4/src/test/osd/TestOSDMap.cc::OSDMapTest.PGTempRespected](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/osd/TestOSDMap.cc#L564) | Swap first/last acting devices through an incremental pg_temp; returned acting vector equals the complete replacement. Reuse original setup, not just a direct helper call. | Blocked: P |
| [v17.2.7/src/test/osd/TestOSDMap.cc::OSDMapTest.PrimaryTempRespected](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/osd/TestOSDMap.cc#L351)<br>[v20.2.4/src/test/osd/TestOSDMap.cc::OSDMapTest.PrimaryTempRespected](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/osd/TestOSDMap.cc#L592) | Set primary_temp to the second acting device through an incremental; mapping selects it as primary. Preserve ordered acting vector assertions. | Blocked: P |
| [v17.2.7/src/test/osd/TestOSDMap.cc::OSDMapTest.CleanTemps](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/osd/TestOSDMap.cc#L373)<br>[v20.2.4/src/test/osd/TestOSDMap.cc::OSDMapTest.CleanTemps](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/osd/TestOSDMap.cc#L614) | Monitor removes redundant pg_temp/primary_temp with exact empty/-1 removal records; capture those deltas and validate Rust application separately. | Outside client scope: M; blocked P for consumer subset |
| [v17.2.7/src/test/osd/TestOSDMap.cc::OSDMapTest.KeepsNecessaryTemps](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/osd/TestOSDMap.cc#L414)<br>[v20.2.4/src/test/osd/TestOSDMap.cc::OSDMapTest.KeepsNecessaryTemps](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/osd/TestOSDMap.cc#L655) | Monitor retains a useful replacement device/primary. Client subset must retain and apply those overrides; cleanup algorithm is not implemented by a client. | Outside client scope: M; blocked P for consumer subset |
| [v17.2.7/src/test/osd/TestOSDMap.cc::OSDMapTest.PrimaryAffinity](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/osd/TestOSDMap.cc#L462)<br>[v20.2.4/src/test/osd/TestOSDMap.cc::OSDMapTest.PrimaryAffinity](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/osd/TestOSDMap.cc#L703) | Replicated and EC pools, 10,000 mappings each, full/zero/half affinities; preserve counts, primary rejection, replicated ordering and 2/3..4/3 tolerance for half weight. Local affinity unit tests cover only narrower cases. | Blocked: P |
| [v17.2.7/src/test/osd/TestOSDMap.cc::OSDMapTest.get_osd_crush_node_flags](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/osd/TestOSDMap.cc#L537)<br>[v20.2.4/src/test/osd/TestOSDMap.cc::OSDMapTest.get_osd_crush_node_flags](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/osd/TestOSDMap.cc#L778) | Inherited root flags 0->123->456->0 for every OSD and missing ID; expose exact node-flag query and reproduce incrementals. | Blocked: D + P |
| [v17.2.7/src/test/osd/TestOSDMap.cc::OSDMapTest.CleanPGUpmaps](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/osd/TestOSDMap.cc#L606)<br>[v20.2.4/src/test/osd/TestOSDMap.cc::OSDMapTest.CleanPGUpmaps](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/osd/TestOSDMap.cc#L847) | Stale mappings, down vs out targets (#37493/#37501), EC same-host remap (#37968), negative full/item upmaps and host collisions. Preserve all pre/post-cleanup client mappings; cleanup producer assertions need monitor API. | Outside client scope: M; blocked P for consumer subset |
| [v20.2.4/src/test/osd/TestOSDMap.cc::OSDMapTest.CleanPGUpmapPrimaries](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/osd/TestOSDMap.cc#L1369) | Tentacle: reduce PG count, invalid/out/redundant primary and deleted pool; exact counts 10/0/11/10/7/8/7/0 around cleanup. Capture resulting primary overrides for client consumption. | Outside client scope: M; blocked P for consumer subset |
| [v17.2.7/src/test/osd/TestOSDMap.cc::OSDMapTest.BUG_38897](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/osd/TestOSDMap.cc#L1128)<br>[v20.2.4/src/test/osd/TestOSDMap.cc::OSDMapTest.BUG_38897](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/osd/TestOSDMap.cc#L1583) | Two pools with custom fixed-first-OSD rules and upmaps; client subsets assert first device 0, forced singleton [1], and replacement first device 10. Balancer calculation is not a client API. | Outside client scope: M; blocked P for consumer subset |
| [v17.2.7/src/test/osd/TestOSDMap.cc::OSDMapTest.BUG_40104](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/osd/TestOSDMap.cc#L1343)<br>[v20.2.4/src/test/osd/TestOSDMap.cc::OSDMapTest.BUG_40104](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/osd/TestOSDMap.cc#L1789) | 5,000 OSDs/10,000 PGs, size-3 placement then bulk upmap cleanup timing. No cleanup performance threshold asserted. Revisit if monitor cleanup is implemented; independent large-map client tests are supplementary. | Outside client scope: M |
| [v17.2.7/src/test/osd/TestOSDMap.cc::OSDMapTest.BUG_42052](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/osd/TestOSDMap.cc#L1399)<br>[v20.2.4/src/test/osd/TestOSDMap.cc::OSDMapTest.BUG_42052](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/osd/TestOSDMap.cc#L1845) | Fixed mapping [0,1,2], full upmap [2,3,5] plus item pairs; cleanup removes invalid overrides. Capture before/after states without implementing cleanup in the client. | Outside client scope: M; blocked P for consumer subset |
| [v17.2.7/src/test/osd/TestOSDMap.cc::OSDMapTest.BUG_42485](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/osd/TestOSDMap.cc#L1483)<br>[v20.2.4/src/test/osd/TestOSDMap.cc::OSDMapTest.BUG_42485](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/osd/TestOSDMap.cc#L1929) | 60 OSDs, two datacenters/three racks each; multilevel size-4 rule and two invalid cross-domain item overrides removed by cleanup. Preserve original domain topology for client replay. | Outside client scope: M; blocked P for consumer subset |
| [v17.2.7/src/test/osd/TestOSDMap.cc::OSDMapTest.BUG_43124](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/osd/TestOSDMap.cc#L1715)<br>[v20.2.4/src/test/osd/TestOSDMap.cc::OSDMapTest.BUG_43124](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/osd/TestOSDMap.cc#L2161) | 200 OSDs, five racks/four hosts/ten OSDs each, size-12 EC multilevel rule; valid item remap survives cleanup. Preserve full vector/domain setup. | Outside client scope: M; blocked P for consumer subset |
| [v17.2.7/src/test/osd/TestOSDMap.cc::OSDMapTest.BUG_48884](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/osd/TestOSDMap.cc#L1865)<br>[v20.2.4/src/test/osd/TestOSDMap.cc::OSDMapTest.BUG_48884](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/osd/TestOSDMap.cc#L2311) | Three hosts/four OSDs each; CRUSH utilization dump expects 3904/3512/384 values. Reporting/utilization formatter is not client placement; revisit if exposed. | Outside client scope: M |
| [v17.2.7/src/test/osd/TestOSDMap.cc::OSDMapTest.BUG_51842](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/osd/TestOSDMap.cc#L1942)<br>[v20.2.4/src/test/osd/TestOSDMap.cc::OSDMapTest.BUG_51842](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/osd/TestOSDMap.cc#L2388) | All four parameters (choose count,type)=(0,1),(3,1),(0,0),(3,0); pool size changes 3->1 and 3->4 invalidate three full upmaps. Preserve all variants and consumer results if porting a subset. | Outside client scope: M; blocked P for consumer subset |
| [v20.2.4/src/test/osd/TestOSDMap.cc::OSDMapTest.ReadBalanceScore1](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/osd/TestOSDMap.cc#L2729) | Randomized topology size from time-seeded C RNG, score range/error assertions for replicated pools. Revisit if read-balancer scoring is exposed; no Rust replacement. | Outside client scope: M |
| [v20.2.4/src/test/osd/TestOSDMap.cc::OSDMapTest.ReadBalanceScore2](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/osd/TestOSDMap.cc#L2800) | Read-balance scoring assertions after map/primary changes; preserve upstream random setup if adding scoring API. Client primary selection requires separate parity. | Outside client scope: M |
| [v20.2.4/src/test/osd/TestOSDMap.cc::OSDMapTest.read_balance_small_map](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/osd/TestOSDMap.cc#L2864) | Primary balancing on small map; validate upstream scoring and optimization assertions only if adding balancer, not merely applying its pg_upmap_primary output. | Outside client scope: M |
| [v20.2.4/src/test/osd/TestOSDMap.cc::OSDMapTest.read_balance_large_map](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/osd/TestOSDMap.cc#L2940) | Large-map primary balance optimization; same producer/consumer boundary, no port of the optimizer. | Outside client scope: M |
| [v20.2.4/src/test/osd/TestOSDMap.cc::OSDMapTest.read_balance_random_map](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/osd/TestOSDMap.cc#L3016) | Random-map primary balance optimization; preserve random setup/sample volume if optimizer enters scope. | Outside client scope: M |
| [v20.2.4/src/test/osd/TestOSDMap.cc::OSDMapTest.rb_osdsize_opt_1small_osd](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/osd/TestOSDMap.cc#L3101) | Heterogeneous-capacity optimal read-balance score, one small OSD; no native-client scoring API. | Outside client scope: M |
| [v20.2.4/src/test/osd/TestOSDMap.cc::OSDMapTest.rb_osdsize_opt_mixed_osds](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/osd/TestOSDMap.cc#L3119) | Heterogeneous-capacity score with mixed OSD sizes; no client-side optimizer. | Outside client scope: M |
| [v20.2.4/src/test/osd/TestOSDMap.cc::OSDMapTest.rb_osdsize_opt_1large_osd](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/osd/TestOSDMap.cc#L3136) | Capacity score with one large OSD; no client-side optimizer. | Outside client scope: M |
| [v20.2.4/src/test/osd/TestOSDMap.cc::OSDMapTest.rb_osdsize_opt_1large_mixed_osds](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/osd/TestOSDMap.cc#L3147) | Capacity score with a large and mixed-size OSDs; no client-side optimizer. | Outside client scope: M |
| [v20.2.4/src/test/osd/TestOSDMap.cc::OSDMapTest.rb_osdsize_opt_score](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/osd/TestOSDMap.cc#L3166) | Cross-check heterogeneous optimal score; preserve scenario if balancer scope changes. | Outside client scope: M |
| [v20.2.4/src/test/osd/TestOSDMap.cc::OSDMapTest.pgtemp_primaryfirst](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/osd/TestOSDMap.cc#L3234) | Tentacle EC optimized/nonoptimized and pg_temp present/absent; six shards, every even nonprimary mask 2..62, stable grouping and inverse shard mapping. Existing PgPool vector/undo APIs allow equivalent assertions; document deriving forward indices from production vector output. Local tests cover only selected layouts, not the full mask sweep. | Ready now (Rust API adaptation) |

### Other adjacent suites

Identifiers for unnamed script sections below are locally assigned; each link points at its actual section or file entry point. A grouped outside-scope script entry covers all its operations, not multiple claimed Rust tests.

| Source / locally assigned case | Disposition and what is needed |
| --- | --- |
| [v17.2.7/src/test/librados/pool.cc::LibRadosPools.PoolCreateWithCrushRule](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/librados/pool.cc#L124)<br>[v20.2.4/src/test/librados/pool.cc::LibRadosPools.PoolCreateWithCrushRule](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/librados/pool.cc#L125) | Blocked: C. Create pool with explicit rule 0 and delete it, with original cluster setup/cleanup. Rust create_pool accepts a rule ID; the missing live gate is not a CRUSH mapper failure. |
| [v17.2.7/src/test/cli/osdmaptool/crush.t::export-import-adjust](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/osdmaptool/crush.t#L1)<br>[v20.2.4/src/test/cli/osdmaptool/crush.t::export-import-adjust](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/osdmaptool/crush.t#L1) | Outside client scope: E; client decode subset blocked F+P. All five commands: createsimple, export, import, adjust weight without/with save. Pin release-specific encoded-size/output differences. |
| [v17.2.7/src/test/cli/osdmaptool/create-print.t::create-export-print](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/osdmaptool/create-print.t#L1)<br>[v20.2.4/src/test/cli/osdmaptool/create-print.t::create-export-print](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/osdmaptool/create-print.t#L1) | Outside client scope: E; capture original exported map and test client decode (F). Decompile/print/clobber assertions are not library APIs. |
| [v17.2.7/src/test/cli/osdmaptool/create-racks.t::rack-map-placement](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/osdmaptool/create-racks.t#L1)<br>[v20.2.4/src/test/cli/osdmaptool/create-racks.t::rack-map-placement](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/osdmaptool/create-racks.t#L1) | Blocked: F+P for --test-map-pg 0.0; original ceph.conf.withracks supplies hierarchy. Preserve exact expected mapping; create/print/clobber portions are E. |
| [v17.2.7/src/test/cli/osdmaptool/test-map-pgs.t::all-pg-placement](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/osdmaptool/test-map-pgs.t#L1)<br>[v20.2.4/src/test/cli/osdmaptool/test-map-pgs.t::all-pg-placement](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/osdmaptool/test-map-pgs.t#L1) | Blocked: F+P. Preserve 500 OSDs, size 3, pg_bits=4, STRAW node/rack topology, all PG counts and CRUSH-vs-random branches. CLI stats formatting/random comparator are E; use the original CRUSH mapping workload for client validation. |
| [v17.2.7/src/test/cli/osdmaptool/tree.t::tree-formats](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/osdmaptool/tree.t#L1)<br>[v20.2.4/src/test/cli/osdmaptool/tree.t::tree-formats](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/osdmaptool/tree.t#L1) | Outside client scope: E. Plain/JSON tree formatting; captured tree metadata can supply a decode subset (F), not a formatter port. |
| [v17.2.7/src/test/cli/osdmaptool/clobber.t::clobber-fsid](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/osdmaptool/clobber.t#L1)<br>[v20.2.4/src/test/cli/osdmaptool/clobber.t::clobber-fsid](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/osdmaptool/clobber.t#L1) | Outside this CRUSH inventory: file clobber/FSID preservation, not mapping assertions; belongs to tooling tests. |
| [v17.2.7/src/test/cli/osdmaptool/help.t::help](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/osdmaptool/help.t#L2)<br>[v20.2.4/src/test/cli/osdmaptool/help.t::help](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/osdmaptool/help.t#L2) | Outside client scope: E. CLI usage output, not native placement. |
| [v17.2.7/qa/workunits/mon/crush_ops.sh::management-sequence](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/qa/workunits/mon/crush_ops.sh#L1)<br>[v20.2.4/qa/workunits/mon/crush_ops.sh::management-sequence](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/qa/workunits/mon/crush_ops.sh#L1) | Outside client scope: M. Entire sequence: rule/class listing and idempotent create/rename/remove; in-use rule protection; buckets/link/unlink/orphans; move/find; reweight/subtree bounds; flat/positional/compat weight sets; class shadows and empty-bucket removal. Capture resulting maps for A/L/P consumer tests (F+C); do not claim these monitor assertions as client ports. |
| [v17.2.7/qa/workunits/rados/test_crushdiff.sh::compare-import-export](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/qa/workunits/rados/test_crushdiff.sh#L1)<br>[v20.2.4/qa/workunits/rados/test_crushdiff.sh::compare-import-export](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/qa/workunits/rados/test_crushdiff.sh#L1) | Outside client scope: E+M. Text/binary/live/offline export/compare/import; unchanged maps show zero movement, changed osd0 weight shows nonzero movement when OSD count>3. Capture original pairs (F+C) if validating client mapping; crushdiff statistics/import is a different API. |

### Search boundaries outside the core suites

- `src/test/librados/test.cc` and `test_cxx.cc` use CRUSH commands in EC
  setup/cleanup. `src/test/neorados/common_tests.cc` does the same;
  `neorados/pool.cc` has an optional rule parameter in its helper. These hits
  are not additional mapper assertions; I/O/pool suites need their own parity
  inventory. `LibRadosPools.PoolCreateWithCrushRule` is explicitly retained above.
- `src/test/osd/types.cc` contains interval/pool encoding tests that mention
  pool CRUSH rules; `TestECBackend.cc`, scrubber and I/O-sequence tests exercise
  server behavior. They do not replace the ordered placement tests above.
  `OSDMapTest.Create`, `parse_osd_id_list`, the three `blocklisting_*` cases
  and `PGTempMap.basic` belong to a separate map/data-structure inventory.
- `src/test/erasure-code/*` CRUSH rule-construction/profile tests are grouped
  as server/plugin rule producers outside client scope (M/E), with review
  pending. Their generated classic/MSR rules remain client inputs, already
  tracked in the mapper/CLI inventories; no claim is made that all EC profiles
  are validated. Revisit specific profiles when importing their generated rules.
- `src/test/cli-integration/balancer/misplaced.t`, dashboard CRUSH-rule API
  tests, monitor stretch tests, `qa/suites` balancer/thrash manifests and
  EC workload manifests are grouped as server/service/workload orchestration,
  outside this algorithm inventory. Preserve a separate live-client gate;
  being outside this inventory is not evidence of compatibility.
- Remaining textual hits under `src/test` include build registrations,
  fixtures, argument parsing, Python bindings, CephFS, configuration tests,
  stress workloads and unrelated uses of the word “crush.” A broad text
  hit alone is not counted as a CRUSH behavioral test.

## Recommended porting order and acceptance

1. **Existing binary-map goldens first:** the 12 ready `test-map-*` files
   cover 87,040 ordered mappings per release (seven 10,240-vector files and
   five 3,072-vector vary-r files). Copy exact binary inputs/expected output,
   preserve each command's tunables and weights, and fail with release,
   fixture, rule, x and replica count on a mismatch. Run without Ceph or a
   cluster after import. Do not reduce samples to make the suite green.
2. **Tentacle functional scenarios:** the 28 NORMAL/MSR `IndepTest` and
   `FirstnTest` variants are ported in Stage 3. Next port the four dedicated
   MSR tests using their original setups, then add both weight-distribution
   shell assertions. `straw2_stddev` may be ported as a diagnostic but must not
   inflate the count of assertion-based parity checks.
3. **Unblock reference preparation:** obtain pinned tools, capture original
   text maps and unequal-weight STRAW setup, and record RNG provenance.
   Resolve the Quincy type-123 adaptation explicitly, then port every Quincy
   case and the firstn/indep, bad-mappings and set-choose regression fixtures.
4. **Missing interfaces:** retain/select choose arguments (A), expose hierarchy
   queries (L/D), and add retry-counter observation for show-choose-tries.
   Import class/choose-args fixtures and both wire feature variants. A failing
   mapper with a runnable fixture is work ready to fix, not an excuse to skip it.
5. **Placement and live gates:** reproduce OSDMap setup/deltas for raw/up/acting,
   primary/affinity and EC positions. Replay class/weight-set transitions,
   then run the original live-client I/O/pool scenarios against both releases.
   Review every E/M disposition individually before declaring scope complete.

A port is complete only when original assertions/variants, source comments,
fixture provenance, execution evidence and the inventory status agree.
Exact goldens and functional tests complement each other; neither alone proves
every possible CRUSH input correct. No ignored/skipped/missing-input test may
count as a passing compatibility gate.

## Reproducing this inventory

Run from this repository with the Ceph checkout at `../ceph`; none of these
commands switches that checkout or needs a running cluster:

```sh
git rev-parse HEAD
git -C ../ceph rev-parse 'v17.2.7^{}' 'v20.2.4^{}'
git -C ../ceph grep -n -E '^TEST(_F|_P)?\(' v17.2.7 -- src/test/crush
git -C ../ceph grep -n -E '^TEST(_F|_P)?\(' v20.2.4 -- src/test/crush
git -C ../ceph ls-tree -r --name-only v17.2.7 src/test/cli/crushtool
git -C ../ceph ls-tree -r --name-only v20.2.4 src/test/cli/crushtool
git -C ../ceph grep -n -E '^  \$ |^\$ ' v20.2.4 -- src/test/cli/crushtool
git -C ../ceph diff v17.2.7 v20.2.4 -- src/test/crush src/test/cli/crushtool
git -C ../ceph grep -n -E '^function TEST_' v20.2.4 -- qa/standalone/crush qa/standalone/mon/osd-crush.sh src/test/test_crush_bucket.sh
git -C ../ceph grep -l -i -E 'crush|chooseleaf' v17.2.7 -- src/test qa/standalone qa/workunits
git -C ../ceph grep -l -i -E 'crush|chooseleaf' v20.2.4 -- src/test qa/standalone qa/workunits
rg -n '#\[test\]|fn test_|#\[ignore' rados/src/crush rados/tests
cargo test -p rados --lib crush:: --offline
git diff --check
```

Use `git show RELEASE:PATH` to read the complete original and
`git show RELEASE:PATH | shasum -a 256` to verify a fixture. Source anchors
are release-specific; do not assume matching line numbers across releases.
All fixture hashes and source anchors in this document were checked against
the pinned local Git objects. Ceph oracle/live execution remains **Not run**.
