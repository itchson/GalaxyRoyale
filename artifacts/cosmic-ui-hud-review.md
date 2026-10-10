# Shared narrow flight HUD geometry

Production ownership was transferred after Root cancelled the stalled isolated local worker; that worker left no edits/test results. This handoff changes only `pixel_ui.rs` and `boss_overlay.rs`, including their focused native tests. No dist, browser, CSS, game simulation or persistence edits were made by this specialist.

`draw_ui_with_boss_panel_and_touch(frame,hud,camera,viewport,mode,boss_panel,minimal,touch)` is the new Browser integration API. The original `draw_ui_with_boss_panel` remains a wrapper with `touch=false`. Root must pass its actual touch state and already-inset logical viewport to the new function.

One stack-only `FlightLayout` provides biome/currency/radar/meter rectangles and meter style. Both painter and boss exclusion adapter use these exact rectangles; the adapter adds safe-area offsets once. Fixed DOM touch hit rectangles are shared through `touch_hit_rects`, while the existing drawn radius28 controls retain their centers. This avoids a second copy of responsive coordinates drifting from real painting.

- Normal533×300 geometry retains biome[12,12,160,28], currency[319,12,142,24], hull[178,252,176,36] and away-from-dock radar[473,156,48,48].
- Width300–391 has compact100×20 biome and110×20 currency panels separated by6px. Smaller extents suppress currency/radar before essential hull. Radar is also omitted below240 logical height.
- Touch320×180 uses a130×36 meter at[102,132], inside the real gap between84/72 hitpads. The centered meter is clamped to that asymmetric gap for transitional widths366..391 too.
- Touch width<290 moves a16px two-row H/S meter above the hitpads;180×140 gives[12,28,156,16]. Optional level/cargo is omitted in this smallest style. Core hull/shield bars/percentages remain.
- Cached RoyaleMono glyph objects use bounded advances for headers/counts/status. The existing normal caption geometry and font caches remain. All six semantic action kinds and three category text kinds now participate in clipped scene reservations.

`cargo test -p game_web pixel_ui -- --nocapture` passed14 tests. `cargo test -p game_web boss_overlay -- --nocapture` passed9 tests. `cargo clippy -p game_web --all-targets -- -D warnings` passed. The new source contains six focused additional tests: three HUD tests and three overlay tests, with retained behavior checks updated to use actual new touch-aware painting rather than assuming optional telemetry must always be hidden at320.

Actual nontransparent cell corners are checked against safe viewport bounds and the shared native rectangles for384 declared width/height/touch/minimal combinations (including fractional widths and ±999k camera precision). Drawn touch corners are checked against their real hit regions. Three asymmetric-inset fixtures compare actual emitted HUD/control/boss corners with full-screen reservations and verify boss/widget clearance. Existing camera/core/player zoom cases remain passing. These are native geometry proofs, not actual browser hit testing or device acceptance.

Two physical boundaries are explicit. Fixed DOM hitpads overlap2px horizontally at safe width180: joystick ends98, fire begins96. Rust HUD changes do not remove that HTML input overlap. Also a180×140 touch view with the essential meter/controls and actual max-zoom player/core exclusion can have no safe100×40 boss panel within the allocator's12px margin. The new regression proves fail-closed behavior in that genuinely overconstrained case; it does not paint over controls or claim a guaranteed panel. Root may later restrict supported touch safe widths or design a separate smaller boss summary, with its own allocator/input acceptance.

Production source fingerprints at handoff:

| File | SHA256 |
|---|---|
| pixel_ui.rs | 8b197c50b6f7556b91aa9f19af1807ddf0ba0177a92bc1145190409803a4e29b |
| boss_overlay.rs | fd672d8d65ad3c8823fedbe1dd586b53abc9f8e6d81f6949e9a9e7f36ca14b2c |

Root owns Browser integration, full checks, both-WASM Clippy/build, actual responsive GPU/GL captures and phone testing. This handoff is source frozen absent a concrete integration defect.
