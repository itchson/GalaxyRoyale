# GALAXY ROYALE native sports-league mark

GALAXY ROYALE is the universe's arena-battle sporting league. The current
[reference](reference.png) supplies forward-leaning athletic lettering, angular
competition wings and an orbital stadium crest with a gold award face.
[Provenance](provenance.json) and [prompt](PROMPT.txt) record the actual built-in
imagegen call; its model/version is not exposed. Root inspected the reference
and native source field. Final browser presentation and user approval remain
separate. The fantasy/adventure mark at ee16cea is superseded; its source and
proof remain in Git history and the historical work folders.

The renderer receives occupied native cells, not a raster billboard. The
512x192 [source field](source-cells.png) is partitioned into eight static
128x96 tiles and a 48x64 crop containing only the gold award face. The complete
nonduplicating union is **44,985 cells /9 transforms**: 44,520 static cells and
465 award cells. No extra glitter is added. Source roles contain 6,704 energy,
24,280 metal-scan and 465 award cells. Native regressions keep handpicked letter
corners stationary while the award moves. Continuous fractional fits preserve
every source cell without a coarse LOD or minimum-cell floor. Cold cache
parsing and FrameBuilder storage can allocate; warmed extraction borrows
cached bodies without cloning them.

The raw reference is 2048x768. Conversion explicitly samples 4:1 with nearest
neighbour, admits alpha>=128 and maxRGB>=40, then quantizes to63 opaque colors
plus transparent index0. [Measured losses](conversion.json) record 61,052
sampled alpha changes, including removal of 13,080 near-black haze/recess
cells; RGB-channel mean absolute error is3.76241/max77. No component-size
filter is used. Lost source detail is not described as restored or lossless.
After conversion, exact indexed cells and the complete partition are preserved.

The [asset-pack](../../../scripts/asset-pack.py) imports prepared PNGs with an
exact provided palette. Native code explicitly consumes title_roles and
subobject coordinates; metadata alone does not animate the asset. Authored
motion is a0.35-source-cell award float, restrained +/-6% award glint, +/-7%
arena energy chase and a12-source-column metal sweep over eight seconds.
Neutral unmasked material remains exact. At the compared poses,7,887 cells
change color; the new energy mask supersedes the earlier mark's smaller mask.
Tests separately enforce amplitude bounds, the narrow scan, neutral-color
invariance and a less-than20% changed-cell ceiling. Reduced Motion freezes
positions and colors to a stable authored pose.

Every occupied submission, including alpha-zero clipped cells, charges both
60k pixel and120k work quotas. Whole-logo admission preflights both quotas and
finite composed corners before mutation. Source group ceilings are45,000
cells /9 transforms. Normal128-transform batch splitting preserves all parts.
UiNode.kind="logo" retains the existing accessible label and measured8:3
rectangle; native extraction centres a uniform continuous fit.
title-backdrop supplies a separate near-black quad with alpha246. No shell,
audio or lifecycle behavior is implemented by this asset.

Focused checks:

```powershell
python -m pip install -r scripts/asset-requirements.txt
python scripts/asset-title-mark-tests.py
cargo test --locked -p game_web title_art:: -- --nocapture
cargo clippy --locked -p game_web --all-targets -- -D warnings
```

Seven Python checks cover independent exact committed-atlas regeneration,
all-cell/RGBA reconstruction, three disjoint masks, losses, indexed PNG
roundtrip, changed-cell fail-closed guards and group budgets. Writers use
canonical LF. Canonical PNG compressed bytes are reused only after independent
dimensions/full decoded RGBA equality, preventing compression differences
from changing provenance or art.

Nine actual native checks cover full/tiny fractional extraction, relative
motion and material bounds, letter invariants, reflected/sheared partial
clips, atomic two-quota admission, batch splitting and joined title parity.
The authored title fixture retains backdrop, logo, all copy and
Launch/Settings/Quit controls at **48,894 cells /157 transforms**. It is a
native fixture, not measurement of Root's final DOM.

The [native software sheet](native-software-contact.png) derives from actual
registered Cargo-test FrameBuilder packets. Its4x supersample/BOX flat-unlit
renderer approximates presentation; it is not WebGPU/WebGL2 execution.
Current packets/logs are in work/title-sports-mark. The prior work/title-mark
proof is historical. Root owns the normal build, actual title captures,
viewport readability, final approval and future user-supplied MP3. No
performance, physical-phone, listening or full concept-parity acceptance is
claimed here.
