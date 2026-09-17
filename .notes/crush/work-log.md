# CRUSH working notes

The public [test status](../../rados/tests/crush/test-parity.md) contains
coverage and remaining work only. Keep investigation and execution history here.
The previous full document is preserved in [test-parity-history.md](test-parity-history.md).

## Continuation after d1f0dc8 — 2026-09-17

Scope: finish point 2 of the porting order (four dedicated Tentacle MSR
tests and two shared weight-distribution assertions), and move notes out of
the public document. No mapper changes were needed.

### References and setup

- Quincy: `b12291d110049b2f35e32e0de30d70e9a4c060d2`.
- Tentacle: `7f793731f1b39eb4f465e960113d2363c311b964`.
- `src/test/crush/crush.cc`, lines 1105–1569: read the complete cluster spec,
  hierarchy builder, host/OSD out helpers, mapping comparisons and all four
  dedicated MSR definitions. Quincy contains no MSR definitions.
- `src/test/crush/crush_weights.sh`: both pinned files are identical; read
  both assertion blocks, embedded map, seeds, replica counts and bc arithmetic.
- `CrushWrapper.h`: optimal means Jewel, MSR defaults are 100/100.
  Root and host STRAW2 IDs follow insertion order, including the second root.
- `CrushCompiler.cc::compile` resets to legacy tunables before parsing text.
  The shell's embedded map has no overrides. `CrushTester.cc` counts every
  non-NONE returned device over the inclusive seed interval.

Searches used `git show`/`git grep` at both commits, without switching the
shared checkout. Selected upstream tests were already fully enumerated in
the archived inventory; the new Rust tests retain adjacent immutable links.

### Port status

All six new scenarios have Coverage: Ported; Readiness: Ready now;
Verification: Passing. The Rust counterparts are in
`rados/tests/crush/functional.rs`, with the four upstream MSR names and
`one_replica_weight_distribution` / `three_replica_weight_distribution`.

- MSR 4 hosts x 3 OSDs: seed 0, choose 3 hosts x 1 OSD, result size 3;
  preserve mapped count after first selected host and first selected OSD fail.
- MSR 3 hosts x 2 OSDs: seed 0, choose 2 x 2, truncate to 3;
  replace positions 0..1 on another host and preserve position 2.
- MSR 5 hosts x 4 OSDs: seed 0, choose 4 x 4, truncate to 14;
  replace positions 0..3 on another host and preserve positions 4..13.
- Multi-root: populate ssd then hdd, each 4 hosts x 3 OSDs; choose 2 x 2
  per root, result size 8, seeds 0..999. Remove OSD positions 1/5 and, in a
  separate transition from all-in, hosts at positions 2/6. Preserve all
  original position-change assertions. Additional assertions inspect every
  item's root and every host in each group, rather than upstream's repeated
  lookup of out[start]. These strengthen coverage without replacing assertions.
- Both distribution checks retain all one million seeds, original weights,
  replica counts, legacy retry tunables and five-place truncated division.
  Setup is constructed directly; compiler/editor and console output are not
  claimed as ported APIs. Diagnostic printing is omitted.

### Validation

Environment: macOS, Apple clang 21.0.0 (`-std=gnu99 -O2`), rustc 1.97.0,
offline Cargo with default features for tests. Initial behavior:

- `cargo test -p rados --test crush functional::msr_ --offline`:
  4 passed, 0 ignored, before any mapper changes.
- `cargo test -p rados --test crush weight_distribution --offline`:
  2 passed, 0 ignored, before any mapper changes.
- `cargo test -p rados --test crush --offline`: 53 passed, 0 ignored.
- `cargo fmt --all -- --check` and `git diff --check`: passed.
- `cargo clippy --workspace --all-targets --all-features --offline -- --no-deps -D warnings`:
  blocked by the two pre-existing unused `OSD_STAT_INTERFACES_*` constants in
  `rados/src/osdclient/pgmap_types.rs`.
- `cargo clippy -p rados --lib --test crush --offline -- -D warnings -A dead-code`:
  passed. No repository lint configuration changed.

`python3 rados/tests/crush/reference/verify-reference.py ../ceph` rebuilt the pinned C
mappers with unmodified source/header bytes and empty acconfig.h. The
[C runner](../../rados/tests/crush/reference/reference-check.c) reproduces the inputs. The script instruments
a temporary copy of the actual Rust ports, not a separate Rust mapper.
All **3,007 exact MSR vectors** match Tentacle; all five device counts in
each distribution run match **both** references:

| Replicas | Device counts 0..4 |
| --- | --- |
| 1 | 243456, 243624, 244486, 243881, 24553 |
| 3 | 723984, 722923, 723153, 723394, 106546 |

