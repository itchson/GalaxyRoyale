# Independent ship progression QA review

Recorded 2026-10-10 19:16–19:22 UTC. Read-only production audit plus focused isolated checks under `work/scout-progression-qa/`. No production files, shared tests/manifests, dashboard or deployment were changed. These are newly discovered progression findings after the HUD/Riveter release; they do not invalidate its renderer acceptance.

## Source and proof identity

The audit began against the frozen gameplay introduced in `9c290ae8fa7baa1a993a867a5e0d0178e0a8bde1`; the successful HUD/Riveter publication metadata identifies exact source `4ac160d6278ac0e728761189e8cf6f2fa54197d1`, cache `galaxy-f37b303ee2258928`. HEAD advanced to bookkeeping commit `50108ed900b70d6d2468e69f462048618b10d2d9` during this review. Current gameplay/browser text matches the published source after normalizing Git/worktree line endings; raw working-file SHA256 values below remain the pre-candidate baseline, not a future cadence repair. Relevant SHA256 values are preserved in `work/scout-progression-qa/hashes.json`:

- `game_core/src/lib.rs`: `e2ef687c9f14d8b75af6b36fbb220b8244979ff35a042d8b0ce6d579fabd6702`.
- `campaign.rs`: `ce3701246d366042841cca3a61ce1c83ea15bbfcd312a05fe5d6d7f991d0b5bc`.
- `campaign_runtime.rs`: `beaa27a19af8c62a23df0b6d6f820e924af51ed40f1d1e78a5cac25f8d7a4e58`.
- `storage.ts`: `2340133e825315fa445469e640ab75e47c3d1bdb34fa332b1c52e68459f463b6`.

The native executable links the actual existing compiled `game_core` library, SHA256 `5aec559ea5a1dc1c423fac226ca300cbed2ab7438eaf6cbe0e98f8ec099de843`, generated after those core inputs. It does not reimplement purchases or campaign mutations. This is not an independent production rebuild, browser/GPU execution or natural human playtest.

## Findings

**P2: maximum-level caption promises an impossible next level.** At 99,000 XP, actual `Game::hud()` reports level100 and `ship_next_xp=101000`, but `campaign::level(101000)` is still100 because the function caps at100. `main.ts:615` prints “next level 101000 XP.” This was reproduced through the actual native public API. No XP is lost; the defect is the advertised reward threshold.

Minimal recommendation: when `hud.ship_level >= 100`, render `SHIP LEVEL 100 / <actual XP> XP / MAX LEVEL. Rank cap 5/5.` Keep actual XP and existing schema; no new `next_xp` sentinel or migration is required. Below100, retain the actual next-level threshold. This is for the next integration session, not a change made by this reviewer.

**P1: some purchased primary fire-rate ranks currently have no actual cadence benefit.** Gameplay's separate actual fixed-tick probe reported13/11/11/10/9/9 ticks between primary shots at ranks0–5, corresponding to approximately4.615/5.455/5.455/6/6.667/6.667 shots per second versus authored5/5.5/6/6.5/7/7.5. Rank1→2 and4→5 therefore plateau. I independently confirmed the source cause: `fire_timer -= dt`, followed by resetting to `1 / fire_rate` after a shot, discards overshoot. The movement/cadence experiment belongs to `work/scout-progression-audit/results.json`; I did not rerun that probe. Its isolated candidate receives a separate independent review.

No XP/currency duplication, repeated craft cost, ownership bypass through ordinary commands, or Sovereign intrinsic-stat leakage was observed in the bounded checks below. Global six-hardware inheritance is intentional: higher permanent ranks remain active when switching to a fresh ship, while new purchases use that selected ship's level. Acquired signature strength is separately capped by its selected XP. Local import accepts structurally valid solo checkpoints; it is not an authoritative anti-cheat service.

## XP and affordability

The actual level1–12 boundary checks agree with cumulative floor `10 × (L−1) × L`:

