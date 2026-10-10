# Production same-art planet LOD handoff

Production source is frozen after the authorized regions-only implementation. `crates/game_core/src/regions.rs` now caches half-resolution copies of the existing four planet and two split-ring templates, doubles their extraction scale under Reduced Cosmetics, and preflights complete planet/ring or nova/spoke groups against the unchanged shared region budget. No new artwork, field selector, palette, material, gameplay state or save schema was introduced.

Source SHA256:

| File | SHA256 |
| --- | --- |
| `crates/game_core/src/regions.rs` | `5119593dc07619072fcead20ebb797d28dce7c66d69c0f6b164c3000cce7fd47` |
| `crates/game_core/src/regions_lod_tests.rs` | `cc047430b93cdf0d06c05b57cce9112d677fd08a468a0091098069d3210b741d` |

Against the 46 production inputs pinned before this task, only `regions.rs` changed; all other 45 input hashes remain identical. The dedicated new test module is the other owned source addition. No `lib.rs`, browser, renderer, manifest, dist, published archive or previous frozen work artifact was changed by this task. Root owns the following normal package build and browser art checks.

## Exact costs and behavior

| Authored group | Full quads | Reduced quads | Object transforms |
| --- | ---: | ---: | ---: |
| Unringed planet | 4,404 | 1,152 | 1 |
| Each split ring | 300 | 106 | 1 |
| Complete ringed planet | 5,004 | 1,364 | 3 |
| Nova core with all 24 spokes visible | 4,428 | 1,176 | 1 body + 1 shared pixel-size transform |

At viewport edges, nova cost is its actual body count plus the number of visible spokes (0–24). The fixed-size stack array computes that count before the body append. An insufficient budget emits neither the core nor any spokes. A fitting full nova retains the previous exact body/spoke packets, color and order. Full planet groups keep original back-ring→surface→front-ring packets when admitted, and now reject as a whole instead of producing orphan rings.

The low cells retain any occupied source 2×2 block and use its existing majority shade with stable row-major tie-breaking. All eight original ordinary-planet shade indices remain; the old nova surface keeps its original four shade indices. Local 48×48 planet / 64×24 ring bodies stay centered and retain their nominal world dimensions through doubled scale. Original per-cell contours are intentionally simplified: the block expansion is bounded by sqrt(2) source-local units, so this is not an exact original silhouette claim.

Gas and earlier layer priority, 6,000/3,000 submitted-cell caps, reduced stable half-field selection, original hash/biome/palette/rotation/parallax, Scar replacement and frozen cosmetic time are unchanged. There is no alternate-art fallback when a complete group cannot fit. New low planets and the old nova can now spend reduced budget that previously went unused; that can legitimately omit later groups. The layer's hash identity is unchanged, but every selected object is not guaranteed admission.

The six additional cached low bodies have fixed 12,288 stored color slots and 192 occupancy words: 50,688 bytes before body/LazyLock headers and one-time authoring temporaries. They initialize lazily once, not per frame. There is no new per-frame heap allocation for the planet helper or nova's fixed 24-item visibility list. This is a source-derived storage count, not an RSS or new cold-frame benchmark.

## Actual validation

On the current production source, `cargo test -p game_core --locked` completed with **148 passed, 0 failed**, including the retained 140 tests and eight new focused regressions. `cargo clippy -p game_core --all-targets --locked -- -D warnings` completed successfully. These were native development-profile checks; no WASM or dist build was performed by this agent.

The new regressions exercise real cached `PixelBody` occupancy and original shade membership; exact and one-below group budget edges; actual back/surface/front packet order; rejection preserving already emitted pixels; 127-transform prefix rollover; nova core/spoke atomicity at fully visible and viewport-edge positions; fitting full-nova packet parity; actual seeded reduced landmarks for all three ordinary kinds in all five non-Scar biome palettes; repeatable zero-cosmetic-time scenes and 6,000/3,000 caps; zero lighting response/emission; and actual cell corners under rotation at near/±999000.3 cameras. The actual max-scale ringed corners remain inside 46 world units, far within the existing 135-unit ordinary visibility margin. The inherited exact Scar packet and gameplay/progression tests also pass.

The historical pre-integration 180-scene LOD study remains immutable at `work/cosmic-lod-followup`. Its 88/90 low ordinary-group admission figure describes the earlier ordinary-only work candidate, not this production composition with the newly retained low nova. Do not quote that figure as the production visible admission rate.

Actual browser/GPU appearance, fine-detail retention at zoom extremes, moving/crowded threat contrast, physical phone frame rate and long-session behavior remain Pending. The production change corrects the proven absent-planet/orphan-ring budget behavior; it does not establish human art acceptance.
