# Boss overlay engine integration review

Read-only source review and bounded native proof, initially 2026-10-10 19:02 UTC; the root-owned focused repair was independently rechecked at 19:06 UTC. This reviewer changed no production files. Root owns the browser/WASM build.

## Findings

The one concrete small-safe-view parity defect found during review is now repaired and independently verified. Initially, `pixel_ui::draw_ui_with_boss_panel` returned before emitting native HUD below a safe width of 220, while `BossOverlay` could allocate a valid compact panel there.

The native reproduction uses full viewport 240 by 180, safe insets [0,20,0,20], zoom 1.6, touch controls, a 20-world-unit player at the camera and the boss core 70 world units north. The adapter returns safe-local [16,12,168,40]. Its initial renderer emitted zero native HUD pixels because the safe viewport width was 200. Root repaired both flight HUD and touch painters to accept safe extents down to 180 by 140, forced minimal telemetry below width 220, and made the allocator explicitly reject smaller safe extents. Menus retain their width-220 guard.

The exact repaired fixture now emits 1,011 native HUD quads and 266 touch-control quads. The controls are unlit and their actual quad bounds stay inside the corresponding DOM hit regions. A safe width of 160 returns no allocation and emits no native HUD, preserving the explicit lower-bound parity. The dedicated production test `narrow_safe_area_emits_the_allocated_compact_panel_and_touch_controls` also passes. Real phone minimum safe dimensions remain a browser/device acceptance requirement.

The independent extraction also measured up to 0.0625 logical pixels of allocated-edge deviation at a camera near (990000.25,-980000.5). Native UI transforms currently add screen offsets to the large f32 camera before renderer upload subtracts it. This consumes a small amount of logical guard precision. In the measured cases actual quad rectangles still cleared the physical player/core and native control footprints. It is a practical large-coordinate precision limit, not proof of exact integer alignment at every world coordinate. A future screen-relative UI transform path would eliminate that precision loss.

No unresolved blocking collision, reservation, or lifecycle defect was found in this bounded review. Conservative no-fit and large-world precision limits below remain explicit.

## Projection and extraction checks

`Game::render_focus(alpha)` interpolates player/boss position and shortest-path heading with the same helper used by extraction. Ordinary authored core position is transformed through that interpolated actor transform and its radius includes actor scale. Encounter identity includes the ECS entity generation. Phase epoch combines actual cycle and tell/attack/recovery phase. Arena opponents protect their rendered center with the minimum core radius.

`BossOverlay::update` projects those points with the same f32 `CameraView`, actual eased zoom and shaken render camera. It recenters the resulting full-screen points before feeding the standalone f64 geometry controller at zoom one, and supplies already-zoomed radii. Insets are never scaled with zoom. The final rectangle is converted to safe-local coordinates only once.

The browser native UI origin equals the shaken camera minus half the full viewport plus the safe left/top inset. World and UI extractions remain separate; the renderer uses world extent for world batches and the fixed logical extent for UI batches. Both geometry bind groups are rebuilt if their shared transform buffer grows. UI materials are unlit. The overlay draw follows world batches, and the semantic UI scene follows native extraction.

Panel width determines maximum glyph advance. The boss renderer clips generation/name and phase to the inset width, while health uses that same width minus 14. The 100-pixel case preserves complete `CORE EXPOSED` and the real health fill in native tests. Authored attack labels fit the compact width; warning charging shortens to `WARN / CHARGE`.

## Reservation parity

Source-derived native reservations match emission:

| Item | Full-screen reserved extent |
| --- | --- |
| Biome/status | safe origin + [12,12,160,28] |
| Currencies | safe origin + [safe width-214,12,142,24] |
| HOME compass | safe origin + [safe width-60,safe height-144,48,48], only beyond 240 world units |
| Hull/cargo/shield | safe origin + [floor(safe width/2-88),safe height-48,176,36], only when rendered |
| Joystick DOM hit region | safe origin + [14,safe height-92,84,84] |
| Fire DOM hit region | safe origin + [safe width-84,safe height-86,72,72] |

The native visual pads fit inside those larger DOM hit regions. Safe-area CSS offsets match the same left/bottom/right origins. Semantic button and text bounds are translated by their group position after intersection with their clip. This includes the actual quick-menu buttons, pause/dock/site actions, toast and destination marker. Hidden and clipped-away elements do not reserve space. Reservations use fixed-capacity storage and overflow fails closed.

The semantic bridge invalidates on relevant DOM changes and continues syncing through drawer/dialogue transitions. Destination movement explicitly invalidates the bridge every frame it is visible. Native reservations are regenerated on every overlay update.

The minimal rescue branch triggers only after valid no-fit. It removes biome/currency/radar reservations and native quads together, retains the hull and actual controls, then reevaluates layout. Minimal state is retained for that encounter to avoid moving-core telemetry flicker. Encounter completion/change or reset restores ordinary telemetry. The essential quick-menu bounds remain hard exclusions in both modes.

## Native commands and observations

Focused production tests:

