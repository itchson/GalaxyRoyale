# Boss rendering integration review

Reviewed the frozen source read-only after the gameplay handoff. No blocking rendering, damaged-occupancy, transform, weakpoint or pooled-buffer defect was found in the inspected paths. This is source/native evidence; root owns integrated browser captures and physical-device acceptance remains pending.

## Source checks

- `game_core/boss_assets.rs` expands the actual hard-alpha indexed atlas once and builds validated head and segment rigs. It does not substitute source PNG textures or placeholder geometry.
- `Game::draw_boss` receives the current ECS `PixelBody`. `PixelRig` partitions its surviving row-major cells into static relative subobjects; missing cells never return. Both the actor bullet sweep and rendering use `actor_transform`. Presentation interpolation uses the previous/current pose, while simulation collision uses the current pose.
- Coil segments use their own surviving bodies, explicit 1.25 scale and the same quarter-turn heading in bullet collision and rendering. Their configured radii and spacing are derived from actual occupied bodies. The final source consumes the two-part segment rigs. A fully visible generation-five Coil has one seven-part head and nineteen two-part modules: 45 object transforms and 13,233 occupied quads.
- Head damage applies the localized metadata core multiplier only after an actual occupied-cell sweep contact. Telegraph/attack armor uses 0.55 damage; recovery armor uses 1.0, and recovery core-region contacts use 1.65. Core holes remain holes. No independently moving collision-bearing limbs are claimed.
- Sanctuary retreat removes the active boss and its modules without crediting a defeat. The highest earned generation is chosen from saturating promotion queues; only actual boss deaths consume progression through recorded per-generation counters. The single `active_boss` arbitration prevents ordinary concurrent boss spawns. Legacy generation sanitation retains old first-generation defeat totals, bounds higher generations by predecessor defeats, and saturates aggregate counts.
- `renderer.rs` clears scratch contents, preserves capacity, pads every ordered batch to a distinct 128-transform table, and retains each draw's instance range/table index. Growing transform buffers rebuilds both world and UI bind groups. World and UI use separate projections, so zoom and camera shake cannot scale UI glyphs or pads.
- `pixel.wgsl` preserves local affine positions and receives role material values from the fixed 32-byte record. Eight bounded lights shade each cell with a flat per-cell color. UI uses zero-response/zero-emission material. Reactor emission is bounded to 0.6, emitter emission to 0.5; the brighter exposed core changes color/material without altering occupancy.
- The compact engine HUD receives family, generation, phase and telegraph text. Root's panel test bounds labels rather than allowing long phase strings to overrun the health panel.

## Finding resolved during review

The first source pass found that Reduced Motion reached interface animation and zoom but did not reach the new reactor pulse. Gameplay added `Game::set_reduced_motion` and `cosmetic_time()`, shared by boss/module rigs and other cosmetic extraction. Root wired the browser setter and gates shake with `self.shake && !self.reduced_motion`. Simulation movement, phase charge and core-open gameplay cues remain active. Invulnerability blinking is also suppressed in Reduced Motion.

## Measured independent proof

Command: `cargo run --release --locked --manifest-path work/boss-render-proof/Cargo.toml`. The isolated harness lock was seeded from the production `Cargo.lock`; the successful final run used the frozen gameplay source. It writes `work/boss-render-proof.json`.

The harness alternates actual public `stress` diagnostics with all fifteen public `boss-<family>-<generation>` diagnostics, then repeatedly rebuilds the paused world and real HUD at zoom 0.6 and 1.6. All 2,400 frames passed exact extraction/packing invariants: preserved combined counts, stable complete geometry/color hashes for each paused scene, valid instance ranges and table slots, finite camera-relative transformed cell centers, material bounds and unlit UI. Actual maximum combined occupancy was 129,345 quads, with 20 padded transform tables. Capacity growth reached 131,072 instances and 32 tables and correctly handled smaller following boss frames. A separate fully visible live Coil-five extraction verified exactly 13,233 rig cells and 45 rig transforms.

The engine's own 37 tests passed (34 unit plus three whole-atlas tests), as did strict native all-target and WASM-library Clippy. Those tests include destroyed core holes, all twenty asset layouts, exact reflected/sheared nested transforms, silhouette differences normalized to a common size, occupied local core bounds, all seven nonempty head roles, and dimensions 1/128 with the 16-part ceiling. Re-running the art generator produced byte-identical atlas, budget and preview files.

`assets/pixels/boss-native-proof.json` is a separate warmed 500-sample-per-scene native extraction benchmark. Its full rigged Coil-five case measured approximately 0.205 ms median and 0.209 ms p95 on this workstation. The final game's head/module rig semantics now match that case, but the benchmark excludes other game systems. Neither proof measures WASM/browser cost, actual GPU uploads/draws, compositing or phone performance.

## Practical limits

Player/contact repulsion uses conservative body circles; projectile collision respects individual surviving cells and holes. Thus a visually open annular boss is not a physical player passage. This is an existing contact-physics scope choice, not a new pixel-rig mismatch. Head localized weakpoints do not imply a separate segment weakpoint damage multiplier. The rig supports hit-flash tint, while the current boss runtime reports impacts through particles/events and shake rather than a per-hull flash state.

The native packing mirror validates the CPU contract, not the GPU driver. Browser visibility/lifecycle, WebGL2/WebGPU presentation, complete encounter readability and physical phones require their own evidence. The explicit renderer ceiling is 384,000 combined quads; current proof scenes stayed below it, without asserting every future combination will do so.

## Final safe-spawn recheck

After the initial browser capture showed a Coil train crossing the player, gameplay corrected head/module initial heading and placement to extend away from the player. Root froze that final source with 63 core tests and 22 web tests. The independently rerun locked harness again passed all 2,400 frames. Its final report, including each paused scene's checked geometry hash, is preserved at `artifacts/boss-render-proof.json` as well as the work path. Maximum stress occupancy remained 129,345 combined quads with 20 padded tables.

Corrected Coil-five diagnostic extraction at the narrow 640×360 base viewport now emits 10,207 combined quads at zoom 0.6 and 2,655 at zoom 1.6: the train is correctly culled as it extends beyond the top screen rather than crossing the player. The uncropped live assembly gate now uses an explicit 2400×2400 world extent and still verifies exactly 13,233 boss rig cells and 45 object transforms. The intermediate failed uncropped assertion used the old narrow framing and detected expected culling, not restored/missing body cells; the final large-extent gate passes. Browser replacement captures will document the corrected presentation separately.
