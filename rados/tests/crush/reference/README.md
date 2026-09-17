# Local reference runners and generated data

These helpers are written for rados-rs. They are **not copied Ceph tests**
and do not implement CRUSH. The C runners construct inputs and link the unmodified
`src/crush/mapper.c` and `hash.c` from the pinned Ceph commits. The functional
runner also links `builder.c` and `crush.c` to verify STRAW construction.

| Program | Purpose and input origin |
| --- | --- |
| [mapper-regressions.c](mapper-regressions.c) | Generates local regression scenarios, not named upstream tests. Reproduction and output provenance: [generated mapper regressions](#generated-mapper-regressions). |
| [reference-check.c](reference-check.c) | Recreates the four dedicated Tentacle MSR test setups, the shared `crush_weights.sh` input and both `bad-mappings.t` cases; prints C results for comparison with the Rust ports. |
| [verify-reference.py](verify-reference.py) | Extracts pinned Ceph sources into a temporary directory, builds the C runner, and compares its results with an instrumented temporary copy of the Rust tests. |
| [prepare-cli.py](prepare-cli.py) | Verifies original inputs against both pinned commits, compiles three text maps with pinned Docker tools, checks all seven upstream test command outputs, and writes six binary maps here. |

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

For `bad-mappings`, the C runner uses the original text map's five devices,
unit weights, STRAW root -1/type 1, hash 0, and rule IDs 0/1 (replicated
FIRSTN and erasure INDEP). It applies the compiler's legacy tunables and calls
the pinned `crush_make_straw_bucket`, asserting all five straws equal 65536.
The audit compares both seed-1/ten-replica results with the actual Rust test,
which asserts the upstream transcript's complete ordered vectors. This checks
placement and builder setup, not the text compiler or binary decoder.

Cargo tests do not execute these helpers; they embed the generated
[mapper-regressions.txt](mapper-regressions.txt),
[mapper-retries.txt](mapper-retries.txt), and compiled maps stored here. Unmodified
upstream maps and transcripts live in [fixtures](../fixtures); investigation
notes live in [.notes/crush](../../../../.notes/crush).

## Pinned crushtool containers

For reference compilation on an ARM64 host, use these official Ceph images
by platform-specific digest. Docker (Colima on macOS) is sufficient; no Ceph
cluster is required. Downloading requires network access; the tool runs offline.

| Release | Expected source commit | Linux ARM64 image digest |
| --- | --- | --- |
| Quincy v17.2.7 | `b12291d110049b2f35e32e0de30d70e9a4c060d2` | `sha256:a70ccb2d8a0e814aa1c009e7541289b99bf039521727b6f44611499e9ca3fada` |
| Tentacle v20.2.4 | `7f793731f1b39eb4f465e960113d2363c311b964` | `sha256:6e6bc7b28fa1b334108a3646af5533dfb50db508efdf5b358eb7dd0dd37a48aa` |

```sh
quincy_image=quay.io/ceph/ceph@sha256:a70ccb2d8a0e814aa1c009e7541289b99bf039521727b6f44611499e9ca3fada
tentacle_image=quay.io/ceph/ceph@sha256:6e6bc7b28fa1b334108a3646af5533dfb50db508efdf5b358eb7dd0dd37a48aa
docker pull --platform linux/arm64 "$quincy_image"
docker pull --platform linux/arm64 "$tentacle_image"
docker run --rm --network none --entrypoint /bin/sh "$quincy_image" -ec 'ceph --version; rpm -qf /usr/bin/crushtool /usr/bin/ceph'
docker run --rm --network none --entrypoint /bin/sh "$tentacle_image" -ec 'ceph --version; rpm -qf /usr/bin/crushtool /usr/bin/ceph'
```

`crushtool` does not support `--version`; use `ceph --version` for the source
commit and check the owning packages for both tools. Verified on 2026-09-17:
Quincy uses `ceph-base`/`ceph-common` version `17.2.7-0.el8.aarch64`, Tentacle
uses `20.2.4-0.el9.aarch64`, and both source commits match the table.
Both tools compiled the original `bad-mappings.crushmap.txt` and reproduced
both `bad-mappings.t` vectors exactly, with networking disabled. Those smoke
checks used temporary container files and did not add generated fixtures.

Keep locally compiled maps and other generated data in `reference/`, with
the command, source inputs, tool version and SHA256;
`fixtures/` is reserved for unmodified files copied from upstream.

## Compiled CLI maps

Run from the repository root, with the above images already downloaded:

```sh
python3 rados/tests/crush/reference/prepare-cli.py ../ceph
```

The script checks every imported text map and transcript against both pinned
Git commits, verifies the container source commit and package versions, then
compiles each map using `crushtool -c /dev/stdin -o /dev/stdout`. Input bytes
come unchanged from `fixtures/`. It replays every original `--test` command,
changing only the input path to `-` to stream the compiled binary into the
container. All weights, tunables, rules, seeds, replica counts and output flags
are retained. Cram's `\t`/`(esc)` notation is decoded for output comparison.
Only after all seven command outputs match for both releases are files written.
No Rust code supplies any generated bytes or expected results.

The three source maps and transcripts are documented with immutable links and
SHA256 in [fixtures/README.md](../fixtures/README.md). `bad-mappings` and
`test-map-firstn-indep` each have two test commands; `set-choose` has three,
covering 36,864 vectors. Generated maps retain the source fixtures' upstream
license notices in `fixtures/COPYING*`.

All three Tentacle binaries equal the respective Quincy bytes followed by
two little-endian u32 MSR defaults (100/100). Both encodings are checked in
and decoded by Cargo tests, which need neither Docker nor a Ceph checkout.

| Generated file | Original text input | SHA256 |
| --- | --- | --- |
| `bad-mappings-quincy.crushmap` | `bad-mappings.crushmap.txt` | `e676cdf4743655ebc5a8efcef7cdb97a0203b3ce6b3bcc0000018adc5ddb6990` |
| `bad-mappings-tentacle.crushmap` | `bad-mappings.crushmap.txt` | `8d8338ee6bf406dbe5f6a7a1697136ce2e449628b25af0a57d342022887ab549` |
| `test-map-firstn-indep-quincy.crushmap` | `test-map-firstn-indep.txt` | `9d262b9c2781313e1d52af38b592358f104285a68c8ce4dff98b35817e5a81fe` |
| `test-map-firstn-indep-tentacle.crushmap` | `test-map-firstn-indep.txt` | `5c0cc6087199c4cf0e2af7e9bd4894ce256a5b5619d11e8f6e1d5c333363b90c` |
| `set-choose-quincy.crushmap` | `set-choose.crushmap.txt` | `39289b7b868a6aa73649749c035887c2e877c8ef1cc39e69f2174b4ad42dc731` |
| `set-choose-tentacle.crushmap` | `set-choose.crushmap.txt` | `8bc67cfd2d1fb08bd192946e271298553daf01d2971d113acc7daf9b89fcb549` |

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

[`mapper-retries.txt`](mapper-retries.txt) contains 1,000 rows for local cases
11..20: all-in inputs except cases 13..16, which mark OSD 0 out to exercise
recursive leaf retries. Every case retains the C result length and ordered
devices. Cases 11 and 12 verify that zero and negative values leave the prior
positive `choose_tries` or `chooseleaf_tries` override unchanged. Cases 13..16
cover positive and zero local/local-fallback retries with a negative repeat.
Cases 17..19 cover `vary_r` 2 or 0 and `stable` 0 with a negative repeat.
The C mapper computes a recursive seed as `r >> (vary_r - 1)` when `vary_r`
is nonzero; the generated cases use only 0 and 2, avoiding an undefined C
shift count while exercising both branches.

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
| 11 / 12 | Positive choose/chooseleaf retries survive zero and negative repeats. |
| 13 / 14 | Local retries: positive and zero values, each with a negative repeat. |
| 15 / 16 | Local fallback retries: positive and zero values, each with a negative repeat. |
| 17 / 18 | Recursive `vary_r`: positive and zero values with a negative repeat. |
| 19 | Recursive `stable=0` survives a negative repeat. |
| 20 | Direct FIRSTN ignores zero and negative choose-tries repeats. |

SHA256:

- Generator: `cd6130979dacc7153c6025c8e54f7c1a1beaafcc6d534469c3f7ce5591cbf96e`.
- Existing vectors: `526fbf16663e42265899c0405e0213996c8cd14f391c4aeb52cd8273f0c3127c`.
- Retry vectors: `9cde2e3b1d28361fd6a6549d2195893798085b1deaa3b24c48b37a6d3f34084e`.

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
        cases = (0, 1, 2, 4, *range(11, 21)) if release == 'quincy' else range(21)
        outputs[release] = {case: sp.check_output([str(root / 'run'), str(case)])
                            for case in cases}
    for case, data in outputs['quincy'].items():
        assert data == outputs['tentacle'][case]
    for case in (3, 7, 8, 9, 10):
        assert all(row.split()[3:] == [b'0']
                   for row in outputs['tentacle'][case].splitlines())
    (reference / 'mapper-regressions.txt').write_bytes(
        b''.join(outputs['tentacle'][case] for case in range(7)))
    (reference / 'mapper-retries.txt').write_bytes(
        b''.join(outputs['tentacle'][case] for case in range(11, 21)))
```
