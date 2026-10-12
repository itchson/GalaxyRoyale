# Compact HUD, detailed controls and earned shields

Tested integration: `f69d6cbe8d58d182da6e45460b42b963655c5fbc`, branch `codex/testing`.
The exact served dual-WASM build is pinned in `tested-preview-f69.json` and source inputs in `source-inputs.json`. Later specialist work is separate. These records describe actual checks, not a claim that all production milestones or human art acceptance are complete.

## Implementation and independent review

- Native 64-cell authored controls, semantic palette masks, bounded atomic admission and fractional quad transforms. The atlas is original authored pixel work, not a generated-image conversion. Masked emitter brightness is unlit; no bloom or dynamic-light claim is made.
- Top-left compact controls; native top-right minimap with right-aligned telemetry beneath. Measured toolbar, context/touch hit targets, safe insets and boss reservations are considered together.
- Curved player hull/shield tracks retain their bounded footprint. Standard 30 HP shield has three 10 HP chunks; real higher capacities produce more chunks and a partial final chunk.
- Recharge is locked for the standard starter. Aegis rank 1 or Guard Coil rank 2 unlocks delayed recharge; eligible ranks and RAICHO levels improve its rate. HUGO starts with its legitimate Guard Coil capacity, not the standard three chunks.
- One compatible owned item per offense/defense/passive slot, with Haven swaps. Manufacturer affinity follows the active RAICHO's main corporation: PAN passive, RENKO offense, HUGO defense. It is independent of the pilot, sponsor and sub-manufacturer on collectible hulls.
- Eight stable champion encounters now have authored KIKAI identities and independent sponsors. Unknown content remains gated. Authored roster approval and natural campaign playthrough remain pending.

Handoffs and independent reviews are archived alongside this file. `affinity-negative.log` preserves the initial independent public-Game probe (2 pass, 4 fail); `affinity-corrected.log` reruns that unchanged probe after the two-file repair (6 pass). The original full unit suite had missed this cross-pilot/hull gap. `native-budget-negative.log` preserves the initial integration failures; corrected tests keep both a legitimate five-action portrait menu and an intentionally rejected over-budget scene.

## Actual validation

| Check | Observed result | Scope |
| --- | --- | --- |
| Rust workspace | 578 pass | 37 engine + 3 external + 314 core + 193 web + 31 layout |
| Browser-shell/studio tests | 351 pass, 1 skip | 352 total; fake DOM is not browser rendering |
| Strict native workspace Clippy | Pass | All targets at tested integration |
| Normal production build | Pass | Both WASM backends, game, studio and 19 PWA assets |
| Rust checkpoint/IndexedDB contract | 42 pass | 21 per backend; supplied fixtures, headless fake IndexedDB |
| Detailed control asset tests | 11 pass | Authored source/atlas checks |
| Portrait asset portability | 15 pass | Decoded RGBA and fail-closed source validation |
| Official GitHub portability repair | Success | Run 38122925138 at 5658bc3; later integration CI was not observed yet |

The desktop browser used an isolated disposable origin, `corner-shield.localhost`, with a newly issued DORA/PAN/RENKO license. User-origin saves were not changed. QA WARP/SCENE commands suspend rewards: these captures are not evidence of naturally earned progression. The test profile retained zero currency, zero carried materials and level 1. Reload returned to the dock and retained its license. Actual retry returned to flight without automatically opening upgrades.

## Desktop renderer observations

- `renderer-webgl2.png` and `renderer-webgpu.png`: actual native console identifies each backend.
- `flight-webgl2-1280.png`, `flight-webgl2-848.png`, `flight-webgl2-320.png`: top-left controls, right navigation and three starter segments at desktop viewport sizes 1280×720, 848×390 and 320×180.
- `status-locked-webgl2.png`: actual 30/30 shield, locked recharge, one passive Flux Drive slot and empty offense/defense slots.
- `settings-settled-webgl2-848.png` and `settings-webgpu-848.png`: settled native settings layout. The earlier `settings-webgl2-848.png` is an immediate post-resize transitional frame and is not used as accepted settled-layout evidence. Scrollable content continues below the visible viewport.
- `combat-webgpu-1280.png`: real fixture combat depletes shield and damages occupied body cells; the native shield warning appears.
- `boss-webgpu-320.png`: first Coil boss and warning widget fit the compact desktop HUD without overlapping the measured toolbar/navigation regions.

At 848×390 and 320×180, actual desktop header buttons measured 28×28 CSS pixels at x=12/43/74/105, y=12. Right telemetry measured `[662,54,180,22]` and `[134,54,180,22]` respectively. These are desktop viewport tests, not physical touch-device tests. Actual coarse-pointer CSS, simultaneous physical-phone input, a 30-minute performance soak, upgraded-shield browser play and human art/audio acceptance remain pending. Reduced motion was checked through the real settings control on this QA profile. The temporary viewport override was reset afterward.
