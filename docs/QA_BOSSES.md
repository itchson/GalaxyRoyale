# Boss progression QA

This is an executable acceptance checklist, not a claim that every content or device check has passed. The independent storage review on 2026-10-10 UTC passed 22 tests, including six new generation cases. The independent native sweep completed all fifteen diagnostic bosses using actual projectiles, and eight checkpoint/save/reload scenarios passed through both real WASM backends. Browser choreography, human balance and physical phone acceptance remain separate.

## Progression and save contract

There are three ordinary enemy families (Dart, Bastion, Coil) with five boss generations each. `boss_generations` is a three-by-five matrix of unsigned 32-bit defeat counters. `boss_kills[family]` is the saturating sum of that family's row. The profile stays schema 3: missing matrices and all-zero legacy rows migrate their existing aggregate into generation one. A supplied nonzero row must match its aggregate and each promoted generation's defeats must be no greater than half its predecessor's defeats. Normalization clones the matrix without granting currency, ranks or revision increases.

Twenty-four ordinary kills reserve a generation-one encounter through `pending[family]`. Base kill surplus derives from generation-one defeats, not the aggregate of every generation. For generation `g` above one, the outstanding queue is `floor(defeats[g - 1] / 2) - defeats[g]`. Spawning an encounter does not consume that credit. A surviving active fight is included in that queue until defeated. Death, sanctuary disengagement, distance retreat and reload retain outstanding credit; live damage, projectiles and entities are transient.

One concrete checkpoint exercises all of these boundaries: Dart defeats `[16,8,4,2,0]`, aggregate 30, ordinary kills 453 and `pending=true` reserve one first-generation fight, retain 45 ordinary-kill surplus and queue one fifth-generation fight. Defeating G5 produces `[16,8,4,2,1]` and aggregate 31; ordinary-kill surplus remains 45. Retrying, saving the same revision or reloading must not add that defeat again. Native runtime reconstruction must prove the surplus calculation; the browser persistence tests prove it retains the source counters exactly.

Champion trials and the nine-entrant league use a separate campaign reward path and must never consume these ordinary queues. A losing league remains a completed loss; it does not grant Sovereign or credits. Returning home or dying in an actual league match records one loss, while reloading an unfinished match follows the documented solo checkpoint rule and leaves it unplayed.

## Reproducible automated checks

Run these from the repository root. Retain the complete command output rather than only a count.

```powershell
node --experimental-transform-types --test scripts/test/storage.test.mjs
node --experimental-transform-types --test scripts/test/campaign.test.mjs scripts/test/input.test.mjs scripts/test/narrative.test.mjs
cargo test --locked -p game_core
cargo test --locked -p engine_core
pnpm test:wasm-contract
cargo run --release --locked --manifest-path scripts/qa/boss-live/Cargo.toml --target-dir work/boss-live/target -- work/boss-live-combat.json
```

The storage suite exercises real IndexedDB transactions through `fake-indexeddb`: all supported old formats, per-family migration, malformed and impossible rows, saturating aggregates, same-schema current/backup migration, caller-owned input copies, equal-revision replay, newer stale-session writes, corrupt-current recovery and invalid imports. It cannot prove combat, WebGPU behavior or a phone's storage lifecycle.

The standalone WASM contract suite builds the actual WebGPU and WebGL2 serialization paths into `work/wasm-contract/`; its probe is feature-gated out of shipping packages. Fresh/champion checkpoints check explicit `null` for Rust `Option::None`. The generation and legacy-boss probes check actual Rust normalization against the strict TypeScript boundary, JSON export and SaveStore reload. All eight scenarios passed in `artifacts/boss-wasm-contract-tests.log`; both shipping JavaScript and WASM exports excluded `checkpoint_contract`.

Native proofs must use source behavior in `Game::tick`, projectile sweeps, occupied-cell damage, spawning and encounter cleanup. Queue proofs need real ordinary defeats, exact promotion thresholds, surplus retained behind an active fight, parallel-family queues, death/retry and reloaded profiles. Direct internal damage/kill helpers are useful unit coverage but are not the independent live-combat sweep. Any live-combat harness must declare its ship/upgrades, aiming method, fixed time step, finite frame/time budget, viewport and rendered counts.

