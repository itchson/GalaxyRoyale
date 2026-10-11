# Galaxy Royale visual system

This guide records the current source and the shared HUD contract. It is not a
visual acceptance report. The previous `galaxy-810fccad52eabc00` browser captures
remain historical evidence; the subsequent HUD and audio integration needs its
own normal-build checks.

## Color and hierarchy

Space stays near black. Navy and muted blue-gray form the background and quiet
surfaces; cyan guides navigation. Player hulls use pale metal and cyan, Dart
threats magenta, Bastion armor amber and Coil bodies jade. Bright projectiles and
authored attack cues must remain distinguishable from cosmetic stars, gas and
planet details. Do not borrow a hostile projectile color for a large decorative
foreground surface.

The native action palette in `crates/game_web/src/ui_scene.rs` is explicit:

| Meaning                    | Native RGB |
| -------------------------- | ---------- |
| Continue / primary action  | `#6AEDC1`  |
| Offense equipment          | `#FF9377`  |
| Defense equipment          | `#5CBFFF`  |
| Passive equipment          | `#9FE18A`  |
| Restart / caution          | `#FFD783`  |
| Exit / title               | `#DC7D82`  |
| General interactive accent | `#58DEEF`  |
| Disabled readable text     | `#8B9BA6`  |

Color supplements an accessible name and a distinct silhouette. Selection is
independent from hover, and disabled state must not erase a control's identity.
The CSS is an interactive semantic fallback; Rust owns the normal visible cell
art, including the measured button states and original icon masks.

## Pixel surfaces and type

Runtime artwork is occupied `PixelBody` cells and object transforms, not the
preview PNG. Indexed atlases reserve palette index zero for transparent cells.
The conversion pipeline uses hard alpha, nearest sampling and no dithering.
Damage removes real occupied cells; surface shading must not refill collision
holes. Additional panels, seams and vents should recolor existing occupancy
unless a silhouette change is explicitly intended.

`RoyaleMono` is the shared original 5×7 mask font with a six-cell advance.
`scripts/build-ui-font.py` produces the semantic font from those masks. Native
glyph scales are whole logical cell steps; do not shrink a label fractionally
to disguise a layout that cannot fit it. Use shorter labels or scroll content.
The wrapper retains at most 256 source characters and 257 bounded line spans.
Different backing raster sizes and flight zoom must not scale menu type or hit
regions. The generated title logo is separate artwork; Tiny5 and its supplied
OFL remain local assets.

## Flight and menu layout

Flight retains the compact status, map, settings and save toolbar. Detailed
ship statistics, recipes, inventory, journal and map information belong to
dedicated accessible menus with a fixed heading, a clear Close control, and
bounded inner scrolling. Do not add repeated navigation frames around every
value. Keep the existing category names and authoritative owned/effective ranks
in crafting views. Carried material is at risk; banked stash and per-ship XP
must not be presented as interchangeable values.

The shared `FlightStatusLayout` supplies one flat, 22-logical-pixel bottom strip
with two compact text rows. Touch uses the gap between the real movement/fire
hit regions when it has enough room, otherwise a clear area above them. The
shell consumes exported strip and context bounds; native painting and boss
reservations use the same geometry. Contextual contact text and a transient
toast share one row, with the toast taking visual precedence. Their text has no
independent frame. Real pointer targets keep at least 44 physical pixels where
the viewport can safely accommodate them; a missing context slot cannot become
an invisible clickable control.

Hull and shield tracks follow the interpolated ship through the unshaken
camera projection. Their logical dimensions stay constant while the world
zooms. If an edge or control makes a track unsafe, omit it and preserve the
compact H/S text fallback; do not pin a false ship marker to the viewport edge.
Warnings keep a readable accent under Reduced Motion rather than relying on a
flash being visible at the instant of inspection.

## Detail, motion and provenance

Parallax, background density and motif admission share the actual frame budget.
Isotope Scar has original authored cells and all-or-nothing group admission;
its software preview is not GPU evidence. Reduced cosmetic density may select
a simpler presentation. Reduced Motion freezes cosmetic drift and shimmer
while retaining real movement, combat timing and warning phases.

Generated sprite/UI sources and deterministic conversion details are recorded
in `artifacts/asset-provenance.json` and `assets/ui/provenance.json`. Campaign
hulls, portraits and emblem cells have original source scripts; see
`assets/ui/CAMPAIGN.md`. Ordinary boss art and exact occupancy are documented in
`docs/ART_DIRECTION.md`. These origins do not imply an unreported model license
or a guarantee about every generated source.

## Acceptance still required

The new strip, projected tracks and reactive audio need actual normal-package
browser captures and listening checks after integration. Physical Pixel 6a and
iPhone 12 safe areas, simultaneous touch, interrupted/background input, sustained
frame time and memory, and PWA lifecycle remain unmeasured. Full natural Scout
progression, moving/crowded threat readability, sustained combat, and human
balance/listening review are separate gates. CPU proofs, posed diagnostic
scenes and desktop snapshots cannot close those gates.
