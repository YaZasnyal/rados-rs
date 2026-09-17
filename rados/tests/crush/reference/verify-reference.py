"""Audit six new ports against pinned C sources; not required by Cargo tests.

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
        for name in ("mapper.c", "mapper.h", "hash.c", "hash.h", "crush.h",
                     "crush_ln_table.h", "crush_compat.h", "../include/int_types.h"):
            data = sp.check_output(["git", "-C", str(ceph), "show",
                                    f"{revision}:src/crush/{name}" if not name.startswith("..")
                                    else f"{revision}:src/include/int_types.h"])
            (dest / ("include/int_types.h" if name.startswith("..") else name)).write_bytes(data)
        (dest / "acconfig.h").touch()
        command = ["clang", "-std=gnu99", "-O2", f"-I{dest}",
                   str(reference / "reference-check.c"), str(dest / "mapper.c"),
                   str(dest / "hash.c"), "-o", str(dest / "run")]
        if release == "quincy":
            command.append("-DQUINCY")
        sp.run(command, check=True)
        outputs[release] = sp.check_output([str(dest / "run")], text=True).splitlines()

    # Instrument a temporary copy so the checked-in ports remain assertion-only.
    source = (repo / "rados/tests/crush/functional.rs").read_text()
    source = source.replace("    out\n}",
        '    println!("ORACLE devices={} x={x} count={count} weights={weights:?} out={out:?}", map.max_devices);\n    out\n}', 1)
    source = source.replace("    // bc scale=5", '    println!("COUNTS replicas={replicas} {counts:?}");\n    // bc scale=5', 1)
    (root / "functional.rs").write_text(source)
    deps = repo / "target/debug/deps"
    rlib = max(deps.glob("librados-*.rlib"), key=lambda p: p.stat().st_mtime)
    sp.run(["rustc", "--edition=2024", "--test", str(root / "functional.rs"),
            "-L", f"dependency={deps}", "--extern", f"rados={rlib}",
            "-o", str(root / "rust-tests")], check=True)
    actual = []
    for selection in ("msr_", "weight_distribution"):
        output = sp.check_output([str(root / "rust-tests"), selection,
                                  "--nocapture", "--test-threads=1"], text=True)
        for line in output.splitlines():
            for marker in ("ORACLE ", "COUNTS "):
                if marker in line:
                    actual.append(marker + line.split(marker, 1)[1])
    assert sum(line.startswith("ORACLE ") for line in actual) == 3007
    assert sorted(actual) == sorted(outputs["tentacle"]), "Rust/Tentacle mismatch"
    counts = sorted(line for line in actual if line.startswith("COUNTS "))
    assert counts == sorted(outputs["quincy"]), "Quincy/Tentacle counts mismatch"
    print("All 3,007 MSR vectors match Tentacle; all five device counts match both releases.")
    print("\n".join(counts))