The repository contains the isolated `scripts/qa/boss-live/` harness and its production-matched lock; generated reports and compiler output stay under `work/`. The harness declares a Royale Sovereign at level 12 with rank-five hardware/signatures. It uses a 1/60-second fixed step, six-second observation period and 120-second firing budget. Read-only actual boss/core coordinates drive joystick magnitude 0.081 for turning, increasing to 0.35 beyond 420 units to follow siege retreats. The harness does not call damage, kill, inventory or reward helpers. All fifteen fights completed after 7.68–12.35 simulated seconds, with charge, their named attack and exposed-core phases observed. Every full profile remained exactly equal before setup and after defeat. Ten seconds at Haven verified cleanup and safety after every case.

The final rerun followed the Coil spawn fix that points the tail away from the actual player. Render extraction at world extents `640×320 / zoom` produced 3,116–15,926 quads at 0.6 and 1,084–10,345 at 1.6, with finite transforms, transform-table bounds and exact emitted pixel totals. These are CPU extraction counts, not a GPU FPS measurement. A separate unupgraded Scout checkpoint with a queued Bastion G5 exercised natural flight/spawn, retreat, reload, actual hostile death after seven seconds and natural retry; counters stayed unchanged. The first sweep exposed incidental asteroid XP in QA fights and stationary-target range disengagement. Its evidence is retained as `work/boss-live-combat-first.*`; gameplay added persistent diagnostic reward isolation, and the harness explicitly followed intentional retreats before the passing rerun. The pre-spawn-fix passing baseline remains in `work/boss-live-combat-before-coil-fix.*`. Final source, atlas and lock fingerprints accompany the tracked result in `artifacts/boss-live-combat.json` and its complete compiler/run output in `artifacts/boss-live-combat.log`.

## Actual browser procedure

Use a disposable QA profile or first export a backup. Mutating scenes are available only with `?qa=1`; they must not be described as regular player progression. Open the backtick drawer, run `PROFILE` and `STATS`, then close it before simulating. `PROFILE` reports the real generation matrix. The established aliases `boss-dart`, `boss-bastion`, `boss-coil` select generation one; explicit `boss-dart-1` through `boss-dart-5`, and equivalent Bastion/Coil commands, are the new diagnostic contract. `champion-0` through `champion-7` are reward-isolated trial fights; `tournament` only moves to its beacon.

| Check | Reproduction and acceptance | Evidence required |
| --- | --- | --- |
| Fifteen identities | Launch each explicit family/generation scene. Verify the actual boss name, family, generation, bounded positive hull and the intended distinct silhouette. | Fifteen scene records; screenshots where silhouette or HUD is disputed. |
| Attacks and telegraphs | Record a finite fixed-step interval for each scene, including damaged and low-health phases. A telegraph must visibly precede its attack and phase text must follow actual runtime state. | Runtime phase/event traces plus browser captures. No invented attack completion. |
| Core shielding and pixel damage | Hit armor and the declared weak point with actual projectiles; destroy armor/segments and revisit the same holes. Validate vulnerability timing, hull effect and surviving occupied pixels. | Native projectile/render evidence and targeted browser observation. |
| Diagnostic rewards | Snapshot counters, XP, currencies, pity, signatures, quests and campaign ownership before setup and after an actual diagnostic boss defeat. All diagnostic rewards/counters remain unchanged. Distinguish ordinary incidental enemies if the scene does not isolate them. | Full before/after snapshots and actual defeat event. A setup-only assertion is insufficient. |
| Natural promotions | Defeat 24 of one base family for G1; defeat two G1 for G2; repeat to G5. Check surplus behind queued/active encounters and queue visibility at Haven. | Counter/queue trace from real fights; native threshold tests separately. |
| Loss and retry | Start an outstanding promoted fight, lose/retreat, then return to combat. Its queue remains; it respawns without a premature defeat/reward. Repeat around save/reload. | Before/after profile and boss HUD, death report, retry evidence. |
| Champion separation | Fight a real champion and a diagnostic champion while ordinary queues are outstanding. Only the real trial awards its once-only patch/blueprint; neither advances ordinary boss generations. | Campaign and generation snapshots. |
| Save races | Two tabs start from one checkpoint. Advance a boss in tab A; tab B attempts a newer stale save. A's checkpoint survives and B visibly pauses on SaveConflict. | Browser observation and the automated transaction regression. |
| Sanctuary and modals | Return to Haven during an ordinary boss fight. No enemy damage/spawn there. Fire opens home services; returning alone does not. Console/dialogue pause, death Retry/title and async preparation cannot resume over another modal/title. | Actual interactions; source review alone does not pass this row. |
| Zoom and fixed UI | Observe each family at 0.6 and 1.6 zoom, then resize landscape. World scale follows zoom; HUD, health bar, queue list and controls retain fixed screen geometry and fit safe areas. | Viewport/quad records and browser capture at both limits. |
| Backend/recovery | Repeat representative deep boss patterns on WebGPU/WebGL2; destroy/recover a graphics device during a live boss. Position, health and queues survive actual recovery. | Current-build backend telemetry and recovery capture. |
| Phone and soak | Physical landscape multi-touch movement+fire, thermal sustained load, rotation, browser/PWA background/foreground and touch cancellation. | Named real devices, duration, measured frame statistics and persistence results. Pending until measured. |