The ordinary Cargo tests do not depend on this audit script, a Ceph checkout,
compilers other than Rust, or external fixtures. No live-cluster gate was run.

At the user's request, both local C runners and the comparison script were
placed in `rados/tests/crush/reference/`, with an explicit origin README.
`fixtures/` contains data and provenance only. The existing regression
generator moved byte-for-byte (its documented SHA256 is unchanged).
The comparison script was rerun successfully from its new path, and all
local documentation links were checked after relocating the notes.

### Next work and known gaps

Continue with point 3: prepare reference binaries for set-choose and the
remaining text-map regressions; preserve the upstream setup. The preceding
source audit confirmed ignored local retry opcodes and SetChooseTries=0
semantics with both C versions. Keep those as additional local regressions;
do not label the newly constructed examples upstream test ports.
Choose arguments are still discarded and belong to point 4.

The old corpus integration test's successful return when input is absent
must not count as an executed compatibility gate. Uniform/list/tree and
full OSDMap placement still need their listed coverage. Passing these six
tests does not close those gaps.

## Continuation after 94ecb01 — 2026-09-17

Started point 3 with the placement assertions in `bad-mappings.t`. The initial
offline suite passed all 53 tests. No local crushtool or Ceph container image
was available, so compiled binary fixtures remain pending. For this one-bucket,
equal-weight case, direct assembly preserves the mapper input without needing
a compiler; the unmodified C builder verifies the STRAW lengths independently.

### Source review and adaptation

Used `git show` at both pinned commits above to read the complete
`src/test/cli/crushtool/bad-mappings.t` and `bad-mappings.crushmap.txt`.
`git diff <Quincy> <Tentacle> -- src/test/cli/crushtool/{bad-mappings.t,bad-mappings.crushmap.txt,set-choose.t,set-choose.crushmap.txt,test-map-firstn-indep.t,test-map-firstn-indep.txt}`
confirmed all six inputs/transcripts are unchanged. A byte comparison of
both imported files also passed; hashes and immutable source links are in
the fixtures README. The archived inventory remains the broader search record.

Read `CrushCompiler.cc::compile` and bucket parsing, both releases'
`CrushWrapper.h::set_tunables_argonaut/legacy`, and
`builder.c::crush_calc_straw/crush_make_straw_bucket` (unchanged between
the references). Compiler defaults are local tries 2, fallback 5, total 19,
descend_once/vary_r/stable 0, straw_calc_version 0. All five equal unit-weight
items have straw length 65536. Rule IDs 0/1, replicated/erasure types,
TAKE(-1), CHOOSE_FIRSTN/INDEP(0, osd), EMIT, seed 1 and ten replicas are retained.

`functional::bad_mappings` covers both locally named cmd-02/cmd-03 placement
cases. Expected vectors are copied from the original transcript, including
the FIRSTN short result and all five trailing INDEP NONE slots. The map is
assembled directly; names, compiler output, CLI formatting and temporary-file
cleanup are not exercised. Binary decoding remains an explicit coverage gap.
No Rust production code changed; the new test passed on its first execution.

The existing audit runner now links unmodified `builder.c` and `crush.c`
alongside mapper/hash, calls the original STRAW constructor and checks each
straw. Its two ordered outputs agree with both the Rust port and the original
transcript for both references. No generated data or C implementation is
vendored; only the two original upstream fixtures were added.

### Validation and remaining work

- `cargo test -p rados --test crush functional::bad_mappings --offline`:
  1 passed, covering both original mapping commands.
- `cargo test -p rados --test crush --offline`: 54 passed, none ignored.
- `python3 rados/tests/crush/reference/verify-reference.py ../ceph`: passed;
  two bad-mappings vectors and STRAW construction match both references;
  existing 3,007 MSR vectors and both million-seed distribution comparisons
  also remain passing.
- `cargo fmt --all -- --check` and `git diff --check`: passed.
- `cargo clippy --workspace --all-targets --all-features --offline -- --no-deps -D warnings`:
  blocked by the same two pre-existing unused `OSD_STAT_INTERFACES_*` constants
  in `rados/src/osdclient/pgmap_types.rs`.
- `cargo clippy -p rados --lib --test crush --offline -- -D warnings -A dead-code`:
  passed; no lint configuration changed.

Next: obtain pinned crushtool binaries for `set-choose`, `test-map-firstn-indep`
and bad-mappings decode coverage. Preserve all 36,864 set-choose vectors,
six rules, three weight profiles and both replica counts. Known ignored local
retry overrides and nonpositive `SetChooseTries` semantics remain separate
mapper regressions to add. Quincy-specific unit cases and STRAW/STRAW2 weight
scenarios, point 4 interfaces and point 5 OSDMap/live gates remain open.

