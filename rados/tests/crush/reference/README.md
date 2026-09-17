# Local reference runners

These programs are written for rados-rs. They are **not copied Ceph tests**
and do not implement CRUSH. They construct inputs and link the unmodified
`src/crush/mapper.c` and `hash.c` from the pinned Ceph commits.

| Program | Purpose and input origin |
| --- | --- |
| [mapper-regressions.c](mapper-regressions.c) | Generates local regression scenarios, not named upstream tests. Reproduction and fixture provenance: [fixtures/README.md](../fixtures/README.md#generated-mapper-regressions). |
| [reference-check.c](reference-check.c) | Recreates the four dedicated Tentacle MSR test setups and the shared `crush_weights.sh` input; prints C results for comparison with the Rust ports. |
| [verify-reference.py](verify-reference.py) | Extracts pinned Ceph sources into a temporary directory, builds the C runner, and compares its results with an instrumented temporary copy of the Rust tests. |

For the functional reference comparison, run from the repository root:

```sh
cargo test -p rados --test crush --offline
python3 rados/tests/crush/reference/verify-reference.py ../ceph
```

The second command requires git, clang, rustc and a Ceph checkout containing
Quincy `b12291d110049b2f35e32e0de30d70e9a4c060d2` and Tentacle
`7f793731f1b39eb4f465e960113d2363c311b964`. It leaves that checkout unchanged.
No Ceph mapper source is vendored here; only an empty generated `acconfig.h`
is added during the standalone C build.

Cargo tests do not execute these helpers. Committed input/expected data live
in [fixtures](../fixtures); investigation notes live in
[.notes/crush](../../../../.notes/crush).
