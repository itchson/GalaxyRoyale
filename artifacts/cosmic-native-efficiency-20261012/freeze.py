"""Freeze the completed measurements; no builds or measured processes run here."""
import hashlib
import json
from pathlib import Path
import subprocess

repo = Path(__file__).resolve().parents[2]
out = Path(__file__).resolve().parent
base = "22a8906cd668e92db0180d725da87b0a2c8408d5"
root_head = "69646456ac3e104c94ca27bc79aeb774dd9ab757"
commit = "d7327c729f483176e26d342b055a665e9471b11c"
runtime_hash = "dfbcd33501214c59dafcf3d4c766b84c20a47f7c79e4567cb05f5c66ab8be3ce"

def git(*args):
    return subprocess.check_output(["git", *args], cwd=repo)

def digest(data):
    return hashlib.sha256(data).hexdigest()

def write_json(path, value):
    path.write_text(json.dumps(value, indent=2) + "\n", encoding="utf8", newline="\n")

def span(pair):
    return str(pair[0]) if pair[0] == pair[1] else f"{pair[0]}–{pair[1]}"

assert git("rev-parse", "HEAD").decode().strip() == commit
assert not git("status", "--porcelain")
changed = git("diff", "--name-only", base, commit).decode().splitlines()
assert changed == ["crates/game_core/src/cosmic_background.rs", "crates/game_core/src/cosmic_perf_tests.rs"]
root_delta = git("diff", "--name-only", base, root_head).decode().splitlines()
assert root_delta == ["artifacts/native-space-next-20261012/assignments.md", "production/project.json"]
background_path = "crates/game_core/src/cosmic_background.rs"
base_background = git("show", f"{base}:{background_path}")
instrumented_background = git("show", f"{commit}:{background_path}")
hook = b'\n#[cfg(test)]\n#[path = "cosmic_perf_tests.rs"]\nmod perf_tests;\n'
assert instrumented_background == base_background + hook
identity = json.loads((out / "pre-instrumentation-identity.json").read_text(encoding="utf-8-sig"))
assert identity["root"]["sourceHash"] == runtime_hash
measurements = json.loads((out / "measurements.json").read_text())
assert measurements["production_root_source_hash"] == runtime_hash
assert digest(Path(measurements["executable"]).read_bytes()) == measurements["executable_sha256"]

write_json(out / "source-identity.json", {
    "base": base,
    "instrumentation_commit": commit,
    "root_evidence_head": root_head,
    "root_runtime_source_hash": runtime_hash,
    "base_to_root_changes": root_delta,
    "instrumentation_changes": changed,
    "background_production_source_exact_after_removing_cfg_test_hook": True,
    "base_background_git_lf_sha256": digest(base_background),
    "measured_instrumented_background_git_lf_sha256": digest(instrumented_background),
    "raw_tree_hash_scope": "Root raw buildIdentity hash recomputed before instrumentation. Own raw tree hash differs; no byte-identical whole-working-tree claim. Canonical tracked native source matches the pinned base except the two test-only files."
})
write_json(out / "check-observations.json", {
    "scope": "Actual command results observed in tool output; compile/Clippy raw transcripts were not retained. Native process measurement logs are retained separately.",
    "checks": [
        {"command": "cargo test --release --locked -p game_core --lib --no-run", "exit_code": 0, "observed_compile_seconds": 18.79},
        {"command": "cargo clippy --locked -p game_core --tests -- -D warnings", "exit_code": 0, "observed_compile_seconds": 4.24},
        {"command": "cargo fmt --all -- --check", "exit_code": 0},
        {"command": "git diff --check", "exit_code": 0}
    ],
    "earlier_formatter_result": "New instrumentation formatting was corrected before compiling/measuring; no production runtime code was changed.",
    "parser_first_attempt": "The native cold test passed. The first Python parser expected the marker at line start, but the Rust harness prefixed it with the test name. Fixed the parser; parser-first-cold.log is retained and excluded from all final distributions."
})

cold_rows = []
for key, label in [("art_ns", "ART"), ("gas_ns", "GAS_ART"), ("total_ns", "Combined")]:
    d = measurements["cold_distributions"][key]
    cold_rows.append(f"| {label} | {d['min_ns']/1e6:.4f} | {d['median_ns']/1e6:.4f} | {d['mean_ns']/1e6:.4f} | {d['p95_ns']/1e6:.4f} | {d['max_ns']/1e6:.4f} |")
