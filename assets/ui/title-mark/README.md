# GALAXY ROYALE native title mark

The original generated [reference](reference.png) supplies the distinctive
two-line angular platinum/graphite wordmark, cyan crystal/inlays and restrained
champagne-gold bevels. [Provenance](provenance.json) and the exact
[prompt](PROMPT.txt) record the built-in imagegen call; its model/version is not
exposed. This is a new proposed title identity, not inferred user approval.

The shipping route is native occupied cells, not a raster billboard. The
512Ã—192 [source field](source-cells.png) is split into eight 128Ã—96 tiles and one
32Ã—64 crystal. Their complete source-cell union is preserved without duplicate
letters. The smaller crystal bounds deliberately exclude the neighbouring
L/A corners; an actual native regression keeps those letter cells stationary.
The final atlas has 41,662 converted occupied cells and 64 palette entries.
Three separately authored nine-cell stellar marks bring runtime cost to
**41,689 occupied cells /12 transforms**. No coarse LOD or minimum-cell floor
is used at small sizes. Cold cache/pack parsing and FrameBuilder storage can
allocate; warmed extraction borrows cached bodies without cloning them.

The raw reference is 2048Ã—768. Conversion intentionally samples it 4:1 with
nearest neighbour, applies alpha>=128, and quantizes to63 opaque colors plus
transparent index0. [Measured losses](conversion.json) report 49,509 sampled
alpha changes, RGB-channel mean absolute error 4.4722/max 77, and zero explicit
component stripping. Detail lost by sampling is not restored or described as
lossless. Exact indexed cells are preserved after this declared conversion.

The actual [asset-pack](../../../scripts/asset-pack.py) imports the prepared
PNGs under a provided exact palette. Authoring group transforms and masks are
validated; the native adapter explicitly consumes `title_roles` and the
matching source coordinates. Metadata alone does not produce animation.
The authored runtime motion consists of a 0.6-source-cell vertical crystal
float, three slow 1.1/0.6-cell stellar offsets, a narrow eight-second metal
sweep and restrained cyan breathing. Neutral unmasked material stays exact.
Reduced Motion freezes positions/colors to a stable pose. All occupied cells,
including alpha-zero clipped submissions, charge the 60k/120k UI quotas.
Whole-logo preflight rejects without partial frames or budget mutation.

`UiNode.kind="logo"` is the existing title bridge contract. DOM supplies an
accessible GALAXY ROYALE label and a responsive 8:3 rectangle; the native helper
centres a continuous uniform fit. `title-backdrop` supplies the separately
requested one-quad near-black scrim. No audio or shell lifecycle is implemented
by this asset. The historical `assets/ui/atlas.json` logo is not a runtime
fallback for this route.

Focused checks:

```powershell
python -m pip install -r scripts/asset-requirements.txt
python scripts/asset-title-mark-tests.py
cargo test --locked -p game_web title_art:: -- --nocapture
cargo clippy --locked -p game_web --all-targets -- -D warnings
```

Seven Python checks cover independent exact committed-atlas regeneration,
all-cell partition/RGBA reconstruction, disjoint masks, declared losses,
indexed PNG roundtrip, changed cell fail-closed guards and group budgets.
Canonical PNG compressed bytes are reused only after independent decoded
dimensions/full RGBA equality, so platform compression differences cannot
silently change provenance or art. Nine actual native checks cover full and
tiny fractional extraction, neutral masks/relative motion, letter invariants,
reflected/sheared partial clips, complete two-quota admission,128-transform
batch splitting and joined title packet parity. The authored joined scene
with backdrop, logo, copy, Launch/Settings/Quit, control guide and build line
retains every isolated component at **45,598 cells /160 transforms**. This is
a native fixture, not measurement of Root's final DOM or browser frame.

The [native software sheet](native-software-contact.png) is derived from actual
registered Cargo-test FrameBuilder packets. Its4Ã— supersample/BOX flat-unlit
renderer is an approximation, not WebGPU/WebGL2 execution. Raw packets/logs and
their exact hashes are retained in `work/title-mark/`. Root owns normal build,
actual title captures, viewport readability, final approval and the future
user-supplied MP3. No performance, physical-phone or listening acceptance is
claimed here.
