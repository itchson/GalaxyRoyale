# Warm custom-background audit after stellar refinement

**The prescribed historical cases do not exercise the new stellar candidate loop.** All six have zero selected and zero submitted stellar neighborhoods. Their workload counts remain exact and their new medians are lower, but this is not evidence that the new sampler is faster or has no material cost. No further measurement process was run after identifying that coverage limit.

Production reference `597f0720704656187056bf57c5d5c1d6bdcfcab3` contains the exact cosmic module from `abb339b59c3d185cbc8342f2c1eeed1c540cfaa9`. Isolated instrumentation is frozen at `8d307a2640f8934a5cfd6ef6ee85489b89ac2ca8` on `codex/stellar-native-efficiency`: one new ignored test module and its `cfg(test)` registration only. Removing that registration reproduces the production cosmic Git blob. Do not cherry-pick measurement hooks into shipping source merely to preserve this evidence.

The previous audit remains historical at native base `22a8906cd668e92db0180d725da87b0a2c8408d5`, instrumentation `d7327c729f483176e26d342b055a665e9471b11c`, broader runtime sourceHash `dfbcd33501214c59dafcf3d4c766b84c20a47f7c79e4567cb05f5c66ab8be3ce`. That is not the new runtime's sourceHash. The entire cosmic source difference is only the stellar sampler/comment/test registration; ART/GAS_ART definitions and cache generators are unchanged. Cold initialization was not rerun, and the old cache timings are not relabelled as current execution. The old report, samples, manifest and measured executable were read only; a separate task-local target directory prevents overwriting the old executable.

## Actual one-process method

One release native test process ran alone with `--exact --ignored --nocapture --test-threads=1`. Both caches were forced before timing. Every case reused a FrameBuilder with 32 warmups and 256 recorded frames. Timers include clear/reuse and the actual native `cosmic_background::render`; input setup, assertions/counting and serialization are excluded. One additional untimed render/sampler coverage pass precedes each case's warmups. The historical process included motif/gas cases around its custom cases; this new process measures custom only. These differences and uncontrolled desktop/CPU scheduling prohibit paired causal attribution.

The inputs are exact to the old custom cases: seed871238449, first row-major [-8,8] native nebula hash3743595620, origin `[1921.350830078125,-8453.685546875]`, camera `[24016.88671875,-105671.0703125]`, 533x300 logical extent divided by zoom, cosmetic time `20+i/60`, Reduced Motion false. Six cases yield **1,536 measured frames**. Native quotas remain 8,000/4,000, with 1,800/900 star reserve; no cap or runtime source was altered.

| Detail | Zoom | Cells | Transforms | Median µs | p95 µs | p99 µs | Maximum µs | Historical median µs |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Full | .6 | 1886 | 30 | 58.7 | 59.2 | 62.3 | 92.8 | 61.6 |
| Full | 1 | 1747 | 16 | 53.7 | 54.2 | 57.7 | 66.1 | 56.6 |
| Full | 1.6 | 1701 | 11 | 51.2 | 51.9 | 56.2 | 67.2 | 53.8 |
| Reduced | .6 | 534 | 27 | 19.7 | 20.0 | 21.9 | 22.5 | 20.9 |
| Reduced | 1 | 464 | 16 | 16.3 | 16.4 | 16.6 | 19.0 | 17.3 |
| Reduced | 1.6 | 444 | 9 | 14.4 | 14.5 | 19.7 | 20.3 | 15.7 |

Cells/transforms are constant across each case and exactly equal to historical values. Min/mean/all 256 raw nanosecond samples, nearest-rank p95/p99 and historical differences are retained in `measurements.json`. Median differences are -4.71% to -8.28%; they describe these two runs, not an improvement caused by the stellar source change. Hash-gate checks still occur, but none of these tiles enters the new candidate-selection loop. A populated-neighborhood paired probe is a future separately authorized measurement, not performed here.

## Tool/machine and evidence

Windows11 Pro26200, AMD Ryzen9 9950X3D (16cores/32logical), rustc/cargo1.97.0, LLVM22.1.6, x86_64-pc-windows-msvc. Release opt3/thinLTO, no features; compile jobs1, OS default CPU affinity. Desktop/background workload is not controlled. `machine-config.json` preserves the actual query/tool output. Native measurement binary SHA256 `d3b41fca6ad4b768c74c6183f6cf421a36f6c006421b7601a14a1906bb67bf96`; its process wall time1.1066204seconds includes startup/cache forcing/output and is not a cache or frame-performance metric.

Compilation used `cargo test --release --no-run --locked -p game_core -j1 --target-dir work/stellar-native-efficiency/target --message-format=json`; `compile-artifacts.jsonl` records successful build and exact executable. `run.py` launched that binary exactly once, captured `warm.log`, and checked exact old/new scenario/count parity before writing measurements. The ignored measurement test passed; cargo fmt/diff checks passed. No additional native test suite or cold process, shader/WASM/production build, browser, listener, GPU/FPS/RSS/phone or whole-game frame measurement was performed. No all-seed or confidence-interval claim is made.

`manifest.json` pins compiled native inputs, original production/stellar/base inputs, raw evidence and the unchanged historical comparison files. `verify.py` performs read-only checks and no native rerun. Root owns production integration and any later populated-star measurement.
