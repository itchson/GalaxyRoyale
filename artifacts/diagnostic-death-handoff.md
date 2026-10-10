# Forced QA death — isolated future candidate

Production remains unchanged. This directory contains an actual copy of the frozen `Game` core, unchanged production modules/assets linked at their original paths, a minimal integration patch, and native proof. There is no new dependency, manifest, save field, event type, renderer or browser implementation. Root's current private deployment does **not** include this candidate.

## Exact failure and candidate

`diagnostic("die")` writes hull zero. In the frozen production tick, eligible regeneration runs before the final death check. R5 restores `0.033333335` hull on the next 1/60s tick, so death never occurs. A timer reset in that command addresses regeneration but is incomplete: a ready Sovereign eighth primary heals four hull independently of the timer.

[integration.patch](C:/Users/mitch/Documents/ChatGPT/galaxy-royale/work/diagnostic-death-fix/integration.patch) extracts the existing end-of-tick death block into private `respawn_if_dead() -> bool`. The original end-of-tick call remains at the same point. A second call immediately after the paused/invalid-time guards returns early when the player was already dead at tick entry. The event, burst, dock cleanup, repair, shield, banking, finished-run and shake path is shared without duplication.

Healthy-player ordering is unchanged: regeneration, firing, movement and actual collision handling run as before, and natural lethal collisions still resolve at the original end-of-tick location. A hull already zero at accepted tick entry cannot be healed by regeneration, a shot signature or dock entry before it dies. Paused/invalid ticks leave a queued zero-hull state pending until a valid unpaused tick. Multiple `die` commands before that tick still resolve one death. This is not an immediate death in the command handler and adds no diagnostic state.

## Measured native negative controls

[public-probe.rs](C:/Users/mitch/Documents/ChatGPT/galaxy-royale/work/diagnostic-death-fix/public-probe.rs) uses only the actual public Game API. The Sovereign case fires seven real primaries, waits twenty idle ticks beyond its cooldown, then commands death and holds fire for one tick. Profiles are declared fixtures, not diagnostic rewards.

| Actual core variant | R5 death events / hull / position | Ready Sovereign death events / hull / new primary |
| --- | --- | --- |
| Frozen baseline | 0 / 0.033333335 / [1800,0] | 0 / 4.05000019 / 1 |
| Timer-reset-only negative control | 1 / 175 / [0,0] | 0 / 4 / 1 |
| Shared-helper candidate | 1 / 175 / [0,0] | 1 / 240 / 0 |

Full serialized profiles remain unchanged in these diagnostic boss cases. Raw results are [baseline](C:/Users/mitch/Documents/ChatGPT/galaxy-royale/work/diagnostic-death-fix/probe-baseline.json), [timer-only control](C:/Users/mitch/Documents/ChatGPT/galaxy-royale/work/diagnostic-death-fix/probe-timer_control.json), and [candidate](C:/Users/mitch/Documents/ChatGPT/galaxy-royale/work/diagnostic-death-fix/probe-candidate.json). `timer-only-lib.rs` is deliberately incomplete experimental evidence, **not** an integration candidate.

## Focused actual Game::tick regressions

[tests.rs](C:/Users/mitch/Documents/ChatGPT/galaxy-royale/work/diagnostic-death-fix/tests.rs) passes **8/8**. The candidate compiled the existing 82 production tests but filtered them from this run; this task does not claim a full production-suite rerun.

- Forced death at regeneration ranks0–5 and accepted steps1/120s,1/60s,0.05s and10s input (the existing tick accepts/clamps large positive input).
- Ready Sovereign eighth-shot healing boundary; a dead player emits no primary.
- Paused/invalid time and duplicate commands; exactly one death and one dock event on the first valid tick, then600 Haven ticks without another death or shot.
- Three generation5 diagnostic families; exact full-profile equality, retained boss queue/counters, removed encounter, no invented run reward/report, and exactly one boss on explicit diagnostic retry.
- Earned run: an authored test enemy is killed by real player projectiles in actual ticks, yielding5 XP; authored collectible fixtures are collected by actual ticks for category currency/materials. No direct XP/credit/kill grant helper is used. Forced return preserves the full profile except its expected cargo bank, finished summary and two corresponding revisions. The summary, checkpoints and reload are exactly stable after subsequent Haven ticks.
- Natural lethal hostile projectile placed at an actual surviving ship cell: the real collision calls `hurt_player`, absorbs shield, damages hull, resets `since_damage=0`, and follows the original final death/reset path after one simulated tick. Full retained-profile/run-summary equality is asserted.
- Natural nonlethal occupied-cell hit: the shield/hull damage and unchanged five-second regeneration delay are asserted.
- Atomic unplayed diagnostic departure and an explicit Haven death: no earned rewards or new run summary are fabricated.

