"""Read-only audit of frozen native measurement inputs and outputs."""
import hashlib
import json
import math
from pathlib import Path
import statistics
import subprocess

repo = Path(__file__).resolve().parents[2]
out = Path(__file__).resolve().parent
manifest = json.loads((out / "manifest.json").read_text())
sha = lambda data: hashlib.sha256(data).hexdigest()
for entry in manifest["source"]:
    raw = (repo / entry["path"]).read_bytes()
    blob = subprocess.check_output(["git", "show", f"{manifest['instrumentation_commit']}:{entry['path']}"], cwd=repo)
    assert (len(raw), sha(raw)) == (entry["working_raw_size"], entry["working_raw_sha256"]), entry["path"]
    assert (len(blob), sha(blob)) == (entry["git_lf_size"], entry["git_lf_sha256"]), entry["path"]
for entry in manifest["evidence"]:
    raw = (out / entry["path"]).read_bytes()
    assert (len(raw), sha(raw)) == (entry["size"], entry["sha256"]), entry["path"]

def log_data(name):
    raw = (out / name).read_text()
    assert "test result: ok. 1 passed; 0 failed" in raw
    rows = [line.split("COSMIC_BENCH_JSON:", 1)[1] for line in raw.splitlines() if "COSMIC_BENCH_JSON:" in line]
    assert len(rows) == 1
    return json.loads(rows[0])

def distribution(values):
    ordered = sorted(values)
    at = lambda p: ordered[max(0, math.ceil(len(ordered) * p) - 1)]
    return {"n": len(values), "min_ns": min(values), "median_ns": statistics.median(values),
        "mean_ns": statistics.mean(values), "p95_ns": at(.95), "p99_ns": at(.99), "max_ns": max(values)}

data = json.loads((out / "measurements.json").read_text())
assert data["production_root_source_hash"] == manifest["root_runtime_source_hash"]
assert len(data["cold"]) == 9
for i, row in enumerate(data["cold"]):
    assert row == log_data(f"cold-{i:02}.log")
    assert row["total_ns"] == row["art_ns"] + row["gas_ns"]
    assert row["occupied_full_low"] == [4762, 1200]
    assert row["cached_bodies"] == 18
for key, value in data["cold_distributions"].items():
    assert value == distribution([r[key] for r in data["cold"]])
warm = log_data("warm.log")
assert len(warm["cases"]) == 18
assert len(data["executions"]) == 10
assert all(row["exit_code"] == 0 for row in data["executions"])
seen = set()
for observed, retained in zip(warm["cases"], data["warm"]["cases"], strict=True):
    assert {k: v for k, v in retained.items() if k != "distribution"} == observed
    assert len(retained["samples_ns"]) == 256
    assert retained["warmup_frames"] == 32
    assert retained["distribution"] == distribution(retained["samples_ns"])
    key = (retained["path"], retained["reduced"], round(retained["zoom"], 1))
    assert key not in seen
    seen.add(key)
    cells = retained["cells_min_max"]
    assert 0 < cells[0] <= cells[1]
    if retained["path"] == "regional_gas":
        assert cells[1] <= (400 if retained["reduced"] else 800)
    else:
        assert cells[1] <= (4000 if retained["reduced"] else 8000)
    if retained["path"] == "nebula_motif":
        assert cells == ([434, 434] if retained["reduced"] else [1672, 1672])
        assert retained["transforms_min_max"] == [4, 4]
assert log_data("parser-first-cold.log")["mode"] == "cold_cache_force"
assert len(seen) == 18
executable = Path(data["executable"])
if executable.exists():
    assert sha(executable.read_bytes()) == data["executable_sha256"]
print(f"PASS {len(manifest['source'])} source pins / {len(manifest['evidence'])} evidence pins; 9 cold / 18 warm cases / 4608 native samples; no native rerun")
print("HANDOFF SHA256", sha((out / "HANDOFF.md").read_bytes()))
print("manifest SHA256", sha((out / "manifest.json").read_bytes()))
