# PAN co-op dock: review candidate

This is a conversion of the actual generated overhead concept, not a replacement
for the shipping dock or an implementation of co-op/invite/join. The raw
`source.png`, generation prompt and tool provenance are retained byte-for-byte.
The tool exposed no model/version. Its requested 128-cell appearance is not an
exact native grid: the actual reference is **1254×1254** with partial alpha.

## Conversion and detail tradeoff

`scripts/qa/pan-coop-dock/convert.py` thresholds alpha at128, samples the complete
canvas onto128×128 using nearest center samples, and keeps every component. No
crop, largest-component filter, outline cleanup, hole filling, dithering or
uniform source-grid assumption is applied. Separate existing `asset-pack`
median-cut passes reserve at most47 metal,8 green and8 cyan opaque colors. The
combined64-entry palette includes index0 transparency. The final material PNGs
then pass through exact provided-palette import into the existing version1 ABI.
These are lossy raster-to-cell operations, not lossless conversion or newly
generated animation.

| Review grid | Occupied cells | Silhouette IoU against thresholded raw reference, after nearest reprojection | Single current PixelRig |
| --- | ---: | ---: | --- |
| 128×128 | 7,100 | 0.978342 | Fits its128-side limit |
| 192×192 | 16,085 | 0.985444 | Exceeds current limit |
| 256×256 | 28,549 | 0.990430 | Exceeds current limit |

The128 version is a compatibility/budget proof, **not the final detail-quality
recommendation**. The192/256 PNGs are comparison evidence, not runtime alternatives. They use the
same candidate palette. The128 variant keeps all four cardinal pads, the open
exterior cutouts, radial deck and green core. Small vents, fasteners, cable
separations and crystal facets collapse or alias; the reference's detailed
craftsmanship is **not fully retained**. The192 comparison introduces one sampled
enclosed transparent cell; neither comparison is automatically repaired.
`grid-comparison.png` shows this at a common384-pixel review size.

`conversion-losses.json` records the complete measurements. At128, thresholding
changes717,441 source alpha values;6,683 occupied cell colors change during
quantization (mean RGB channel error3.12343, maximum91, RMSE5.87361). Nearest mask
reprojection creates5,534 opaque and9,411 transparent mismatches against the raw
thresholded source. These figures characterize the declared conversion, not
perceived quality or GPU fidelity.

## Material and animation contract

Four disjoint native-grid parts partition the7,100 cells exactly once:

| Part | Occupied cells | PixelRig material role |
| --- | ---: | --- |
| Deck/recesses | 4,611 | Armor |
| Bright alloy | 1,786 | Trim |
| Cyan screens | 182 | Trim |
| Green signals/crystal | 521 | Emitter |

All children use finite zero relative translations under the deck parent. The
body remains centered on the full128-cell canvas. A future host chooses the
world footprint explicitly; this candidate assigns no collision radius,
facility centers, safe radius, rewards or multiplayer state.

The433-cell exterior green lamp mask selects three authored palette states:
baseline, dim and bright,700ms each. The dim/bright states change384/316 cells
respectively, only inside that mask. The central crystal and all non-lamp cells
stay unchanged; every frame has exactly the same occupancy, role map and affine
bases. Lamp shading uses nearest existing green colors to declared gains
1/.78/1.35; it does not flash the whole object. These are explicit authored
states, not imagegen-produced frames.

`pixel_rig` is an opt-in extension, not automatically consumed by the game. An
adapter can construct `PixelRig::new(128,128,roles,parts)` and the selected full
`dock-N` PixelBody. For Reduced Motion freeze body state0 and rig time0; do not
advance this timeline. `PixelRig` still supplies its normal material/tint
semantics. The proof holds charge0.2/time0/hit0 across states, giving signal
emission≈0.132 and preserving unchanged geometry; charge here is a shading
parameter, not a gameplay weapon or dock action.

Preflight the complete local object before appending: at most8,000 occupied
submissions,4 transforms, all four transformed corners finite and within the
declared1,000-cell proof extent. FrameBuilder has no global cap/rollback; the
proof's caller guard is not an engine-wide quota. Alpha-zero submissions would
still count. A128 transform table splits batches; it does not truncate a group.
The live damaged body is authoritative; role masks never refill removed cells.

## Reproduce and inspect

From the repository root, install `scripts/asset-requirements.txt` in a normal
Python3.12 environment (Pillow12.3.0). No new Python dependency is introduced.

```powershell
python scripts/qa/pan-coop-dock/convert.py
python scripts/qa/pan-coop-dock/test_conversion.py
cargo test --locked --manifest-path scripts/qa/pan-coop-dock/native/Cargo.toml
cargo clippy --locked --manifest-path scripts/qa/pan-coop-dock/native/Cargo.toml --all-targets -- -D warnings
cargo run --locked --manifest-path scripts/asset-native-preview/Cargo.toml -- assets/pipeline/pan-coop-dock/pack.json pan-lamps assets/pipeline/pan-coop-dock/native-extraction.json
python scripts/qa/pan-coop-dock/convert.py --compact-native
python scripts/asset-audit.py --pack assets/pipeline/pan-coop-dock/pack.json --extraction assets/pipeline/pan-coop-dock/native-extraction.json --output assets/pipeline/pan-coop-dock/native-audit.json
python scripts/qa/pan-coop-dock/convert.py --compact-native
python scripts/asset-native-render.py --input assets/pipeline/pan-coop-dock/native-extraction.json --output assets/pipeline/pan-coop-dock/native-preview.png --scale 4
cargo run --locked --manifest-path scripts/qa/pan-coop-dock/native/Cargo.toml -- assets/pipeline/pan-coop-dock/rig-extraction.json
python scripts/asset-native-render.py --input assets/pipeline/pan-coop-dock/rig-extraction.json --output assets/pipeline/pan-coop-dock/rig-preview.png --scale 4
```

The12 Python tests include full committed-byte regeneration, exact indexed
roundtrip, all material unions, actual pad/cutout landmarks, three lamp states,
budget/parent rejection and measured-loss checks. Generated source/mask PNGs use
the existing fixed-filter/stored-DEFLATE writer; their bytes are reproducible
without platform-specific zlib compression. The4 actual native tests cover
PixelRig colors/materials/all corners, real PixelBody damage, caller preflight
and crossing the128-transform table. The existing native adapter extracted
three actual frames at0/700/1400ms with7,100 cells/4 transforms each; its audit
matches every palette color and four corners (observed error0).

The candidate directory's scoped Git attributes preserve reference/provenance
and generated data bytes across checkout platforms. Validation hashes those
assets raw; surrounding source code hashes normalize CRLF to LF, with the actual
Windows input hashes retained separately. `python scripts/qa/pan-coop-dock/verify.py`
performs a read-only pin/packet audit. This does not claim a Linux test run.
Native packet JSON is minified without changing decoded cell/corner/material
data; rerun compaction before auditing freshly emitted pretty packet JSON.

`native-preview.png` is flat software rendering of actual PixelBody/FrameBuilder
packets. `rig-preview.png` is flat software rendering of actual PixelRig packets
under a reflected/sheared fractional affine, not a GPU screenshot. Neither
reproduces GPU lighting/filtering. `lamp-states.png` and `grid-comparison.png` are
authoring/reference review images. Shipping integration, co-op semantics, both
GPU backends, moving human readability and physical performance remain pending.
