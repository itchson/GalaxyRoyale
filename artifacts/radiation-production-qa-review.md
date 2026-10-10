# Independent actual-core Isotope Mote QA

Execution/source review 2026-10-10 19:56–20:04 UTC; evidence frozen at 20:06 UTC. **Accepted for bounded integration of the final frozen review repair.** Only reviewer-owned `work/radiation-production-qa/` files were changed. This is an isolated actual `Game` candidate; it is not merged production, browser, GPU, phone or natural human progression acceptance.

## Candidate and scope

The feature is one ordinary family-0 enemy in Radial Wastes, not environmental radiation/DOT, heat/cooling, a new damage type or save field. It reuses the real ordinary Dart actor, shared projectile/collision/sanctuary/reward paths, indexed destructible body and the existing first-boss credit. Its explicit `biome-mote` QA scene suppresses rewards and tracked run history.

The final complete core copy remains SHA256 `f65a4a4ea80126325ff88343a65f45a011dd368cb9d656b02a72b35ce2151164`. The corrected adapter/controller are separate linked files, so that unchanged core-file hash does not identify V1 behavior by itself. The final authoritative source set is Gameplay's `frozen-hashes.json`, SHA256 `8b2a1180863dfe9106576c5617a6fff4d2daaea898c98a774e7be8b854b260ed`. Its candidate library is `a46a1821641aaa1c68deb7eeb4805838129abf717bce2a863d3c04dadb6bdcdd`.

The core baseline intentionally remains `0a574a35c9d23f955fff05246da2cf4bf9a8065f5db58c63c365124f982337e6`, preceding root's separately reviewed shared early-death helper. This proof cannot substitute for testing the merged implementation. Current local production changes are neither reverted nor included implicitly.

## Concrete findings and repairs

Two actual issues were found and sent to Gameplay/root in a real asynchronous source/test exchange:

1. **F32 phase timing mismatch.** V1 emitted its first full pulse on charge tick 67 and its second on tick 87: 20 ticks / 0.3333 seconds apart. The charge boundary used bare `age >= 1.1`, while the later pulse used an epsilon. Gameplay corrected boundary normalization; the final independent run observes ticks 66 and 87, exactly 21 ticks / 0.35 seconds apart at 60 Hz.
2. **Explicit stress exclusion missing.** The accepted integration plan excludes stress variants, but V1's installer omitted `stress_mode`. In a declared low-count stress fixture, the actual ordinary spawn hook installed a Mote on its first Dart tick. Gameplay added the explicit guard. The final independent test exercises 128 actual ordinary spawn ticks, including Darts, and installs no Mote in stress mode.

Negative evidence is retained as `v1-cadence-execution.log`, `v1-stress-policy.log` and `v1-independent-tests.exe`. Gameplay preserves the exact V1 sources under its `v1/` directory and the intermediate timing-only revision under `v2-timing/`. The initial reviewer capacity fixture also armed one tick too early; that fixture error was corrected using a separate identical actual-Game encounter to locate the due tick. It is not counted as a runtime defect.

## Independent final execution

`build.ps1` verifies the frozen candidate/source hashes, copies its actual core into the owned directory, appends only the owned crate-child test module and compiles against the same existing dependency artifacts. It does not change a controller algorithm. Private scene/geometry fixtures explicitly remove unrelated waves/rocks, expose the real ECS state or reserve existing bullet slots; no direct helper kills, damage or reward grants are used. Attacks, actual player projectiles, movement, collision and rewards still run through `Game::tick`.

The final owned executable ran **8/8 new tests successfully, with 98 existing tests filtered**. Full output is `execution.log`; derived observations and fingerprints are `results.json` and `hashes.json`.

