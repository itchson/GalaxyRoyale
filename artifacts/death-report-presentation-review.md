# Death report presentation — independent source review

Recorded 2026-10-10 19:53 UTC. Root accepted the bounded presentation option below after this review. This reviewer read production source and the already frozen actual-core proof; only this work directory was written. No browser, phone, normal-package or remote execution is claimed here. The observed local core now contains the shared early-dead simulation helper; that local integration is distinct from the previously published `be3ae02adcf514d2f3a6e9687115a3c7da344d2d` / `galaxy-79774fbb09d2dfb9` release.

## Finding and accepted treatment

`showDeath()` reads `hud.last_run` without evidence that the current death finished a run. The heading `EXPEDITION ENDED` can therefore present an older saved report as this QA death's expedition: forced death at Haven, a protected diagnostic boss, or `combat → die` before a valid outbound tick preserves the existing history. This is a QA presentation ambiguity, not a reward or checkpoint duplication.

The smallest reliable treatment needs no freshness detector:

- For non-null `hud.last_run`, label the existing values **LAST RECORDED EXPEDITION**.
- For null history, show **NO EXPEDITION RECORDED**.
- Preserve the real death screen, Retry/Title actions and the current dock stash/ship-level row. Preserve all saved history and natural-death totals.

Root accepted and owns this UI implementation. This reviewer has not executed or accepted its browser layout. The existing `last_run !== null` condition reliably distinguishes recorded history from absent history; it does not distinguish a newly completed run from an older one. The universal wording is truthful in both cases.

## Source evidence

At the inspected snapshot, `apps/game/src/main.ts:1122` pauses the runtime and inputs on death. At line 1140 it reads the post-step `hud.last_run` and renders duration/distance, kills, points and XP/materials. It then displays current stored materials and ship level. The HTML at `apps/game/index.html:190` describes the expedition as ended; there was no independent recorded-history label in the inspected implementation.

`crates/game_core/src/campaign_runtime.rs:33` creates/updates the private run only while outside Haven and under the actual tracking rules. Its `finish_run()` at line 41 takes that private `Option<RunSummary>`; only a present run replaces `profile.campaign.last_run` and increments revision. With no current run, the previous record remains intact. `Game.run` is private at `crates/game_core/src/lib.rs:443`; HUD publishes only the retained `last_run`.

The shared helper at `crates/game_core/src/lib.rs:1033` emits death, performs the existing reset/repair/bank/finish path and returns before healing/combat when already dead at valid tick entry. `enter_dock()` at line 2505 calls `finish_run()` and emits `dock` even if there was no run. Consequently death/dock are not fresh-report events.

`crates/game_web/src/browser.rs:148` may process up to five actual fixed ticks before serializing one HUD/events response. Its profile response at line 247 is conditional on any revision change, not run completion. The frontend handles the post-step HUD and death event in `main.ts:813` onwards. An earlier frontend-observed outside-Haven frame is not guaranteed before a first outbound lethal tick.

## Rejected inferred signals

| Proposed signal | Concrete failure |
| --- | --- |
| Previous/current `last_run` content differs | Two legitimate completed runs may have equal totals. Equality would hide a real report. Object identity is also recreated by serialization. |
| Profile revision or non-null frame profile | Banking, discoveries, courier delivery, arena result handling and other retained changes also alter revision. A run finish is not identified. |
| Death or dock event | Both exist on the accepted no-new-run forced reset. Neither event carries report freshness. |
| HUD expedition time | Safe remote sanctuaries can track a run while expedition time is zero; reward-isolated boss QA can advance expedition time without tracking a run. Reset clears expedition time. |
| Frontend `scenario` or QA flag | `SCENE die` overwrites the scene label with `die`. QA-enabled sessions may still contain genuine completed combat runs. |
| Frontend saw position outside Haven before death | Multiple fixed ticks can occur in one response. Protected QA scenes also place the player outside Haven without a tracked run. |

If future product design requires a selective **NO NEW EXPEDITION REPORT** message, the reliable source is the private current-run presence/actual `finish_run` outcome. A narrow transient event or return value could expose that outcome. It need not be persisted in the profile. That is unnecessary for the accepted universal history label, so no candidate detector or schema change was authored.

## Existing actual execution supporting this review

The frozen independent public-Game proof is `work/diagnostic-death-qa/results.json`, described in that directory's `review.md`. It links Engine's frozen work-only candidate, not the newly integrated local production core. This task did not rerun or count it as a new integration test.

- Haven and all three protected diagnostic families each emitted one death and zero postmortem primaries, retained the entire checkpoint exactly, and continued to expose the prior report: 12 seconds, 2000 peak distance, three enemies, 15 XP and two collected materials.
- The atomic unstarted `combat → die` case retained prior history and invented no duration/rewards.
- The actually simulated remote sanctuary run replaced history with 0.050000004 seconds and zero gains. Thus zero expedition danger/time would not mean no recorded run.
- Real ordinary enemy AI/projectile/contact ticks produced a lethal collision at tick 449, a truthful 7.4833765-second summary and stable exact save/reload. That report remains meaningful under the universal history label.

Engine independently confirmed that its frozen helper adds no report field/event and exposes no existing reliable freshness signal. This was an actual asynchronous review exchange, not a fabricated meeting or additional test execution.

## Identity and limits

The observed source and existing proof hashes are frozen in `evidence.json`. Read-only source inspection found no reason to alter the saved prior summary or invent a new timestamp, run, reward or revision. Root owns implementation and ordinary native/WASM/build/browser verification of its new label. Physical-phone readability/touch/lifecycle/60 FPS/soak and naturally played Scout/full campaign acceptance remain pending separately.
