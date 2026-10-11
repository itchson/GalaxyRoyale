# Native nebula morphology revision

Frozen source: `dca11c3a0e24c81e6f62f4302253aeaddbd87814`, parent `27e0b403e96689598eb57fb91805d315de1c3e08`, branch `codex/nebula-natural-morphology`. Root integrated this as `3fae2b0`. Only `cosmic_background.rs`, `deep_nebula_tests.rs`, and `cosmic_detail_tests.rs` changed. Tracked checkout is clean. No preview, browser, shared manifest, shader, gameplay, or source-art atlas change was made.

## Problem and implementation

The persisted Root `artifacts/co-op-native-integration-20261012/nebula-webgpu-final.jpg` has two conspicuous enclosed oval cavities. Its SHA256 is `b8c0a4b84c8d8ce237cba9fa951030354925ee28e9a4778fb1cfe3e396a8e6b9`. The source combined elliptical cloud envelopes and three elliptical negative pockets with two roughly parallel hard ridges.

The revision authors uneven branching paths with varying widths, broad faint rough haze, connected gas knots, short fine threads, and tapered dust clefts entering from the outer sky. Correlated multiscale variation breaks uniform edges; it does not replace authored structure with independent random noise. Density and cell color/occupancy are cached once. The veil threshold is 0.06 to retain faint outskirts. No new per-frame heap work is introduced by these source samplers.

Existing palettes, material encoding/emission, tint functions, seeded selection, 0.08 parallax, world motion, Reduced Motion freezing, culling, and whole-object preflight are retained. Four parts/transforms remain. Full source costs 1,672 cells (448 veil + 274 billows + 950 fine), reduced costs 434 (118 + 70 + 246), below the unchanged 2,200/1,100 motif bounds. Cosmic 8,000/4,000 and star reserves 1,800/900 are unchanged; regional gas 800/400 belongs to its separate 6,000/3,000 region budget and was not edited. Materials remain unlit; source shade/emission is not a physical world light or gas simulation.

Primary morphology references were read, not imported: NASA's [NGC 3627 views](https://science.nasa.gov/asset/webb/webb-and-hubbles-views-of-spiral-galaxy-ngc-3627/) and [M81 image explanation](https://science.nasa.gov/asset/hubble/hubble-photographs-grand-design-spiral-galaxy-m81/). They inform uneven diffuse dust lanes and clustered structure. The approved native sports UI/logo is unchanged.

## Actual checks

- `cargo test --locked -p game_core cosmic_background:: -- --nocapture`: 28 passed, 3 explicit exporters ignored; `final-native.log`.
- `cargo clippy --locked -p game_core --all-targets -- -D warnings`: passed; `final-clippy.log`.
- Explicit existing `export_actual_deep_nebula_packets` native test: 1 passed; `final/export.log`. Six actual FrameBuilder/CameraView packets cover full/reduced, time 0/18/36, each at zoom 0.6/1/1.6. The last exporter ran after final runtime source changes and before the final test-only oracle update; the exported runtime source matches the committed source.
- New flood oracle reproduces baseline enclosed source regions of 97 and 98 samples (`baseline/cavity-regression.log`). Final has only a 2-sample isolated fleck and passes the same <=48 bound. It tests the observed failure rather than an exact output count.
- Final 121x81 density sampling has 2,276 diffuse samples, 1,732 dense, and 5,793 sky. Tests require bounded finite density, diffuse transition exceeding dense knots, substantial knots, asymmetric outline, and predominantly dark surroundings.
- Existing native tests still cover far-coordinate projection, fractional/reflected transforms, seed/revisit, motion/Reduced Motion, unlit materials, cell budgets, atomic short-quota rejection, and crossing the 128-transform batch boundary. No full workspace/WASM build was run by this author.

Draft/baseline negative logs remain historical evidence. They are not final passing results.

## Images and browser reproduction

`final/native-software-contact.png` and the 18 views in `final/software/` were generated from those actual native packets with the retained NumPy/Pillow radial-material model in `final/render_contact.py`. It approximates radial coverage, source alpha/emission and gamma at 2x logical raster with 2x offline sampling; it does not execute WGSL. The first column isolates the same candidate's hard cells (the inherited script wording "unchanged filament packets" does not mean old source geometry). The baseline sheet and packets are retained separately. Full and reduced final sheets were visually inspected: diffuse irregular outlines replace the two oval holes; the faint thin threads still need browser inspection.

For the declared default seed **1196576601**, motif hash **1087913753**, the native camera is **[29410.63671875, 11408.638671875]**. On an isolated QA save/origin with `?qa=1`, the existing console command is:

```text
WARP 29410.637 11408.639
```

This location is seed-specific; it is not an assertion about another saved profile. Existing WARP deliberately stops the run/discards carried materials and suppresses QA rewards, so use isolated QA state. No `SCENE nebula` command is claimed. Root's normal GPU/WebGL2 reproduction, threat contrast, moving-view presentation, human approval, phone testing and soak performance remain pending. This author performed no browser activity.

## Peer coordination

Engine confirmed no cosmic file overlap and supplied the preserved material/budget contract. Gameplay was busy implementing real RTC/WASM acceptance and explicitly has not inspected or accepted this candidate. Root inspected the final saved software view and found the diffuse silhouette improved; that is software review, not final GPU acceptance.

`manifest.json` pins the committed inputs, all retained local evidence, and the Root baseline screenshot. `verify.py` verifies bytes without building or writing outputs.
