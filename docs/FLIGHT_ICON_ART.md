# Detailed flight controls

The current authoring set contains ten original **64×64** control objects in
`assets/ui/flight-icons/atlas.json`. They are newly drawn cell silhouettes and
panel clusters, rather than enlarged copies of the historical 32-cell KIKAI
icons. The native flight-control loader already consumes this version1 schema;
the 32-cell pack remains historical/separate. This refinement changes the asset
cells and palette only, not the loader, animation code, layout or hit targets.
It covers controls, not every game asset.

All controls share neutral platinum bevels, dark graphite recesses, authored
alloy planes and cyan indicators, matching the athletic sports-league mark in
`assets/ui/title-mark/`. Generic repeated diagonal blue-gray bands are replaced
with specific wing facets, folded plate faces, gear chamfers, cartridge cuts,
cross-arm planes and locator-arm recesses. Small fixed gold fasteners/locators
occupy at most3% of each object; they do not signal ownership or gameplay state.
Each central motif is deliberately
larger than its seams or rivets. The status control is an overhead RAICHO;
map has three folded plates and route nodes; settings has an open gear bore;
save has an open data aperture; close has crossing rails; resume has a strong
triangle; restart has a return arrow; title has the Haven roof and docking
throat; quit has a power-ring gap; fullscreen has four separate locator arms.
Intentional transparent apertures and disconnected parts are retained.

Every occupancy cell, open aperture, outer bound, accent cell and emitter cell
is unchanged from the first64-cell pack (`e86980bb…`). The10 controls still cost
14,709 cells. Cyan source colors and semantic animation masks remain exact;
only neutral material faces and unmasked fixed gold fittings are reauthored.

## Pack contract

The outer object has `version: 1`, `palette` and `sprites`. Palette entries are
32-bit RGBA in little-endian channel order, with index zero transparent and all
other entries opaque. There are 21 distinct entries, below the 64-color cap.
Sprite keys are `icon_status`, `icon_map`, `icon_settings`, `icon_save`,
`icon_close`, `icon_resume`, `icon_restart`, `icon_title`, `icon_quit` and
`icon_fullscreen`. Every sprite contains:

- `width: 64`, `height: 64` and 4,096 row-major palette `indices`;
- `accent_cells`, sorted unique **linear source-cell indices**;
- `emitter_cells`, a nonempty strict subset of `accent_cells`.

The two masks select already occupied cells. No mask can capture immutable
metal/secondary colors. The host may tint only accents for semantic state and
vary only emitter color/emission subtly for hover/busy state. Geometry and
occupancy remain fixed; Reduced Motion should hold the emissive animation
phase. The metadata alone does not install animation or input behavior.
The host should retain the current save pending/error accent meaning.

Fit each object continuously inside its actual measured control rectangle;
the 64-cell source is retained under fractional cell transforms. Do not resize
or replace it with a smaller bitmap. Fractional source cells do not guarantee
that every cell receives a sample at a tiny physical display size. The
authoring review therefore includes explicit 14/28/42-pixel nearest samples.
They are **software authoring previews**, not native packet or GPU evidence.
At the 14-pixel sample the main forms stay distinct while fine seams fade.
Human readability, actual GPU blending, hover/busy animation and physical
phone input remain separate acceptance gates.

## Occupied-cell budgets

| Object | Occupied | Accent | Emitter |
| --- | ---: | ---: | ---: |
| Status | 1,291 | 113 | 18 |
| Map | 2,036 | 178 | 15 |
| Settings | 1,802 | 100 | 36 |
| Save | 1,503 | 36 | 12 |
| Close | 1,289 | 98 | 16 |
| Resume | 1,362 | 156 | 26 |
| Restart | 1,373 | 97 | 18 |
| Title | 1,591 | 51 | 18 |
| Quit | 1,322 | 100 | 27 |
| Fullscreen | 1,140 | 312 | 76 |

All ten total 14,709 cells. Status/map/settings/save total 6,632 cells per
instance group. The UI's existing 60,000 submitted-cell and 120,000 work caps
remain unchanged. Charge every instance's occupied cells, including clipped
alpha-zero submissions; reused cache entries do not make repeat draws free.
Admit each icon atomically. This budget table is authoring data, not a frame
rate or device measurement. For example, the existing dense 15,588-cell UI
fixture plus the 34,594-cell DORA portrait plus those four icons totals 56,814;
other simultaneous controls still require actual host budget checks.

## Exact authoring and reproduction

Use Python with the repository's pinned dependency:

```powershell
python -m pip install -r scripts/asset-requirements.txt
python scripts/asset-flight-icons.py --output assets/ui/flight-icons
python scripts/asset-flight-icons-tests.py
```

Pillow is used for integer-cell drawing and exact decoding. The source PNG
writer emits standard RGBA PNG, filter zero and explicit stored DEFLATE blocks
without linked compression-library choices. Every source byte can therefore
be checked on Windows/Linux. `asset-pack.py` imports the resulting source PNGs
with the provided palette, exact binary alpha, all components, no resizing and
no quantization. Source/manifest raw SHA-256 values are retained in the pack.
Runtime JSON is compact and canonical LF; readable provenance is separate.

The tests regenerate the complete committed atlas, all ten source PNGs, both
review images and provenance and require byte equality. They decode every PNG
back to its exact indexed RGBA cells, verify visual landmark coordinates and
open apertures, enforce bounds/palette/mask/work budgets, reject malformed
roles and alpha, and distinguish fine 64-cell clusters from doubled 32-cell
blocks. The encoding test also covers multi-block stored PNG streams.
The sports pass additionally checks a frozen all-cell occupancy/role digest,
handpicked facet/fitting coordinates, the restrained gold budget, exact cyan
role colors and neutral graphite/platinum contrast. Counts alone cannot pass
if an edge, aperture or role cell moves.

`authoring-overview.png` presents the actual source objects at 2× nearest.
`compact-readability.png` presents 14/28/42-pixel software samples enlarged for
inspection. No AI bitmap was generated for this set and no conversion loss is
claimed or hidden: the source cells were authored directly by the script.
The sports revision's source and14/28/42px previews were inspected; they retain
ten distinct main motifs while their thinner seams fade at14px. This is an
authoring/software assessment. Final normal native/GPU presentation of the
revised colors, state animation and small-size user readability remain Root's
integration gate; no new browser or physical-device claim is made here.
Future intentional artwork changes require review of the regenerated sources,
mask landmarks and budget evidence before committing them.
