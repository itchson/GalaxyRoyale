import hashlib
import json
import statistics
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
trials = [json.loads((HERE / f"trial-{n}.json").read_text(encoding="utf-8-sig")) for n in range(3)]
rows = []
for index in range(12):
    samples = [trial["cases"][index] for trial in trials]
    first = samples[0]
    assert all(s["corporation"] == first["corporation"] and s["cosmetics"] == first["cosmetics"] and s["world_zoom_equivalent"] == first["world_zoom_equivalent"] for s in samples)
    assert all(s["baseline"]["packet"] == first["baseline"]["packet"] and s["candidate"]["packet"] == first["candidate"]["packet"] for s in samples)
    row = {k: first[k] for k in ["corporation", "cosmetics", "world_zoom_equivalent"]}
    for path in ["baseline", "candidate"]:
        row[path] = {
            "packet": first[path]["packet"],
            "median_trial_quantile_ms": {
                q: statistics.median(s[path]["timing"][f"{q}_ms"] for s in samples)
                for q in ["p50", "p95", "p99"]
            },
            "worst_trial_p95_ms": max(s[path]["timing"]["p95_ms"] for s in samples),
            "worst_trial_p99_ms": max(s[path]["timing"]["p99_ms"] for s in samples),
        }
    row["median_trial_p50_added_ms"] = row["candidate"]["median_trial_quantile_ms"]["p50"] - row["baseline"]["median_trial_quantile_ms"]["p50"]
    rows.append(row)
summary = {
    "source_commit": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
    "trials": 3, "fixture_pairs_per_trial": 12, "warmup_per_path": 60, "iterations_per_path": 600,
    "timed_frames": 3 * 12 * 2 * 600,
    "percentile_domain": "nearest-rank per 600-frame trial; table uses median of the three trial quantiles, not a pooled percentile",
    "cold_callback_ms": [t["cold"]["first_dock_callback_ms"] for t in trials],
    "cold_game_render_ms": [t["cold"]["total_game_render_ms"] for t in trials],
    "cold_domain": trials[0]["cold"]["includes"],
    "scope": trials[0]["scope"],
    "cases": rows,
}
(HERE / "summary.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
lines = ["| Corp / quality / zoom | Quads old → dock | p50 old → dock (ms) | dock p95 / p99 (ms) |", "|---|---:|---:|---:|"]
for r in rows:
    b, c = r["baseline"], r["candidate"]
    bt, ct = b["median_trial_quantile_ms"], c["median_trial_quantile_ms"]
    lines.append(f"| {r['corporation']} / {r['cosmetics']} / {r['world_zoom_equivalent']:.1f} | {b['packet']['quads']:,} → {c['packet']['quads']:,} | {bt['p50']:.4f} → {ct['p50']:.4f} | {ct['p95']:.4f} / {ct['p99']:.4f} |")
(HERE / "table.md").write_text("\n".join(lines) + "\n", encoding="utf-8")

paths = [
    "Cargo.toml", "Cargo.lock", "crates/engine_core/Cargo.toml", "crates/game_core/Cargo.toml", "crates/game_web/Cargo.toml",
    "crates/engine_core/src/pixels.rs", "crates/engine_core/src/pixel_rig.rs", "crates/engine_core/src/frame.rs",
    "crates/game_core/src/lib.rs", "crates/game_web/src/dock_art.rs", "crates/game_web/src/dock_art_tests.rs", "crates/game_web/src/lib.rs",
    "assets/pipeline/pan-coop-dock/native256/pack.json",
]
def pin(p):
    data = p.read_bytes()
    return {"bytes":len(data), "sha256":hashlib.sha256(data).hexdigest()}
pins = {
    "source_commit": summary["source_commit"],
    "tracked_status": subprocess.check_output(["git", "status", "--porcelain", "--untracked-files=no"], cwd=ROOT, text=True),
    "source_files": {p: pin(ROOT / p) for p in paths},
    "actual_linked_libraries": {},
    "evidence": {},
}
deps = HERE / "target/release/deps"
for pattern in ["libengine_core*.rlib", "libgame_core*.rlib", "libgame_web*.rlib", "libserde_json*.rlib"]:
    matches = list(deps.glob(pattern))
    assert len(matches) == 1, matches
    p = matches[0]
    pins["actual_linked_libraries"][str(p.relative_to(HERE)).replace("\\", "/")] = pin(p)
for name in ["probe.rs", "summarize.py", "build.log", "probe-build-record.json", "dock-runtime-benchmark.exe", "trial-0.json", "trial-1.json", "trial-2.json", "summary.json", "table.md", "platform.json"]:
    pins["evidence"][name] = pin(HERE / name)
(HERE / "input-and-results-pins.json").write_text(json.dumps(pins, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"cold_callback_ms":summary["cold_callback_ms"], "max_median_trial_dock_p95_ms":max(r["candidate"]["median_trial_quantile_ms"]["p95"] for r in rows), "max_worst_trial_dock_p99_ms":max(r["candidate"]["worst_trial_p99_ms"] for r in rows), "added_p50_ms_range":[min(r["median_trial_p50_added_ms"] for r in rows),max(r["median_trial_p50_added_ms"] for r in rows)], "source_commit":summary["source_commit"]}, indent=2))
