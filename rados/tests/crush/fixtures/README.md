# Ceph CRUSH golden fixtures

All `.crushmap` and `.t` files here are unmodified copies of
`src/test/cli/crushtool/` from both pinned releases:

- v17.2.7: `b12291d110049b2f35e32e0de30d70e9a4c060d2`
- v20.2.4: `7f793731f1b39eb4f465e960113d2363c311b964`

Every imported file was compared byte for byte across these commits. One
copy serves both releases; this is 87,040 distinct expected mappings, not
174,080 independent cases. Expected results come from the committed Ceph
transcripts, never from the Rust implementation. No external tools or local
paths are needed to run `cargo test -p rados --test crush --offline`.
The additional generated regression fixture is documented separately below.

The first line of each `.t` retains the original crushtool command. Rust
loads the referenced binary map, applies those tunables and OSD weights,
executes the same rule for every seed and replica count, and checks exact
ordered vectors (including short results and NONE slots) and result-size
histograms. Console formatting and the map-modified advisory are CLI-only
and are not asserted. The helper checks the complete seed sequence and
sample count and internal histogram consistency before invoking the mapper,
so an early mapper failure cannot hide incomplete fixture data. See the
[execution history](../../../../.notes/crush/test-parity-history.md#stage-2-mapper-fixes) for the original
failures, mapper corrections and passing results.

Provenance: paths in the table are relative to `src/test/cli/crushtool/`;
links point to the exact releases above. Adaptations to fixture bytes: none.
`cmd-01` in test source comments is a locally assigned identifier for the
single unnamed CLI command in each transcript.

Ceph's upstream copyright/license notices are preserved in `COPYING`,
`COPYING-LGPL2.1` and `COPYING-LGPL3`, copied from v17.2.7. These third-party
fixtures retain their upstream licensing; the workspace MIT declaration
does not replace it.

| File | Source v17.2.7 / v20.2.4 | SHA256 |
| --- | --- | --- |
| `test-map-a.crushmap` | [Quincy](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/test-map-a.crushmap) / [Tentacle](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/test-map-a.crushmap) | `3eacd04c60fa0143e298c49f6609a5b405fc01fc668f80386da28a8ea40781ca` |
| `test-map-bobtail-tunables.t` | [Quincy](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/test-map-bobtail-tunables.t) / [Tentacle](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/test-map-bobtail-tunables.t) | `49302596753ab66180af0c2e06761bd5e59763f01df60730a5b5e122bd8fbc86` |
| `test-map-firefly-tunables.t` | [Quincy](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/test-map-firefly-tunables.t) / [Tentacle](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/test-map-firefly-tunables.t) | `cc296abd018b76820566c6ae448108b945c00fb57892ed923b26d006628b3b94` |
| `test-map-hammer-tunables.crushmap` | [Quincy](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/test-map-hammer-tunables.crushmap) / [Tentacle](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/test-map-hammer-tunables.crushmap) | `c1bf51b90ccdcc7dccef50a5d92384d444c44d75b7c3b4d915b8b30c03170b57` |
| `test-map-hammer-tunables.t` | [Quincy](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/test-map-hammer-tunables.t) / [Tentacle](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/test-map-hammer-tunables.t) | `411c3fe95a514e7f3890879e40f8f1851a987cf57220574c453e3683728d22f0` |
| `test-map-indep.crushmap` | [Quincy](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/test-map-indep.crushmap) / [Tentacle](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/test-map-indep.crushmap) | `7de3cd6cc1410bd0cf3729ba08c9572f4f6a9c9b696103f938b243e0a6682b9b` |
| `test-map-indep.t` | [Quincy](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/test-map-indep.t) / [Tentacle](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/test-map-indep.t) | `a0a91533a680350a67699b16380280de15108ec6e87e4f1e780cc6f73ad25732` |
| `test-map-jewel-tunables.crushmap` | [Quincy](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/test-map-jewel-tunables.crushmap) / [Tentacle](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/test-map-jewel-tunables.crushmap) | `4a54a318d1e9f33d1ed405edec87366add5e443433e4fcc40206f130fda46880` |
| `test-map-jewel-tunables.t` | [Quincy](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/test-map-jewel-tunables.t) / [Tentacle](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/test-map-jewel-tunables.t) | `268b70f78c0d6490a31c92a73545832f77f95cb16c44564d0c7a837dc038da0c` |
| `test-map-legacy-tunables.t` | [Quincy](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/test-map-legacy-tunables.t) / [Tentacle](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/test-map-legacy-tunables.t) | `9889916c3eace87448c043d6134b44347724e8207d7429dcb17950081336b3d3` |
| `test-map-tries-vs-retries.crushmap` | [Quincy](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/test-map-tries-vs-retries.crushmap) / [Tentacle](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/test-map-tries-vs-retries.crushmap) | `df6669db23b0184704bb7b9b99d3d184f5d1ece01ef1607008365ac6df5c3d15` |
| `test-map-tries-vs-retries.t` | [Quincy](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/test-map-tries-vs-retries.t) / [Tentacle](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/test-map-tries-vs-retries.t) | `fe39f33798696c16590154f2dbb7de4331873994fc1c2c41ca31642e103f9f5d` |
| `test-map-vary-r-0.t` | [Quincy](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/test-map-vary-r-0.t) / [Tentacle](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/test-map-vary-r-0.t) | `30c80294d774f6a90e078a79eb8c7acffdfd20d3d7ae092d74fe981d9b4790f5` |
| `test-map-vary-r-1.t` | [Quincy](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/test-map-vary-r-1.t) / [Tentacle](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/test-map-vary-r-1.t) | `f6fb2fed94167395b4efa80b7efb3b93c5fc14e93af183dcda19114723c2892b` |
| `test-map-vary-r-2.t` | [Quincy](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/test-map-vary-r-2.t) / [Tentacle](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/test-map-vary-r-2.t) | `d6602de4e9601beec2dbcb394761a3068b5e9fc22b56482577ca62d98f18ceae` |
| `test-map-vary-r-3.t` | [Quincy](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/test-map-vary-r-3.t) / [Tentacle](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/test-map-vary-r-3.t) | `294dbf3fe352e7a8c6ffda508832049ec1512841aad4dcd584d229ffb377e464` |
| `test-map-vary-r-4.t` | [Quincy](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/test-map-vary-r-4.t) / [Tentacle](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/test-map-vary-r-4.t) | `27d83afe8c9927849c9b7aa655620fea3658a97f392a9f4d8b7306c444095e24` |
| `test-map-vary-r.crushmap` | [Quincy](https://github.com/ceph/ceph/blob/b12291d110049b2f35e32e0de30d70e9a4c060d2/src/test/cli/crushtool/test-map-vary-r.crushmap) / [Tentacle](https://github.com/ceph/ceph/blob/7f793731f1b39eb4f465e960113d2363c311b964/src/test/cli/crushtool/test-map-vary-r.crushmap) | `3c8979266fd148244ddd3dd05455f460c2fa411c5d40f4d7d788227716ae2b57` |

## Generated mapper regressions

[`mapper-regressions.c`](../reference/mapper-regressions.c) is a local input generator,
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
fixtures = Path('rados/tests/crush/fixtures')
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
                   str(fixtures.parent / 'reference/mapper-regressions.c'), str(root / 'mapper.c'),
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
    (fixtures / 'mapper-regressions.txt').write_bytes(
        b''.join(outputs['tentacle'][case] for case in range(7)))
```