## Compiled CLI ports after 240e078 — 2026-09-17

Continued point 3 using the pinned ARM64 crushtool containers documented in
`reference/README.md`. Imported four unchanged upstream files: set-choose text
map/transcript and test-map-firstn-indep text map/transcript. Both pinned
commits contain byte-identical inputs; no test fixture was invented or edited.

### Source review and reference preparation

Read the complete maps, all command blocks, rule definitions, weight profiles,
seed/replica ranges and result-size histograms. The firstn/indep map's duplicate
`rack1` entry is intentional input and was retained. `git show`/`git diff` at
both pinned commits confirmed identities. `git grep -l -E
'set_choose_local_tries|set_choose_local_fallback_tries|bad-mappings|test-map-firstn-indep'`
searched `src/test/crush`, `src/test/cli/crushtool` and `qa/standalone/crush`
for both releases; selected cases agree with the archived upstream inventory.
Read both mappers' rule override handling and FIRSTN retries, and both
`CrushTester.cc` bad-mapping predicates (short vector or any NONE slot).

`reference/prepare-cli.py ../ceph` verifies all six text inputs/transcripts
against the pinned Git contents and confirms each image's source commit and
matching ceph-base/ceph-common package version. It streams each original map
through `crushtool -c /dev/stdin -o /dev/stdout`, then replays every original
test command with the generated binary on stdin. All seven complete outputs
match for each release, including Cram-escaped histogram tabs and advisories.
Only input/output transport changes; no mapper or expected output is adapted.

Six generated binary maps are stored in `reference/`, separately from original
`fixtures/`, with SHA256 and reproduction commands. Each Tentacle map equals
the Quincy bytes plus two u32 MSR defaults 100/100; Rust decodes both variants
and checks there are no unconsumed bytes. Ordinary Cargo tests remain offline.

### Rust ports and results

- `golden::bad_mappings_compiled`: original rules 0/1, seed 1, ten replicas,
  exact vectors including the five INDEP NONE slots; closes binary decode
  coverage for the earlier direct-assembly port. The existing functional test
  remains a second setup for the same two cases, not additional upstream cases.
- `golden::firstn_indep_compiled`: rules 0/1, seed 1, replicas 1..10; checks
  the complete bad-mapping report, including ten exact vectors and absence of
  errors for the ten successful calls. No exact success vectors are invented.
- `golden::set_choose_all_in`, `set_choose_out_devices`,
  `set_choose_partial_weights`: all 36,864 vectors, all six rules, both replica
  counts, all original weights and statistics. OSD 6's 0.1 weight is truncated
  to 6553 as in crushtool. Extended the existing golden helper to handle rule
  ranges and explicit weights while retaining complete sequence/count/histogram
  validation before invoking the mapper.

All five new tests passed on their first execution, with no production change.
Passing set-choose does not resolve the known ignored local retry opcodes:
its fallback override equals the map's value 2, and its local retry value 2
is already covered by that fallback budget. Separate discriminating local
regressions and nonpositive SetChooseTries cases remain necessary.

Validation (macOS ARM64, pinned Linux ARM64 containers):

- `python3 rados/tests/crush/reference/prepare-cli.py ../ceph`: all seven
  command outputs match both releases; six compiled maps written.
- `cargo test -p rados --test crush golden:: --offline`: 17 passed initially.
- `cargo test -p rados --test crush --offline`: 59 passed, none ignored.
- `cargo fmt --all -- --check`, `git diff --check`, local documentation link
  checks and documented fixture/map SHA256 checks: passed.
- Full workspace/all-target/all-feature Clippy remains blocked by the two
  pre-existing `OSD_STAT_INTERFACES_*` unused constants in pgmap_types.rs.
  `cargo clippy -p rados --lib --test crush --offline -- -D warnings -A dead-code`
  passed. No lint configuration changed.

Next in point 3: discriminatory retry regressions, Quincy-specific unit cases,
STRAW zero/perturbed weights and STRAW2 reweight. Choose arguments, device-class
queries, retry counters, full OSDMap placement and live-cluster gates remain
open; these CLI ports do not establish complete CRUSH compatibility.

## Conventional retry overrides after 3919ba6 — 2026-09-17

Completed the retry-regression row in continuation order 3. This adds local
C-reference cases only; it is not a port of an invented upstream test name.

### Source review and reference preparation

