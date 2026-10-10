# Production cosmic background handoff

Current production source is `crates/game_core/src/cosmic_background.rs`, called by the scoped `Game::render_background`. The old fixed large Haven clouds, old streamed coarse custom clouds and their per-Game templates were removed. The existing `regions::background_with_density` path, Isotope Scar, combat, saves, rewards and normal particles are unchanged. Root owns the new near-neutral-black renderer clear and actor ambient grade; neither is implemented by this module.

The art is original authored occupancy generated once into cached `PixelBody` objects: two logarithmic spiral arms with actual gaps and facet star knots, a compact galactic bulge, a torn gas filament plus a separate child thread, and broken five-lobed supernova ejecta with a small central remnant. No raster runtime, image/model generation, imported art, external dependencies, per-frame occupancy rebuild or gameplay hazard is used. Reduced templates select an occupied representative per2×2 block and draw at double local scale; they preserve the motif type, not every original vacancy.

Four star layers have parallax factors0.09/0.19/0.38/0.68 and unit-cell sizes0.70/0.85/1/1.25. Each384-unit streamed field caches four actual region samples at its **unencoded physical birth-space corners**. Smoothstep blends density/color across shared corners. Star admission/color is evaluated from each immutable birth position; camera or current player biome never rerolls a whole starfield. A smooth radial falloff gives Haven a quiet central sky. The six authored outer density weights are0.14/0.54/0.80/0.29/0.43/0.65; violet, green, warm iron, muted radiation and cold colors remain restrained point tints.

Motifs have their own1200-unit seeded grid at parallax0.08. `hash(seed ^ 0x636f736d, grid, 83) % 31` assigns Galaxy0, Nebula1, Supernova2, otherwise no motif. Identity is independent of camera, zoom, time and quality. Visibility/budget can omit a whole object; it never becomes another kind of art. Each motif has exactly two local body transforms. Galaxies rotate their flattened arm child by0.008rad/s; nebula threads sway at most1.4local units and their body tilts at most0.035rad; nova shells expand gently and retain a faint remnant rather than flashing the screen. Reduced Motion freezes all local geometry, color and opacity at time0 while camera parallax still follows movement.

Custom extraction is capped at **8,000 / 4,000 occupied quads**, separately from the unchanged region **6,000 / 3,000** cap. Motif preflight reserves1,800 /900 star slots, charges all occupied cells including faint/overlapping ones, and accepts or rejects both body parts atomically. Stars submit as grouped local cells through `push_cells_affine_material` with scalar/fixed-array field scratch; no tiny absolute-position `push_pixel` is used. Cached art initializes once; normal FrameBuilder storage growth remains possible. All custom materials are unlit, emission0. Existing128-transform batch splitting is retained.

The parent encodes `origin + camera * (1-factor)` once. The supplied camera is the interpolated **unshaken extraction camera** and viewport is already divided by eased zoom. The renderer subtracts camera before multiplying small local cells. Browser shake is applied later. Module inputs reject invalid camera/viewport/time and clamp finite practical camera/world viewport bounds to±1M /8192.

## Actual validation

Nine focused native tests pass, all **140 core tests** pass, and `cargo clippy -p game_core --all-targets -- -D warnings` passes. No dist/WASM/browser build was performed by this agent. Tests cover deterministic seeded revisits, smooth shared birth-space edge values, actual sparse/dense star extraction, source profile/RNG/events unchanged by rendering, real seeded motif packets across both qualities and zoom extents, moving/animated packets, exact Reduced Motion freeze, atomic budget rejection with prior frame cells, far-world local projection, invalid no-op and authored negative space.

In32 declared seeds at a533×300 world extent, Haven emitted584stars total (18.25mean) and the declared outer position[27000,-19000] emitted2162 (67.5625mean). These are deterministic fixture counts, not playtest or device scores. Default-seed centered motifs for later normal browser warp inspection are:

| Motif | Camera/world warp center | Actual full / reduced motif quads |
|---|---|---:|
| Galaxy | [13387.087,61155.58] | 978 /439 |
| Nebula | [29410.637,11408.639] | 694 /324 |
| Supernova | [-20372.748,30055.252] | 473 /228 |

`proof.rs` compiles the **actual production module** directly with its exact extracted pure hash helpers and actual current Game region library. `source-pins.json` records source/library identities. It extracts32 native custom-background frames: four seeded centers, two qualities, two live times and two Reduced Motion times. `packet-proof.json` verifies counts, unlit materials, transform bounds and eight byte-identical Reduced Motion pairs. `contact-sheet.png` presents24 software views across zoom0.6/1/1.6; no region planets, Scar, actors, hostile shots or UI are composited into these art-only views.

The software preview uses current straight-alpha/pow2.2 shading with Root's declared neutral clear[.0001,.00011,.00014]. It approximates convex-quad pixel-center coverage, not GPU/GL raster edges, framebuffer/display parity or threat readability. At minimum zoom individual star and motif cells can be subpixel.

An optimized standalone custom-module benchmark with existing debug native dependencies, reused FrameBuilder,30warmup and300samples measures median/p95 extraction milliseconds: Haven0.0403/0.0416, Galaxy0.0704/0.0732, clamped8192² far-world extent0.4827/0.4932. Last frames contain18/1046/4418quads respectively. This is desktop native CPU extraction only: no complete Game tick, WASM, browser GPU, sustained thermals or mobile FPS claim.

Root's normal dual-backend build, seeded warp captures, moving/crowded Mote/boss contrast, perceived galaxy/nova motion and physical Pixel6a/iPhone12 remain Pending. The prior Vesper, warning and palette freezes are unchanged and unapplied by this task.
