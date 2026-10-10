# Galaxy Royale art direction

The direction reference in `artifacts/concepts/galaxy-royale-direction.png` was generated with the session image tool. Runtime ships, asteroids and dock use the separately generated sprite sheet, converted into editable local pixel cells in `assets/pixels/atlas.json`. Rust expands those indexed cells into PixelBodies, with individual quad rendering, object transforms, localized damage and cell animation.

The visual palette uses near-black navy, an ivory and cyan player hull, cyan dock guidance, magenta Dart threats, amber Bastion armor and jade Coil bodies. Offensive points are coral, defensive cyan and passive gold. Different silhouettes and bright enemy shots give encounters readable roles against the subdued space background. North-facing source ships align with movement through the actor heading plus a quarter-turn transform.

Stars use layers with their own parallax and brightness. Authored nebula cells form coarse shapes and leave darker space around combat. Directional exhaust, restrained debris, impact flashes, localized hull holes, boss telegraphs and brief camera shake support the action. Cosmetic density and reduced motion change presentation while preserving simulation.

## Generated sprite cells

`scripts/convert-atlas.py` crops four source columns at the explicit row boundary y=410, retains the largest 8-neighbor alpha component, fits its tight bounding box with nearest sampling and packs a common palette. The palette contains index zero for transparent cells and 63 opaque colors; there is no dithering. Colors are little-endian RGBA u32 values. Runtime asteroid mineral variants preserve occupancy.

| Sprite | Frame | Occupied quads |
| --- | --- | ---: |
| Player | 48 × 48 | 931 |
| Dart | 32 × 32 | 426 |
| Bastion | 40 × 40 | 938 |
| Coil head | 32 × 32 | 584 |
| Coil segment | 16 × 16 | 87 |
| Asteroid | 40 × 40 | 1,067 |
| Dock | 64 × 64 | 1,454 |
| Dart boss | 64 × 64 | 2,045 |

Total occupancy across the eight source bodies is 7,532 cells. Actual scene cost depends on spawned bodies, visibility, particles and backgrounds. The dock center and crater gaps remain transparent. The small Coil module bounds repeated segment cost.

## Generated interface cells

`artifacts/concepts/galaxy-royale-ui-atlas.png` is the RGBA source for `assets/ui/atlas.json`. Its logo, panel, button and seal have separate explicit crop regions; the button crosses the source midpoint. `scripts/convert-ui-atlas.py` keeps all thresholded components, including separated logo characters, star and ornaments. It uses hard alpha at 128, tight aspect-preserving fit, nearest sampling and a shared 64-entry palette. `assets/ui/atlas-preview.png` magnifies every output cell eight times.

| Interface asset | Frame | Occupied quads |
| --- | --- | ---: |
| Logo | 256 × 104 | 15,528 |
| Panel | 96 × 48 | 3,475 |
| Button | 96 × 20 | 1,417 |
| Seal | 40 × 40 | 804 |

The complete interface asset set contains 21,224 occupied cells. Engine rendering owns its pixel visuals. Semantic DOM controls retain input, keyboard focus and accessible names, with a bridge supplying layout and state to Rust. Integrated browser checks remain necessary for visible labels, hit targets, scrolling and control states.

The local Tiny5 font and its supplied SIL Open Font License 1.1 are retained at `apps/game/public/fonts/Tiny5.ttf` and `OFL.txt`. UI provenance records both file hashes. Generated source provenance records the session tool, actual source hashes and conversion settings; the specific image model identifier was not exposed in the artifact metadata.

RoyaleMono is original 5×7 cell artwork defined in `crates/game_web/src/pixel_ui.rs`. The semantic browser font `apps/game/public/fonts/RoyaleMono.ttf` is generated from those exact masks by `scripts/build-ui-font.py` using fontTools (build tooling only). It uses the same six-cell advance and uppercase masks, so DOM wrapping and Rust quad glyph placement agree. The shipped visual letters remain individually transformed occupied pixel objects. Campaign hulls, insignia and additional portraits are original authored cells; see `assets/ui/CAMPAIGN.md` for their reproducible source and atlas contract.

## Verification and production evidence

Conversion checks verify exact dimensions, row-major indices, hard alpha, common palettes, crop boundaries, sampled silhouettes and byte-identical reruns. The UI check additionally verifies that disconnected components survive. Evidence is recorded in `work/ui-atlas-tests.log`; source and output hashes are in `artifacts/asset-provenance.json` and `assets/ui/provenance.json`. Script help contains reproducible Python/Pillow commands.

Physical phone performance and final visual acceptance remain separate checks. Audio uses an authored chip/synth score and synthesized cues; optional offline music generation is a later asset experiment.
