# Haven service pipeline example

This is a durable authoring and native-extraction example. It is not referenced by the shipped Game asset loader.

`dock.png` preserves the actual accepted `assets/pixels/atlas.json` dock palette and indices: 64×64, 1,454 occupied cells and an intentionally open landing bay. That dock originates from the generated sheet recorded in `artifacts/asset-provenance.json`; its model identifier and licensing guarantee were not supplied. `service-0.png`/`service-1.png` are original 12×8 cell-authored lamps in the same palette. All four detached lamp components are retained, 32 occupied cells per sprite. No new concept generation or model download was performed for this example.

`manifest.json` explicitly defines two group frames. Each includes the dock and two reused service instances: 1,518 submitted cells and three transforms. A reflected child is nested under a nonuniform/sheared parent. Frames change lamp colors while retaining the same geometry. Full affine composition and occupied-cell bounds are recorded in `pack.json`.

Reproduce conversion with the pinned Python environment described in `docs/ASSET_PIPELINE.md`:

```powershell
python scripts/asset-example.py --atlas assets/pixels/atlas.json --output-dir assets/pipeline/haven-service
cargo test --locked --manifest-path scripts/asset-native-preview/Cargo.toml
cargo run --locked --manifest-path scripts/asset-native-preview/Cargo.toml -- assets/pipeline/haven-service/pack.json service-pulse work/haven-native.json
python scripts/asset-native-render.py --input work/haven-native.json --output work/haven-native.png
python scripts/asset-audit.py --pack assets/pipeline/haven-service/pack.json --extraction work/haven-native.json --output work/haven-audit.json
```

The preserved `native-extraction.json` came from actual repository `PixelBody`/`FrameBuilder` execution before the final utility guards. Its two frames each contain 1,518 cells, three transforms and one batch. `native-preview.png` is a flat software raster of those actual packet corners. This earlier run did not record sampled timeline timestamps. The original native extraction source fingerprint was not retained, so this output is historical proof rather than final-source execution acceptance.

`historical-audit.json` compares every extracted color, cell corner and occupied bound to the final example pack. Its explicit historical mode checks frame order and declared durations without claiming timestamped final execution. To reproduce that audit:

```powershell
python scripts/asset-audit.py --pack assets/pipeline/haven-service/pack.json --extraction assets/pipeline/haven-service/native-extraction.json --output work/haven-historical-audit.json --historical-extraction
```

Local checks passed 26 Python tests, including exact existing game/UI/boss atlas round trips, and strict compile-only native Clippy. Four earlier native tests passed before the final empty-body guard. The final five-test executable compiled but Windows Application Control blocked execution (OS error 4551, never executed). `validation.json` records this distinction; the final native execution gate awaits a clean runner. No blocked binary was renamed/copied and no system policy was changed.

Authoring metadata, anchors and this preview timeline are not automatic runtime semantics. Live animation, damage/collision changes, Reduced Motion and runtime integration require an explicit Game consumer. GPU backend presentation, moving human readability and physical-phone performance are pending.
