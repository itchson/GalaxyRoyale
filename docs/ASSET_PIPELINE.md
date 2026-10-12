# Pixel asset pipeline

Galaxy Royale draws occupied engine cells. PNG files are authoring inputs and review outputs; importing a pack does not introduce textured sprites or make generated concept art an accepted game asset.

Optional per-cell surface maps follow the same reviewed-source pipeline. Semantic heights and sparse signal masks are packed through the real Rust encoder, bound to the original atlas, and cached for matching native assets. See [PIXEL_SURFACES.md](PIXEL_SURFACES.md) for the normal/luminance contract and current acceptance boundaries.

`scripts/asset-pack.py` is an offline, deterministic pack/unpack/validation tool. Exact conversion is the default: native grid, binary alpha, all components, original dimensions and exact colors. The current game/UI atlas readers keep their version-1 ABI: `palette` contains little-endian u32 RGBA, index0 is transparent, and each named sprite has width, height and row-major indices. Default exact packs have 64 entries, padding unused entries with a duplicate opaque color. A smaller/different palette is possible explicitly, but each runtime consumer's existing limits still apply. The game loader accepts sprites up to128; the UI loader supports its existing larger logo. The packer's generic256-side limit does not override either consumer.

## Fresh checkout

Use Python3.10+ and the pinned Pillow version. No model, network service or GPU is needed for conversion. Native extraction also needs the repository's Rust toolchain.

```powershell
python -m venv work/asset-venv
work/asset-venv/Scripts/python.exe -m pip install -r scripts/asset-requirements.txt
work/asset-venv/Scripts/python.exe scripts/asset-pack-tests.py -v
cargo test --locked --manifest-path scripts/asset-native-preview/Cargo.toml
```

On Linux/macOS use the environment's `bin/python`. The checked-in isolated native utility has its own lockfile/workspace and depends on the actual repository `engine_core`; it does not change shared manifests or enter the release build.

## Authoring contract

A manifest is schema 1. Each sprite has a safe logical identifier and explicit source PNG path. Optional `crop` is `[x,y,width,height]`, bounded within the source. No automatic tight crop, padding, layout inference or largest-component filtering occurs. A `cell_grid:[2,2]` means each two-by-two source block is one cell; every block must be exactly uniform. It is grid decoding, not downsampling arbitrary art. Transparent hidden RGB is canonicalized to index 0 and counted in evidence; it is not visible material. Animated PNGs must be split into declared frames rather than implicitly taking the first frame.

The whole import permits at most 16,777,216 decoded source pixels (repeated inputs charge again), 1,048,576 stored cells, 256 sprites and a 32 MB JSON input. Source-header budget checks happen before PNG decode. `budgets.source_pixels` can reduce that work bound. Preview output is also bounded to 16,777,216 pixels.

```json
{
  "schema": 1,
  "palette": { "mode": "exact", "max_colors": 64 },
  "sprites": [
    {
      "name": "rock",
      "source": "rock.png",
      "cell_grid": [1, 1],
      "alpha": { "mode": "exact" },
      "components": { "mode": "all" },
      "resize": { "mode": "none" },
      "topology": "solid",
      "anchor": [0, 0]
    }
  ],
  "provenance": { "source_type": "original-authored-cells" }
}
```

`solid` requires one four-neighbor connected occupied body and zero enclosed transparent cells. Exterior flood fill includes every image edge, even when a shape divides empty corner regions. Use `holes-allowed` for landing bays, antennas, separated logo letters, articulated hulls and intentionally open geometry. Opaque dark crater floors remain material; later actual gameplay damage can remove cells and create collision holes. The tool never fills artist-authored holes automatically.

