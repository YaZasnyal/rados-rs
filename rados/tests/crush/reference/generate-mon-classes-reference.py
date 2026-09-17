"""Capture retained TEST_mon_classes map states from pinned isolated clusters.

Run: python3 rados/tests/crush/reference/generate-mon-classes-reference.py
Requires the pinned ARM64 Docker images and the adjacent bounded topology
script live-mon-classes-preflight.sh.
"""
from pathlib import Path
import subprocess as sp
import sys
import tempfile
import time

reference = Path(__file__).resolve().parent
preflight = reference / "live-mon-classes-preflight.sh"
check = sys.argv[1:] == ["--check"]
releases = (
    ("quincy", "17.2.7", "b12291d110049b2f35e32e0de30d70e9a4c060d2",
     "a70ccb2d8a0e814aa1c009e7541289b99bf039521727b6f44611499e9ca3fada"),
    ("tentacle", "20.2.4", "7f793731f1b39eb4f465e960113d2363c311b964",
     "6e6bc7b28fa1b334108a3646af5533dfb50db508efdf5b358eb7dd0dd37a48aa"),
)
state_rules = {"asdf": (1,), "abc": (1, 2), "class2": (1, 2, 3)}
docker = ["docker", "run", "--rm", "--network", "none", "-i", "--entrypoint"]


def command(name, script):
    script = "conf=$(echo /tmp/rados-mon-classes.*/ceph.conf); " + script.replace("ceph ", "ceph --conf \"$conf\" ")
    sp.run(["docker", "exec", name, "bash", "-ec", script], check=True, stdout=sp.DEVNULL)


def capture(name, state, output):
    command(name, f"ceph osd getcrushmap -o /tmp/{state}.crushmap")
    sp.run(["docker", "cp", f"{name}:/tmp/{state}.crushmap", str(output)], check=True, stdout=sp.DEVNULL)
    return output.read_bytes()


def vectors(image, data, rule):
    output = sp.check_output(docker + ["crushtool", image, "-i", "-", "--test", "--rule", str(rule),
                                       "--num-rep", "3", "--min-x", "0", "--max-x", "19", "--show-mappings"], input=data)
    rows = output.decode().splitlines()
    assert len(rows) == 20, (rule, len(rows))
    return [line.split("[", 1)[1].rstrip("]").replace(",", " ") for line in rows]


generated = {}
manifest = []
for release, version, revision, image_digest in releases:
    image = "quay.io/ceph/ceph@sha256:" + image_digest
    identity = sp.check_output(docker + ["ceph", image, "--version"], text=True)
    assert f"ceph version {version} ({revision})" in identity, identity
    name = f"crush-mon-classes-{release}-{int(time.time())}"
    log = Path(tempfile.mkstemp(prefix=f".mon-classes-{release}-", suffix=".log")[1])
    with log.open("w") as output:
        process = sp.Popen([
            "docker", "run", "--rm", "--name", name, "--network", "host", "--user", "ceph",
            "-e", "CRUSH_PREFLIGHT_HOLD=300", "-v", f"{preflight}:/preflight.sh:ro", image,
            "bash", "/preflight.sh",
        ], stdout=output, stderr=sp.STDOUT)
        try:
            deadline = time.monotonic() + 150
            while "LIVE PREFLIGHT PASS" not in log.read_text():
                if process.poll() is not None:
                    raise RuntimeError(log.read_text())
                if time.monotonic() > deadline:
                    raise TimeoutError(f"{release} cluster startup")
                time.sleep(1)
            with tempfile.TemporaryDirectory(prefix=".mon-classes-", dir=reference) as temporary:
                work = Path(temporary)
                command(name, "ceph osd crush class create CLASS; ceph osd crush class create CLASS; ceph osd crush class rename CLASS TEMP; ceph osd crush class rename TEMP CLASS; ceph osd erasure-code-profile set myprofile plugin=jerasure technique=reed_sol_van k=2 m=1 crush-failure-domain=osd crush-device-class=CLASS; ! ceph osd crush class rm CLASS; ceph osd erasure-code-profile rm myprofile; ceph osd crush class rm CLASS; ceph osd crush class rm CLASS")
                command(name, "ceph osd crush set-device-class aaa osd.0; ceph osd crush set-device-class bbb osd.1; ceph osd crush set-device-class ccc osd.2; ceph osd crush rm-device-class 0; ceph osd crush rm-device-class 1; ceph osd crush rm-device-class 2")
                generated[release, "removed"] = capture(name, "removed", work / "removed.crushmap")
                command(name, "ceph osd crush set-device-class asdf all; ceph osd crush rule create-replicated asdf-rule default host asdf; ceph osd crush rm-device-class all")
                generated[release, "asdf"] = capture(name, "asdf", work / "asdf.crushmap")
                command(name, "ceph osd crush set-device-class abc osd.2; ceph osd crush move osd.2 root=foo rack=foo-rack host=foo-host; ceph osd crush rm-device-class osd.2; ceph osd crush set-device-class abc osd.2; ceph osd crush rule create-replicated foo-rule foo host abc; ceph osd crush set-device-class hdd osd.0; ! ceph osd crush set-device-class nvme osd.0")
                generated[release, "abc"] = capture(name, "abc", work / "abc.crushmap")
                command(name, "ceph osd crush rm-device-class all; ceph osd crush set-device-class class_1 all; ceph osd crush rule create-replicated class_1_rule default host class_1; ceph osd crush class rename class_1 class_2; ceph osd crush class rename class_1 class_2")
                generated[release, "class2"] = capture(name, "class2", work / "class2.crushmap")
        finally:
            sp.run(["docker", "stop", "--time", "5", name], stdout=sp.DEVNULL, stderr=sp.DEVNULL, check=False)
            process.wait(timeout=15)
            assert not sp.check_output(["docker", "ps", "-q", "--filter", f"name={name}"]).strip()
            log.unlink(missing_ok=True)
    for state, rules in state_rules.items():
        data = generated[release, state]
        for rule in rules:
            for x, row in enumerate(vectors(image, data, rule)):
                manifest.append(f"{release} {state} {rule} {x} [{row}]")

for (release, state), data in generated.items():
    path = reference / f"mon-classes-{state}-{release}.crushmap"
    if check:
        assert path.read_bytes() == data, f"stale {path}"
    else:
        path.write_bytes(data)
    print(f"{path.name}: {len(data)} bytes")
manifest_path = reference / "mon-classes-vectors.txt"
manifest_data = ("\n".join(manifest) + "\n").encode()
if check:
    assert manifest_path.read_bytes() == manifest_data, f"stale {manifest_path}"
else:
    manifest_path.write_bytes(manifest_data)
print(f"mon-classes-vectors.txt: {len(manifest)} rows")
