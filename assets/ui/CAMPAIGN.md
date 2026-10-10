# Authored campaign cells

`campaign-atlas.json` contains original pixel art baked from the silhouettes,
emblems and character masks in `campaign-source.py`. Regenerate it with
`python assets/ui/campaign-source.py`. The source uses only Python's standard
library. `campaign-preview.png` is a nearest-neighbor contact sheet of the
actual atlas cells; it is not a texture used by the game.

The atlas uses the existing schema: version 1, 64 packed little-endian RGBA
colors, and named sprites with width, height and row-major palette indices.
Index 0 is transparent. The eight champion hulls and Royale reward hull are
shared by the game simulation and UI, so their occupied pixels and palette
are the same assets rather than separate menu representations.

Hull surfaces use authored armor facets, beveled panel seams, vent banks and
small hardware contours. The Voidcoil ring has segmented reactor arcs; the
Royale/Sovereign reward has warm gold plating and prismatic inlays. These
details recolor existing occupied cells, preserving every collision outline.
The atlas currently uses 38 opaque colors within its 64-slot palette.

| Index | Champion / craft | Hull and patch identity |
| --- | --- | --- |
| 0 | Lyra Voss / Dawnneedle | Amber needle, star/arrow patch |
| 1 | Korr-9 / Bulwark | Red shoulder cruiser, shield patch |
| 2 | Sable Myrr / Nightrail | Violet forked serpent, coil patch |
| 3 | Xil of Aurora / Prism | Cyan split crystal wings, diamond patch |
| 4 | Maera Quill / Windrunner | Emerald swept wings, feather patch |
| 5 | Orun Vale / Ironwake | Orange ram prow, anvil patch |
| 6 | Vexa Null / Voidcoil | Magenta ring with fins, eye/ring patch |
| 7 | Atlas-0 / Sunspear | Pale gold spear, crowned lance patch |
| 8 | Royale reward | Gold crown prow with prismatic inlays |
| 9 | Haven Scout icon | Compact cyan scout |

`campaign_ship_0` through `campaign_ship_9` are 32×32 and nose-up. Index 9 is
the selection icon; the existing generated Haven player body remains separate.
`campaign_patch_0` through `campaign_patch_7` are 24×24, with transparent gaps
and distinct central emblems. New `campaign_portrait_3` through
`campaign_portrait_7` are 24×32. The first three dialogue portraits retain their
existing authored bodies and palettes in the Rust UI module.

UI nodes use `kind: "campaign-ship"`, `"campaign-patch"`, or `"portrait"` and
numeric `value` for the index. Values are floored and clamped; nonfinite values
select index 0. Nodes enlarge with an integer cell scale and preserve aspect;
provide at least the native dimensions. UI materials are unlit. Selection,
disabled colors and group-relative clipping do not change their occupancy.