## Evidence ledger

| Scope | Current state | Actual evidence |
| --- | --- | --- |
| Browser save validation/persistence | Passed automated checks | `artifacts/boss-shell-tests.log`: 22/22, including six generation-specific regressions. |
| Written specialist review | Active asynchronous review | `work/boss-qa-review.json`; actual messages from production/gameplay/engine, with conclusions distinguished from tests. |
| Fifteen-boss native combat/render sweep | Passed declared CPU probes | `artifacts/boss-live-combat.json` and `.log`: final Coil spawn fix, 15/15 actual fights, full-profile isolation, phase coverage, finite extraction, sanctuary safety and actual G5 loss/retry. Human balance remains untested. |
| Cross-language generation snapshots | Passed both WASM backends | `work/boss-wasm-contract-test.log`: eight actual Rust-serializer/strict-TypeScript/IndexedDB save/reload scenarios; shipping export isolation checked. |
| Final compact-settings shipping QA exclusion | Passed compiled-package scan | `artifacts/boss-shipping-export-review.json`, recorded 2026-10-10 18:04:55 UTC: no checkpoint probe in public/dist JS or binary exports, unchanged backend hashes match, eighteen game-only cache assets under `galaxy-5a206c9f1e2acf40`. Recomputed content hash and complete asset list match `dist/`. This is packaging evidence, not an offline browser test. |
| Compact settings presentation | Passed independent capture review at 720×360 | `artifacts/boss-settings-720-zoom-default.jpg`: all nine controls and five footer actions have readable labels inside the card; the default zoom thumb is approximately 40%. `artifacts/boss-browser-capture-review.json` records source hashes and scope. Physical touch and other viewport sizes remain separate. |
| Deep Coil minimum-zoom presentation | Limited capture reviewed | `artifacts/boss-coil-g5-zoom-min.jpg`: readable boss/phase/player HUD and full-hull Scout. Head is partly behind the HUD and tail extends beyond the viewport; the still does not prove all segments or attack timing. |
| Current-build browser all-fifteen acceptance | Pending | Prior release browser proofs do not pass new content. |
| Physical phones and sustained performance | Pending | No physical mobile or soak measurement claimed. |

The final representative browser procedure and capture hashes are retained in `artifacts/boss-browser-validation.json`. Maximum zoom was selected through the real settings range; wheel minimum and increment were separately observed. The final WebGL2 Bastion capture and scalar telemetry are `artifacts/boss-bastion-g5-webgl2.jpg` and `artifacts/boss-browser-webgl2-stats.txt`. These captures establish representative rendering and fixed UI placement; all-content choreography, human balance and physical device acceptance remain pending.