warm_rows = []
for c in measurements["warm"]["cases"]:
    d = c["distribution"]
    quality = "reduced" if c["reduced"] else "full"
    warm_rows.append(f"| {c['path']} | {quality} | {c['zoom']:.1f} | {span(c['cells_min_max'])} | {span(c['transforms_min_max'])} | {d['median_ns']/1000:.2f} | {d['p95_ns']/1000:.2f} | {d['p99_ns']/1000:.2f} |")
w = measurements["warm"]
machine = json.loads((out / "machine-config.json").read_text(encoding="utf-8-sig"))
report = f"""# Native cosmic CPU efficiency audit

The nine fresh-process cache samples have a combined median of **4.8528 ms**, with **4.8263–4.9329 ms** observed. In 4,608 measured warm extractions, the selected full nebula costs about **51 µs** at the median versus **13 µs** reduced. These are native CPU measurements of specific extraction paths, with no GPU, browser FPS, phone, RSS or whole-game frame performance claim.

## Source and execution identity

- Production Root runtime sourceHash: `{runtime_hash}`. The existing `buildIdentity()` recomputed this exact value at Root `{root_head}` before instrumentation. The observed d749 build/package metadata is preserved in `observed-root-build.json`.
- Native baseline: `{base}`. Root's final evidence-only head differs from that base only in `assignments.md` and the production project ledger; tracked native runtime source is unchanged.
- Frozen instrumentation: `{commit}` on `codex/cosmic-native-efficiency`. Only the new ignored `cosmic_perf_tests.rs` and its `cfg(test)` module hook changed. Removing that hook produces the exact baseline cosmic source bytes in Git's LF domain. Production code and budgets were not edited.
- Own pre-instrumentation raw-tree sourceHash is `{identity['own']['sourceHash']}`, which differs from Root's raw-tree hash. This audit does not claim the whole working trees are byte-identical. Native source/Git blob and raw measurement inputs are pinned separately in `manifest.json`.
- Actual release executable SHA256: `{measurements['executable_sha256']}`. It was compiled from the instrumentation before the commit; no instrumentation source changed between that compile/capture and the committed byte checks.

## Cold cached art

One explicit ignored release test ran alone in each of **nine fresh sequential processes**. `Instant` surrounds `LazyLock::force(ART)`, then `LazyLock::force(GAS_ART)`. Startup, cell counting and serialization are outside those timers. Full and reduced bodies are initialized together: **18 cached bodies**, with **4,762 full / 1,200 reduced occupied cells** summed across the nine pairs. These are occupancy counts, not resident-memory measurements. No separate reduced-quality cold-cost advantage was measured.

| Cache | Minimum ms | Median ms | Mean ms | p95 ms | Maximum ms |
| --- | ---: | ---: | ---: | ---: | ---: |
{chr(10).join(cold_rows)}

The sample count is nine; nearest-rank p95 and p99 both equal the maximum. Raw native samples and separate process wall times are in `measurements.json` and `cold-00.log` through `cold-08.log`. Process wall times are not used as cache timing.

## Warm FrameBuilder extraction

One subsequent release process forces both caches before timing. Each of **18 cases** reuses its own `FrameBuilder`: **32 warmup frames and 256 measured frames**, moving cosmetic time from 20 seconds at 1/60-second increments. Timers include `FrameBuilder.clear()` and actual native extraction; they exclude input setup, cell/count validation, sample recording and JSON output. Reused buffer capacity avoids measuring initial builder setup; this is not an allocator audit, and any later capacity growth remains inside the timer.

Seed `{w['seed']}`; selected actual nebula hash `{w['motif_hash']}`; birth origin `{w['motif_origin']}`; unshaken camera `{w['camera']}` (`origin / 0.08`). It is the first actual nebula found by a bounded row-major [-8,8] grid search. World viewport is 533×300 divided by zoom; Reduced Motion is false. This is one deterministic selected area, not an all-seed or worst-case sample.

`nebula_motif` extracts the actual four-part nebula alone. `custom_background` extracts the actual custom motifs, stellar neighborhoods and star field. `regional_gas` extracts the separately budgeted regional gas only, passing the existing 6,000/3,000 regional quota and retaining its 800/400 gas cap. Regional planets, other regional scenery, gameplay objects and simulation are not included.

| Path | Detail | Zoom | Submitted cells | Transforms | Median µs | p95 µs | p99 µs |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
{chr(10).join(warm_rows)}

Submitted counts are actual `RenderFrame.pixel_count`; transforms are the sum of actual batch transform counts. Every measured motif frame retains its exact full/reduced cell cost and four transforms. Actual custom counts stay within the unchanged 8,000/4,000 quota; regional gas counts stay within 800/400. Samples include changing gas culling/counts as cosmetic time advances. Full distributions, including minima/maxima and all nanosecond samples, are retained rather than inferred from displayed rounded values.

Regional gas can cost more CPU than the larger single nebula while emitting fewer cells. Its many transforms and streamed-field/candidate loops are consistent with that observation; this capture does not isolate the cause with a profiler. No renderer rewrite or budget increase is justified by this one bounded audit. Separate-path medians must not be summed into a whole-game FPS estimate.

## Configuration, checks and limits

AMD Ryzen 9 9950X3D, 16 cores / 32 logical processors; Windows 11 Pro 10.0.26200; `{machine['ram_bytes']}` physical RAM bytes reported by CIM. Rust 1.97.0 / Cargo 1.97.0, x86_64-pc-windows-msvc, LLVM 22.1.6. Release profile uses opt-level 3 and thin LTO. CPU affinity is OS default. Power/thermal state, desktop background workload and concurrent Root activity were not controlled. Case order was fixed and not randomized; one warm run provides per-frame variation, not independent-run confidence bounds. Windows `Instant` results in these logs are quantized to 100 ns.

Actual native release compilation and strict core test-target Clippy passed; final formatting/diff checks passed. `check-observations.json` records observed tool results without inventing compiler transcripts. Ten final native measurement processes succeeded sequentially; each ran one selected ignored measurement test. No general full-suite, WASM or GPU test was run for this audit.

The first cold native test also passed, but its Python parser rejected the Rust harness's prefixed marker. `parser-first-cold.log` preserves that attempt; it is excluded from final distributions. The fixed runner then captured the nine cold samples and one warm process recorded above. No runtime failure was hidden or reclassified.

This audit measures cache construction and warmed native packet extraction. It does not measure GPU upload/shading/alpha fill, physical display quality, browser scheduling, startup including compilation/module loading, RSS, allocator totals, moving combat, crowded multiplayer or phone hardware. No listener/server/browser/Studio was started or modified, and Root production/runtime files were not changed.

## Reproduction and retained evidence

Build the native instrumentation with `cargo test --release --locked -p game_core --lib --no-run`, locate the emitted test executable, then run each exact ignored test alone with `--exact --ignored --nocapture --test-threads=1`. `run_measurements.py` records the actual executable path used here; if a rebuild changes the executable filename, update a copy of the runner rather than overwrite these frozen results. All subprocesses are sequential with a 60-second timeout. `verify.py` verifies the manifest and compares retained logs to the measurement JSON without executing the native binary.
"""
(out / "HANDOFF.md").write_text(report, encoding="utf8", newline="\n")

