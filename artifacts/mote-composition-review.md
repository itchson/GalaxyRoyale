# Mote production composition review

**No concrete composition defect found.** Root's frozen composition preserves the accepted V2/review-repair1 Mote behavior and the separately accepted shared early-death helper. This review read production and wrote only this directory. It did not rebuild, execute the full production suite, alter the candidate, edit source, publish or claim new browser/phone acceptance.

Source was pinned after root's explicit freeze signal. Production core lib SHA256 is `ec2b3c897d4b5dc1ec70891d383f19a008fc0425a5dedd691c298cb41ddd3cea`; accepted isolated V2 manifest remains `8b2a1180863dfe9106576c5617a6fff4d2daaea898c98a774e7be8b854b260ed`. These are separate identities: the isolated candidate remains anchored to pre-death-helper baseline `0a574a35…337e6`, while this composition includes both changes.

`compare.py` verifies the accepted candidate's frozen artifacts, unchanged linked production modules and all four copied assets; it records production identities and auditable raw diffs under this directory. Run with the explicit frozen lib hash:

```powershell
python work/mote-integration-review/compare.py EC2B3C897D4B5DC1EC70891D383F19A008FC0425A5DEDD691C298CB41DDD3CEA
```

The command passed. `fingerprints.json` pins all reviewed sources, assets, Cargo.lock, the console entry point, durable death tests and root's inspected test log. Source differences were read in full:

| Production component | Difference from accepted V2 |
|---|---|
| `radiation_enemies.rs` | Production module comment only |
| `mote_controller.rs` | Production title and explicit pre/postguard alignment comment only |
| `mote_art.rs` | Relative production atlas include path and formatting only |
| `radiation_enemy_tests.rs` | Byte-identical; all16 accepted tests retained |
| Four authored assets | Byte-identical atlas, source, provenance and preview |
| `lib.rs` | Local production module imports, formatting, accepted shared death helper and durable test module |

There are no absolute workstation/work-candidate module imports in the production Mote sources. `include_str!("../../../assets/pixels/mote-atlas.json")` resolves from the new production art module to the copied authoritative atlas. Runtime still uses indexed cells; the PNG is not loaded as a render texture. Local `#[path]` attributes resolve controller/art/tests within the production source directory; the test module remains cfg(test).

The ordinary spawn hook still passes the actual spawned position's biome. Final stress/arena/QA/variant exclusions, immutable ticket and no-extra-RNG semantics are unchanged. `update_motes` remains after existing biome enemy dispatch and before generic AI/shared integration. Generic AI skips the optional Mote before incrementing age; the Mote updater increments it once. Its returned pose is integrated once, shared post-integration sanctuary enforcement remains afterward, and the actual all-lifetime bullet sweep/collision/reward paths have no Mote-specific bypass.

The controller retains f64 phase and pulse clocks and the pulse interval begins on its first scheduled tick. Per-Mote six-shot bounds, reservations across actors, the final shared180/300 projectile cap and consumed suppressed pulses are unchanged. Current/predicted camera intersection, physical clearance, shake reserve,0.125-world-unit corner inset, settle admission and warning-loss cancellation survived composition exactly.

The body extraction override still uses the **current damaged body**, interpolated position and shortest heading. Cue-aware culling includes the warning envelope; the authoritative actor scale and the same heading+PI/2 transform are used. Body material remains response0.75/emission0 and cue cells remain unlit. This source comparison preserves the earlier independent near/far packed-cell and live-emitter results; it does not convert them into new production GPU evidence.

The shared death helper runs immediately after paused/nonfinite/nonpositive-dt guards and before simulation time, control, regeneration, dock repair and primary fire. It returns after a forced dead-entry reset. Healthy ticks preserve normal simulation/collision ordering and call the same helper after camera follow. Therefore an eligible R5 repair or Sovereign eighth-shot heal cannot revive the already dead entry hull, and natural end-of-tick deaths retain the shared event/checkpoint path. The durable eight real-Game regressions are present alongside the16 Mote tests.

Console help and the existing QA-gated scene whitelist both include `biome-mote` in `apps/game/src/main.ts`. The actual diagnostic still delegates to the runtime, resets through the existing dock path and enables scene-wide reward isolation. There is no new save field, automatic grant or radiation background hazard.

Root's `artifacts/mote-integration-rust-tests.log` was inspected. It records **217 passed /0 failed** across37 engine,106 game-core and74 web tests, including all16 Mote and eight death tests. This is root-executed evidence, not a second test execution by this reviewer. Strict native/two-WASM checks and the release build were still root's next steps at the review freeze.

Remaining limitations are unchanged: the cleared stationary12-extent native fixtures all fire, but the declared moving8-second/max-zoom60Hz fixture produces zero emitting ticks; human encounter pacing remains Pending. Later unrelated collision/shake can invalidate the predicted camera, and full-canvas warning admission does not exclude overlaid HUD/touch widgets. Actual WebGPU/WebGL2 cue contrast, zoom easing, crowded impacts, reduced motion, save/reload and physical Pixel6a/iPhone12 frame-time acceptance still require their own observed production evidence. Neither this source review nor the preserved native tests assert those outcomes.
