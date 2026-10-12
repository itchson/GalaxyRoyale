# Deep nebula gas — native implementation

Frozen commit **c3904b8f57b9c96cf5e5f2b5672b3bc9e3d020a5**, branch `codex/deep-nebula-gas`, parent **56af06fe7c6ec16c551e02ab5fb1ab92483a5430**. Only `cosmic_background.rs`, `cosmic_detail_tests.rs`, and new `deep_nebula_tests.rs` changed. Tracked checkout is clean. Root owns integration, browser review and publication.

The rare nebula gains two cached overlapping gas volumes behind its existing fine hard-cell filaments. Smooth multi-scale noise, a warped birth field, sparse dust-channel occupancy, localized warm ridges and low-alpha source colors form billows instead of isolated dots. The template is sampled once at two densities. Game seed still determines motif identity, origin, rotation, scale and birth-space tint. Extraction streams borrowed cells without cloning a body or constructing a per-frame list.

| Nebula part | Full source / cells | Reduced source / cells | World spacing / soft footprint |
| --- | --- | --- | --- |
| Diffuse veil | 40×24 / 438 | 20×12 / 105 | 3 / 8.1 full; 6 / 16.2 reduced |
| Turbulent billows | 48×32 / 437 | 24×16 / 110 | 2.5 / 6 full; 5 / 12 reduced |
| Fine lanes | 160×96 / 1,029 | 80×48 / 263 | Existing 0.5 / 1 hard cells |
| Fine thread | 96×48 / 224 | 48×24 / 60 | Existing 0.5 / 1 hard cells |

Complete cost is **2,128 full / 538 reduced cells and four transforms**. All four parts preflight one shared remaining quota before any append. One cell short preserves prior packets and budget. The 127-existing-transform case keeps the complete cloud across two valid batches. Alpha-zero submissions still cost cells. Galaxy/Nova geometry and the regional gas helper are unchanged.

The custom pool remains **8,000/4,000**, including the existing **1,800/900 star/system reserve**. Regional gas remains **800/400 charged inside the separate 6,000/3,000 region pool**. No cap, probability, fallback or gameplay change was introduced. Nebula cost increases by 875/215 cells and two transforms, so crowded views may omit more whole motifs.

Nominal volume canvases are 120×72 and 120×80 world units before motif scale/rotation. These broaden the old narrow ribbons; they do not enlarge collision or interaction geometry. Full/reduced canvases agree, while occupied corners and soft footprints differ. Every tested corner remains inside the existing 180-world-unit conservative culling radius, including ±999,000 cameras and non-square views. Birth-origin encoding and **0.08 parallax with one renderer camera subtraction** remain unchanged.

Distinct continuous curl fields move veil and billows by at most 1.472 and 0.9216 world units per axis before parent scale. Slow relative rotation and alpha breathing give restrained depth. Reduced Motion actually sets cosmetic time to zero: cells, colors and transforms are byte-identical at time20/901. Source shade plus existing soft material emission0.04/0.09 provides a small brightness adjustment. Light response is zero. No point light, bloom, shader, material ABI, hard-quad minimum or renderer path was added.

## Actual checks

- `cargo test -p game_core --locked cosmic_background -- --nocapture`: **24 passed, 0 failed, 2 explicit exporters ignored**.
- `cargo test -p game_core --locked`: **373 passed, 0 failed, 2 ignored**; zero doc tests.
- Strict native `cargo clippy -p game_core --locked --all-targets -- -D warnings` and portable-core WASM library Clippy passed. This is not a new normal dual-backend build or GPU execution.
- Explicit new ignored exporter: **1 passed**, writing six actual FrameBuilder/CameraView packets and 18 projected views for both densities, time0/18/36 and zoom0.6/1/1.6.
- Tests retain seed/revisit and simulation profile/RNG/event checks, atomic budget guards, exact transform-slot accounting, all-corner far bounds, reflection/shear, arbitrary fractional geometry, actual camera movement, continuous small time steps and Reduced Motion equality.

The first whole cosmic run retained four obsolete two-part/default-material assertions; its failure log remains `legacy-contract-first-tests.log`. Updated tests preserve their geometric/budget intent while accounting for the two explicit soft parts. No final test failure remains.

## Software proof and limits

`native-software-contact.png` and `software/` consume actual native packet centers, bases, materials and RGBA. `render_contact.py` models the existing radial/coverage/emission shader equations offline in linear color, using a declared 2× physical raster and 2× offline sampling. It does not execute WGSL. Its left column isolates the same candidate's unchanged hard filaments; it is not a separately built historical package. The sheet was actually viewed: broad muted billows, a persistent torn channel and subordinate fine ridges remain coherent in full/low and all three zooms. Art peer's read-time software inspection found no concrete art blocker; normal GPU/threat contrast remains pending.

Both backend paths continue to consume the same existing `pixel.wgsl`; the new packets use its existing signed-emission soft encoding. Root must verify settled/moving WebGPU and WebGL2 appearance, whole-scene admission and foreground cue contrast. No browser, physical device, human play, GPU timing, allocation-count, cold-cache latency or RSS measurement occurred here.

The added cache stores 3,120 color slots plus occupancy: **12,872 source-derived bytes**, excluding headers, initialization strings/temporary allocations and allocator overhead. Standard FrameBuilder capacity growth remains possible. No zero-allocation or performance claim is made.

`manifest.json` pins current working bytes, committed source blobs, relevant unchanged engine/shader inputs and actual evidence. Focused tests do not execute either ignored output writer. Frozen outputs should not be regenerated in place by a peer.
