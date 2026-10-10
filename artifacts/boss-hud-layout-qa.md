# Independent boss-HUD layout prototype QA

Review scope: isolated `work/boss-hud-layout/` source/tests and comparison with the current pixel projection/UI geometry. Only this QA note was written by the reviewer. The existing compiled test executable was run without rebuilding or editing the prototype. No production renderer/gameplay, browser or physical device was exercised. Published runtime `65194683a28b00397ed3f29df989149b8d93412e` is unchanged by this review; task `task-426419e2` remains the next prototype/integration work.

## Result and actual evidence

No blocking safety defect remains in the frozen compact/cache prototype's emitted placements. The final independent runs passed **42/42 tests** at 18:53 UTC on 2026-10-10: 11 unit tests in `boss_hud_layout_prototype-3e2594f3ab7f2858.exe` and 31 integration tests in `layout-6a86070fd67355a5.exe`, both under `work/boss-hud-layout/target/debug/deps/` with `--test-threads=1`. Earlier independent 28- and 35-test runs covered the original anchors and pre-cache compact source. Engine separately reported strict all-target Clippy, a 2,400-frame native geometry replay and native CPU timing; these are engine-owned evidence, not browser acceptance or reviewer-measured performance.

The compiled tests cover an 8,190-input viewport/zoom/core-position sweep, 1,200 direct brute-force search comparisons, deterministic replay, core/player protection, both directions of floating-point tangency rounding, hard widget exclusions, partial visibility, invalid/overflow input, fractional safe areas, f32 zoom endpoints, immediate unsafe-anchor movement, 350 ms hold, 24-pixel improvement/interruption, phase changes, small jitter, clock rewind, compact rescue and exact free-search challenger identity. Synthetic widget rectangles are explicitly fixtures, not captured physical-phone hit bounds.

SHA256 fingerprints verified during the final independent run:

| File | SHA256 |
| --- | --- |
| `work/boss-hud-layout/src/lib.rs` | `97ae0496ec27b89510b46ad21785a377d2ca6ae0d88d1fe95dcbc30a1758e353` |
| `work/boss-hud-layout/tests/layout.rs` | `7fec0734936cb9fa163f83e6374315b30850f0ce3501c0b3201bbb78179910d6` |
| `work/boss-hud-layout/Cargo.toml` | `eb69c174c8634cbd4ec24e9f50df10080d7f2703ffb272eae22c550983012d60` |
| Compiled unit executable | `c1f0d158be75237545df77626cb8e82bb19d6c84047094b2656636f17156a961` |
| Compiled integration executable | `1a5d14efdadabf668e8fc7280b507805d0d64db36fa2923dc4a76e1fcbca5287` |

## Projection and geometry contract

The primary 220×40 candidates are safe-area-relative upper left `(12,68)`, upper right `(W−232,68)` and lower center `(floor(W/2−110),H−98)`. When blocked, the frozen prototype tries 168/140/100×40 compact panels at additional fixed anchors, then searches integer origins inside 12-pixel safe-screen margins. Safe edges snap inward to integer logical pixels. Returned rectangles use full logical-screen coordinates; their width/height do not scale with world zoom.

World focus projects as `(world − actual_render_camera) × actual_eased_zoom + full_logical_viewport/2`. This agrees with the separate world/UI projections in `crates/game_web/src/renderer.rs`. The camera must include enabled shake, and focus positions/radii must come from the same interpolated presentation time. Root is adding `Game::render_focus`; the published `BossHud` positions alone do not establish interpolated render geometry.

Safe-area insets must not be multiplied by zoom. For a declared 533×300 full viewport, `[top,right,bottom,left]=[13,7,5,31]`, and world offset `(40,−60)` from the render camera, the reference full projection is `(290.5,114)` at 0.6 and `(330.5,54)` at 1.6. Safe-relative coordinates are `(259.5,101)` and `(299.5,41)` after subtracting left/top once. These are independent arithmetic reference values, not claimed browser observations. The prototype correctly keeps full projection separate from inset-adjusted candidates.

Root identified a real draft type-boundary defect: production `CameraZoom::value()` is f32, whose 1.6 endpoint promotes to `1.600000023841858` in f64. The draft literal f64 upper bound rejected it. Engine corrected the bound to the actual promoted endpoint and added a test for 0.6/1.0/1.6 f32 values plus rejection of the next f32 ULP above maximum. That regression passed in the independent run.

