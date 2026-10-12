# Native dock art integration contract

This accompanies the immutable candidate from `92fb9f677a94c6740b6a14380a7dd403219795b2`.
The candidate README and archived packets describe its earlier offline proof.
The source review below covers the separate runtime adapter at
`7d3673e3a2f051aeefac6540e443ef7fc5d0ad9c`; it does not record a new GPU,
human readability, co-op or physical-device acceptance.

The retained 1254×1254 source is one strictly overhead dock. Its flat central
deck, four arms/pads, alloy inset panels, vents and crystal clusters remain
recognizable in the actual 256-cell authoring and native software previews.
Exterior cutouts are retained. No source-art or tile-seam defect was found in
this bounded review, so the source raster, palette, indexed cells and previous
conversion-loss measurements have not been regenerated.

## Corporate material roles

| Role | Occupied cells | Runtime color policy |
| --- | ---: | --- |
| Graphite deck / armor | 18,844 | Preserve authored palette |
| Pale alloy / trim | 7,075 | Preserve authored palette |
| Cyan screens / trim | 705 | Preserve authored palette |
| Green signals / emitter | 1,925 | PAN original; RENKO red or HUGO blue |

The main manufacturer determines signal hue, independently of a pilot's
secondary crystal manufacturer or tournament sponsor. It must not recolor the
neutral alloy or cyan instruments. Palette preservation here means the colors
submitted to PixelRig; existing role-based tint, emission and world lighting
remain active and are not evidence of a changed source palette.

All 1,925 signals may receive the main corporate hue, including the central
core. Only the explicit 1,528 **exterior** lamp mask changes authored pose.
The central core remains the selected corporation's state-0 color. Three
same-occupancy poses cycle every 0.7 seconds; Reduced Motion selects pose 0
and time 0. With the adapter's fixed charge 0.2, the current PixelRig emitter
tint is time-independent: its pulse path affects Core parts, and this dock
has no Core part. Adding a time-dependent core effect later would be a new
presentation change, not something proved by the earlier lamp captures.

## Geometry and bounded admission

Four cached 128×128 tile bodies each retain four disjoint material roles.
Tile centers are `(-64,-64)`, `(64,-64)`, `(-64,64)`, `(64,64)` in source
coordinates. Compose the host affine, source scale 0.5 and tile translation;
do not multiply the tile origin by a second source scale. Every admitted
frame submits 28,549 cells once using 16 transforms. No coarse bitmap or
minimum source-cell size is substituted at smaller fits.

The runtime adapter uses a **local** 32,000-cell / 16-transform preflight,
then appends the complete group. Rejection must append nothing and leave its
quota unchanged, permitting the existing home body fallback. This is not a
new global FrameBuilder limit. Its cached bodies and rigs avoid the offline
proof utility's per-run construction.

The existing station host scale 1.5 gives occupied bounds approximately
±94.5 world units. This cosmetic body does not redefine the 240-unit Haven
safety radius, the four facility centers, their 24-unit hit radius, docking
or save semantics. Finite transformed corner validation is separate from
the earlier proof host's ±1,000 near-origin policy. Runtime source tests
include far-coordinate projection, but small-cell GPU visibility and visual
precision require actual backend observation.

## Provenance and outstanding review

`../generation-provenance.json` is the original generation record. Its
`promptFile: "prompt.txt"` maps to the retained byte-exact
`../generation-prompt.txt`; this filename mapping is not a new generation.
The source remains SHA256
`eab5fa7c3b604451fdb5fadff75c2138d614aa8d8bbcedd31402527c59380918`.
The generated APR portrait was a style reference only. Neither generated
animation nor perfect raster-to-cell conversion is claimed.

Source-side peer exchange established corporate signal isolation, fixed
central-core pose, exact fractional tile composition and unchanged gameplay
footprints. This note records source inspection and viewing existing source,
authoring and native software images. Actual browser appearance, moving
interaction clarity, phone cost and playable co-op remain separate gates.