The unchanged baseline reproduces failure in the new forced-regeneration test. The exact same **two natural-combat tests pass on both baseline and candidate**, providing a positive behavioral control for real lethal collision timing and nonlethal regeneration. Candidate library and tests both pass strict native Clippy with `-D warnings`.

Independent QA linked the frozen candidate's actual public Game API, without private health/kill/spawn grants. Its accepted probe covers R5, advanced boss counters, sanctuary/Haven, duplicate/pause/invalid requests, Sovereign shot8, retry and report/reload retention. A no-fire ordinary combat run reached natural lethal contact at tick449 (~7.48s), with12 hit events and one death. Its separate report owns those observations and exact hashes; no browser claim follows from either native proof.

## Intentional report/time boundary

An already-dead tick-entry hull does not advance simulation time, create shots, earn loot or add a dead frame to a run. An existing actual run is finished once at its previously measured values. `diagnostic("combat")` immediately followed by `diagnostic("die")` **before any valid outbound tick** has no current run to finish; the previously retained `last_run` stays unchanged. A Haven-only forced death likewise does not manufacture a new summary. Future UI integration must not label an older retained summary as freshly earned solely because a QA death event appeared with no active run. Positive-health natural collision deaths retain their original time advance and finalization ordering.

This candidate does not change existing active-match save/checkpoint rules, boss retry queues, collected progression, or reward isolation. Public diagnostics move the player into QA scenes; the isolated tests are not an ordinary onboarding/playtest or a mobile control acceptance run.

## Reproduce and integrate

From the repository root, using existing Rust1.97.0 native dependencies:

```powershell
.\work\diagnostic-death-fix\build.ps1
.\work\diagnostic-death-fix\death-tests.exe diagnostic_death_audit:: --nocapture
.\work\diagnostic-death-fix\death-baseline-tests.exe diagnostic_death_audit::natural_ --nocapture
.\work\diagnostic-death-fix\check.ps1
.\work\diagnostic-death-fix\probe.ps1
```

The baseline forced-regeneration command intentionally exits101:

```powershell
.\work\diagnostic-death-fix\death-baseline-tests.exe diagnostic_death_audit::forced_death_cannot_be_undone_by_already_eligible_regeneration --exact --nocapture
```

`prepare.py` refuses changed production `lib.rs` rather than silently rebasing. It writes only here and records all read-only core/engine source, pixel-atlas and shared-manifest hashes in [source-hashes.json](C:/Users/mitch/Documents/ChatGPT/galaxy-royale/work/diagnostic-death-fix/source-hashes.json). Commands and actual outputs are retained in the build/test/negative-control/Clippy logs. The candidate rlib is `libgame_core_diagnostic_candidate.rlib`; a public probe can link it as `--extern game_core=work/diagnostic-death-fix/libgame_core_diagnostic_candidate.rlib -L dependency=target/debug/deps`.

Only `integration.patch`'s runtime hunks belong in production. The absolute module paths and appended `diagnostic_death_audit` module are isolated build scaffolding. Root can move `tests.rs` into a private production test module during a future integration. There are no production edits in this task and no WASM/dual-renderer/browser/physical-phone acceptance or publication. Those remain root's future integration gates.

Frozen identities:

- Production `lib.rs`: `0A574A35C9D23F955FFF05246DA2CF4BF9A8065F5DB58C63C365124F982337E6`.
- Candidate `candidate-lib.rs`: `DF502B969F767842C12FBE330CF2DB9DB6BCAE8455F8A6C5761CD64E781C5321`.
- Tests: `598D4212DC60B24C4B32835BAA47E845B666D15C5ADB02F00181FF3B973EDE7E`.
- Candidate rlib: `635B0D0564FF9003D918509137FB43A73892898F9D654960AAA335A2970CF7EE`.
