# Independent weapon cadence candidate review

Recorded 2026-10-10 19:23–19:34 UTC. **Accepted for production integration** after independent actual public-`Game` native execution. This fixes a newly discovered progression issue in the published HUD/Riveter source; it does not invalidate that release's renderer acceptance. This reviewer changed only owned `work/` files. Root began integrating the accepted patch after receiving the verdict; a rebuilt or republished release is outside this proof.

## Source identity and correspondence

Published baseline: commit `4ac160d6278ac0e728761189e8cf6f2fa54197d1`, cache `galaxy-f37b303ee2258928`, raw core input SHA256 `e2ef687c9f14d8b75af6b36fbb220b8244979ff35a042d8b0ce6d579fabd6702`. Gameplay's frozen candidate has SHA256 `f038546fb7b630ccff18bf2b15ce6e2f5d973d2c9283dbec27fae62aa299e5e9`; its compiled actual-core library has SHA256 `74f99e1150ae7e216b576d911286e670f1226fe8b56cf11a8a3f54ddb29e871f`.

The candidate uses the actual core source and unchanged real modules/assets through explicit path attributes. Its public `Game` API is unchanged. Inspection of `prepare.py` and the three-hunk patch confirms that runtime changes are limited to positive cooldown decrement with sub-microsecond boundary normalization, clearing negative cooldown during inactive/safe ticks, and adding the next period after an actual shot. There is still one firing branch per tick. Damage, stat multipliers, profile schema, rewards and Echo creation are unchanged.

At 19:34:14 UTC, an independent full-text comparison found root's integrated runtime core identical to the frozen candidate after removing only work-specific module path attributes/test declaration and the production `cfg(test)` cadence-module declaration, and normalizing line endings. The seven other real source-module hashes still match the candidate's build inputs. This verifies source correspondence; it is not a production build or browser result. Original inputs, integrated-source hash, comparison rules, candidate/library/harness/executable/output fingerprints are in `work/weapon-cadence-qa/hashes.json`.

## Actual independent results

The isolated executable links the frozen actual-core library and calls public `Game::tick`, counts emitted `shot` events and compares complete profiles. It does not reproduce the timer or grant/kill actors directly. Its declared Scout fixture has inherited armor/shield/regeneration rank 5, rate rank under test, otherwise default hardware, micro-thrust magnitude 0.081 and the ordinary Dart diagnostic. Longer windows aim real joystick/fire at the current boss hull or exposed core; real projectiles may defeat it, after which diagnostic isolation suppresses ordinary waves. All six Scout windows ended with 175 hull and no profile changes, so deaths/resets did not invalidate the cadence measurements.

| Rate rank | Authored shots/s | Actual primaries / 30 s | Actual interval ticks at 60 Hz |
| ---: | ---: | ---: | --- |
| 0 | 5 | 150 | 12 |
| 1 | 5.5 | 165 | 10–11 |
| 2 | 6 | 180 | 10 |
| 3 | 6.5 | 195 | 9–10 |
| 4 | 7 | 210 | 8–9 |
| 5 | 7.5 | 225 | 8 |

Every rank strictly improves actual primary cadence. All intervals fall within floor/ceiling of `60 / authored_rate`; noninteger periods retain residual through alternating intervals. Separate five-second windows emitted 25/28/30/33/35/38 primaries, including the initial ready shot. These bounded counts should not be interpreted as long-term rates by simply dividing by five: the sampled interval runs from tick 1 through tick 300.

All eight actual owned crafted ships at inherited rate rank 5 emitted 38 primaries in the same five-second window with exact eight-tick intervals. Actual owned Sovereign emitted 45 at its intrinsic `7.5 × 1.2 = 9` shots/s, with six/seven-tick intervals. Complete profiles stayed unchanged, confirming that cadence extraction did not transfer XP/rewards or overwrite global ranks.

- Initial fire, three seconds of released-input idle, held fire at Haven, held fire at a remote sanctuary, return home and an actual diagnostic death/retry each produced one ready primary and a full subsequent twelve-tick rank 0 period. Neither a negative idle timer nor accumulated catch-up debt carried into the next interval.
- A three-tick release after the initial shot preserved the remaining cooldown at every rank. Resume waits were 9/8/7/7/6/5 ticks; release neither erased the wait nor restarted a full period.
- Pause held for 100 ticks produced no primaries. Zero, negative, NaN and infinite time produced no primaries; 60 calls with 0.5-second input, clamped by the actual game, emitted at most one primary per call. All firing paths used the actual one-event-per-tick assertion.
- Paired actual games with/without Echo had identical primary events and entity counts before the trigger. After 180 held-fire safe ticks, 100 paused ticks and a three-tick release immediately before the threshold, evolution 1 added exactly three live bullet entities on primary 8; evolution 5 added exactly seven on primary 6. Whole profiles remained unchanged. Safe input, pause and release did not manufacture a bonus count.

Detailed actual shot ticks, intervals, ship IDs, hulls, release/reset probes and Echo observations are in `work/weapon-cadence-qa/results.json`. Gameplay separately reported seven focused actual-core tests passing; those are authored specialist results, while the public-API executable/results above were independently run by this reviewer.

## Existing diagnostic limitation

The initial independent run reproduced a pre-existing QA helper limitation: `diagnostic("die")` writes health 0, but an already eligible rank 5 nanite regeneration tick heals it above zero before the final death check. The death/retry acceptance fixture therefore explicitly uses regeneration 0, with rank 5 armor/shield. This does not demonstrate resurrection after ordinary lethal damage: `hurt_player` resets `since_damage`, and ordinary collision damage occurs after that tick's regeneration. No production diagnostic change was included in the cadence patch. The first failed fixture is not hidden as a successful death test.

No blocking defect was observed in the frozen cadence candidate. Human weapon feel, naturally earned rank affordability, full natural progression/campaign, browser/audio scheduling and physical phone 60 FPS/multitouch/lifecycle/soak remain pending. These native diagnostic probes are neither a human playtest nor new publication acceptance.

## Reproduction

Using the preserved frozen library and matching existing dependency artifacts:

```powershell
rustc --edition=2024 work/weapon-cadence-qa/main.rs -L dependency=target/debug/deps --extern game_core=work/weapon-cadence-fix/libgame_core_cadence_candidate.rlib --extern serde_json=target/debug/deps/libserde_json-b89616ee5c7ce7b6.rlib -o work/weapon-cadence-qa/cadence-proof.exe
./work/weapon-cadence-qa/cadence-proof.exe
```

The successful run exits0 and writes actual results JSON. These commands depend on preserved local compiled libraries, not a fresh-checkout dependency setup. Gameplay's work-only `build.ps1` intentionally guards its original input hash and will reject the now-integrated production source rather than silently testing a different candidate; use the identified baseline to regenerate that exact isolated candidate. Root owns durable integrated regression tests and normal native/WASM/build validation.
