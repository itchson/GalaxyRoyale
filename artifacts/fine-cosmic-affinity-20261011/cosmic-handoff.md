# Fine cosmic detail — isolated implementation

Frozen code: `aff95d69b3055191355bfa619a5ea307da852c3c`, branch `codex/cosmic-fine-detail`, parent `f69d6cbe8d58d182da6e45460b42b963655c5fbc`. Only `crates/game_core/src/cosmic_background.rs` and new `cosmic_detail_tests.rs` changed. No integration, source push, server, public build, art asset, shader, UI, audio, simulation or save change was performed.

The original READY proposal and its baseline evidence remain under `work/cosmic-particle-refinement`. This implementation is a separate candidate for Root integration and normal browser review.

## Concrete visual/data change

The existing galaxy, torn nebula and nova use cached half-unit full source cells with clustered low-alpha ridges, dust, core shading and real empty lanes. Their reduced bodies independently sample the same physical fields at unit spacing instead of max-filling coarse blocks across dark lanes. Fourteen actual source RGBA colors per motif provide restrained plum/dust/amber structure; the brightest small core regions remain localized. The nova retains an explicitly absent arc. Fine source cells are actual local engine quads, not an enlarged old bitmap or a minimum raster-size clamp.

Both densities preserve each part's nominal physical canvas and its existing hierarchy/pose scales. Actual occupied extrema are not identical: the tested full/reduced extrema differ by at most 2.5 physical units. All six pairs are constructed once in `LazyLock`; extraction streams borrowed cells without a new intermediate Vec/body. FrameBuilder still owns its ordinary reusable output allocations; allocation counts and cold initialization latency were not measured.

| Motif / part | Full grid / cells | Reduced grid / cells |
| --- | --- | --- |
| Galaxy arms | 192×192 / 1,253 | 96×96 / 318 |
| Galaxy core | 48×48 / 346 | 24×24 / 87 |
| Nebula lanes | 160×96 / 1,029 | 80×48 / 263 |
| Nebula thread | 96×48 / 224 | 48×24 / 60 |
| Nova rim | 128×128 / 901 | 64×64 / 223 |
| Nova core | 24×24 / 100 | 12×12 / 36 |

Whole motif costs are **Galaxy 1,599/405**, **Nebula 1,253/323**, **Nova 1,001/259** full/reduced, always two parts/transforms. They are below the agreed 2,200/1,100 per-motif ceilings. The twelve cached bodies contain 95,120 color slots; color and occupancy arrays total 392,376 bytes, excluding object headers, initialization strings/temporaries, allocator overhead and RSS. This is source-derived storage, not allocation or process measurement.

The continuous local curl is bounded to 0.14 physical units per axis for the outer part and 0.07 for the core/thread. Existing restrained group rotations, alpha breathing and nova pulse remain. A 0.001-second actual packet step stays below 0.002 physical displacement with at most one byte of color change in the declared cases. Reduced Motion uses time zero and produces exactly equal cells/colors/transforms at time 20 versus 901.

## Contracts preserved

Seeded motif identity, hash probability, unencoded birth origin, 0.08 parallax, conservative culling radius 180, camera-relative encoding and the single renderer camera subtraction are unchanged. Seed selection never comes from the current camera biome; tint remains sampled at motif birth space. Full/reduced changes density, not gameplay or motif identity.

Custom cosmic remains **8,000/4,000 total**, including the **1,800/900 star/system reserve**. Regional gas remains **800/400 inside the separate 6,000/3,000 regional pool**. Gas, stars and stellar-system algorithms are byte-identical normalized functions to the parent commit. The Scar and all regional body code are untouched. Both child parts preflight their complete occupied count against shared remaining capacity; one cell short appends neither part and leaves prior packets/budget intact. The 127-existing-transform proof emits both parts across two valid batches without dropping cells. All source occupancy costs are charged, including any alpha-zero result.

These motifs use default unlit materials (`b[2..] == [0,0]`); breathing is alpha/color presentation, not bloom, an emitted light, a hazard or a gameplay particle. Existing core frame/material ABI and both renderer paths are unchanged. Full-detail motif costs are higher, so an exhausted wide-view budget can omit later whole motifs more often. No cap increase, partial child fallback, identity substitution or new selector was introduced.

## Actual checks

- `cargo test -p game_core --locked cosmic_background -- --nocapture`: 19 passed / 1 explicitly ignored export; 301 filtered. Includes the existing thirteen tests and six new assertion tests.
- `cargo test -p game_core --locked`: **320 passed / 0 failed / 1 explicitly ignored export**, zero doc tests, exact frozen commit. The export is not counted as passed by this run.
- Explicit ignored export invocation: **1 passed**, actually wrote 12 native FrameBuilder packet sets with 36 CameraView zoom views at 0.6/1.0/1.6, times 0/18, three motifs and both densities.
- Strict native all-target core Clippy passed. Strict core `wasm32-unknown-unknown` library Clippy passed. This is portable core compilation, not separately executed WebGPU/WebGL2 or a normal release build.
- Actual tests cover all occupied cell corner composition under nonuniform/reflected/sheared test parents, fractional projected widths below 0.5 logical pixels, ±999,000 far cameras with non-square extents, atomic quotas and 128-transform boundaries, finite bounds, deterministic repeated extraction, seeded view/quality consistency, frozen motion and unchanged Game profile/combat RNG/events.

Commands/logs and source inputs are pinned in `manifest.json`. The first test run caught a reduced nova core-ray gap between coarse centers; correcting the common source sampler retained the ray. Initial failure/warnings and the initial software sheet remain historical. `refine.py` is the historical first scoped editor, not a final regeneration command. The only later test-only edit prints individual cached part counts; the final frozen full suite and native Clippy were rerun afterward.

## Native software contact proof and limits

`packets/` contains actual native extraction and actual CameraView projected corners. `render_contact.py` consumes those polygons and their exact packed RGBA values; it does not recreate the motif algorithm. `native-software-contact.png` and the 36 `software/` images use fourfold offline sampling with flat alpha over near-black. They demonstrate emitted geometry/poses and software silhouettes. They do not run WGSL, lighting, real GPU blending/compositing or a browser. The contact sheet was actually inspected; the software shows muted spiral/torn-lane/open-rim structure, with fine sampling visibly delicate at zoom 0.6.

Root's normal WebGPU and WebGL2 captures must assess final readability, moving subpixel coverage, threat/Mote cue contrast, whole-scene admission and the chosen colors before visual acceptance. Human art approval, physical devices, GPU/phone performance, allocator counts, cold cache initialization and sustained soak remain unmeasured. No claim that every half-unit cell is visible on every raster is made.

## Review/reproduction

Read-only identity verification: `python work/cosmic-fine-detail/verify.py`. Focused peer execution can use `cargo test -p game_core --locked cosmic_background -- --nocapture` and its own log; it will not execute the explicitly ignored output writer. The exported output requires an explicit absolute `COSMIC_DETAIL_PROOF_DIR` plus `--ignored --exact`; peers should preserve the frozen evidence and choose their own owned output path if separately authorized.

Root owns integration, the normal build, browser acceptance, durable board updates and publication. The code branch has not been merged or pushed by this specialist.
