# Next boss readability change

Design review only. Production source is frozen; this note changes no game, renderer or asset code and supplies no new visual proof. Evidence is the actual captures recorded in `work/boss-browser-art-review.md` and the current source described below.

## What the evidence establishes

The current boss HUD in `crates/game_web/src/pixel_ui.rs` is a fixed 220×40 logical-pixel panel centered horizontally at y=44..84. Name, phase and health are already readable, and the renderer's separate UI projection keeps those cells the same screen size at zoom 0.6..1.6. The observed problem is placement: centered northern boss heads pass behind that panel.

`boss_diagnostic` starts a head 210 world units north of the player. With a centered camera, its head-center screen y is `H/2 - 210*zoom`: 54 at H=360/zoom=0.6, directly inside the HUD; -156 at zoom=1.6, beyond the top edge. These are source-derived projection examples, not measurements of every animated screenshot. `Game` follows the player rather than framing both combatants. Changing the panel alone can remove HUD occlusion, but cannot bring an offscreen core into view.

The latest minimum-zoom Dart/Coil images show HUD-covered head portions. Maximum-zoom Bastion images show natural viewport cropping plus HUD-covered armor. Neither proves full reactor visibility. Distinct metal contours, open bays, player hull and HUD text survive both WebGPU and WebGL2 in the captured portions.

## Recommendation: one panel, three reserved anchors

Keep the existing 220×40 panel, fixed 5×7 font, health meter and text hierarchy. Move that same panel between three discrete anchors in the safe-area-adjusted logical UI viewport:

| Anchor | Top-left logical position | Purpose |
| --- | --- | --- |
| Upper left | `(12, 68)` | Below biome and pause/header controls |
| Upper right | `(W-232, 68)` | Below currency and pause/header controls |
| Lower center | `(floor(W/2-110), H-98)` | Above the hull/cargo meter; between touch pads |

Choose the candidate that clears the projected vulnerable core and player, then prefer the one farther from the head center. Do not add another bar, floating full-size label or enlarged world-space text. The lower candidate is a fallback, not a permanent second HUD row.

Use one pure `boss_panel_layout` helper plus a small presentation-only anchor state in the browser. Inputs are the actual world projection, projected core/player points, safe UI extent and reserved widget rectangles. Existing `BossHud.position` and `core_position` supply the target data; `CameraView::world_to_screen` supplies the same projection math as the renderer. Subtract the safe-area left/top offsets after projecting into the full logical viewport. Use the actual eased zoom and shaken render camera. Do not use CSS backing pixels or target zoom.

For the current fifteen ordinary bosses, protect a 32-logical-pixel radius around the core and 24 around the player. The largest authored core radius is 8.5 world cells; Bastion G5 scale 1.34 at zoom 1.6 gives an 18.224-pixel core radius, leaving almost 14 pixels for clearance and presentation interpolation. Derive or expose the world core radius if later content exceeds this budget; do not silently assume all future reactors fit it. Head overlap is a soft score because a giant head can occupy a substantial viewport, whereas covering the attackable core is a hard failure.

Retain the current anchor while it stays safe. Relocate immediately if it covers a protected point. Return to a preferred anchor only at a phase transition or after a sustained safe interval of approximately 350 ms; never delay removal of a panel covering the core. Place the panel directly at its integer anchor without sliding animation. Reduced motion therefore needs no special moving-HUD path. Validate actual pause/quick-menu, Home compass, meter and touch-control bounds as exclusions; suppress flight HUD while a menu/dialogue is open, as today.

At the observed minimum logical extent 533×300, the upper panels have a 69-pixel horizontal gap and the lower panel starts 94 pixels below their bottom. A 32-radius core disc cannot overlap two candidates across those gaps. This is useful geometric reasoning for the proposed layout, not a passed implementation test. Smaller extents need a separately checked compact fallback; this review does not claim the three-anchor guarantee below 533×300.

Do not automatically change the player's zoom or camera target as part of this correction. A core outside the viewport remains outside it. Use an unobstructed later encounter frame or a wider world extent for full-art review. An offscreen-target indicator or encounter camera would be a separate gameplay/UI decision.

## Acceptance for a future implementation

- Native layout tests at UI extents 533×300, 640×360, 720×360 and 844×390, with zooms 0.6, 1.0 and 1.6. Sweep core/player points through each anchor boundary and test safe-area offsets, invalid coordinates and reserved widget bounds. Wherever a clear candidate exists, the emitted panel must not intersect the core/player protection discs.
- Verify exact world-to-UI projection with the same eased zoom/render camera used for world drawing. If tick-time `BossHud` positions cause visible disagreement with interpolated bodies, provide the small render-time core/center descriptor from `Game` rather than masking it with arbitrary larger margins.
- Replay northern approach, lateral attacks, close pursuit and recovery for all three G5 families. The panel stays at one anchor while safe, immediately clears a threatened core, and does not oscillate at a stationary boundary. It remains exactly 220×40 with font scale 1 at every zoom.
- Capture one complete, unobscured recovery core per family on visible WebGPU and WebGL2 tabs. Record actual viewport/zoom/phase scalars beside the images. At maximum zoom, accept natural world-edge cropping, but do not accept a visible recovery core hidden by the HUD. A cropped or charging-core frame cannot count as exposed-core proof.
- Capture short landscape with touch controls, safe-area insets and low hull. Boss panel, cargo meter, joystick, fire pad, pause and Home compass must remain readable and disjoint. Physical Pixel 6a/iPhone 12 performance and touch acceptance remain a device test, not an inference from desktop screenshots.

This proposal addresses the observed occlusion with a small layout helper and one retained anchor. It does not redesign combat, alter collision/destruction, add a second camera controller, or claim validation that has not been performed.
