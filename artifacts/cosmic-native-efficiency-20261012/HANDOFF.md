# Native cosmic CPU efficiency audit

The nine fresh-process cache samples have a combined median of **4.8528 ms**, with **4.8263–4.9329 ms** observed. In 4,608 measured warm extractions, the selected full nebula costs about **51 µs** at the median versus **13 µs** reduced. These are native CPU measurements of specific extraction paths, with no GPU, browser FPS, phone, RSS or whole-game frame performance claim.

## Source and execution identity

- Production Root runtime sourceHash: `dfbcd33501214c59dafcf3d4c766b84c20a47f7c79e4567cb05f5c66ab8be3ce`. The existing `buildIdentity()` recomputed this exact value at Root `69646456ac3e104c94ca27bc79aeb774dd9ab757` before instrumentation. The observed d749 build/package metadata is preserved in `observed-root-build.json`.
- Native baseline: `22a8906cd668e92db0180d725da87b0a2c8408d5`. Root's final evidence-only head differs from that base only in `assignments.md` and the production project ledger; tracked native runtime source is unchanged.
- Frozen instrumentation: `d7327c729f483176e26d342b055a665e9471b11c` on `codex/cosmic-native-efficiency`. Only the new ignored `cosmic_perf_tests.rs` and its `cfg(test)` module hook changed. Removing that hook produces the exact baseline cosmic source bytes in Git's LF domain. Production code and budgets were not edited.
- Own pre-instrumentation raw-tree sourceHash is `21e90c5262c573c7c2bf366fd517de79dc58cde61b8302d7e130c39797090d9a`, which differs from Root's raw-tree hash. This audit does not claim the whole working trees are byte-identical. Native source/Git blob and raw measurement inputs are pinned separately in `manifest.json`.
- Actual release executable SHA256: `339062839a6a71711cad4a39029f527a594c2e1bfc7482e5dfad3fbc9d59e232`. It was compiled from the instrumentation before the commit; no instrumentation source changed between that compile/capture and the committed byte checks.

## Cold cached art

One explicit ignored release test ran alone in each of **nine fresh sequential processes**. `Instant` surrounds `LazyLock::force(ART)`, then `LazyLock::force(GAS_ART)`. Startup, cell counting and serialization are outside those timers. Full and reduced bodies are initialized together: **18 cached bodies**, with **4,762 full / 1,200 reduced occupied cells** summed across the nine pairs. These are occupancy counts, not resident-memory measurements. No separate reduced-quality cold-cost advantage was measured.

| Cache | Minimum ms | Median ms | Mean ms | p95 ms | Maximum ms |
| --- | ---: | ---: | ---: | ---: | ---: |
| ART | 4.5220 | 4.5397 | 4.5530 | 4.6171 | 4.6171 |
| GAS_ART | 0.2968 | 0.3039 | 0.3049 | 0.3158 | 0.3158 |
| Combined | 4.8263 | 4.8528 | 4.8579 | 4.9329 | 4.9329 |

The sample count is nine; nearest-rank p95 and p99 both equal the maximum. Raw native samples and separate process wall times are in `measurements.json` and `cold-00.log` through `cold-08.log`. Process wall times are not used as cache timing.

## Warm FrameBuilder extraction

One subsequent release process forces both caches before timing. Each of **18 cases** reuses its own `FrameBuilder`: **32 warmup frames and 256 measured frames**, moving cosmetic time from 20 seconds at 1/60-second increments. Timers include `FrameBuilder.clear()` and actual native extraction; they exclude input setup, cell/count validation, sample recording and JSON output. Reused buffer capacity avoids measuring initial builder setup; this is not an allocator audit, and any later capacity growth remains inside the timer.

Seed `871238449`; selected actual nebula hash `3743595620`; birth origin `[1921.350830078125, -8453.685546875]`; unshaken camera `[24016.88671875, -105671.0703125]` (`origin / 0.08`). It is the first actual nebula found by a bounded row-major [-8,8] grid search. World viewport is 533×300 divided by zoom; Reduced Motion is false. This is one deterministic selected area, not an all-seed or worst-case sample.

`nebula_motif` extracts the actual four-part nebula alone. `custom_background` extracts the actual custom motifs, stellar neighborhoods and star field. `regional_gas` extracts the separately budgeted regional gas only, passing the existing 6,000/3,000 regional quota and retaining its 800/400 gas cap. Regional planets, other regional scenery, gameplay objects and simulation are not included.