Lossy choices must be explicit: `alpha:{mode:"threshold",threshold:128}`, `components:{mode:"largest"}`, `resize:{mode:"nearest-fit",size:[48,48]}`, or `palette:{mode:"quantize",max_colors:64}`. Evidence records changed alpha cells, removed component cells, dimensions, occupied counts before/after resizing and changed color cells. Quantization is median-cut with no dithering. These choices do not certify that thin art, readable silhouettes or useful components survived; review the output. Unknown/misspelled policies fail.

Use `palette:{mode:"provided",max_colors:64,colors:[...]}` for a fixed accepted palette. RGBA sources must match its colors exactly unless a quantization mode is explicitly chosen. For truly exact legacy index round trips, `unpack` creates indexed PNGs plus `encoding:"indices"` and the complete original palette. This retains duplicate and unused palette entries and distinguishes two identical-color indices. RGBA alone cannot recover that distinction. Palette/transparency mismatches fail instead of guessing.

## Commands and exact round trips

```powershell
python scripts/asset-pack.py pack --manifest assets/pipeline/haven-service/manifest.json --output work/haven-pack.json
python scripts/asset-pack.py validate --input work/haven-pack.json
python scripts/asset-pack.py preview --input work/haven-pack.json --output work/haven-cells.png --zoom 4
python scripts/asset-pack.py unpack --input assets/pixels/atlas.json --output-dir work/atlas-export
python scripts/asset-pack.py pack --manifest work/atlas-export/manifest.json --output work/atlas-repacked.json
```

Unpack requires an absent/empty output directory. Packing cannot overwrite its manifest or a declared source PNG. Existing game, UI and boss atlas tests compare every dimension, palette entry, index and occupied count. Runtime-specific extension fields such as legacy boss `rigs` are preserved verbatim as explicit manifest `extensions`; their gameplay/rig semantics are not validated by this generic cell tool. No existing runtime atlas is overwritten or migrated by these tests.

## Groups, transforms and animation

Additive `asset_metadata` contains the authoring policies, actual source hashes, topology/occupied bounds, budgets, provenance, groups and animation references. Legacy game/UI loaders ignore it. An `anchor` is a finite authoring annotation; the utility does not silently change the engine's centered body-local origin to that anchor.

A group has ordered named `parts`, each referencing a sprite and optionally an earlier parent part. A transform has `position`, `basis_x` and `basis_y`. Full affine composition preserves reflected, nonuniform and sheared parents. Bounds include all four corners of every occupied cell; they are not a transformed texture rectangle. Every reused sprite instance charges its complete occupied count. Group preflight checks every child before emitting any packet. The default maximum is60,000 occupied cells and256 transforms; the engine splits batches at128 transform slots. A128 limit is not incorrectly imposed on a whole valid group. Author budgets may reduce those bounds, never increase the audited maxima. Masks/culling cannot be used to undercount submitted work.

An animation contains a bounded list of explicit `sprite` or `group` frames, positive integer `duration_ms` and boolean `loop`. The authoring tool validates references and durations. The opt-in native review adapter implements group-frame selection at declared time boundaries. The production game does **not** automatically instantiate those groups, play these animations, replace entity collision bodies, apply anchors or install abilities. Any future runtime adapter must explicitly decide how animated occupancy interacts with damage and physics. Existing `PixelRig` semantics remain unchanged.

## Real checked-in example and native proof

[Haven service manifest](../assets/pipeline/haven-service/manifest.json) reuses the accepted generated dock's exact palette/indices and adds original service-lamp cells in that palette. The open landing bay and all detached lamps are retained. Two frames share occupancy/geometry while changing screen colors. A reflected child is nested under a nonuniform/sheared parent. The example and generated pack/PNGs are durable source artifacts, separate from shipping runtime assets.

```powershell
python scripts/asset-example.py --atlas assets/pixels/atlas.json --output-dir assets/pipeline/haven-service
cargo run --locked --manifest-path scripts/asset-native-preview/Cargo.toml -- assets/pipeline/haven-service/pack.json service-pulse work/haven-native.json
python scripts/asset-native-render.py --input work/haven-native.json --output work/haven-native.png
python scripts/asset-audit.py --pack assets/pipeline/haven-service/pack.json --extraction work/haven-native.json --output work/haven-audit.json
```

