"""Sequential native release test processes; never starts a listener or build."""
import hashlib
import json
import math
from pathlib import Path
import statistics
import subprocess
import time

repo = Path(__file__).resolve().parents[2]
out = Path(__file__).resolve().parent
exe = repo / "target/release/deps/game_core-2b707a8b4a4ebce6.exe"
assert exe.is_file()
base = "cosmic_background::perf_tests::"
executions = []

def invoke(name, label):
    command = [str(exe), base + name, "--exact", "--ignored", "--nocapture", "--test-threads=1"]
    started = time.perf_counter_ns()
    result = subprocess.run(command, cwd=repo, capture_output=True, timeout=60)
    wall_ns = time.perf_counter_ns() - started
    raw = result.stdout + result.stderr
    (out / f"{label}.log").write_bytes(raw)
    executions.append({"label": label, "argv": command, "exit_code": result.returncode, "process_wall_ns": wall_ns})
    assert result.returncode == 0, raw.decode(errors="replace")
    rows = [line.split("COSMIC_BENCH_JSON:", 1)[1] for line in raw.decode().splitlines() if "COSMIC_BENCH_JSON:" in line]
    assert len(rows) == 1
    return json.loads(rows[0])

def distribution(values):
    ordered = sorted(values)
    quantile = lambda p: ordered[max(0, math.ceil(len(ordered)*p)-1)]
    return {"n": len(values), "min_ns": min(values), "median_ns": statistics.median(values), "mean_ns": statistics.mean(values), "p95_ns": quantile(.95), "p99_ns": quantile(.99), "max_ns": max(values)}

cold = [invoke("measure_cold_cosmic_caches", f"cold-{i:02}") for i in range(9)]
assert all(c["occupied_full_low"] == cold[0]["occupied_full_low"] for c in cold)
warm = invoke("measure_warm_cosmic_extraction", "warm")
assert len(warm["cases"]) == 18
for case in warm["cases"]:
    assert len(case["samples_ns"]) == 256
    case["distribution"] = distribution(case["samples_ns"])
result = {"production_root_source_hash": "dfbcd33501214c59dafcf3d4c766b84c20a47f7c79e4567cb05f5c66ab8be3ce", "base": "22a8906cd668e92db0180d725da87b0a2c8408d5", "executable": str(exe), "executable_sha256": hashlib.sha256(exe.read_bytes()).hexdigest(), "cold": cold, "cold_distributions": {k: distribution([r[k] for r in cold]) for k in ("art_ns", "gas_ns", "total_ns")}, "warm": warm, "executions": executions, "scope": "native CPU cache initialization and FrameBuilder clear/reuse+extraction only; no GPU FPS, RSS/device performance or whole-game frame timing"}
(out / "measurements.json").write_text(json.dumps(result, indent=2)+"\n", encoding="utf8", newline="\n")
print("Cold distributions ns:", json.dumps(result["cold_distributions"]))
for c in warm["cases"]:
    print(c["path"], "low" if c["reduced"] else "full", c["zoom"], "cells", c["cells_min_max"], "transforms", c["transforms_min_max"], "median_us", round(c["distribution"]["median_ns"]/1000, 3), "p95_us", round(c["distribution"]["p95_ns"]/1000, 3))
print("PASS 9 fresh processes, 18 warmed cases/4608 measured frames; all processes sequential")
