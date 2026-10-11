# PAN co-op dock: detailed tiled candidate

This preserves the finer panels, vents, pad edges and crystal facets from the
generated reference as a **review candidate**. The shipping Haven is unchanged.
The adjacent frozen native128 candidate is a compatibility/budget proof; its
coarser image is not the recommended final detail target.

The source is the exact 1254×1254 RGBA reference retained at `../source.png`,
SHA256 `eab5fa7c3b604451fdb5fadff75c2138d614aa8d8bbcedd31402527c59380918`.
It was generated through the built-in image_gen tool; no model/version was
exposed. APR portrait imagery supplied a style reference, not geometry.
The original retained provenance calls its prompt `prompt.txt`; the byte-exact
copy beside this candidate is named `../generation-prompt.txt`. This is a
filename mapping, not a new prompt or generation. `../generation-provenance.json`
retains the original metadata unchanged.

## Conversion and native contract

Alpha is explicitly thresholded at 128, then the entire canvas is center-sampled
with nearest sampling to 256×256. No crop, component stripping, hole filling or
silent resize occurs. Colors are explicitly nearest-matched within the frozen
64-color native128 palette's metal, green and cyan strata. The import then uses
the existing asset-pack converter with that exact provided palette. This is a
lossy raster conversion, followed by exact indexed-cell import.

Four quadrants retain every sampled cell. Each quadrant has one 128×128
PixelBody and four disjoint PixelRig roles: graphite deck (armor), pale alloy
(trim), cyan screens (trim), green signals (emitter). Their reference centers
are `(-64,-64)`, `(64,-64)`, `(-64,64)`, `(64,64)` in 256-source coordinates.
A shared parent scale of 0.5 produces the same 128-unit reference footprint.
Compose complete affine matrices, including the parent, rather than rebuilding
TRS from them. Every local part remains within the existing 128-cell PixelRig
dimension limit. The four rigs submit **28,549 cells / 16 transforms** once:

| Material | Cells |
| --- | ---: |
| Deck | 18,844 |
| Alloy | 7,075 |
| Screens | 705 |
| Signals | 1,925 |

The proof host admits the whole group before appending, with local ceilings of
32,000 cells / 16 transforms and finite corners within ±1,000 reference units.
This extent gate is an offline near-origin proof policy, not a world-coordinate
runtime adapter or a new FrameBuilder/global limit. Damage removes real current
PixelBody cells; the rig does not restore template cells. A future runtime
adapter must cache the bodies and rigs: the proof utility rebuilds them for
offline extraction. Its `pixel_rig_tiles` extension is explicit authoring data,
not metadata automatically consumed by the live Game.

Three authored lamp states retain exactly the same occupancy, transforms and
material groups. Of 1,528 exterior lamp-mask cells, the dim and bright states
change 1,391 and 1,188 colors respectively; the central core and all other
colors stay fixed. The manifest describes 700 ms per state. The proof host
explicitly selects the state and freezes state 0 / rig time 0 for Reduced
Motion. PixelRig time is fixed at 0, charge 0.2 and hit flash 0 for these captures;
its real emitter tint/material is recorded. No animated frames were generated
by a model and no live-game animation is implied.

## Measured loss and review evidence

`conversion-losses.json` records the source/decoded sampling measurements:
717,441 source alpha values changed by hard-alpha conversion; 26,943 occupied
color cells changed by palette matching; RGB channel MAE 3.184735, RMSE 6.181149,
maximum channel error 138. Reprojected source-mask IoU is 0.990430, with 3,248
false opaque and 3,334 false transparent source pixels. The sampled dock is
one connected component, with no enclosed holes; exterior arm/pad cutouts are
retained. The separate source128/192/256 comparison is `../grid-comparison.png`.

`authoring-states.png` shows the exact indexed source, while `native-preview.png`
comes from the existing actual PixelBody/FrameBuilder utility. `zoom-preview.png`
uses actual PixelRig packets and actual CameraView corner projection at zoom
0.6 / 1.0 / 1.6 with logical viewport 320×240. Its renderer auto-fits the packet
bounds and draws flat color; these are enlarged software reviews, not literal
viewport screenshots, GPU lighting/aliasing results, or an animation capture.
All three zoom panels show the same static state 0 (the utility's `/700ms`
caption is inherited duration metadata). Small-cell GPU visibility and distant
world precision remain unmeasured by these near-origin views.

Exact native exports are stored as lossless `*-extraction.json.gz` archives.
`archive.json` pins both compressed bytes and full decoded SHA256/length. Gzip
headers have an empty filename and timestamp 0; compressed encodings are pinned
evidence, not promised byte-equal across different zlib implementations.
The archival reader bounds each decoded export to 32 MiB. `native-audit.json`
references the original decoded export's hash; no proof rows were removed.

## Reproduction

Use the repository's pinned Python/Pillow requirements. From the repository root:

```text
python scripts/qa/pan-coop-dock/test_256.py
python scripts/qa/pan-coop-dock/verify_256.py
cargo test --locked --manifest-path scripts/qa/pan-coop-dock/native256/Cargo.toml
cargo clippy --locked --manifest-path scripts/qa/pan-coop-dock/native256/Cargo.toml --all-targets -- -D warnings
```

Authoring is separate from verification:

```text
python scripts/qa/pan-coop-dock/convert_256.py --output work/pan-coop-dock256/regenerated
cargo run --locked --manifest-path scripts/asset-native-preview/Cargo.toml -- assets/pipeline/pan-coop-dock/native256/pack.json pan256-lamps work/pan-coop-dock256/native-extraction.json
cargo run --locked --manifest-path scripts/qa/pan-coop-dock/native256/Cargo.toml -- work/pan-coop-dock256
```

The candidate tests check exact committed authoring bytes, source-mask retention,
palette and material partition, real landmarks/cutouts, same-occupancy lamp states,
native PNG round trips, and budget rejection. Native tests cover all-cell emission,
nested reflected/sheared corners, continuous zoom, real damage holes, atomic
short-budget rejection and crossing the 128-transform batch boundary.
Shipping integration, collision/interaction scale, both GPU backends, moving
human readability, co-op operation and physical-phone cost are pending.
