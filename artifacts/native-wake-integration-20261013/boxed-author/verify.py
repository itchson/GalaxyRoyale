"""Freeze/verify actual native review evidence without building or mutating runtime."""
from pathlib import Path
import hashlib
import json
import subprocess
import sys

work = Path(__file__).resolve().parent
root = work.parents[1]


def digest(path):
    data = path.read_bytes()
    return {"bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()}


if "--freeze" in sys.argv:
    pins = json.loads((work / "source-pins.json").read_text(encoding="utf-8"))
    excluded = {"MANIFEST.json", "verification.json", "verification.log"}
    manifest = {
        "version": 1,
        "candidate": pins["candidate"],
        "base": pins["base"],
        "reference_root": pins["reference_root"],
        "scope": "Native test-only regression and source-backed ownership review; no runtime change or live browser proof.",
        "source_semantics": "Canonical Git blob pins in source-pins.json; all evidence byte hashes are actual raw stored files.",
        "evidence": [
            {"path": p.relative_to(work).as_posix(), **digest(p)}
            for p in sorted(work.rglob("*"))
            if p.is_file() and p.relative_to(work).as_posix() not in excluded
        ],
    }
    (work / "MANIFEST.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")

manifest = json.loads((work / "MANIFEST.json").read_text(encoding="utf-8"))
for pin in manifest["evidence"]:
    assert digest(work / pin["path"]) == {"bytes": pin["bytes"], "sha256": pin["sha256"]}, pin["path"]
pins = json.loads((work / "source-pins.json").read_text(encoding="utf-8"))
for pin in pins["source"]:
    data = subprocess.check_output(["git", "show", pins["candidate"] + ":" + pin["path"]], cwd=root)
    assert data == (work / pin["snapshot"]).read_bytes(), pin["path"]
    assert len(data) == pin["bytes"] and hashlib.sha256(data).hexdigest() == pin["sha256"], pin["path"]
    reference = subprocess.check_output(["git", "show", pins["reference_root"] + ":" + pin["path"]], cwd=root)
    assert (data == reference) == pin["identical_to_reference_root"], pin["path"]
    if not pin["identical_to_reference_root"]:
        assert reference == (work / pin["reference_root_snapshot"]).read_bytes(), pin["path"]

assert "23 passed; 0 failed; 1 ignored" in (work / "co-op-final.log").read_text(encoding="utf-8-sig")
assert "boxed_solo_world_restores_ecs_transients_and_deterministic_continuation_after_teardown ... ok" in (work / "co-op-final.log").read_text(encoding="utf-8-sig")
assert "4 passed; 0 failed; 0 ignored" in (work / "guest-bridge.log").read_text(encoding="utf-8-sig")
assert "test result: ok. 0 passed;" in (work / "native-first.log").read_text(encoding="utf-8-sig")
assert "assertion failed: !game.chunks.is_empty()" in (work / "native-first-exact.log").read_text(encoding="utf-8-sig")
assert "left: Err(StaleMembership)" in (work / "native-second.log").read_text(encoding="utf-8-sig")
assert "PASS: owned test rustfmt and git diff --check" in (work / "fmt-final.log").read_text(encoding="utf-8-sig")
result = {
    "verified": True,
    "candidate": manifest["candidate"],
    "canonical_source_pins": len(pins["source"]),
    "runtime_inputs_byte_identical_to_root5e": sum(p["identical_to_reference_root"] for p in pins["source"]),
    "raw_evidence_pins": len(manifest["evidence"]),
    "native_coop_passed": 23,
    "native_coop_ignored": 1,
    "guest_bridge_passed": 4,
    "retained_zero_test_attempt": True,
    "retained_fixture_failures": 2,
    "runtime_defect_reproduced": False,
    "browser_or_rtc_run_by_this_review": False,
}
(work / "verification.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
print(json.dumps(result, indent=2))
