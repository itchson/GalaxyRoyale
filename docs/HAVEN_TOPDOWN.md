# Haven overhead art

The center dock and four Haven facilities use flat overhead decks. The facilities replace the previous upright facades and vertically offset diamond pads with cached 44 × 44 object-local `PixelBody` cells. Roofs and flush perimeter rails share an isotropic 0.55 world scale and the exact facility center, without elevation offsets.

Forge has an octagonal fabrication well and four cardinal work clamps. Hangar has a chamfered cyan service roof, open landing aperture and a shuttle seen from above. Journal has four green archive pods around survey glass. Arenas has a gold annular track and cardinal scoring decks. Single-cell bevels, vents, fasteners and recessed lanes supply detail without depicting upright walls or towers.

The center dock retains the exact existing 64 × 64, 1,454-cell occupied mask, open bay and conservative collision radius. Its existing cells receive planar roof seams, radial berths, flush sockets and guidance lanes. The generated atlas PNG/JSON and historical asset-pipeline example remain unchanged; this is a Rust cell-color treatment of that accepted mask.

Facility interaction centers remain `[-120,-80]`, `[120,-80]`, `[-120,95]`, `[120,95]`, with the existing 24-world-unit hit radius and home gate. This change does not add simulation, progression, save fields or moving collision parts.

Signals animate their colors relative to local roof cells; arena signals use angular phase around the track. The dock reuses the host's existing local signal wave. Both follow the existing cosmetic clock, so Reduced Motion freezes the colors and geometry. Roofs remain stationary.

| Object | Submitted occupied cells | Transforms | Maximum actual world corner radius |
| --- | ---: | ---: | ---: |
| Center dock | 1,454 | 1 | 48.211 |
| Forge | 1,584 | 2 | 13.120 |
| Hangar | 1,702 | 2 | 14.645 |
| Journal | 1,404 | 2 | 12.876 |
| Arenas | 1,284 | 2 | 12.542 |

The five visible structures submit 7,428 cells across nine transforms. These counts include each facility's roof and rail. All facility corners fit inside the unchanged interaction radius. The dock's measured corner radius above is distinct from the engine's conservative collision radius; the native regression verifies that conservative radius is unchanged.

## Actual verification

The source change is commit `773fc6c2092e06a038eda0175852ba941569e2ab`, based on `6152611ec88653946f809f200a24a0de482cf265`. All 231 `game_core` native tests and strict all-target Clippy passed. The tests verify overall silhouette distinction, cardinal roof symmetry, isotropic centered transforms, exact dock-mask preservation, finite bounded extraction, live signal changes and Reduced Motion behavior.

The separate [native proof](../artifacts/haven-topdown/proof.rs) creates a real default `Game`, extracts actual render packets before and after 60 fixed ticks, then verifies another 60 ticks with Reduced Motion. Every structure has changed signal colors with unchanged corners/counts; the persisted profile stays unchanged. The [metrics](../artifacts/haven-topdown/metrics.json), [evidence](../artifacts/haven-topdown/evidence.json) and native logs record that scope.

The [contact sheet](../artifacts/haven-topdown/contact-sheet.png) magnifies the center dock and facilities separately, so it is not a common world scale. The [world overview](../artifacts/haven-topdown/world-overview.png) uses the real extracted positions at a common two pixels per world unit. Both are flat software rasterizations of native packets, not GPU or browser captures. They are separate from the historical generated-dock conversion contact sheet.

To reproduce from this source checkout with the Rust toolchain and Python 3.12/Pillow 12.3.0:

```powershell
powershell -NoProfile -File artifacts/haven-topdown/run.ps1
python artifacts/haven-topdown/render.py work/haven-topdown-repro
```

The runner writes a new executable, raw packet JSON and images under ignored `work/haven-topdown-repro`; it does not overwrite this frozen evidence. Browser material/lighting, minimum-zoom readability, human evaluation and physical-phone acceptance remain pending integration.