Core and player protection use **screen-space radius floors**, not diameters: core 32 with eight logical-pixel padding; player 24 with four. Effective radius is `max(actual_scaled_world_radius × zoom + padding, floor)`. The floors are not caps. Any partly visible disc remains protected; exact edge/corner contact is treated as occupied. Reserved widgets have a strict four-pixel clearance. Head position guides soft preference, but the entire head armor/tail silhouette is not a hard protected region in this contract.

Current native exclusion geometry is biome `(12,12,160,28)`, currency `(W−214,12,142,24)`, centered hull/cargo `(W/2−88,H−48,176,36)`, and home radar `(W−60,H−144,48,48)`. Touch visuals center at `(56,H−50)` and `(W−48,H−50)` with radius 28; actual semantic hit rectangles may be larger. Integration must reserve the actual native and semantic bounds after safe-area placement, including visible quick-menu items, toast, pause/fullscreen/site/dock controls. Root has explicitly accepted actual `ui_scene` reservations. Reserving only the stand-alone fixture rectangles would not prove those dynamic paths.

## Hysteresis assessment

The controller retains a clear current anchor, immediately leaves an unsafe one, and switches between safe upper candidates after a continuously observed challenger has at least 24 pixels more head clearance for 350 ms, or on a real phase-epoch change. Clock rewind clears pending promotion. Equal preferences deterministically favor left. Exact threshold/interruption and one-pixel jitter cases passed.

Returning to a higher-priority anchor or wider panel intentionally bypasses the 24-pixel head-clearance requirement while retaining the 350 ms/phase gate. That preference exception should remain explicit in the integration contract; it is not a safety defect. A retained free-search rectangle stays fixed while clear, and a different free-search origin starts its own challenger timer.

`InvalidInput` intentionally preserves the entire controller state, including a pending challenger. Consequently a wall-clock hold can finish after an invalid/unobserved interval. This is compatible with fail-closed geometry but is not proof that the challenger stayed visible/clear throughout the gap. Engine agreed the caller must reset on encounter replacement, menus/dialogue, recovery and other presentation discontinuities. `phase_epoch` must represent actual encounter/phase changes, rather than changing every draw or accidentally reusing a previous encounter's identity.

## Remaining integration gates and adversarial cases

| Case | Expected behavior / acceptance | Current scope |
| --- | --- | --- |
| All primary anchors blocked or safe viewport too small | Compact/free search returns a clear ≥100×40 panel when one is available, without reusing an unsafe stale rectangle. A truly overconstrained supported scene still needs an essential-HUD/reflow policy; permanent disappearance is unacceptable. | Compact geometry passed; integrated readable fallback Pending |
| Core/head partly offscreen at maximum zoom | Any visible core disc remains protected; whole armor/tail protection is outside this contract. Observe all three families while moving them into view, rather than accepting a clipped still as full silhouette coverage. | Geometry covered; integrated presentation Pending |
| Actual largest owned-player radius at 533×300 and promoted f32 maximum | Use the actual occupied-cell radius, not the 18-unit fixture. Engine calculated Scout about 19.9003 and Sovereign about 19.11; the conservative 20-unit sweep and compact cases passed. | Native regression passed; actual adapter/device Pending |
| Small safe viewport plus radar/touch controls/expanded quick menu/toast | No intersection with actual control hit bounds, including partly clipped widgets; fallback remains readable. A low-width candidate may overlap several controls even if a wide viewport passes. | Static fixtures covered; actual dynamic reservations Pending |
| Reserved gap exactly 4 and core corner tangency | Contact remains excluded; greater clearance may be accepted. | Exact-boundary regressions passed |
| 24-pixel preference at 349/350 ms, interruption, phase change and rewind | Stable safe placement; no one-pixel chatter, stale promotion or delayed escape from a now-unsafe anchor. | Native regressions passed; integrated motion Pending |
| Modal/invalid interval followed by another encounter | No inherited stale challenger or old unsafe placement; explicitly reset the caller's controller state at the boundary. | Source contract reviewed; caller integration Pending |
| Asymmetric/fractional insets at eased 0.6/1.6 zoom with shake | Full projection and reservations use one coordinate space; panels snap inward and never scale with world zoom. | Native geometry covered; browser/device Pending |

