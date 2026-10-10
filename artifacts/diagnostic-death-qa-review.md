# Independent diagnostic-death candidate QA

Recorded 2026-10-10 19:40–19:48:34 UTC. **Accepted for bounded future simulation integration.** This is an isolated actual-core candidate, not a change to the published game. This reviewer changed only `work/diagnostic-death-qa/`, ran focused native public-Game probes and read source. No production/shared manifest/dashboard/deployment edits, authenticated remote execution, browser/GPU execution or broad test reruns were performed.

## Identity and source review

Current publication metadata identifies commit `be3ae02adcf514d2f3a6e9687115a3c7da344d2d`, cache `galaxy-79774fbb09d2dfb9`. That publication retains the original forced-death QA helper behavior. Local metadata is historical identity evidence; this reviewer did not verify authenticated remote execution.

Frozen production core input SHA256 is `0a574a35c9d23f955fff05246da2cf4bf9a8065f5db58c63c365124f982337e6`. Engine's work-only candidate source is `df502b969f767842c12fbe330cf2db9db6bcae8455f8a6c5761cd64e781c5321`; compiled library is `635b0d0564ff9003d918509137fb43a73892898f9d654960aaa335a2970cf7ee`. All recorded frozen production inputs still matched at independent hash capture. The harness links that candidate through the unchanged public `Game` interface. Source, candidate/library, baseline library, harness/executables, logs and results fingerprints are in `hashes.json`.

Inspection of the bounded patch confirms an early `respawn_if_dead()` call after pause/nonfinite/nonpositive-time guards and before simulation, plus reuse of that same helper at the existing end-of-tick location. The helper retains the original death event, particle burst, reset/repair, checkpoint and shake operations. An already-dead tick returns without running healing, firing, movement or reward/time accrual. A player alive at entry still runs the ordinary simulation and end-of-tick lethal-collision path. No profile field, authority rule or reward was added.

Merely resetting regeneration time would miss Sovereign's primary-eight signature repair. The shared early guard correctly covers both nanites and that automatic heal. Source-level natural damage still resets `since_damage=0` in `hurt_player`; the ordinary collision occurs after that tick's regeneration.

## Independent actual execution

`baseline-proof.exe`, linked against the actual unmodified production core, reproduced the existing defect at the first eligible R5 Haven probe: zero death events instead of one. Its expected failure is preserved in `baseline-failure.log`. It is not recorded as a pass.

`death-proof.exe`, linked against the frozen candidate, exited 0 and wrote `results.json`; stdout is preserved in `execution.log`. The checks invoke real `Game::tick`, inspect actual events/HUD/profiles and use only the public diagnostic/navigation interface. No private kill/hurt/spawn/grant API or reimplemented death algorithm was used.

- At Haven, Dart G1, Bastion G5, Coil G5 and a remote sanctuary, already eligible rank 5 regeneration could not revive forced health 0. Two `die` commands before the valid tick emitted exactly one death and no postmortem primary. The ship returned to `[0,0]`, with full hull/shield, safe/dock true and no active boss.
- The declared existing checkpoint carries currency `[7,8,9]`, Scout XP 60, stock 9 and advanced generation matrices `[[18,9,4,2,1],[6,3,1,0,0],[26,13,6,2,1]]` plus pending credits. Currency, all ten XP slots, hardware/signatures, kill/generation/pity/pending state, stored materials, patches and league results were retained. Whole profiles stayed identical in Haven/protected-boss scenes. The remote sanctuary's actually simulated three ticks produced its truthful 0.05-second run summary; no reward was invented.
- Ten paused ticks and zero/negative/NaN/infinite time deferred death without events or profile changes. One valid unpaused tick then emitted one death. Sixty subsequent held-fire Haven ticks emitted neither another death nor a primary, and left the completed checkpoint unchanged.
- Owned Sovereign fired seven real primaries, then waited thirteen released-input ticks so its weapon was ready. Forced death plus held fire emitted one death and zero primaries; primary 8's repair did not run. The protected diagnostic checkpoint stayed identical.
- All five scene-return probes and the Sovereign boundary probe allowed one explicit real-shot retry without another death or profile reward. The existing last-run record remained unchanged through the retry setup.

The independent ordinary collision probe used a declared previously saved currency/XP/stock fixture, regeneration rank 5, no armor/shield and no firing. Public `diagnostic("combat")` creates the existing deterministic standard-enemy scene; subsequent ticks use the actual enemy AI, projectiles, contacts and production `hurt_player` collision route. It is a simulation probe, not naturally reached human progression.

| Actual ordinary collision observation | Value |
| --- | --- |
| First/only death | Tick 449, about 7.4833336 simulated seconds |
| Actual hit events before/including death | 12 |
| Finished run time | 7.4833765 s (f32 accumulation) |
| Finished peak distance | 1600.7543 world units |
| Run enemies / asteroids / XP / points / materials | All zero; no player firing/reward grants |
| Retained existing currency / Scout XP / stock | `[7,8,9]` / 60 / 9 |
| Subsequent Haven report/checkpoint | Stable; no repeated death |
| Actual `Game::new` reload | Complete profile and last-run equal; no death event |
| Explicit retry | One real primary; no death or reward mutation |

This demonstrates one truthful completed run on actual lethal combat damage while keeping the ordinary tick-end semantics. The original collision baseline was not independently replayed here; Engine separately owns its baseline/candidate natural-combat parity proof. Its eight focused actual-core regression passes were reported by Engine and their source was read, not counted as an independent test run by this reviewer. The independent public probe verifies retained existing stock; Engine's authored pickup fixture separately covers actual unbanked cargo collection/banking. Neither is claimed as a human cargo-gathering playtest.

## Accepted no-new-run edge and UI limitation

Atomic `combat → die` before any valid outbound flight tick invents no flight duration or rewards; the candidate preserves the prior `last_run`. Forced death at Haven and reward-isolated boss scenes likewise retain the prior report. Root accepted this native preservation policy.

Current `showDeath()` at `apps/game/src/main.ts:1140` unconditionally renders `hud.last_run` beneath `EXPEDITION ENDED`. Consequently these QA-only no-new-run deaths can display a previous expedition report. This is an explicit **nonblocking QA presentation limitation**, not evidence of a new run or natural reward/currency duplication. Root owns any future truthful no-new-run presentation guard. No fabricated UI report, timestamp or reward was introduced to hide the edge.

## Reproduction and limits

With the preserved candidate library and matching existing dependencies:

```powershell
rustc --edition=2024 work/diagnostic-death-qa/main.rs -L dependency=target/debug/deps --extern game_core=work/diagnostic-death-fix/libgame_core_diagnostic_candidate.rlib --extern serde_json=target/debug/deps/libserde_json-b89616ee5c7ce7b6.rlib -o work/diagnostic-death-qa/death-proof.exe
./work/diagnostic-death-qa/death-proof.exe
```

The exact baseline reproduction substitutes `target/debug/deps/libgame_core-684f3d0a8e530f28.rlib`; expected exit 1 is the first R5 forced-death failure. The work-only engine build script guards the original core input hash and must be rebased explicitly if production changes. These reproduction commands depend on the identified existing compiled dependency artifacts, not a fresh-checkout setup.

No blocking simulation defect was observed in the frozen candidate. Production integration, normal native/WASM/build checks, browser death/report/retry presentation and publication remain root-owned future work. Physical-phone touch/lifecycle/readability/60 FPS/soak, natural Scout progression and full campaign/boss play remain separate pending gates.
