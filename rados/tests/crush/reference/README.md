# Local reference runners and generated data

These programs are written for rados-rs. They are **not copied Ceph tests**
and do not implement CRUSH. They construct inputs and link the unmodified
`src/crush/mapper.c` and `hash.c` from the pinned Ceph commits.

| Program | Purpose and input origin |
| --- | --- |
| [mapper-regressions.c](mapper-regressions.c) | Generates local regression scenarios, not named upstream tests. Reproduction and output provenance: [generated mapper regressions](#generated-mapper-regressions). |
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

Cargo tests do not execute these helpers; they embed the generated
[mapper-regressions.txt](mapper-regressions.txt) stored here. Unmodified
upstream maps and transcripts live in [fixtures](../fixtures); investigation
notes live in [.notes/crush](../../../../.notes/crush).

## Generated mapper regressions

[`mapper-regressions.c`](mapper-regressions.c) is a local input generator,
linked with unmodified Ceph `src/crush/mapper.c` and `hash.c`. It is not an
upstream test port and contains no Rust-derived expected values. Sources:
[Quincy mapper](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/crush/mapper.c),
[Tentacle mapper](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/crush/mapper.c).
The rule steps, bucket IDs, weights, tunables and output limits are explicit
in the generator and mirrored in [`regressions.rs`](../regressions.rs).

[`mapper-regressions.txt`](mapper-regressions.txt) contains 11,200 rows:
`scenario out_mask seed result_length [ordered device IDs...]`. Each of
cases 0..6 covers all 16 availability masks of a two-host/two-OSD-per-host
STRAW2 map and seeds 0..99. Empty results and `CRUSH_ITEM_NONE` are preserved.
Cases 0, 1, 2 and 4 were generated independently with both releases and are
byte-identical (6,400 vectors). MSR cases 3, 5 and 6 apply only to Tentacle.
Cases 7..10 check additional malformed MSR blocks; their results are always
empty and are asserted directly instead of storing another 6,400 empty rows.
No negative MSR fanout is passed to the C oracle: that input has no safe C
contract and is covered by Rust validation tests.

| Case | Behavior |
| --- | --- |
| 0 | Chained INDEP skips unresolved failure domains. |
| 1 | FIRSTN honors explicit chooseleaf retries even with descend_once enabled. |
| 2 | A conventional rule skips CHOOSE_MSR. |
| 3 | MSR rejects a conventional CHOOSE step with an empty result. |
| 4 | EMIT clears its working set. |
| 5 / 6 | MSR FIRSTN / INDEP: fanout 2 x 2, truncated to three results, all availability masks. |
| 7 / 8 | MSR block missing TAKE / EMIT. |
| 9 | An invalid second MSR block discards output from the valid first block. |
| 10 | Device TAKE followed by CHOOSE_MSR is invalid. |

SHA256:

- Generator: `6e93c0fe5c21313a5162f9c36ca87b6198ea8da08ae3dacf2fd0796f61edece9`.
- Vectors: `526fbf16663e42265899c0405e0213996c8cd14f391c4aeb52cd8273f0c3127c`.

Generation environment: macOS, Apple clang 21.0.0 (clang-2100.1.1.101),
`-std=gnu99 -O2`. The only build adaptation is an empty generated
`acconfig.h` for these standalone C files. Source and header bytes, including
`include/int_types.h`, are taken directly from the pinned commits.

Reproduce from the Rust repository root with Python 3, clang and a Ceph Git
checkout containing both commits (adjust `ceph` below). This does not change
that checkout. Only regeneration needs these tools; Cargo embeds the result.

```python
from pathlib import Path
import subprocess as sp
import tempfile

ceph = Path('../ceph')
reference = Path('rados/tests/crush/reference')
revisions = {
    'quincy': 'b12291d110049b2f35e32e0de30d70e9a4c060d2',
    'tentacle': '7f793731f1b39eb4f465e960113d2363c311b964',
}
outputs = {}
with tempfile.TemporaryDirectory() as directory:
    for release, revision in revisions.items():
        root = Path(directory) / release
        (root / 'include').mkdir(parents=True)
        for name in ('mapper.c', 'mapper.h', 'hash.c', 'hash.h', 'crush.h',
                     'crush_ln_table.h', 'crush_compat.h'):
            (root / name).write_bytes(sp.check_output(
                ['git', '-C', str(ceph), 'show', f'{revision}:src/crush/{name}']))
        (root / 'include/int_types.h').write_bytes(sp.check_output(
            ['git', '-C', str(ceph), 'show', f'{revision}:src/include/int_types.h']))
        (root / 'acconfig.h').touch()
        command = ['clang', '-std=gnu99', '-O2', f'-I{root}',
                   str(reference / 'mapper-regressions.c'), str(root / 'mapper.c'),
                   str(root / 'hash.c'), '-o', str(root / 'run')]
        if release == 'quincy':
            command.append('-DQUINCY')
        sp.run(command, check=True)
        cases = (0, 1, 2, 4) if release == 'quincy' else range(11)
        outputs[release] = {case: sp.check_output([str(root / 'run'), str(case)])
                            for case in cases}
    for case, data in outputs['quincy'].items():
        assert data == outputs['tentacle'][case]
    for case in (3, 7, 8, 9, 10):
        assert all(row.split()[3:] == [b'0']
                   for row in outputs['tentacle'][case].splitlines())
    (reference / 'mapper-regressions.txt').write_bytes(
        b''.join(outputs['tentacle'][case] for case in range(7)))
```