- Used `git -C ../ceph grep -n -i -E 'choose.*tries|tries.*choose|chooseleaf.*(vary|stable)' <commit> -- src/test/crush src/test/cli/crushtool qa` for both pinned commits. The complete outputs are `.renchik/test-parity/task-1-upstream-quincy.log` and `task-1-upstream-tentacle.log`. `src/test/crush/crush.cc` has `SetChooseLeafTries` cases; no upstream test covers all nonpositive/repeated conventional override variants.
- Read both pinned `src/crush/mapper.c::crush_do_rule_no_retry` switch blocks and every Rust `crush_choose_firstn` caller. Quincy lines 941–969 and Tentacle lines 890–918 give the signed guards: choose/chooseleaf tries require `arg1 > 0`; local/local-fallback/vary-r/stable require `arg1 >= 0`.
- The C recursive FIRSTN path uses `r >> (vary_r - 1)` only when `vary_r` is nonzero. Cases use 0 and 2, avoiding undefined C shift counts. `mapper-regressions.c` linked against unchanged mapper/hash sources at both commits generated 1,000 new rows; outputs were byte-identical.

### TDD and implementation

`cargo test -p rados --test crush regressions::rule_retry_overrides_match_ceph --offline` initially failed before mapper changes: case 11, seed 0 returned Rust `[3, 0, 1, 2]` instead of pinned-C `[3, 0]`; see `.renchik/test-parity/task-1-red.log`. The corrected focused command passed all 1,000 C vectors; see `task-1-green-focused.log`.

The conventional executor now ignores nonpositive choose overrides, retains
per-rule local retry/fallback state, and passes that state into each recursive
FIRSTN call. Nonnegative vary-r/stable values retain their full unsigned value
inside the mapper; a negative repeat no longer truncates into a behavior-changing
byte. `mapper-retries.txt` records all exact ordered results and lengths.

### Remaining work

This closes only the zero/nonpositive conventional retry-setting row. Remaining
order-3 Quincy mapper cases and STRAW/STRAW2 weight cases, plus later choose
arguments, location/retry observability, OSDMap and live-cluster work, remain
open.

### Review follow-up

Direct FIRSTN cases 21..24 now cover local and local-fallback retries with
the same positive, zero, negative and repeated guards. Cases 21/22 use an
all-in collision input; cases 23/24 mark OSD 0 out. Both positive/zero pairs
produce different ordered C vectors, so the direct path observes each setting.
The vector fixture now has 1,400 rows. The rerun reference log records every
pinned `git show`, clang build and C runner invocation together with the
per-case SHA256 and cross-release comparison result in
`.renchik/test-parity/task-1-round1-reference.log`.

## Quincy mapper and STRAW weights after 930aa2e — 2026-09-17

The historical inventory's Quincy FIRSTN wording was incorrect. Those five
cases are `CHOOSELEAF_INDEP`; current status corrects this without rewriting
the archived snapshot. Both pinned `CrushWrapper::create()` paths select Jewel
tunables and `straw_calc_version=1`.

Existing NORMAL INDEP tests retain all source inputs/assertions and now cite
Quincy. `reference-check.c` builds their hierarchy with raw type 123 and type
3, compares every ordered output, then `verify-reference.py` compares actual
Rust ports across 100, 100, 100, 100, and 108 calls. Both references produce
`558f98bbf0b260dd`, `bf44299af70e19fe`, `86205296e425f70b`,
`2496ef9ab5778af4`, and `d5f68f99ffabd047`. This proves only the test-harness
Erasure mapping; raw-123 decode remains unported.

`weights.rs` ports `straw_zero` (10,000), `straw_same` (100,000), and
`straw2_reweight` (1,000,000). Pinned `builder.c` supplies STRAW lengths.
Both C implementations and Rust have complete-output digests
`12722a47fde289ef`, `32c9040dd1425108` (12 differences), and
`65e72f17e5a64b0b`. Reference libc reports unseeded `rand()%10 == 7`, yielding
the original integer-divided weight 45871. `straw2_stddev` is Not ported: it
only prints diagnostics. `task-2-initial-focused.log` records 3 focused passes
in 7.04s before any production change. The separately retained
`task-2-reference-initial.log` is only a C/Rust audit-harness evolution mismatch
while new C scenarios preceded their Rust comparison, never a behavior failure.

### Review fix round 1

The review found that Rust copied its legacy STRAW lengths into map fixtures but
the audit discarded C's emitted arrays. `verify-reference.py` now captures both
pinned C `STRAW` lines, reads the arrays from the executing audit-mode Rust
maps, asserts equality, and preserves those C lines in its successful output.
The original full-output digest check remains. `straw_zero`, `straw_same`, and
`straw2_reweight` now cite both immutable source definitions; their setup and
assertions are identical between Quincy and Tentacle. Focused test logs retain
the two pre-existing `pgmap_types.rs` dead-code warnings as accepted unrelated
noise; no production warning suppression changed.
