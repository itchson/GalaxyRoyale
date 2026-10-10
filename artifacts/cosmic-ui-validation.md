# Dark cosmic space and responsive pixel menus

The Rust/WASM engine now draws a black space base with bounded, seeded distant pixel galaxies, nebula filaments and nova remnants. Four star layers vary spatial density and tint without rerolling when the camera moves. Cached occupied cells retain their own nested transforms; reduced-motion mode freezes cosmetic animation. Existing biome planets and region scenery remain composed alongside these new motifs. Lighting applies a small bounded biome grade only to responsive world materials; UI and unlit scenery retain their authored colors.

The command menu has six dedicated pages: Upgrades, Hangar, Campaign, Journal, Settings and Save. Fixed navigation and primary actions surround a clipped scrolling page. Selected tabs differ from hover/focus; mint primary actions, coral offensive, cyan defensive, green passive and subdued red exit controls guide navigation. Save export/restore reside in the Save page, and Restore is a real keyboard-operable button. Keyboard arrows/Home/End move between tabs. Narrow portrait navigation uses two columns; very short landscapes use a horizontally scrolling strip with the selected tab brought into view.

Native HUD painting and boss reservations consume the same allocation-free layout. Narrow touch layouts move or reduce secondary telemetry and keep hull/shield meters clear of input controls. At logical widths180/181, the joystick hit rectangle narrows by2/1 units and its drawn center follows that exact rectangle. Normal widths182+ retain exact native control packets across540 independent comparisons. Actual shared HUD, cell-corner, inset and reservation tests cover384 viewport/touch/minimal/far-camera cases. A genuine180×140 touch viewport can have no safe100×40 boss-panel position; that geometry still fails closed instead of overlapping critical controls.

## Final automated checks

- `cargo test --workspace`:282 passed (37 engine,140 gameplay,105 web).
- `npm test`:89 passed.
- Strict native workspace/all-target Clippy and both `wasm32-unknown-unknown` backend Clippy checks pass.
- `npm run build`:normal WebGPU/WebGL2 WASM, strict TypeScript, game/PWA and independent Studio builds pass.
- Final normal cache:`galaxy-95d0a4d39116971e`,18 cached game assets. Earlier initial/menu-only caches are interim component evidence.

The independent cosmic/shader review executes actual affine-corner, transform-pool, fixed-tick preservation and Naga uniform-layout checks. The CPU profile uses actual public Game extraction with frozen simulation snapshots:86,400 warm frames across48 fixtures and three processes. Max observed cold extraction2.2106ms; highest fixture warmP95/P99.3394/.3654ms. Those figures exclude GPU, browser UI and advancing simulation. They are not FPS or phone acceptance. Ordinary large region planets can be omitted under the existing reduced cosmetic budget; cosmetic density remains bounded and independent of gameplay.

## Evidence and remaining gates

See cosmic-ui-engine-review.md, cosmic-background-proof.md, cosmic-background-path-review.md, cosmic-render-profile.md, cosmic-ui-hud-review.md and cosmic-ui-input-edge-review.md for actual specialist contributions, test commands, source fingerprints and limits. The input-edge addendum supersedes only the earlier two-pixel touch-hit overlap. The installed local HUD worker was interrupted without edits/tests; responsive native code was completed and checked by the Engine specialist. Its recorded state is interrupted, never completed.

Final current-package browser observations, independent package review, exact committed archive and private publication are recorded separately after they exist. Physical Pixel6a/iPhone12 testing, real simultaneous touch/safe insets, sustained GPU frame-time/memory and30-minute soak, natural campaign balance, human audio review, broad narrow moving Mote readability, full context-recovery/PWA matrix and authenticated remote gameplay remain pending. The broader production slice remains in Review.