| Actual independent probe | Observed result |
| --- | --- |
| Full charge/pulse/recovery cycle | First two pulses on charge ticks 66/87; six actual hostile rays each; no third pulse before the next charge at tick 361. Recovery begins at tick 96. All old rays are gone by tick 327. |
| Indicated gap and stationary Scout | Ray centers lie about 80/120/160 degrees from the locked gap, outside the 120-degree wedge plus its 20-degree buffer; actual velocities are 110 world units/s. A stationary fresh Scout receives no hit/death and retains full hull through this measured cycle. |
| Final charge lock | An explicitly declared 8-world-unit player displacement during the final charge retains the exact locked gap, completes a real attack and does not silently abort into Waiting. |
| Pause/invalid/large time | Paused and zero/negative/NaN/infinite ticks leave actual controller age, pose, clock and profile unchanged with no shots. A 20-second positive input advances only the existing 0.05-second clamped tick. |
| Combined primary/Mote shared cap | A real primary takes the last remaining slot at the due pulse; total bullets stay 180. The dropped first pulse is consumed, never replayed. Only the second scheduled pulse uses freed capacity 21 ticks later. |
| No-fit then minimum supported geometry | At an explicit unsupported 120×80 world viewport, 900 ticks stay nonfiring. After restoring 320×180 UI / 1.6 zoom world extents, the actual enemy approaches/settles in 671 ticks and then shows a fresh 66-tick charge before firing. This is a declared geometry fixture, not a human pacing judgement. |
| Diagnostic defeat, retreat, reload | Real player projectiles defeat the diagnostic Mote in 21 ticks. Full saved profile remains exactly equal through victory, Haven reset, remote retreat and `Game::new` reload; live enemy/controller state is not persisted. |
| Ordinary rewards and first-boss threshold | Two actual player-projectile defeats in a declared non-QA reward fixture take 21/24 ticks. Kills 22→23→24, selected XP 1320→1325→1330, an actually accepted seeded bounty 0→1→2, and G1 pending becomes true only at 24. Uncollected drops grant no direct currency; unrelated generation counters/material stash remain unchanged. Dock/reset/Haven ticks and reload preserve the final checkpoint exactly. |
| Stress policy | 128 actual low-count stress-mode ordinary spawn ticks, including Dart-family spawns, install no variant after the final guard repair. |

The normal-reward fixture uses `biome-mote` to install the actual entity and then explicitly enables the ordinary reward route. It is not a claim that the enemy was naturally encountered by a player. Its declared starting XP, hardware ranks and banked stock are test inputs; the five XP per defeat and pending/bounty increments are produced only by real projectile defeats. Independent stress coverage does use the actual ordinary spawn hook.

## Actual browser-save boundary, without a browser

The ordinary fixture writes its actual Rust profile to `ordinary-checkpoint.json`. `checkpoint-boundary.mjs` imports the current real TypeScript `validProfile`, `normalizeProfile` and `SaveStore`, and uses fake IndexedDB. The final execution passes strict validation, exact normalization, exact persistence/reload, and ignores an equal-revision attempted duplicate currency/XP gain. The retained checkpoint has revision 8, kills `[24,0,0]`, selected XP 1330, G1 pending `[true,false,false]`, currency `[7,8,9]` and bounty progress 2. See `checkpoint-boundary.json` and its actual log.

This is native JSON → current TypeScript save code, not the WASM serializer, browser IndexedDB, offline lifecycle or device evidence. It confirms the enemy introduces no new persisted schema requirement; root still owns normal dual-WASM serialization/build acceptance after integration.

## Read-only integration assessment

The adapter snapshots real player refuge, current/predicted follow camera, live viewport and shared remaining capacity before mutable ECS iteration. It dispatches shots once through the existing bullet path and reserves commands across entities. Generic enemy AI skips the Mote component, while shared physics still integrates once. Sanctuary enforcement and lifetime swept projectile protection remain shared. Invalid controller inputs retire without rewards. Birth identity is immutable and based on actual spawn biome/grid/ticket, with no additional loot RNG draw.

Rendering reuses the live damaged body, actual interpolated transform and phase-dependent pixels; it does not swap in a pristine raster sprite. The controller's warning fits world viewport geometry. That geometric fact does not establish GPU presentation or freedom from semantic HUD/touch-control occlusion; actual visible captures must inspect the full cue at supported zooms and small landscape sizes.

Gameplay's own 16 actual-core tests and strict native library/test Clippy passes were reported and source-read, not independently rerun or added to this reviewer's eight passes. Their moving-camera, destructible emitter, hostile-impact/reflection and multi-enemy-cap coverage remains explicitly specialist-owned evidence.

## Reproduce and remaining gates

With the preserved frozen work candidate and its identified existing dependency artifacts:

```powershell
./work/radiation-production-qa/build.ps1
$env:GALAXY_RADIATION_QA_CHECKPOINT = 'work/radiation-production-qa/ordinary-checkpoint.json'
./work/radiation-production-qa/independent-tests.exe radiation_independent:: --nocapture
Remove-Item Env:GALAXY_RADIATION_QA_CHECKPOINT
node work/radiation-production-qa/checkpoint-boundary.mjs
```

No blocking issue remains in the measured final candidate scope. Root owns merged-source native/WASM/strict checks, normal dual-WASM output and test-export exclusion, source/build correspondence, visible WebGPU/WebGL2 cue/material/control checks and publication. Full naturally reached Scout progression/enemy balance, human evasive play, physical-phone input/readability/lifecycle/60 FPS/soak and authenticated remote acceptance remain pending. Neither native tick timing nor an 8-test wall-clock duration is a game frame-rate benchmark.
