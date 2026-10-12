# Native PAN material registration and isolated relighting scene

Frozen source: `a07919178e17ecd7e27e7316d451d3db0f4df690`, parent `817f99ba95daea4a9cad299383425c75fe535ae8`, branch `codex/pan-material-render`. Only nine `game_core` source/test files changed. Root owns the matching shader, browser boundary, shared assets, integration, builds, listeners, Studio records, and actual renderer review.

## Runtime contract

- The cached PAN HIKARI asset consumes the already committed native-encoded `assets/pixels/pan-surface/material.json`. It validates version, asset, encoding, exact approved source SHA metadata and computed FNV-1a64 of the included atlas bytes, exact complete ordered coarse/fine occupied indices, enabled Surface words with zero low-seven transform-selector bits, and original signal masks. FNV is a deterministic byte binding, not a cryptographic integrity claim. No new crypto dependency.
- The immutable map adds 4 bytes per field cell: a 2,304-word coarse index map and 1,205-word aligned fine map, totaling 14,036 payload bytes. It is allocated once through the existing LazyLock asset cache. Normal fields are decoded from native Surface words; no per-frame authored-float re-encoding or metadata cloning.
- Existing RGBA, retained 48-cell hull, 96-cell detail, occupancy, collision, damage filtering, transform composition, 3,000-cell object budget, two-transform admission, animation callbacks, and alpha behavior remain in use. Coarse holes remove all corresponding fine cells. The intact PAN pair submits 662 coarse plus 1,205 fine cells. Coarse masks are zero; exactly the original 31 fine signal cells have mask 255. The mapped fine group uses emission 0.35; this becomes actual signal-only emission through the linear mask. Other assets retain legacy words/materials.
- Actual native co-op render reconstructs the same immutable PAN asset only for a matching PAN identity, starter ship 0, canonical ship scale, and valid retained body. Existing body pages/signatures/occupancy remain unchanged. Legacy raw `append_occupied` still emits raw retained bodies. The exact host/guest appearance test uses reduced motion: recent-hit timing is absent from guest metadata, so this change does not claim full guest damage-flash parity.

## QA scene and browser hooks

`Game::diagnostic(&str) -> bool` now reports acceptance. Existing calls with ignored semicolon results continue to compile; Root must preserve the bool through the WASM/browser shell and check it before reporting a successful Scene/WARP command. The native command is `material` (the shell's `scene material` maps to it).

The scene accepts only a living current PAN starter with the compiled map ready, no carried material/training cargo, no real active run, arena, active boss, or co-op. There is no required current Haven-radius predicate: already isolated empty QA warp locations are valid. Refusal restores transient diagnostic state and does not mutate the Profile, world, or pose. It does not register another Profile or upgrade abilities.

Accepted entry uses existing diagnostic isolation, resets to dock, clears other transient entities/visual pools, and places the real starter at (450, 0). Native fixed ticks run only the real movement/controller, confinement, time, interpolation, and camera. They skip world generation, enemies, weapons, abilities, rewards, exploration, and RNG. Exactly two modest warm/cool moving light inputs surround the ship; reduced motion freezes their position-relative motion. Existing HUD/STATS position, camera, and time expose the scene. Dock/Home, physical return to the dock radius, or another accepted scene clears the flag. It is never saved.

## Actual validation

- `cargo test -p game_core --lib`: **504 passed, 0 failed, 11 explicit exporters ignored** in `core-source-final.log`.
- Strict native `cargo clippy -p game_core --all-targets -- -D warnings`, strict WASM `cargo clippy -p game_core --target wasm32-unknown-unknown -- -D warnings`, and `cargo fmt --all -- --check` passed; logs retained.
- Thirteen fine-asset tests passed, covering exact source/mask/word binding, malformed metadata, repeat keys, actual projectile contact/damage, counts, callbacks, quota admission, nested/reflected/sheared affine transforms, and zoom endpoints. Three new material tests supplement the existing fine-asset coverage.
- Two new native replica tests perform real Game capture, stage, body import, complete commit, and actual view rendering, with exact PAN mapped-cell parity for two participants. The damaged case uses actual `hit_segment`, damage and reimport; raw body geometry remains exact and material detail follows the resulting holes.
- Three new scene tests cover 600 fixed ticks with held fire and exact Profile/RNG invariance, truthful seven-case refusal plus co-op refusal, real controller motion, reduced-motion lights, explicit scene switches, and physical dock exit.
- The explicit ignored exporter was actually run with `GALAXY_PAN_MATERIAL_EVIDENCE=work/pan-material-render/fixtures`. Its `native-material-scene.json` contains two actual native Game packets at tick 0 and 120: 1,867 material cells and two lights each, one world entity, exact unchanged Profile, logical viewport 640 by 360. This is cell/light input evidence, not a screenshot or GPU proof.

Earlier failures are retained: the first focused test still expected zero PAN fine-group emission; a temporary focused compile ran before the new scene test file existed; and the first full run exposed two old frame-selection helpers that selected only zero emission. The helpers now select 0.35 only for mapped PAN while preserving the old substantive count/damage assertions. The final full run and strict checks passed after those corrections. No failure logs were replaced with passing claims.

## Remaining integration and review

Root owns matching browser bool handling and console help, cherry-pick/integration, real WebGPU and WebGL2 relighting comparison, gameplay regressions, and any Studio updates. Independent QA has inspected the source and will verify the frozen raw/Git pins and exported map indices/masks. No browser, GPU, visual-quality acceptance, physical-device, performance, multiplayer RTC, or mixed-version compatibility claim is made by these native results.
