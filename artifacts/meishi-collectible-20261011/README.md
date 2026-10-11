# Collectible MEISHI card — 11 October 2026

Implemented on `codex/testing`; final browser package is identified by `tested-build.json` (bdecb3d, source hash 9665aaa536d8644bf5531c83efaa322d3e5e36cd7db1074bc10cd4e6dd5bba67). Later documentation/evidence-only commits can carry the same runtime/package hashes. This is bounded card acceptance, not whole-game release approval.

## Delivered

Native cached packed-pixel fittings form a main-manufacturer card: PAN forest/green, RENKO maroon/red, HUGO navy/blue; neutral platinum corners, layered enamel rails, separate corporate seals, animated existing KIKAI portraits, current RAICHO/abilities and retained point balances. Exactly eight recessed numbered sockets show earned patches only. Unknown champion identity stays hidden. Close is the only control. A reserved strip keeps scrolled text clear of its target.

The built-in image generator produced the [PAN art reference](../concepts/meishi-collectible-20261011/pan-collectible-reference.png), with [prompt](../concepts/meishi-collectible-20261011/PROMPT.txt) and [provenance](../concepts/meishi-collectible-20261011/provenance.json). Runtime fittings are separately authored native assets adapting that direction; generated text, portrait variations and balances are not canon or automatically converted runtime art.

## Actual validation

- `shell-tests.log`: 416 pass, one existing Windows symlink skip (417 total).
- `native-tests.log`: all 216 game_web library tests pass. `clippy.log`: strict native library/test Clippy passes. `typecheck.log`: TypeScript check passes. `normal-build.log`: normal production build produces both WASM renderer packages, game and Studio.
- Independent reviewer executed 33 focused copied TypeScript/DOM tests and inspected native evidence; see `independent-review.md` and the separate `close-clearance-review.md`.
- The native nine-profile/all-earned fixture compares joined and isolated emitted components. Worst fixture is 45,367/60,000 submitted cells; no quota increase. This is a representative native fixture, not a measured full live DOM/HUD packet. Continuous 22.5-logical-unit fit preserves occupied patch cells. Rejected socket budgets suppress their paired later emblems. Reduced Motion freezes decorative color animation.
- Root operated one QA browser tab, sequentially using isolated profiles. Actual console `renderer` reports confirm WebGPU and WebGL2. PAN final-package captures cover 1280x720, 480x700, 848x390, 601x560 and 320x568. `layout-measurements.json` records eight wells, Close-only controls, no horizontal overflow, and at least seven CSS pixels between Close bottom and content top. Smaller portrait windows scroll internally; Control+End reaches all eight sockets at 320x568. Escape closes to flight. The PWA Apply update at Haven flow preserved the existing QA identity/starter state through two actual updates.
- `pan-webgpu-1280.png`, `pan-webgl2-1280.png` and the smaller PAN captures show the final package. `hugo-webgl2-1280.png` and `renko-webgpu-1280.png` show the preceding ddd82b1 package, recorded in `initial-native-build.json`; only the later Close-clearance CSS differs. Their independently selected blue/green and red/blue hair/horn combinations verify main-manufacturer card colors do not follow horn color. No patches were awarded or saves injected to produce these screenshots.
- Spark completed a read-only layout audit via the installed wrapper with danger-full-access/never inherited explicitly. Its static top-row calculation omitted real insets/seal placement, but Root reproduced scrolled text bounding boxes entering the Close target region and implemented the reserved strip. Retained `spark-layout-review.md` is raw reviewer output, not blanket acceptance of every inferred number.
- Final captured browser warning/error list is empty. Physical phone testing, all-eight-earned browser progression, subjective human readability approval and a 30-minute performance soak were not performed in this card pass.

Only the owned game preview was replaced; the QA tab was closed and viewport reset. The user's Studio tab remains open. `manifest.json` pins this evidence archive as bytes; source pins identify the runtime files without duplicating the source tree.
