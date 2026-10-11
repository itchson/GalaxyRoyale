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

## Current detailed-cell production standard

New assets follow [the exact asset pipeline](ASSET_PIPELINE.md) and [the asset brief](ASSET_BRIEF.md). The historical generated-sheet recipes above describe their inspected inputs; they do not authorize silent resizing, alpha changes or component removal for future art.

Keep the same faction/category palette and silhouette language across ships, facilities, enemies and UI. Build detail from a readable silhouette, a few directional material planes, then restrained panels, vents, fasteners and lamps. Dark crater floors are occupied material on intact rocks; landing apertures and authored hull gaps are intentional transparent cells. Pixel count alone does not establish visual quality. Inspect normal gameplay size, both zoom limits and biome contrast before accepting detail.

The current four procedural rock families have connected opaque interiors and deterministic geometric/color variation. Haven facilities and action icons use original finer cells within their existing footprints; native software contact sheets and actual browser captures are separate evidence. The text-only flight footer uses the smaller original RoyaleMicro cell font; attached beveled hull/shield chunks animate while retaining immediate health truth. Compact-viewport human readability remains an explicit review gate.

## Fifteen ordinary bosses

`assets/pixels/boss-source.py` authors fifteen distinct 64-unit hull designs and five Coil modules. It writes `boss-atlas.json`, measured `boss-budgets.json`, and the optional Pillow contact sheet `boss-preview.png`. These are original authored cells rather than image-model output. Each head has its own silhouette, armor panel layout, actual transparent openings and occupied reactor; later tiers add forked wings, citadel bays or annular/gill shapes. Native sizes are 64, 72, 80, 88 and 96 cells. The common 64-entry hard-alpha palette retains magenta Dart, amber Bastion and jade Coil recognition, with layered metal shading, rails, vents and firing hardware.

| Family | Generation 1 | Generation 2 | Generation 3 | Generation 4 | Generation 5 |
| --- | ---: | ---: | ---: | ---: | ---: |
| Dart head | 1,760 | 2,318 | 3,251 | 3,779 | 5,563 |
| Bastion head | 2,664 | 3,247 | 4,561 | 5,224 | 6,161 |
| Coil head | 1,990 | 2,755 | 2,886 | 3,949 | 5,633 |
| Coil module | 84 | 168 | 152 | 312 | 400 |

`boss_atlas.rigs` provides a row-major per-cell part index, an earlier-parent translation hierarchy, and an occupied local core position/radius. The shared engine `PixelRig` partitions the current damaged body into seven nested hull/armor/collar/lens/emitter objects. It derives each part's local cells from surviving occupancy and composes their exact affine placements into the actor hierarchy, including shear. No pristine child template restores removed cells. Parts remain rigid with the collision hull; color and emission animate. Independently moving collision limbs would require a separate hitbox assembly change.

The actual design review with gameplay agreed on telegraph, attack and recovery phases: charge brightens the emitter objects and recovery opens a brighter pulsing reactor. The rig supports flashing surviving cells, while current boss hits use debris, hit events and presentation shake. The gameplay module owns localized recovery-core bonus damage, generation thresholds, movement and projectile mechanics. Maximum engine material emission is 0.6 for the core and 0.5 for emitters. Source colors and occupancy remain unchanged. Reduced motion freezes cosmetic presentation time while preserving charging and open-core phase cues.

The deepest Coil composition is 5,633 head cells plus nineteen 400-cell modules: 13,233 quads and 45 transforms, fitting one 128-transform batch before other scene objects. This is a per-boss budget; total gameplay cost also includes other enemies, bullets, particles, background, lights and UI. Whole-atlas native tests verify normalized silhouette differences, every role's occupied cells, local core bounds, damage holes, exact positions under nested nonuniform shear, dimensions 1 and 128, and the 16-part engine ceiling.

Reproduce the art with `python assets/pixels/boss-source.py` and proofs with `cargo test -p engine_core` and `cargo run --release -p engine_core --example boss_rig_proof`. The measured development-workstation report is `assets/pixels/boss-native-proof.json`: 500 warmed samples per scene. The deepest Coil's native CPU extraction measured about 0.205 ms median and 0.209 ms p95 in that run. Those figures exclude simulation, GPU work, browser/WASM overhead and physical phones. The contact sheet has been visually reviewed as development evidence; integrated browser and device acceptance remain separate.

## Approved Kikai detail benchmark

The user approved the generated DORA expression reference in `assets/concepts/kikai/dora-reference-expression-sheet.png` as the project's desired detail and quality. Use coherent color clusters, fine mechanical planes, readable silhouettes and restrained localized highlights, with independently authored animation parts. `KIKAI_ART.md` declares the reference-to-cell conversion and actual 96×128 native portrait proof. Preserve those occupied cells at fractional scale instead of silently replacing them with a coarse silhouette. Final raster coverage still depends on viewport resolution.

PAN uses green, RENKO red and HUGO blue for main-manufacturer hair, home accents and starter hull/jet themes. Crystal horns identify the independent sub-manufacturer. OPR#1337 is PAN with RENKO horns: green long hair and red crystals. Arena sponsor branding is independent of a competitor's manufacturers. Corporation colors do not replace the offensive/defensive/passive loot readability rules.

Global asset refinement is recorded under task-ef6ed92a; the native portrait does not imply every historical asset has already reached this benchmark. Detailed shooting-star head and coherent bounded emissive particles are task-72663b49. Inspect actual runtime size, parallax, both zoom limits and low-density presentation before accepting each asset.
