# Ordinary per-pilot Ion Wake native candidate

Branch: `codex/coop-ion-wake`, based on `5e4b629e91f3334582ffa30d5e65af98675f8c39`. The frozen title branch is preserved. This isolated candidate does not change Root's current F4 runtime. Only four Rust files in `crates/game_core/src` plus this evidence directory are changed. No manifests, browser, rendering, UI, audio, or servers are changed.

## Reproduction and implementation

The baseline admits ordinary Ion Wake ID5 but has no per-pilot wake state or update. The added regression uses an actual admitted profile, real ECS motion/enemy and `tick_co_op`. At the baseline the target remains at 100 health rather than 99.9; `baseline-reproduction.log` records the actual failing run. With the candidate it reaches 99.9, with the retained solo profile unchanged.

`Pilot` caches the effective equipped ID5 rank and family-two evolution, and owns its transient timer/samples. Guests use validated effect-only `wake_evolution`; no remote defeat history enters the temporary Profile. Champion Windrunner and Royal wake remain outside this ordinary-ability candidate.

Each fixed tick captures each pilot's pre-integration origin, as solo does. After collision/respawn and home repair, the real native wake stage ages samples, checks the current owner sanctuary and speed, admits at most one sample, then applies the solo health damage rules. Births use the current heading and the origin minus 14 units forward. Cadence is 0.2 seconds through evolution three, 0.15 from four; strict emission speed threshold is greater than half that pilot's actual admitted speed. Point damage reaches enemies/bosses only, excludes protected targets, uses 6 DPS scaled by 1 + 0.25*(effective rank - 1), and keeps the existing boss armor multiplier. Nonfatal wake damage leaves PixelBody cells intact, matching solo. Real lethal hits call `kill_actor`, with exact owner ID/epoch scoped around the call and the enclosing owner restored. Overlapping kills use stable pilot slot order and remove each target once. Co-op does not grant solo progression or mutate saved profiles.

Evolution tiers retain solo dimensions:

| Evolution | Samples/pilot | Sample lifetime | Reach | Connected damage |
| --- | ---: | ---: | ---: | --- |
| 1 | 12 | 2.0 s | 20 | none |
| 2 | 16 | 2.6 s | 20 | none |
| 3 | 16 | 2.6 s | 28 | none |
| 4 | 24 | 3.4 s | 28 | none |
| 5 | 24 | 3.4 s | 28 | consecutive samples <=160 units |

Evolution-five connected links are computed once per pilot (at most 23). They exclude Haven and canonical region sanctuaries via `regions::segment_protected`, including sites outside the host's nearby render-site cache. Co-op cannot start in an arena; therefore this canonical sanctuary check applies to every admitted co-op session. This fixes the cache limitation for distant pilots without changing solo source.

Eight memberships hold at most 192 samples. Wake creates no projectile entities and no unbounded emission backlog. Births use the existing bounded ability event and 0.3-second transient-light systems (global caps 128 events and 32 lights). Samples age while the owner is safe, but sanctuary prevents births and damage. Pausing preserves phase. Death clears that pilot's samples/timer before the late wake stage; leave drops the owning Pilot; rejoin gets empty state and a fresh epoch. Session end restores the original boxed solo Game, including its original wake state. A birth light already emitted before leave/death may finish its bounded 0.3-second lifetime in the shared light system; no departed membership trail can keep damaging.

## Wire contract requiring Root browser changes before integration

`OwnedLoadout.wake_evolution` is an optional numeric effect claim. The wire remains `galaxy-coop-1` / loadout version one, consistent with the existing optional evolution fields. Root owns strict browser field allowlists and validation; do not integrate/rebuild this native candidate until those recognize the exact contract. Offline effect/loadout claims do not prove durable ownership.

| Value | Native result |
| --- | --- |
| omitted | `None`; admitted evolution one; old serialized bytes unchanged |
| explicit integer 1..5 | `Some(value)`; accepted only with `slots[2]` ordinary ID5 and existing valid rank/category/ownership/XP constraints |
| `null`, boolean, string, negative, fraction, integer >255 | rejected by serde parsing |
| integer 0, 6..255 | serde can parse; native admission rejects as `InvalidLoadout` |
| supplied while ordinary ID5 is absent or replaced | `InvalidLoadout` |
| unknown combat/save/trail fields | rejected by the existing `deny_unknown_fields` |

Native export omits the field at evolution one or when ordinary ID5 is not effectively equipped. It emits evolution 2..5 only for that ordinary effect. Explicit one remains a legal claim but is not normally exported. Raw native serializer fixtures in `wire-fixtures/` were extracted from actual passing native test output, without rewriting their JSON payloads. `candidate-pins.json` pins their file bytes and all four Rust sources. No arbitrary combat state or boss-history save fields travel in this contract.

## Actual checks

All commands ran in the isolated Windows worktree, offline, using its existing Rust dependencies/cache. No broad workspace build or browser/runtime test was performed.

- Baseline: `cargo test -p game_core --lib co_op::wake_tests --offline -- --nocapture` — failed as expected, 0 passed / 1 failed; health 100 vs 99.9. Raw `baseline-reproduction.log`.
- Final Ion Wake: same command — 13 passed / 0 failed / 0 ignored; 528 filtered. Raw `wake-tests-attempt-3.log`.
- Existing co-op coverage: `cargo test -p game_core --lib co_op --offline -- --nocapture` — 115 passed / 0 failed / 3 ignored; 423 filtered. Raw `co-op-regression.log`. Ignored test names/reasons remain in that log.
- Focused compile: `cargo check -p game_core --offline` — passed. Raw `game-core-check.log`.
- Source whitespace check: `git diff --cached --check -- crates/game_core` — passed; empty `diff-check.log`. Raw Cargo logs preserve their emitted trailing blank lines.

The 13 tests exercise the real first host/guest ticks, all five actual XP rank caps, real guest destruction and owner epochs, overlap attribution, boss armor and cell preservation, eight-pilot real tick resource bounds, threshold/ordinary equipment gates, sanctuary owner/target protection, remote-site segment fencing, pause/input timeout/expiry, death/leave/rejoin/restored solo state, and omission/null/range/unknown-field serializer rules. A separate differential runs the actual isolated solo and co-op wake stages for all 25 rank/evolution builds over 360 active plus 210 stationary steps; this verifies that stage, not a complete solo-versus-RTC game session.

Raw failed intermediate attempts are preserved: the first expanded test compile assumed PixelBody was serde-serializable and used the wrong ParticipantId helper; the next attempt revealed two remote-site test fixtures were still inside the host cache. Both fixture/test issues were corrected, and the final tests above actually passed. No passing result is inferred from source inspection.

## Open review and physical gates

This is a native gameplay candidate for independent review. `engine_fx` reviews the frozen source separately; no review result is asserted here. Root must update browser strict wire validation, review and integrate/rebuild after acceptance, and verify genuine host/guest RTC behavior and guest admission/effective build with the new optional field.

The existing solo persistent trail renderer still reads `Game.wake`; this candidate's membership-local samples are gameplay state only. Existing birth light/event calls are native, but persistent per-pilot trail drawing, guest publication/replica effects and audiovisual delivery are not implemented or accepted by this scope. No RTC, guest effect, visual, audio, mobile physical-play, or perceived acceptance claim follows from these native tests. Root must address the visual/replication design and conduct actual runtime checks before calling Ion Wake fully usable in co-op.