```powershell
cargo test -p game_web boss_overlay -- --nocapture
cargo test -p game_web pixel_ui::tests -- --nocapture
```

Both exit with code zero: five overlay adapter tests and eleven native pixel UI tests. They include expanded quick-menu retention, encounter telemetry restoration, actual f32 projection, clipped semantic bounds, complete compact phase glyphs/health, invalid panel rejection and minimal hull retention. These were focused checks rather than another broad workspace rerun.

The independent proof lives in `work/boss-overlay-proof/main.rs`. It links the actual compiled native libraries directly:

```powershell
rustc --edition=2024 work/boss-overlay-proof/main.rs -L dependency=target/debug/deps --extern game_web=target/debug/deps/libgame_web.rlib --extern engine_core=target/debug/deps/libengine_core-c6e44304da45d14c.rlib --extern game_core=target/debug/deps/libgame_core-684f3d0a8e530f28.rlib -o work/boss-overlay-proof/quad-proof.exe
./work/boss-overlay-proof/quad-proof.exe
```

Compilation and execution exit with code zero. The proof subtracts a no-boss baseline to isolate actual emitted boss quads, reconstructs all four quad corners after the renderer's camera-relative transformation, checks unlit material fields, allocated bounds and actual physical target/control exclusion. It uses the actual core radius from the diagnostic Bastion generation-five content and a conservative 20-world-unit player. The core location is deliberately moved in the presentation focus to challenge allocation; this is a controlled projection fixture, not a gameplay encounter proof.

Across full logical sizes 320 by 180, 400 by 240, 533 by 300 and 720 by 360; zoom 0.6/1.6; zero or fractional asymmetric insets; and near/far cameras, the proof emitted 30 placements and checked 13,030 actual boss quads. Two inputs returned conservative no-fit after minimal mode: 320 by 180, zoom 0.6 and insets [11.2,20.8,9.3,35.4], at both tested camera positions. Those remain explicit overconstrained fixtures rather than a claim that every arbitrary safe-area combination can fit essential HUD, controls and protected targets.

`work/boss-overlay-proof/results.log` and `hashes.json` preserve the initial reproduction. `final-results.log` and `final-hashes.json` record the repaired exact-fixture run and final source/library fingerprints. The independent proof additionally asserts that its repaired compact allocation actually emits HUD, that the 266 touch quads lie within the two DOM hit rectangles, and that the explicitly unsupported safe160 view does not claim a placement.

Final focused repair command:

```powershell
cargo test -p game_web narrow_safe_area_emits_the_allocated_compact_panel_and_touch_controls -- --nocapture
```

It exits zero with one targeted test passed; unrelated tests are filtered. The 32-case production quad proof still places 30 cases and checks 13,030 boss quads after the repair. The two conservative no-fit cases and 0.0625-pixel maximum far-camera edge deviation are unchanged.

Final SHA256 fingerprints:

| File | SHA256 |
| --- | --- |
| crates/game_web/src/boss_overlay.rs | FD9FF6B24CF69D3C43144571815D071C251485B002CBA8BCAD27BCED8B02810F |
| crates/game_web/src/pixel_ui.rs | C66A24BF4C105DF8FC5121C8C4AB8A797A19BD3EB51D5DFC067F2DF68A7DD94A |
| crates/game_web/src/browser.rs | 2BE9F952D91A3994688076F0C0A57FC468C254BD05073A6F14469156045143C5 |
| target/debug/deps/libgame_web.rlib | 3C855AFA333442366B826940E5315F8575C2AAC4E0B6D50E7D233DE35FAF361F |
| work/boss-overlay-proof/main.rs | 2B50A5E5487CAE8E2762C46CF534A63600A325D795D35611A0A6DA0D37D0DEB1 |
| work/boss-overlay-proof/quad-proof.exe | D89EA72F631D1E7DC0D2B832588A64337698CC4780E110BCEBC49464AF524A41 |

## Lifecycle source checks

Overlay state resets on changed encounter or absent boss, pause-state transition, non-flight UI mode, resize, graphics recovery, diagnostic scene change and return home. Modal console/dialogue/death paths set mode zero and pause; regular menus use mode three. Closing modals restores the appropriate UI mode and resets simulation input. Visibility loss pauses an active expedition, resets input and suspends audio; frame drawing stops while hidden. Resuming through the menu resets the overlay through `set_paused(false)`.

Inset/touch/semantic changes are validated on the next frame; changed geometry invalidates exact no-fit caching and unsafe rectangles immediately refresh. Reduced motion gates shake and snaps zoom to its target; the same camera/zoom still feeds both projection and drawing. The overlay has no animated panel transition.

The last visible boss bounds remain available for paused-console inspection; their persistence while hidden is documented in the browser API and does not itself draw a stale panel.

## Acceptance boundary

This review exercises portable production geometry and actual native quad extraction. It does not execute browser lifecycle events, GPU uploads, WebGPU/WebGL2 compositing, touch input or physical phones. Root must inspect actual compact health/phase text, widget exclusion and modal/resize/recovery transitions in the final browser build. No shipping or mobile-performance acceptance is claimed.

