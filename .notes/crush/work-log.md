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
