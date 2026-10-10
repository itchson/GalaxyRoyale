# Standalone boss HUD layout prototype

This directory contains a dependency-free Rust presentation prototype. It does not edit or execute the production game, renderer, browser, or deployed website. The source was frozen after the bounded-cache revision on 2026-10-10 at 18:53 UTC. Root owns production integration.

## Contract

`LayoutController::update(&FrameInput) -> Outcome` preserves the existing `FrameInput`, `WorldDisc`, and `Outcome` fields. The controller returns the complete `Placement`, including its exact rectangle; retaining only an anchor or width loses the free-search position.

Coordinates are full logical-screen pixels. World zoom never scales the panel. Returned origins are integral and the safe area is snapped inward. Supported input extents are finite positive values up to 4096 by 2160, at most 64 valid reserved rectangles, and the actual eased zoom from 0.6 through the promoted f32 1.6 endpoint. Larger extents are `InvalidInput`, distinct from a valid but overconstrained `NoClearAnchor`.

The panel height is always 40. Width preference is 220, 168, 140, then 100. Every candidate stays inside the safe area with a 12-pixel margin. A genuinely overconstrained view is allowed to return no placement; the controller never forces a panel over the player's ship, visible vulnerable core, or a reserved control.

A visible core receives a protected radius of `max(32, world_radius * zoom + 8)`. The player receives `max(24, world_radius * zoom + 4)`. These values are radii, not diameters. Actor scale must already be included in the supplied radius. Larger objects enlarge protection; 32 and 24 are floors, not caps. Partly visible discs are protected. Completely offscreen discs are ignored for intersection checks. Disc tangency and a reservation's expanded edge count as blocked. Reservation clearance is 4 logical pixels.

## Preferred positions and compact rescue

The 220 by 40 panel tries upper left, upper right, then lower center. The upper pair selects the greater distance from the projected boss head; a tie selects left. Upper origins are 12 pixels from the safe side and 68 from its top. Lower center sits 98 pixels above the safe bottom.

Compact widths add middle left/right, lower left/right and upper center. If those anchors fail, an integer-origin search examines all permitted rows, using sorted forbidden x intervals to skip obstacles. Its fixed arrays contain at most 66 intervals. The emitted candidate is always validated by the independent closest-point predicate.

Both directions of floating-point interval error are tested: an underestimated tangent retries the next integer, and an overestimated interval probes its integer boundary before skipping it. A 1,200-case direct brute-force oracle independently enumerates integer origins and compares the exact result. The finite row/rectangle limits bound the work; they do not make every fresh synthetic search cheap.

## Retention, hysteresis and cache

An unsafe retained panel relocates immediately. A safe panel moves to a meaningfully better candidate after a continuous 350 ms hold or a real phase-epoch change. Same-priority, same-width switches require 24 pixels of clearance improvement. Returning to a wider panel or a preferred anchor tier bypasses the 24-pixel threshold but still requires the hold or phase change. Free-search challengers match their full rectangle, so a moving candidate cannot inherit another rectangle's hold.

The primary fixed anchors are evaluated on every valid frame. Compact preference reuse is limited to 350 ms from the fresh fallback search, and requires both the retained rectangle and the exact cached preference to pass current core/player/control checks. A changed screen, clock rewind or phase change refreshes that preference. An unsafe current or cached preference forces a fresh search on that frame.

An exact no-fit result can be reused without a time limit only while raw screen, projection, core, player and reserved contents/length are unchanged. This is a full equality key rather than a hash. Any geometry-key change forces fresh evaluation, and tests verify that opened space is discovered on the same frame. Head position is excluded from the no-fit key because preference cannot create empty space.

Successful caching can defer discovery of a purely preferred safe compact rectangle by up to approximately 350 ms before its separate 350 ms challenger hold. Consequently that optional change can take approximately 700 ms. Safety relocation remains immediate; newly clear primary anchors remain evaluated every frame.

Invalid input preserves state. A no-fit frame clears the pending challenger but retains the remembered placement for future safety validation. Call `reset()` on encounter identity changes, modal/dialogue boundaries, hidden/resumed presentation, and other discontinuities. Invalid or unobserved gaps may otherwise allow an existing wall-clock hold to complete when valid updates return. Clock rewind resets pending promotion.

## Production adapter handoff

Use the engine's actual `CameraView::world_to_screen` for the interpolated player, boss head and authored core. It must use the same eased zoom and shaken presentation camera as world rendering. Both actor position and heading interpolate; a rotated core must be derived from that same interpolated transform.

