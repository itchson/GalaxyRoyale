# Independent RAICHO fine source review

Verdict: no blocker in the frozen source-art pack. Importing this pack does not
constitute runtime or collision acceptance. Prefer the retained damaged 48-cell
body plus its sparse fine overlay for the first integration.

Reviewed exact commit `7884a7cff0476bacb354770700ef03228f146425`, parent
`f69d6cbe8d58d182da6e45460b42b963655c5fbc`. Atlas SHA256:
`319aa1fd8cf1112ef33addd06739063c72b92ecca5ca460afcc05aada77666be`.
The isolated snapshot contains 22 exact Git blobs: all 15 committed new files,
three existing tool/dependency inputs, and four existing engine/core files.
No branch change, source edit, build, native helper, service, browser or publish
was performed. All review outputs are under this work directory.

## Occupancy, damage and transform contract

The source tests compare every reconstructed 48-cell position with a scalar
f32 transcription of the pinned `identity.rs` fields. Static source inspection
agrees with those fields (`identity.rs:366` onward). This is a checked source
transcription, not execution of `identity::starter_body`.

| Manufacturer | Coarse cells | Full fine cells | Fine overlay | Base + overlay | Full connected components / void cells |
| --- | ---: | ---: | ---: | ---: | --- |
| PAN | 662 | 2,648 | 1,205 | 1,867 | 1 / 0 |
| RENKO | 688 | 2,752 | 1,176 | 1,864 | 1 / 96 |
| HUGO | 1,040 | 3,736 | 1,740 | 2,780 | 1 / 0 |

All six sprites map every occupied fine cell to an occupied reference cell by
`(x / 2, y / 2)` with integer division. PAN and RENKO are exact four-quarter
occupancy expansions. RENKO retains the original 24 enclosed coarse void cells
as 96 fine void cells, while its material remains four-connected. The independent
flood-fill oracle confirms this without using the packer's topology routine.

HUGO's declared inset removes 424 of the 4,160 doubled fine quarters. All 1,040
coarse positions remain represented, but 198 have incomplete coverage: 31 have
one quarter, 164 two, three three and 842 all four. Its four restored extrema
are connected, bounds are exact and it has no enclosed void. Matching occupied
bounds therefore does not mean identical visible collision material. This is
an intentional documented presentation change, not a pack defect.

The centered convention matches `PixelBody::center` (`pixels.rs:254`): fine
center `(x + .5 - 48) * .5` falls a quarter unit either side of its coarse
center. Every fine corner lies inside the mapped coarse cell. The existing
full affine composition (`transform.rs:95`) preserves that containment under
reflection, shear and nonuniform scale; the frozen source suite checks every
corner against its offline affine oracle. No native f32 rounding proof ran.

`PixelBody::damage` clears occupancy separately from stored color
(`pixels.rs:167`). A runtime adapter must consult surviving occupancy, rather
than use the intact reference or a color-only predicate. The independent probe
exhaustively removes each of the 2,390 coarse cells in the source lookup and
checks both full and overlay maps: all mapped fine cells disappear, and other
fine cells remain. That proves the lookup rule, not live projectile integration.

## Masks, materials and submission costs

The seven sorted masks are an exact disjoint partition of each sprite's occupied
positions, with every role represented. Signals are sorted occupied subsets
of accent/reactor/exhaust roles. Overlay material and signal masks are exact
restrictions of the corresponding full surface. Its occupied positions and
colors exactly match `fine_cells`. The 32-entry palette has transparent zero
and alpha 255 on every occupied entry. Corporate tint cannot reach the glass,
neutral metal, recess or fastener masks.

All full surfaces meet 4,000 cells and overlays meet 1,800. A full fine route is
one transform per intact ship; retained base plus overlay is two. Repeated
instances cost again. Source-group limits are authoring limits, not evidence
that a crowded runtime frame fits. Both parts of an overlay route must be
preflighted together and charged even if clipping or alpha masking hides cells.
Do not draw a full fine surface and its fine overlay together. Do not replace
the collision body, infer new exhaust origins or change damage/shield behavior
from this metadata.

For the first runtime pass, use original body occupancy and actor affine, compose
a centered .5 child for the overlay, and filter by the live coarse grid. Confirm
actual damaged collision holes, fragment/rebind behavior, transformed corners,
127/128 transform-table boundaries and crowded-scene caps in native integration
tests. Real state may affect the small signal mask; Reduced Motion freezes only
cosmetic phase. These checks remain required because no adapter is installed.

## Actual visual inspection

Viewed the committed `authoring-overview.png` and `compact-review.png` with the
image viewer. PAN has a narrow canopy/central warm reactor and green nacelles;
RENKO has long forward rails, preserved apertures and red trim; HUGO has a broad
shield deck and blue outer trim. Canopy seals, paired ribs/channels, recessed
vents, nozzle lips and fasteners remain coherent at the enlarged source view.
Their common alloy, glass and warm reactor language is consistent across the
three silhouettes. No unrelated recoloring or aperture fill was observed.

At the declared 29-pixel software sample, fine vent/fastener structure is lost
and the overlay alone is visibly sparse. The canopy/reactor and broad hull
differences remain recognizable; 48/77 samples show more of the authored
assemblies. Those are nearest-sampled authoring previews, not measured game
zoom, lighting, DPR or human acceptance. In particular, the sheet does not prove
the colors of the actual retained old body plus new overlay combination.

## Executed checks and remaining scope

On this Windows host, Python 3.12.10 / Pillow 12.3.0:

1. `python work/raicho-fine-independent-review/run_source_tests.py`: frozen
   suite 18/18 pass. It regenerates atlas, manifest, provenance, six source PNGs
   and two preview PNGs byte-for-byte in an owned temporary directory, and checks
   exact indexed/RGBA import, role corruption, topology, affine bounds and costs.
2. `python work/raicho-fine-independent-review/independent_probe.py`: 5/5 pass,
   covering raw partitions, independent topology, HUGO quarter coverage, every
   coarse damage lookup, all fine corners and exact submission counts.

The provenance's source-code hashes normalize CRLF to LF; atlas/manifest/PNG
domains remain raw. Regeneration from exact LF Git blobs passes. The generic
indexed PNG roundtrip preserves cells/palette/top-level extension but drops
sprite-level masks, as explicitly documented. Keep the authored atlas.

Art's frozen handoff and Windows/Linux source logs were inspected and copied
with their exact hashes; their Linux run is peer evidence, not my execution.
No native Game/render extraction, GPU/GL2, animation, frame time, phone or
natural-play acceptance is claimed. Root owns a separate runtime decision and
normal browser evidence. The code/pack reviewed here is immutable.