The layout decision must be recalculated against current exclusions before drawing. On `NoClearAnchor` or `InvalidInput`, a caller must not blindly draw its last `Placement`. Compact fallback and semantic alignment require their own renderer/UI tests and current-build browser captures before the new HUD task is Complete.

A valid 320×180 landscape viewport made the original fallback limitation concrete. Its three primary rectangles are `(12,68,220,40)`, `(88,68,220,40)` and `(50,82,220,40)`. All contain the centered player at `(160,90)` even before adding its protected radius. The earlier compiled regression confirmed no primary anchor at the promoted f32 maximum with a conservative 20-unit player radius. The final compact regression now finds a clear smaller panel, including seven native-widget fixtures. This proves geometry rescue for the declared fixture, not all actual mobile controls or rendered text.

Requested extension checks are an independent brute-force reference for bounded small grids, exact disc/rectangle tangencies, fractional safe edges, fully blocked screens, and finite worst-case search work. A retained free-search rectangle must be validated afresh and stay fixed while safe, rather than following every moving core. Unsupported viewport extents must be distinguished from a genuine geometric no-fit result.

## Compact-extension findings and resolution

The frozen extension uses 168/140/100 widths at height 40, adds side/mid anchors, and searches integer origins with sorted forbidden x intervals when fixed anchors fail. Extents are bounded to 4096×2160 and at most 64 reserved rectangles. Larger extents return `InvalidInput`, rather than falsely claiming geometric no-fit. Direct brute-force comparison checks candidate rectangles by closest-point collision, independently of the search's interval formulas.

One concrete defect was sent to engine before freeze: at disc radius 32, vertical distance 25.6 and center x=44.8, the interval formula yields right edge `63.99999999999999`. `floor(right)+1` chooses x=64, whose direct Euclidean distance is exactly 32, so the original outer filter rejected the tangent and abandoned the width. Engine moved final collision validation inside the search so x=65 is considered. The named regression passed independently. The opposite rounding case—r24, dy0.024, cx40.000012000003, interval right64 while direct distance exceeds24 by an ULP—also has a passing boundary-probe regression. Neither original edge emitted an unsafe panel; both affected completeness of placement search.

A second finding concerned challenger identity: matching only anchor and width let a changing free-search origin inherit another origin's 350 ms timer. Engine now compares full placement for `Search`; the moving-origin regression passed independently. The retained exact rectangle prevents immediate per-frame tracking while clear.

For the **pre-cache** source, engine reported repeated-update native CPU median/p95 of 0.100/0.200 µs for the normal 533×300 fixture, 13.200/18.800 µs for a 320×180 compact fixture with seven widgets, and 9.211/9.440 ms for 64 covering stripes at 4096×2160. This historical bound motivated the cache. Engine's refreshed final `results/run.log` was read back at 18:54 UTC and distinguishes cold work from retained/cache reuse:

| Final native fixture | Samples | Median / p95 |
| --- | --- | --- |
| Normal 533×300 retained | 500 | 0.100 / 0.200 µs |
| Compact 320×180 + seven widgets retained | 500 | 0.200 / 0.200 µs |
| Same compact fixture fresh controller/search | 500 | 13.800 / 14.000 µs |
| Synthetic 4096×2160 + 64 covering stripes, exact cached no-fit | 500 | 0.300 / 0.300 µs |
| Same maximum-extent no-fit, fresh controller/search | 30 | 9.7672 / 12.0455 ms |
| Synthetic 960×540 + 64 covering stripes, fresh no-fit | 30 | 330.900 / 334.600 µs |

The covering stripes defeat the single-enclosing-widget shortcut. These are engine-measured desktop native CPU figures, not reviewer-measured browser, GPU or phone results. Changing geometry can legitimately invalidate a cached no-fit every frame; the fresh cost still matters. Integrated measurement is required, and finite loops alone do not establish a phone's 16.67 ms budget.

Production's current nearest-pixel resize scaling keeps ordinary logical height around 360 and below 540, so the 2160-row timing is deliberately beyond normal production viewport shape. A 4K output is scaled into fewer logical rows. It must not be presented as measured normal production cost. Widths above the prototype's 4096 logical-pixel limit remain an explicit unsupported-layout edge.

