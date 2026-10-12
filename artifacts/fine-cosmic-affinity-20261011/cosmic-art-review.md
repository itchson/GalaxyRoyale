# Independent fine cosmic source/software art review

Reviewed on 2026-10-11. No blocking source-art or material-contract defect was
found within the inspected scope. This is an independent specialist reading
and software-image assessment, not human acceptance or GPU combat contrast.

Frozen source is `aff95d69b3055191355bfa619a5ea307da852c3c`, integrated by Root
as `65ecaf288d9abd55149beb3aa0a6ae40c9e1944d`. Both actual current Root cosmic
source/test files are LF-equivalent to those frozen Git blobs. No source,
candidate evidence, runtime asset, frame cap, selector or probability was edited.
All new review files remain in this owned `work/cosmic-art-review/` directory.

## Actual inspected inputs and verification

The Engine handoff is at
`C:/Users/mitch/.codex/worktrees/galaxy-hud-navigation/galaxy-royale/work/cosmic-fine-detail/`.
I read its source, focused tests, handoff, manifest, exporter and software
renderer, and inspected the following images with `view_image`:

- `native-software-contact.png`, all 12 packet sets/36 declared zoom views.
- `software/galaxy-False-18.0-1.6.png` and `galaxy-False-0.0-0.6.png`.
- `software/nebula-False-18.0-1.6.png` and `nebula-False-0.0-0.6.png`.
- `software/supernova-False-18.0-1.6.png` and `supernova-True-18.0-1.6.png`.
- The current 64-control authoring overview, with the previously inspected
  frozen 96-cell RAICHO source overview as the second material benchmark.

| Exact input | SHA256 |
| --- | --- |
| Cosmic source, committed/LF | `05f78c74354b4c80f14757aba659a66cc981fba2c0eecc02fb6b33bda8e7f9c8` |
| Engine handoff, raw | `fc2947b082653f10c77a72f59f90d75751385ec19b30b6de72c06927ab7e0ba6` |
| Engine manifest, raw | `d7425a4e5e2b9f7afda0ba7c8e995b805eb83d37d751ca83ae097110af8a1a02` |
| Native software contact sheet, raw | `d2bd847396f9c7ec906f96499a34a59dbac5821e9aa30020bcbcdc4c35ae319d` |
| 64-control atlas, raw | `e86980bbb99f3c1423900283f78e7bba2aac4b90d02b4a0855291c2e5ff8ee73` |
| 96-RAICHO atlas, raw | `319aa1fd8cf1112ef33addd06739063c72b92ecca5ca460afcc05aada77666be` |

`evidence.json` pins every read native packet, each image above, the software
renderer and verifier, source files and comparison packs. The read-only Engine
`verify.py` actually passed **69 committed inputs / 84 raw evidence pins / 12
packet sets**; output is preserved in `read-only-verify.log`. My bounded parser
read all 12 packets, checked their hashes against the frozen manifest, all 36
views' exact declared cell counts, two transforms per set, four finite corners
per cell and bounded RGBA values. This did not create or rerun native packets.

| Motif | Actual full cells | Actual reduced cells | Parts/transforms |
| --- | ---: | ---: | ---: |
| Galaxy | 1,599 | 405 | 2 |
| Nebula | 1,253 | 323 | 2 |
| Supernova | 1,001 | 259 | 2 |

The proof deliberately poses each kind at origin zero, hash 93, scale 1.1,
seed 17, times 0/18, using native CameraView with 192×144 logical extent at
zoom 0.6/1.0/1.6. It is actual engine extraction of an isolated authored motif,
not a natural seeded encounter, full scene or normal browser screenshot.
The software renderer consumes exact emitted polygons/RGBA, samples at 4×,
composites flat alpha over `(3,4,6)` and downsamples BOX. It does not execute
WGSL, real GPU rasterization, linear-light blending, lighting or bloom.

## Concrete composition and material findings

1. **Galaxy:** the winding arms and localized warm center read as one spiral
   form. Interrupted inner/outer arm ridges retain deliberate dark lanes;
   empty wedges and the broad untouched surrounding field prevent a filled
   disc. The 1.6 software view exposes different cluster densities along the
   arm rather than a uniform stippled stroke. At 0.6 the arm texture becomes
   faint and much less articulated, while the central shape persists.

2. **Nebula:** two broken plum ridges follow a coherent bent lane, with a
   separate shorter gray-green thread. The central split is real absent
   material, not a black painted stripe. Full and reduced software views keep
   that main gap and silhouette; the reduced edge has visibly coarser clusters.
   It reads as sparse distant gas, not a new nearby hazard or pickup glow.

3. **Nova:** an asymmetric broken rim surrounds a small warm cross-like core.
   The open arc and uneven edge density make it different from a complete
   decorative ring. The reduced shell loses some fine edge articulation while
   retaining the opening and core. The software sheet remains near-black
   outside those localized structures.

The source fields constrain occupancy/color to arm distance, lane distance,
shell radius and fixed **two-unit birth clusters**. The hash supplies stable
cluster variation within those shapes; it does not independently scatter
bright speckles across every fine source cell. Full bodies sample at 0.5 local
units, reduced bodies at 1.0 from the same physical field, with real cached
positions and color ramps. Fine detail is actual additional occupied quads
and spatial clusters, not a doubled old bitmap.

The muted plum/dust tones and small cream/amber centers relate to the warm
reactor/brass highlights and cool neutral recesses in the control/RAICHO
benchmarks. The distant material appropriately has much lower alpha and
contrast than their opaque alloy, canopy and manufacturer accents. Consistency
here is coherent clustered shading and localized highlights, not making gas
look metallic or copying a foreground corporate palette across the sky.

## Recommendations and remaining gates

No scoped color/density/cap patch is proposed from these software images.
Keep the existing source hierarchy, voids and whole-group admission. The
custom **8,000/4,000** budget includes the **1,800/900** star/system reserve;
regional gas **800/400** remains inside its separate **6,000/3,000** pool.
Source uses default unlit motif material; brighter alpha breathing is not
emission, a dynamic world light or a new gameplay particle system.

Root's actual normal-backend review should judge the delicate 0.6 ridges under
moving camera/rotation, the relationship of the motif core to nearby hostile
projectiles/Mote warnings, and whole-scene admission when several forms share
the existing budget. Enlarging or brightening source merely to mimic the
software sheet is not justified without that observation. Static times 0/18
and source continuity tests do not establish a pleasing real-time animation.

Native assertion/Clippy outcomes in the handoff remain Engine's prior evidence,
not new passes by this reviewer. This review claims no new native execution,
browser/GPU contrast, listening, human art approval, natural campaign play,
phone performance, allocator/RSS measurement or sustained soak. Those gates
remain separate from source/software review.
