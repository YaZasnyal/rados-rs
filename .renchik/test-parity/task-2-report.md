# Task 2 report — Quincy mapper and STRAW weights

Status: DONE. Base `930aa2e`; no production CRUSH changes.

## Implementation

- Attached `weights.rs`; ported Quincy `CRUSHTest.straw_zero` (10,000),
  `straw_same` (100,000), and `straw2_reweight` (1,000,000), retaining their
  original assertions, inputs, integer division order, and item 14 `0x300000`.
- Captured unmodified pinned `builder.c` STRAW lengths with
  `straw_calc_version=1`. The standalone C reference records the macOS/Apple
  clang 21.0.0 libc realization `rand()%10 == 7`, so item 1 becomes 45871.
- Reused the five existing NORMAL INDEP tests only. Adjacent Quincy links state
  the mapping-only raw-123 to Erasure adaptation; no `RuleType` decoder change
  and no duplicate counting tests.

## C and type-adaptation evidence

`python3 rados/tests/crush/reference/verify-reference.py ../ceph` rebuilt
unmodified mapper/hash/builder/crush C sources from Quincy
`b12291d110049b2f35e32e0de30d70e9a4c060d2` and Tentacle
`7f793731f1b39eb4f465e960113d2363c311b964`. For each full Quincy domain it
compares raw type 123 with type 3 before hashing, then compares the actual
Rust NORMAL output stream: 100 `toosmall`, 100 `basic`, 100 `out_alt`, 100
`out_contig`, and 108 progressive calls. Digests are respectively
`558f98bbf0b260dd`, `bf44299af70e19fe`, `86205296e425f70b`,
`2496ef9ab5778af4`, `d5f68f99ffabd047` in both C releases and Rust.

The C helper initially emitted one result because its shared weight helper
assigned all hierarchy buckets type 1. Corrected source setup is root type 5,
rack type 3, host type 1; child IDs, items and weights remain source-shaped.
This was reference-fixture preparation, never a production failure.

The same audit compares complete ordered-output word-FNV-1a streams for C and
Rust: `straw_zero` `12722a47fde289ef`, `straw_same`
`32c9040dd1425108` (12/100,000 differing), and `straw2_reweight`
`65e72f17e5a64b0b`. The digest starts at `1469598103934665603`; for each
vector it XORs/multiplies length then each u32 item by `1099511628211`.

## Execution evidence

- Initial focused test: `.renchik/test-parity/task-2-initial-focused.log`.
  Its initial legacy-STRAW placeholder-length RED was fixed by the C-built
  setup; no production behavior changed. `task-2-reference-initial.log` is a
  harness-evolution mismatch before ports existed, not a behavior failure.
- Focused final: `task-2-green-focused.log` — 3 passed.
- Full: `task-2-crush.log` — 63 passed, 0 failed, 0 ignored.
- C audit: `task-2-reference.log` — passed, including all counts above.
- Formatting/diff: `task-2-fmt.log`, `task-2-diff-check.log` — passed.
- Lint: `task-2-clippy.log` — passed with the existing `-A dead-code` scope.

## Disposition and concerns

`straw2_stddev` remains Not ported: upstream prints diagnostic standard
deviations and supplies no acceptance assertion. Full CRUSH coverage remains
open; status and work-log preserve that limitation. The source-audit search
commands/results remain in `task-2-source-audit.md`.
