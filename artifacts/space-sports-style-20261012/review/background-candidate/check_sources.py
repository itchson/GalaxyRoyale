"""Read-only exact-source and retained nebula packet checks."""

import hashlib
import json
from pathlib import Path
import re
import subprocess

repo = Path("C:/Users/mitch/.codex/worktrees/galaxy-hud-navigation/galaxy-royale")
baseline = "4e014bff3a84b356c5254b4291560e38accef64d"
commit = "61492ce2d3e360375c2a3eb047ca330fd4c937c4"


def blob(revision, path):
    return subprocess.check_output(["git", "show", f"{revision}:{path}"], cwd=repo)


def pin(path):
    data = blob(commit, path)
    return {"path": path, "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()}


changed = subprocess.check_output(
    ["git", "diff-tree", "--no-commit-id", "--name-only", "-r", commit], cwd=repo
).decode().splitlines()
expected = {"crates/game_core/src/" + name for name in [
    "cosmic_background.rs", "cosmic_detail_tests.rs", "regions.rs",
    "regions_lod_tests.rs", "region_immersion_tests.rs",
]}
assert set(changed) == expected
unchanged = []
for path in [
    "crates/game_core/src/lib.rs", "crates/game_core/src/asteroid_art.rs",
    "crates/game_core/src/shield_progression.rs", "crates/game_core/src/radiation_enemies.rs",
    "crates/game_core/src/isotope_scar.rs", "crates/engine_core/src/frame.rs",
    "crates/engine_core/src/camera.rs", "crates/game_web/src/renderer.rs",
    "crates/game_web/src/pixel.wgsl", "crates/game_web/src/present.wgsl",
]:
    assert blob(commit, path) == blob(baseline, path), path
    unchanged.append(pin(path))
regions = "crates/game_core/src/regions.rs"
assert blob(commit, regions).split(b"static PLANETS:")[0] == blob(baseline, regions).split(b"static PLANETS:")[0]


def function(source, name):
    # Exact standalone function text, not a Rust AST equivalence claim.
    result = re.search(r"(?ms)^(?:pub(?:\([^)]*\))? )?fn " + re.escape(name) + r"\b.*?^}", source)
    assert result, name
    return result.group(0)


cosmic = "crates/game_core/src/cosmic_background.rs"
old, new = blob(baseline, cosmic).decode(), blob(commit, cosmic).decode()
functions = {}
for name in [
    "motif", "stellar_system", "parent", "draw_gas_part", "render_gas",
    "gas_density", "gas_flow", "star_cells", "draw_part", "colored",
]:
    assert function(old, name) == function(new, name), name
    functions[name] = hashlib.sha256(function(new, name).encode()).hexdigest()

peer = repo / "work/background-immersion"
nebula = []
for reduced in [False, True]:
    for time in [0, 18]:
        name = f"motif-1-reduced-{str(reduced).lower()}-time-{time}.json"
        before = json.loads((peer / "before" / name).read_text())
        after = json.loads((peer / "after-final" / name).read_text())
        for key in ["kind", "quads", "transforms", "reduced", "time"]:
            assert before[key] == after[key], (name, key)
        assert len(before["views"]) == len(after["views"]) == 3
        for a, b in zip(before["views"], after["views"]):
            assert a["zoom"] == b["zoom"] and len(a["cells"]) == len(b["cells"])
            for x, y in zip(a["cells"], b["cells"]):
                assert x["corners"] == y["corners"] and x["rgba"] == y["rgba"], name
        nebula.append({"file": name, "quads": after["quads"], "transforms": after["transforms"], "three_views_geometry_rgba_exact": True})

print(json.dumps({
    "commit": commit,
    "baseline": baseline,
    "scope": "Independent exact source/function parity and retained nebula corner/RGBA packet checks; no native test or browser rerun",
    "changed_inputs": [pin(path) for path in changed],
    "unchanged_inputs": unchanged,
    "region_policy_prefix_exact": True,
    "cosmic_function_text_parity": functions,
    "nebula_packet_parity": nebula,
}, indent=2))
