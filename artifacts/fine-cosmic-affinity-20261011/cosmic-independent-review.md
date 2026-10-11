# Independent fine-cosmic source and packet review

Scoped acceptance: no concrete blocking defect found in frozen `aff95d69b3055191355bfa619a5ea307da852c3c`, parent `f69d6cbe8d58d182da6e45460b42b963655c5fbc`. This is an independent source/native-packet review, not browser or final art acceptance. Only `cosmic_background.rs` and the new `cosmic_detail_tests.rs` changed. Peer source and archived proof were read only; no production, dist, server, API, Studio, dependency or peer evidence mutation occurred.

## Actual independent checks

- Exact peer checkout HEAD and clean tracked status verified. Its read-only verifier passed **69 committed input pins / 84 raw evidence pins / 12 actual packets**.
- Reran `cargo test -p game_core cosmic --locked -- --nocapture` against that exact checkout: **19 passed, 0 failed, 1 ignored, 301 filtered**. The ignored output-writing exporter was not rerun. This includes the thirteen retained presentation checks and six new geometry/budget/motion tests. The peer's full-suite/Clippy results are separate owner evidence, not my execution.
- Independently parsed all twelve frozen exports: **36 CameraView views / 29,040 submitted cells**, times 0 and 18, three motif kinds and both quality modes at zoom 0.6/1/1.6. Every exported count matched the actual packet length, both transforms were present, all four corners were finite, all RGBA bytes were bounded and alpha positive, and cell side lengths were positive. This validates exported data, not a second native exporter execution.
- Viewed the actual native software contact sheet. Its distinct spiral, torn lane and open rim are apparent in the software polygons; they remain delicate at minimum zoom. The sheet uses flat-alpha offline supersampling and is explicitly not GPU rendering or moving-threat contrast evidence.

| Whole motif | Full quads | Reduced quads | Transforms |
| --- | ---: | ---: | ---: |
| Galaxy | 1,599 | 405 | 2 |
| Nebula | 1,253 | 323 | 2 |
| Nova | 1,001 | 259 | 2 |

All costs are below the agreed 2,200/1,100 per-motif ceiling. Full source uses half-unit cells, reduced uses unit cells. Actual packet-side measurements include full-detail lengths below 0.5 logical units at zoom 0.6; there is no authored minimum-size floor.

## Source conclusions

**Placement and quality:** `motif()` keeps the same seeded grid/hash/origin/scale; `render()` enumerates the same fixed fields, and tint is sampled from motif birth space. Camera/zoom do not select different art. A quality change selects the two cached versions of the same motif. Their physical canvases and hierarchy scales match; their *occupied outlines are not identical*. The actual source-extrema test allows and proves at most 2.5 physical units of difference. This precision is correctly stated in the peer handoff and should remain in integration descriptions.

**Budget and batching:** `draw_motif()` preflights the sum of both occupied bodies before any append, then charges the entire submitted occupancy. One cell short leaves both parts, caller quota and existing packets unchanged. The actual 127-transform-prefix test verifies a complete two-part admission across the 128-transform batch boundary with valid indices. The retained wide-view matrix checks full/reduced custom totals at near/far cameras and all three zooms: actual quads equal stars plus motif quads, never exceeding **8,000/4,000**. The **1,800/900** star/system reserve is preserved. Regional gas remains **800/400 within its separate 6,000/3,000 regional cap**; regions/Scar are unchanged. Higher full-detail costs may omit a later whole motif in crowded wide views. Omission is a bounded admission result; it never substitutes a planet or another motif identity.

**Transforms, animation and materials:** local borrowed cells flow through composed affine transforms; renderer camera subtraction still happens once. The executed tests cover reflected/nonuniform/sheared parents, fractional projection, ±999,000 cameras, finite corners and conservative radius 180. Curl is bounded to 0.14 physical units per axis for the first part and 0.07 for the second. A small declared time step changes geometry continuously; Reduced Motion sets cosmetic time to zero and the tested colors/cells/transforms are exactly frozen. Default hard material `[0,0]` remains unlit. No light source, hazard, collider or soft-glow layer is invented. Gas/star/system algorithms were not changed by this commit.

**Gameplay:** `lib.rs`, regions and engine frame/camera inputs are unchanged from the parent. The module mutates only the presentation builder and its local quota. It neither owns ECS state nor advances combat RNG, rewards, XP, profile fields, collisions or save schema. The retained actual Game render test again verified Profile bytes, RNG and event count unchanged by render/quality/motion settings.

**Allocation scope:** the twelve cached bodies are initialized once, then extraction streams their borrowed cells without a per-part intermediate Vec/body. This is a source observation. Cold initialization, output allocator calls, complete Game frame time, GPU upload and performance were not measured in this review.

## Pins and remaining gates

`input-pins.json` pins exact Git bytes and all twelve packet files. `packet-inspection.json`, `native-tests.log`, `peer-verification.log`, this report and the reviewed peer handoff/manifest/contact are pinned by `manifest.json`. The pre-change `BASELINE.md` and its pins remain unchanged.

Normal integrated WebGPU/WebGL2 captures must still assess moving subpixel coverage, low-alpha visibility, background/Mote cue contrast and whole-scene admission. Human art approval, phone/physical hardware, browser FPS, cold load, allocation measurement and sustained soak remain Pending. A finite projected quad does not prove that every cell wins a raster sample or reads clearly against combat.