The native adapter loads the actual pack into `PixelBody`, composes nested `AffineTransform`s and emits through the actual `FrameBuilder`. Each example frame emits 1,518 occupied cells/3 transforms. Native tests exercise 129 transform slots, exact animation boundaries, reflected/sheared corners and all-or-nothing budget rejection. `asset-audit.py` checks every actual cell color and all four affine corners against the packed source, with a disclosed 0.0001-world-unit f32 tolerance.

The checked-in `native-extraction.json` and `native-preview.png` preserve the earlier actual native execution. Local Windows Application Control blocked the final test executable with OS error 4551; no renamed binary or policy bypass was attempted. Subsequent clean Windows and Ubuntu [CI run 38108345034](https://github.com/itchson/GalaxyRoyale/actions/runs/38108345034), at exact source `07d3f131016e8d2caaae5ce9b7efb9890f276c4d`, passed all five final native tests and newly extracted/audited the accepted dock through the actual engine. Its per-run packet/audit files are retained as CI artifacts and the observed job results in `artifacts/presentation-refinement-20261011/foundation-ci.json`.

`historical-audit.json` still audits the earlier packet geometry/colors/order without claiming final executable or sample-time proof. The software PNG does not claim GPU filtering, lighting, live Game animation, browser readability or physical performance. See the example README for the precise historical validation boundary.

## Generation to acceptance

1. Start from the current accepted palette, object size, occupied silhouette, collision envelope and per-object budget. Use the existing dock/ship/boss artwork as references, with a concrete asset brief and intended layers/groups.
2. For new raster reference art, use the installed **imagegen** skill's built-in tool, or an explicitly selected local ComfyUI workflow. Save tool/model metadata actually exposed, prompt, source hash and supplied licensing notes. An unknown model stays unknown. These are concept references under `artifacts/concepts/`, not runtime cells.
3. Inspect the generated sheet. Author or edit a native pixel grid; alternatively choose explicit crop/alpha/resize/quantization policies and inspect their measured losses. Preserve separated components unless the brief explicitly allows removal. Never call an arbitrary generated high-resolution image a perfect pixel atlas.
4. Pack, validate, inspect cell/topology/budget metrics and generate a native extracted preview. Review silhouette, directional materials, palette contrast, detail density, pivot/parent placement and any animated mask changes against gameplay intent.
5. Only then adapt the reviewed pack to the intended runtime consumer. Verify real damage holes, group budget rejection, animation/Reduced Motion and both backend presentations. Human moving-play, hearing and physical-phone gates remain separate; native proofs cannot mark those passed.

The earlier `convert-atlas.py` and `convert-ui-atlas.py` remain reproducible historical conversion recipes for their particular inspected sheets. Their hard-coded crops and explicit historical loss policies are not the default of this reusable pipeline. No generation service or global model configuration is installed by these scripts.

## PAN co-op dock conversion candidate

[PAN dock candidate](../assets/pipeline/pan-coop-dock/README.md) retains the actual
1254×1254 imagegen reference and records explicit alpha128/nearest128 sampling
and material-stratified palette losses. Its64-color pack preserves all sampled
components and exterior pad cutouts, with7,100 occupied cells split into four
logical parts. Three authored lamp-only color states share identical occupancy.
The existing native utility proves exact PixelBody extraction, and a scoped
`scripts/qa/pan-coop-dock/native` proof exercises the actual PixelRig, material
roles, fractional/reflected/sheared corners, damage holes and bounded admission.

192/256 comparison views show the detail tradeoff but exceed the current
single-rig128-side contract. No engine limit is enlarged and no shipping dock is
replaced. The reference's finest texture/crystal details are not fully retained
at128; GPU/live-game/co-op and human/device acceptance remain separate gates.