The final cache's source safety was reviewed: primary anchors are always freshly computed; successful fallback reuse requires a freshly clear retained rectangle and exact cached preferred rectangle, unchanged screen, no rewind/phase change, and age at most 350 ms. No-fit reuse requires exact screen/projection/core/player/reserved values and length. Unsafe rectangles force a fresh search on that frame. No source overlap defect was found in this policy. Dedicated regressions for every no-fit geometry class, newly blocked memo/current, age/phase/rewind and primary promotion passed independently in the 42-test run. A safe compact preference can take cache refresh plus challenger hold (roughly 700 ms) absent phase change; unsafe escape remains immediate. Cache safety acceptance does not imply browser/device performance acceptance.

## Initial production-adapter source review

Root subsequently authorized read-only inspection of `crates/game_web/src/boss_overlay.rs`, its browser hooks, `Game::render_focus` and compact native painting. The adapter projects through the actual f32 `CameraView`, subtracts full viewport/2, then passes those recentered points into the layout's camera-zero/zoom-one projection. Its core/player radii are already multiplied by the actual eased world zoom. This applies world zoom once while retaining screen-space floors and fixed panel pixels. Safe-local returned bounds subtract left/top once, and the existing UI camera/inset transform restores the full-screen origin. `render_focus` uses the same position and shortest-heading interpolation as rendering. Enabled shake is fed through the same presentation camera.

Root's source resets on encounter identity changes, absence of a boss, pause transitions, non-flight UI, resize, diagnostics, return home and graphics recovery. This closes the documented caller discontinuity requirement in ordinary menu/recovery paths. Touch reservations match the current final CSS safe-area-adjusted 84×84 joystick and 72×72 fire hit regions. Actual integration tests and captures remain root-owned and Pending at this source snapshot.

Three concrete adapter findings were sent to root:

1. `Reserved::scene` originally filters to `kind == "button"`, while the bridge emits visible toast and destination marker nodes as `text`. Those nodes draw after the native panel and can cover it. Reserve their actual clipped visible bounds or explicitly mark blockers; a captured quick-menu/button fixture alone is insufficient.
2. At 100 pixels the 14-glyph limit truncates actual 15-character phases `Siege broadside` and `Serpent pursuit`. Use meaningful compact labels and cover every authored phase, rather than testing only `Core exposed`.
3. The adapter test currently allows no panel for a valid 320×180 touch scene. A genuinely overconstrained layout requires a UI reflow/collapse policy or explicit supported minimum viewport; silent permanent loss of health/phase does not satisfy readable small-screen HUD acceptance.

Root subsequently corrected the first two findings in source: clipped `text` and `button` bounds are reserved, traversal/coordinates are bounded, and a toast/destination regression exists; meaningful `BROADSIDE`, `PURSUIT` and `COIL RUSH` compact labels plus all-authored-phase coverage were added. This correction was read back at approximately 18:47 UTC. Root is compiling/running the adapter tests; the provisional 320×180 allowance remains under integration review. The bounded performance policy is accepted at native source/test level, with actual integrated measurements pending. These are source-review findings, not invented observed browser failures. Root test evidence and representative browser/physical acceptance remain Pending.

At 18:54 UTC the production `crates/game_web/src/boss_layout.rs` copy was verified byte-identical to the final independently tested prototype (`97ae0496ec27b89510b46ad21785a377d2ca6ae0d88d1fe95dcbc30a1758e353`). Adapter/readback fingerprints at that snapshot were `boss_overlay.rs`=`efbb64d04df574fc25dbae436245a72eaf23a8ecdc3fa13389704306fedbd260`, `browser.rs`=`9187ef88d50a94a4380c445e7fc7f277cb723e909cb1477635a5adbf9420c3c6`, and `pixel_ui.rs`=`f3de71dff0624caf1e6b7922d55856f2f904ee7f1f8165bf6750b593b3bd400c`. These establish inspected source, not a new published build. Reservations use the same 128-group/512-node traversal limits and finite coordinate checks as `ui_scene::draw`.

## Reproduction

Engine's isolated source commands are:

```powershell
cargo test --manifest-path work/boss-hud-layout/Cargo.toml
cargo clippy --manifest-path work/boss-hud-layout/Cargo.toml --all-targets -- -D warnings
cargo run --release --manifest-path work/boss-hud-layout/Cargo.toml
```

They compile only the standalone prototype. The independent review used the already compiled test executable to avoid writing files owned by engine. Native geometric success does not alter published desktop/phone acceptance, the natural Scout playthrough, all-fifteen browser choreography or the existing physical-device worksheet's Pending statuses.