| Path | Detail | Zoom | Submitted cells | Transforms | Median µs | p95 µs | p99 µs |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| nebula_motif | full | 0.6 | 1672 | 4 | 51.00 | 51.70 | 53.80 |
| nebula_motif | full | 1.0 | 1672 | 4 | 51.30 | 52.10 | 55.30 |
| nebula_motif | full | 1.6 | 1672 | 4 | 51.30 | 52.10 | 54.00 |
| nebula_motif | reduced | 0.6 | 434 | 4 | 13.10 | 13.30 | 13.60 |
| nebula_motif | reduced | 1.0 | 434 | 4 | 12.80 | 13.10 | 20.90 |
| nebula_motif | reduced | 1.6 | 434 | 4 | 12.70 | 12.80 | 13.70 |
| custom_background | full | 0.6 | 1886 | 30 | 61.60 | 62.50 | 64.90 |
| custom_background | full | 1.0 | 1747 | 16 | 56.60 | 58.50 | 61.50 |
| custom_background | full | 1.6 | 1701 | 11 | 53.80 | 54.70 | 57.60 |
| custom_background | reduced | 0.6 | 534 | 27 | 20.90 | 21.20 | 23.30 |
| custom_background | reduced | 1.0 | 464 | 16 | 17.30 | 17.50 | 17.60 |
| custom_background | reduced | 1.6 | 444 | 9 | 15.70 | 15.80 | 17.60 |
| regional_gas | full | 0.6 | 770–776 | 84–86 | 68.20 | 69.10 | 70.80 |
| regional_gas | full | 1.0 | 312–316 | 34–37 | 35.00 | 35.50 | 39.40 |
| regional_gas | full | 1.6 | 188–190 | 16 | 25.20 | 25.50 | 30.90 |
| regional_gas | reduced | 0.6 | 376–381 | 79–80 | 47.85 | 48.40 | 50.80 |
| regional_gas | reduced | 1.0 | 163–164 | 31–32 | 25.10 | 25.40 | 27.60 |
| regional_gas | reduced | 1.6 | 107 | 16 | 17.90 | 18.00 | 18.10 |

Submitted counts are actual `RenderFrame.pixel_count`; transforms are the sum of actual batch transform counts. Every measured motif frame retains its exact full/reduced cell cost and four transforms. Actual custom counts stay within the unchanged 8,000/4,000 quota; regional gas counts stay within 800/400. Samples include changing gas culling/counts as cosmetic time advances. Full distributions, including minima/maxima and all nanosecond samples, are retained rather than inferred from displayed rounded values.

Regional gas can cost more CPU than the larger single nebula while emitting fewer cells. Its many transforms and streamed-field/candidate loops are consistent with that observation; this capture does not isolate the cause with a profiler. No renderer rewrite or budget increase is justified by this one bounded audit. Separate-path medians must not be summed into a whole-game FPS estimate.

## Configuration, checks and limits

AMD Ryzen 9 9950X3D, 16 cores / 32 logical processors; Windows 11 Pro 10.0.26200; `66094223360` physical RAM bytes reported by CIM. Rust 1.97.0 / Cargo 1.97.0, x86_64-pc-windows-msvc, LLVM 22.1.6. Release profile uses opt-level 3 and thin LTO. CPU affinity is OS default. Power/thermal state, desktop background workload and concurrent Root activity were not controlled. Case order was fixed and not randomized; one warm run provides per-frame variation, not independent-run confidence bounds. Windows `Instant` results in these logs are quantized to 100 ns.

Actual native release compilation and strict core test-target Clippy passed; final formatting/diff checks passed. `check-observations.json` records observed tool results without inventing compiler transcripts. Ten final native measurement processes succeeded sequentially; each ran one selected ignored measurement test. No general full-suite, WASM or GPU test was run for this audit.

The first cold native test also passed, but its Python parser rejected the Rust harness's prefixed marker. `parser-first-cold.log` preserves that attempt; it is excluded from final distributions. The fixed runner then captured the nine cold samples and one warm process recorded above. No runtime failure was hidden or reclassified.

This audit measures cache construction and warmed native packet extraction. It does not measure GPU upload/shading/alpha fill, physical display quality, browser scheduling, startup including compilation/module loading, RSS, allocator totals, moving combat, crowded multiplayer or phone hardware. No listener/server/browser/Studio was started or modified, and Root production/runtime files were not changed.

## Reproduction and retained evidence

Build the native instrumentation with `cargo test --release --locked -p game_core --lib --no-run`, locate the emitted test executable, then run each exact ignored test alone with `--exact --ignored --nocapture --test-threads=1`. `run_measurements.py` records the actual executable path used here; if a rebuild changes the executable filename, update a copy of the runner rather than overwrite these frozen results. All subprocesses are sequential with a 60-second timeout. `verify.py` verifies the manifest and compares retained logs to the measurement JSON without executing the native binary.
