"""Audit functional ports against pinned C sources; not required by Cargo tests.

Run from the repository root after cargo test -p rados --test crush:
    python3 rados/tests/crush/reference/verify-reference.py ../ceph
Requires git, clang, rustc and the two commits in the Ceph checkout.
"""
from pathlib import Path
import subprocess as sp
import sys
import tempfile

ceph = Path(sys.argv[1]).resolve()
reference = Path(__file__).resolve().parent
repo = reference.parents[3]
refs = {
    "quincy": "b12291d110049b2f35e32e0de30d70e9a4c060d2",
    "tentacle": "7f793731f1b39eb4f465e960113d2363c311b964",
}

with tempfile.TemporaryDirectory(prefix="crush-functional-reference-") as temporary:
    root = Path(temporary)
    outputs = {}
    for release, revision in refs.items():
        dest = root / release
        (dest / "include").mkdir(parents=True)
        (dest / "crush").mkdir()
        for name in ("crush/mapper.c", "crush/mapper.h", "crush/hash.c", "crush/hash.h",
                     "crush/crush.h", "crush/crush_ln_table.h", "crush/crush_compat.h",
                     "crush/builder.c", "crush/builder.h", "crush/crush.c", "include/int_types.h"):
            data = sp.check_output(["git", "-C", str(ceph), "show",
                                    f"{revision}:src/{name}"])
            (dest / name).write_bytes(data)
        (dest / "acconfig.h").touch()
        command = ["clang", "-std=gnu99", "-O2", f"-I{dest}", f"-I{dest / 'crush'}",
                   str(reference / "reference-check.c"), str(dest / "crush/mapper.c"),
                   str(dest / "crush/hash.c"), str(dest / "crush/builder.c"),
                   str(dest / "crush/crush.c"), "-lm", "-o", str(dest / "run")]
        if release == "quincy":
            command.append("-DQUINCY")
        sp.run(command, check=True)
        outputs[release] = sp.check_output([str(dest / "run")], text=True).splitlines()

    # Instrument a temporary copy so the checked-in ports remain assertion-only.
    source = (repo / "rados/tests/crush/functional.rs").read_text()
    source = source.replace("    out\n}",
        '    println!("ORACLE devices={} x={x} count={count} weights={weights:?} out={out:?}", map.max_devices);\n    out\n}', 1)
    source = source.replace("    // bc scale=5", '    println!("COUNTS replicas={replicas} {counts:?}");\n    // bc scale=5', 1)
    source = source.replace('        assert_eq!(out, expected, "rule {rule_id}");',
        '        println!("BAD_MAPPINGS rule={rule_id} out={out:?}");\n        assert_eq!(out, expected, "rule {rule_id}");', 1)
    (root / "functional.rs").write_text(source)
    deps = repo / "target/debug/deps"
    rlib = max(deps.glob("librados-*.rlib"), key=lambda p: p.stat().st_mtime)
    sp.run(["rustc", "--edition=2024", "--test", str(root / "functional.rs"),
            "-L", f"dependency={deps}", "--extern", f"rados={rlib}",
            "-o", str(root / "rust-tests")], check=True)
    actual = []
    for selection in ("msr_", "weight_distribution", "bad_mappings"):
        output = sp.check_output([str(root / "rust-tests"), selection,
                                  "--nocapture", "--test-threads=1"], text=True)
        for line in output.splitlines():
            for marker in ("ORACLE ", "COUNTS ", "BAD_MAPPINGS "):
                if marker in line:
                    actual.append(marker + line.split(marker, 1)[1])
    assert sum(line.startswith("ORACLE ") for line in actual) == 3007
    assert sum(line.startswith("BAD_MAPPINGS ") for line in actual) == 2
    assert sorted(actual) == sorted(outputs["tentacle"]), "Rust/Tentacle mismatch"
    shared = sorted(line for line in actual if not line.startswith("ORACLE "))
    assert shared == sorted(outputs["quincy"]), "Quincy/Tentacle mismatch"
    print("All 3,007 MSR vectors match Tentacle; all five device counts match both releases.")
    print("Both bad-mappings vectors and the STRAW builder setup match both releases.")
    print("\n".join(shared))