| Level | XP floor | Next XP | Purchase rank cap | Scout drive gain before bought drive |
| --- | ---: | ---: | ---: | ---: |
| 1 | 0 | 20 | 1 | 0% |
| 2 | 20 | 60 | 1 | 2.5% |
| 3 | 60 | 120 | 2 | 5% |
| 4 | 120 | 200 | 2 | 7.5% |
| 5 | 200 | 300 | 3 | 10% |
| 6 | 300 | 420 | 3 | 12.5% |
| 7 | 420 | 560 | 3 | 15% |
| 8 | 560 | 720 | 4 | 17.5% |
| 9 | 720 | 900 | 4 | 20% |
| 10 | 900 | 1100 | 4 | 22.5% |
| 11 | 1100 | 1320 | 4 | 25% |
| 12 | 1320 | 1560 | 5 | 27.5% |

Scout drive is100 speed/260 acceleration times the XP factor and `(1 + 0.04 × drive rank)`. XP drive stops increasing at12; later levels have no further drive/rank reward in current content. Crafted ships start from180/400 before family bonuses; their XP starts independently atzero. Sovereign retains fixed intrinsic rank-five drive,248.4 speed/480 acceleration, with a minimum displayed level12 and no extra XP multiplier. These match current campaign documentation. Their human feel is not established by formulas.

Each rank costs ten matching category points. Ordinary enemies grant5 XP and have20% chance of one random-category point; asteroids grant2 XP and have25% chance of one random-category point independently of25% material chance. Bosses drop one point of each category; champions directly grant three matching points once; completed site events grant three matching points once. Point pickups must actually be collected.

Under an explicitly simplified uniform-drop model, collecting every drop and excluding bosses/quests/champions, ten points of one category take approximately150 ordinary defeats on average, yielding750 XP (a level9 threshold), or120 asteroids yielding240 XP (level5). Six materials alone average24 asteroid defeats, but the16-capacity hold requires banking for retained construction stock. These are analytical expectations, not measured session length, seed results, natural balance or guaranteed income. They explain why affordability can trail the earliest XP gates; human progression testing is needed before tuning either curve.

## Focused actual checks

The native isolated proof passed **88 assertions**. It checked every level1–12 floor and just-below boundary, every rank purchase floor0/60/200/560/1320, just-below level and nine-point rejection with the complete profile unchanged, exact cost/rank/revision success, craft rejection/success/repeat atomicity, dock-only operations, fresh crafted XP, Scout→Dawnneedle→Scout independent XP/global hardware/signature cap, same-ship idempotence, reload equality, and Sovereign→Scout→Sovereign intrinsic isolation. Its level100 caption reproduction is recorded separately from passes in `native-results.json`.

The actual native snapshots then passed **25 strict TypeScript/SaveStore assertions** using `fake-indexeddb`: crafted/switch/return persist exactly, caller mutations cannot change stored XP, equal-revision XP/currency rewrites are ignored, malformed XP and unowned selected ships are rejected before import, and a valid winning Sovereign checkpoint imports/reloads exactly. This exercises transactions and the real validator, not physical storage lifecycle.

Reproduction with the identified compiled library:

```powershell
rustc --edition=2024 work/scout-progression-qa/main.rs -L dependency=target/debug/deps --extern game_core=target/debug/deps/libgame_core-684f3d0a8e530f28.rlib --extern serde_json=target/debug/deps/libserde_json-b89616ee5c7ce7b6.rlib -o work/scout-progression-qa/progression-proof.exe
./work/scout-progression-qa/progression-proof.exe
node --experimental-transform-types work/scout-progression-qa/save-check.mjs
```

These commands depend on that existing production-matched local native build. Source/output fingerprints, generated reports and harnesses remain isolated under `work/`.

The existing campaign tests cover selected-ship XP kills, level-gated global purchases, atomic craft/material storage, death retention, once-only league rewards and intrinsic Sovereign state. Existing storage tests cover version migrations, ownership/XP/cargo validation, whole campaign reload and stale writes; campaign DOM tests distinguish losing/winning season actions. Those tests were read, not broadly rerun here. Natural early Scout flight, point gathering, full campaign/boss play, human tuning, named-phone60FPS and physical lifecycle remain pending.