Root's agreed adapter feeds `Projection { camera: (0, 0), zoom: 1 }` and recentered projected points: subtract half of the full logical extent from each screen point. Supply already-zoomed radii so the adapter adds only logical clearance. This shares renderer projection instead of introducing separate f64 rounding.

Safe insets remain in full-screen coordinates for the controller. Only subtract the actual left/top safe inset immediately before native UI drawing, whose origin adds that inset back. Do not scale safe-origin offsets with world zoom.

Supply visible native biome/currency/radar/hull/touch areas plus actual clipped semantic button, quick-menu, toast and mobile hit bounds. Use the current visible bounds during transitions. Filter hidden or zero-size controls; do not pass invalid rectangles. The 64-rectangle limit is explicit.

The returned width drives text clipping and compact naming. At a one-pixel RoyaleMono scale, `G5 / BASTION` and `CORE EXPOSED` each occupy 71 pixels with six-pixel glyph advances, fitting a 100-pixel panel with 14 pixels of inset. Health, generation/family and phase must remain readable. The geometry tests do not prove font, color or browser readability; root must exercise the actual 100-pixel rendering before accepting it.

The largest existing owned player body is Haven Scout at approximately 19.90026 world radius after its 0.7 actor scale. Sovereign is approximately 19.11 and the other campaign hulls are smaller. Tests use a conservative 20-world-unit radius, protecting 36 logical pixels at zoom 1.6.

## Native verification

Commands from the workspace root:

```powershell
cargo test --manifest-path work/boss-hud-layout/Cargo.toml
cargo clippy --manifest-path work/boss-hud-layout/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path work/boss-hud-layout/Cargo.toml --check
cargo run --release --manifest-path work/boss-hud-layout/Cargo.toml
```

All exit with code 0: 11 private unit tests and 31 integration tests, strict Clippy clean, formatting clean. The tests include the promoted f32 zoom endpoints; fractional asymmetric insets; exact disc and widget tangency; 24-pixel and 350-ms boundaries; interrupted candidate identity; phase/rewind/reset; invalid-state preservation; changed-core/control cache invalidation; opened-space recovery for every geometry class; 1,600 deterministic frames; 8,190 viewport/zoom/core samples; and 631 actual-largest-player samples at 533 by 300.

The release geometry replay emits 2,400 panels: 1,255 upper left and 1,145 upper right, with zero no-fit frames. Compact fixtures separately exercise the retained 100 by 40 search rectangle at (12,45) on a 320 by 180 view with seven native-widget fixtures. These widget fixtures are source-derived geometry, not recorded browser hit bounds.

Runtime measured on the Windows workstation with Rust 1.97.0. Timing is native CPU geometry only:

| Scenario | Samples | Median | p95 |
| --- | ---: | ---: | ---: |
| Normal 533 by 300 retained | 500 | 0.100 microseconds | 0.200 microseconds |
| 320 by 180, seven widgets, cached rescue | 500 | 0.200 microseconds | 0.200 microseconds |
| Same compact case, fresh controller/search | 500 | 13.800 microseconds | 14.000 microseconds |
| 4096 by 2160, 64 covering stripes, exact cached no-fit | 500 | 0.300 microseconds | 0.300 microseconds |
| Same maximum synthetic case, fresh search | 30 | 9.767 milliseconds | 12.046 milliseconds |
| 960 by 540, 64 stripes, fresh search | 30 | 0.331 milliseconds | 0.335 milliseconds |

Cache-hit measurements hold the input geometry and presentation timestamp fixed. Fresh measurements reset the controller for each sample. An earlier uncached run reached 12.364 ms p95; this variability is disclosed rather than treated as an acceptance threshold. The 2160-row case is a maximum synthetic bound; root reports production logical height at most approximately 539. Changing adversarial geometry requires a fresh search and is not accelerated by an exact no-fit cache.

`results/test.log`, `results/run.log`, `results/checks.log` contain the actual final command output. `results/hashes.json` records frozen source/test executable hashes. The independent reviewer ran the test binaries separately.

## Remaining acceptance

Root must verify the integrated camera adapter, actual semantic reservations, compact labels/phase/health legibility, reduced-motion behavior and immediate escape from moving targets in real browser scenes at zoom 0.6 and 1.6. The source proof does not exercise gameplay, GPU upload, WebGPU/WebGL2 compositing, touch behavior or physical phones. It does not claim shipping or phone acceptance.

