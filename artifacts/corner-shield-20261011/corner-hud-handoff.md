# Compact corner HUD handoff

Source: `e4fe3f67e665707f190273821273abfc12c040ba`, branch `codex/compact-corner-hud`, parent `f6c7140348f752d53c3e30f96cf09376c256c455`. Five scoped source files; no assets/core/renderer/bridge/CSS changes. Root already integrated this commit separately.

The normal allocation is safe-area-local minimap `[W-50,6,44,44]`, status `[W-186,54,180,22]`; both share right edge `W-6`. Text slots remain `[x+4,y+2,w-8,5]` and `[x+4,y+12,w-8,5]`. Full-screen exports retain the existing ABI and add safe left/top exactly once. No bottom duplicate or visible status frame is painted. Existing yellow home, purple tracked mission, circular distance clamp, heading and unshaken/zoom-independent map semantics are unchanged.

The actual measured toolbar union decides clearance. Text widths try 180/140/100 at constant 3×5 RoyaleMicro cells, and the text can move just beneath the toolbar while the map retains its top anchor. A measured right-side obstruction can move the entire group down. Player/core/control/status-hit conflicts omit optional navigation. Compact touch boss mode preserves its real rescue space with essential H/S text immediately above actual touch pads; it never suppresses boss health to preserve optional map art. Curved player-relative vitals and their44×12 envelope are unchanged.

`pixel_ui::micro_text_right` is the same allocation-free, caller-budgeted whole-cell-clipped extractor, right-aligned within the supplied slot. It preserves leading characters on overflow; font/cell scale is never enlarged. Root's ui_scene status-text/warning-text adapter must call it (Root confirmed integration). Semantic full 44 physical-minimum contact hits remain distinct from the5 logical glyph row. Shell must gate unsafe hits against actual map/header/touch BCRs; root confirmed this fix separately. These tests do not measure CSS hitboxes.

A real existing defect was found: Reserved::scene omitted fullscreen-icon although flight_header included it. The strengthened actual-node test failed before the one-entry allowlist repair (`fullscreen-reservation-negative.log`). The final compact 320×180 touch fixture places its real 100×40 boss at [103,127] without crossing any of the five measured icons, pads, player/core exclusions or native status. At200px safe touch width, retained actual tests likewise preserve a compact boss. Genuinely overconstrained 180px touch fixtures remain fail-closed.

Validation actually executed in this isolated checkout:
- `cargo test -p game_web --locked -- --nocapture`: 184 native web tests + 31 existing layout integration tests, 215 total pass,0 fail.
- `cargo clippy -p game_web --locked --all-targets -- -D warnings`:pass.
- `git diff --check`:pass.
- Six new tests cover measured top-left toolbar/fullscreen, declared 320×180/848×390/1280×720 ratios and insets at zoom 0.6/1/1.6/reduced state; actual nontransparent four-corner containment; right-aligned nested-affine clipping and both budgets; real compact boss rescue/reservation.

Prior failure logs are retained separately, not counted as final passes. Browser/GPU/WebGL2/physical-device acceptance, latest Root CSS and new 64-cell icon module are outside this native source freeze. No normal build, preview server, browser, merge or push was run. The cosmic refinement is parked READY with no source edits and 13 baseline tests, under a separate proposal directory. Shield-capacity visuals are the next separately scoped task, not part of this commit.