# Compile inputs are pinned, not copied. These include all tracked core/engine
# Rust sources plus manifests; asset hashes are not needed by these measured
# procedural caches, and the base-to-Root diff above proves unchanged ancestry.
paths = git("ls-files", "crates/game_core", "crates/engine_core", "Cargo.toml", "Cargo.lock", "rust-toolchain.toml").decode().splitlines()
paths = sorted(p for p in paths if p.endswith(".rs") or p.endswith("Cargo.toml") or p in ("Cargo.lock", "rust-toolchain.toml"))
source = []
for path in paths:
    blob = git("show", f"{commit}:{path}")
    raw = (repo / path).read_bytes()
    assert raw.replace(b"\r\n", b"\n") == blob, path
    source.append({"path": path, "git_lf_size": len(blob), "git_lf_sha256": digest(blob), "working_raw_size": len(raw), "working_raw_sha256": digest(raw)})
evidence = []
for path in sorted(out.iterdir()):
    if path.is_file() and path.name != "manifest.json":
        raw = path.read_bytes()
        evidence.append({"path": path.name, "size": len(raw), "sha256": digest(raw)})
write_json(out / "manifest.json", {"schema": 1, "instrumentation_commit": commit, "base": base,
    "root_runtime_source_hash": runtime_hash, "source_domain": "committed Git LF bytes and actual working raw bytes, separately declared",
    "evidence_domain": "actual raw retained bytes", "source": source, "evidence": evidence})
print(f"Frozen {len(source)} native source pins and {len(evidence)} evidence files; no new measurements")
