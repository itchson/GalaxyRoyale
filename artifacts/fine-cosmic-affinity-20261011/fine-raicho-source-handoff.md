# Fine corporate RAICHO source-art handoff

Source commit: `7884a7cff0476bacb354770700ef03228f146425` on
`codex/raicho-fine-detail`, based on
`f69d6cbe8d58d182da6e45460b42b963655c5fbc`.

Only the new `assets/pixels/raicho-fine-detail/` pack, two dedicated Python
scripts and `docs/RAICHO_FINE_ART.md` are committed. No Rust, game/shell source,
shared manifest, Studio state, server, browser, published package or runtime
collision body changed. Nothing was pushed.

## Inspect

- `assets/pixels/raicho-fine-detail/atlas.json`: version1 packed cells plus
  per-sprite `material_cells`, `fine_cells`, `signal_cells`, and top-level
  authoring contract `raicho_detail`.
- `assets/pixels/raicho-fine-detail/manifest.json`: provided32 palette, exact
  binary alpha, no resizing, all components, one-surface half-cell groups.
- `assets/pixels/raicho-fine-detail/provenance.json`: actual source/atlas/script
  fingerprints and conversion/budget policy. Script hashes explicitly
  normalize checkout CRLF; PNG/manifest/atlas hashes use raw bytes.
- `assets/pixels/raicho-fine-detail/authoring-overview.png`: actual96 source
  cells and a declared2× nearest authoring review, not native extraction.
- `assets/pixels/raicho-fine-detail/compact-review.png`: declared software
  29/48/77 output-pixel samples of each full surface and isolated fine overlay.
  These are not measured runtime zoom/DPR/display samples.
- `docs/RAICHO_FINE_ART.md`: integration alternatives, material/color roles,
  submission budgets, damage mapping and pending gates.

Atlas SHA256:
`319aa1fd8cf1112ef33addd06739063c72b92ecca5ca460afcc05aada77666be`.
Authoring overview SHA256:
`6fe5cba94bab08a49eb0fdd7c3b51dba0c322b430af89b71ce7322157112fbce`.
`work/raicho-fine-detail/evidence.json` pins all15 committed inputs and both
actual check logs; text-source Git equivalence is checked with CRLF→LF only.

## Exact source counts and tolerances

| Surface | Old48 occupied | Full96 occupied | Fine overlay | Old+overlay | Signals |
| --- | ---: | ---: | ---: | ---: | ---: |
| PAN HIKARI | 662 | 2,648 | 1,205 | 1,867 | 31 |
| RENKO HIBANA | 688 | 2,752 | 1,176 | 1,864 | 27 |
| HUGO MAMORI | 1,040 | 3,736 | 1,740 | 2,780 | 31 |

All full bodies are exactly one four-neighbor connected component. PAN/HUGO
have zero enclosed transparent cells. RENKO intentionally retains24 reviewed
coarse rail-aperture cells as96 fine void cells; it is not labeled born-solid.
HUGO's proposed full surface has a declared one-fine-cell perimeter inset and
four connected original extrema. None of the six sprites has an occupied cell
outside its reviewed48 occupied envelope. No silhouette/envelope tolerance is
allowed: mapping, extrema and masks are exact integer-cell checks.

Fine coordinates use centered96 cells with local0.5 basis, then the existing
actor affine/world scale. Each fine cell belongs to
`[floor(source_x/2),floor(source_y/2)]`. A future adapter must remove all mapped
fine cells when that coarse cell is destroyed. The test proves that mapping
specification without a running projectile or renderer. Reflected/sheared
nonuniform parent matrix and all-cell-corner bounds are checked by the offline
pack utilities; this is not a native f32/GPU roundoff proof.

Retained48 + fine overlay is the collision-preserving proposal. Full96 is a
separate proposed presentation surface, not an automatic collision replacement.
Do not append the full surface and its fine overlay together: that duplicates
detail. Every reused instance counts again. Per-surface cap4,000; overlay1,800;
pack authoring groups cap4,000 cells/four transforms. Existing engine frame caps
are neither changed nor claimed sufficient for an arbitrary crowded scene.

The seven explicit material roles partition all occupied positions and permit
only their expected palette ramps. `signal_cells` are a strict occupied subset
of manufacturer/reactor/exhaust roles. Overlay roles and signals exactly match
their full source. Glass/alloy/recess/bolts never inherit manufacturer tint.
No animation, emission, input-dependent thrust or damage adapter is installed;
Reduced Motion behavior is a documented future host contract, not a test pass.

## Actual validation

`python scripts/asset-raicho-fine-tests.py` passed18/18 on:

- Windows11: Python3.12.14, Pillow12.3.0, `windows-tests.log`.
- WSL2 Ubuntu-24.04: Python3.12.3, Pillow12.3.0, `linux-tests.log`.

Both runs independently regenerate all committed atlas/manifest/provenance,
six source PNGs and two preview PNGs byte-for-byte. They check hard alpha,
RGBA/indexed PNG roundtrip, integer geometry against the reviewed source's f32
formulas, old occupied envelope, deliberate topology, exact material landmarks,
new fine clusters, restricted corporate/signal roles, repeated-instance costs,
four-corner affine mapping and malformed/removed-cell rejection. The generic
PNG roundtrip retains indexed cells/palette and top-level extension, not
sprite-level role metadata; keep the authored JSON for those masks.

No Rust/game tests, native preview, GPU/browser execution, human play, phone,
performance timing, generation model or publishing action was claimed here.
Root/independent specialists own subsequent source review and runtime choice.
