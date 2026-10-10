# Radiation Mote: isolated actual-core candidate

Status: **V2 / review repair1** frozen for independent Engine/QA review. **Not integrated or published.** This directory is the only output of this task. No production file, shared manifest, dependency, browser setting or save schema was changed. The original V1 sources/artifacts/evidence are preserved under `v1/`; the intermediate V2 timing checkpoint is preserved under `v2-timing/`. V1's measured20-tick pulse spacing was a timing defect and is not accepted as the authored21-tick behavior.

The source base is `crates/game_core/src/lib.rs` SHA256 `0a574a35c9d23f955fff05246da2cf4bf9a8065f5db58c63c365124f982337e6`, after the accepted weapon-cadence fix and **before** root's later early-death helper. Those two work streams must be composed explicitly. This candidate was not silently rebased. `source-hashes.json` preserves the baseline; `frozen-hashes.json` pins candidate code, art, native evidence and read-only linked source. Final V2 library SHA256 is `a46a1821641aaa1c68deb7eeb4805838129abf717bce2a863d3c04dadb6bdcdd`; final freeze manifest SHA256 is `8b2a1180863dfe9106576c5617a6fff4d2daaea898c98a774e7be8b854b260ed`.

## Scope and gameplay

The Mote replaces one eligible ordinary **Dart / family0** spawn born in **Radial Wastes / biome4**. Its immutable selection hashes the actual spawn grid, seed and existing `Brain.phase` bits; one-in-four eligible tickets select it. Selection consumes no extra RNG and never changes at biome borders. It excludes bosses, champions, arenas, existing variants, ordinary QA spawning, explicit stress mode and out-of-domain birth positions.

The original authored40×40 tri-pylon body has712 occupied cells, three disjoint emitter masks of142/143/143 cells,15 defined opaque palette colors and11 colors actually used. Runtime uses indexed cell quads, never the PNG preview. Source and provenance are copied unchanged from the frozen original prototype except the provenance now distinguishes defined versus used colors. There is no AI image/model claim.

Body scale is0.6; radius is derived from occupied geometry, approximately13.820534 world units. Ordinary spawned tier, hull and contact damage remain unchanged. No environmental radiation damage, heat, cooling, burn, bonus loot, new XP source, second spawning system or saved variant state is introduced.

The attack has a1.1s charge, with its player-facing escape wedge locked for the final0.35s. Six rays lie at80/120/160/200/240/280 degrees relative to that wedge; they leave a120-degree indicated gap with20-degree boundary buffers. Two pulses occur at attack0 and0.35s, then a4.4s recovery follows the0.5s attack. Rays travel110 world units/s, deal0.6×the ordinary contact damage and use the existing hostile four-second lifetime. At declared tier1, this is5.4 damage per ray. These are authored starting values, not human balance acceptance.

Each ray belongs to its nominal pylon. Its muzzle is a surviving occupied cell of that pylon, transformed by the same heading+quarter-turn and scale used for body collision. Partial damage moves that muzzle to a surviving cell; destroying its complete mask removes its rays and lane cues. A different pylon never silently replaces it.

V2 accumulates transient duration clocks in f64 and starts the second-pulse interval from the first scheduled pulse's actual tick. Carried charge overshoot therefore cannot shorten the real interval. It remains one scheduled pulse per tick, including a consumed suppressed pulse; there is no catch-up volley. No timer or phase is saved.

## Camera and safety policy

The fixed180-unit orbit in the pure prototype could never fully admit its warning at533×300 UI / zoom1.6. This candidate uses a bounded approach-and-hold controller instead:

- At most five stable candidate positions: the preferred180-unit player-relative point clamped into the camera rectangle, plus four corners in birth-determined order. Closest legal position wins; a legal retained target stays stable.
- The rectangle is the intersection of current and predicted end-of-tick camera views. The host predicts the existing follow using the actual controlled player velocity and unchanged follow coefficient; it never changes the camera, viewport or zoom.
- Target and direct approach path reject the physical player radius+Mote radius+8-unit clearance and sanctuary radius+Mote radius+6-unit clearance. Steering removes an inward velocity component that would overshoot player clearance before the shared integration.
- The whole conservative radius+35-unit cue envelope,2-unit guard and a5-unit/current-larger shake allowance must fit. Reduced Motion needs no shake allowance. Candidate points are inset by0.125 world units to prevent far-world f32 corner rounding from invalidating a legal target.
- Warning starts only within3 units of the target and below8 units/s. Current and predicted next-step cue bounds are checked before every pulse. Losing a legal target or warning visibility cancels the warning/attack and requires a fresh charge; suppressed pulses do not become burst debt.
- A viewport that cannot fit the complete cue remains nonfiring and continues bounded movement. The120×80 world fixture explicitly exercises this policy; the twelve tested supported extents all admit real shots.

Haven plus the relevant deterministic site neighborhood remain world-space sanctuaries. Distant actors fetch their own neighborhood; the player's global refuge result is independent of that cache. The ordinary shared post-integration body guard and every-lifetime swept bullet boundary still apply. Retreat/range retirement uses the existing removal path without kills, rewards or boss queue mutation. Invalid controller input retires the enemy rather than leaving a frozen threat.

The conservative camera prediction covers the measured moving-player fixtures. An unrelated collision or a larger same-tick impact can change the final player/camera pose after this controller ran; native fixtures do not establish every possible combined collision/impulse case. Browser review must inspect actual warnings during zoom easing, camera shake and crowded contact. The safety and damage guards remain authoritative even when a visual warning is interrupted.

## Minimal integration contract

`integration.patch` contains **runtime lib.rs hooks only**, anchored to the stated baseline. Root owns composing them with later source. It adds one private module, one ordinary spawn hook using `regions::at(seed, spawn)`, one update before generic enemy AI/shared integration, an optional ECS component that skips generic AI, one actor extraction override with cue-aware culling, and one explicit diagnostic dispatch. It changes no public Profile/Hud/input fields.

Copy these candidate sources when independently accepted:

| Candidate file | Proposed production destination |
|---|---|
| `radiation_enemies.rs` | `crates/game_core/src/radiation_enemies.rs` |
| `mote_controller.rs` | `crates/game_core/src/mote_controller.rs` |
| `mote_art.rs` | `crates/game_core/src/mote_art.rs` |
| `radiation_enemy_tests.rs` | `crates/game_core/src/radiation_enemy_tests.rs` |
| `assets/mote-*` | `assets/pixels/mote-*` |

For that destination, change the sole atlas include in `mote_art.rs` from `include_str!("assets/mote-atlas.json")` to `include_str!("../../../assets/pixels/mote-atlas.json")`. Other relative module paths already match the proposed files. There is no new Cargo dependency. Root owns any browser console help/whitelist addition for `biome-mote` and the final dual-WASM build.

`diagnostic("biome-mote")` resets to a deterministic nearby Radial Wastes fixture, clears nearby rocks, selects an actual eligible spawn ticket and enables the existing entire-scene reward-isolation flag. Actual damage, bullets, cells and phases stay live. It does not fake a win, modify saved pending boss reservations or award currency/XP. Reset/return home clears that scene flag.

Shots reserve remaining slots across Motes within the same ECS update after existing Riveter commands have been dispatched; `spawn_bullet` remains the final shared180/300 hard cap. Each pulse consumes its phase slot even if capacity or destroyed emitters suppress it. Motion is integrated exactly once. Body/render extraction always uses the current damaged body and interpolated world pose with0.75 light response,0 emission; cosmetic density does not remove actors or warnings.

All ordinary kills retain the existing five XP, one Dart kill/24-kill credit, accepted family bounty progress and ordinary random collectible drop rule. No kill path is duplicated in the new module. Repeated removal/kill callbacks after a real defeat retain once-only behavior.

## Reproduce isolated evidence

From the repository root:

```powershell
powershell -NoProfile -File work/radiation-production-candidate/build.ps1 -FrozenSource
work/radiation-production-candidate/mote-tests.exe radiation_enemies::tests --nocapture
```

The preparer without `-FrozenSource` intentionally rejects changed production lib.rs rather than silently rebasing. The frozen build uses the stored actual Game copy and existing compiled dependencies; unchanged linked module fingerprints were verified when freezing. No manifest was added.

Independent test modules can be appended to a separately owned candidate-lib copy as crate-child `#[cfg(test)]` modules. They can inspect the actual private Game ECS and `radiation_enemies::Mote.controller`; birth/phase-age/target getters exist only for tests. Public native API remains Game tick/input/render/profile plus the explicit diagnostic command. This access is test scaffolding, not a new browser or persistence contract.

## Observed evidence and pending work

The final V2 candidate's **16 focused actual-Game tests pass**;82 existing baseline tests compile but were filtered rather than rerun. Strict native `clippy-driver -D warnings` passes for both the library and complete test compilation. `native-proof.log` records the actual result.

`viewport-proof.json` records twelve stationary fresh-Scout fixtures: UI533×300/720×360/400×240/320×180 at zoom0.6/1/1.6. All fire after112–205 ticks (1.87–3.42s) from a head born210 units above the player, with complete cue bounds and a measured minimum physical clearance above8 units. The narrowest320×180/1.6 case fires at205 ticks with minimum gap9.4576 units. These fixtures use tier1 bodies, stationary players and cleared unrelated threats; they are visibility/physics evidence, not natural difficulty evidence.

`timing-proof.json` records actual emitted hostile entities, not a duplicate timer implementation. From fresh charge,1/120s steps emit first/second at ticks132/174;1/60s at66/87;0.05s at22/29. Fresh charge is at least1.1s and actual spacing at least0.35s in all three cases. A declared irregular sequence also passes with charge1.112008s and spacing0.353003s. At most six new rays appear in any tick. The added stress test proves an actual eligible ticket is excluded without consuming its identity/RNG or granting progress, then remains eligible when stress mode is disabled.

The moving-player maximum-zoom fixture measures post-follow and interpolated camera bounds at1/60 and0.05s ticks. During its declared eight-second full-joystick reversal path,1/60 produces34 visible warning/attack ticks and intentionally interrupts all pulses;0.05 produces48 visible ticks and two actual emitting ticks. The test verifies those real emitting ticks as well as cancellation. This does not measure human dodge success or acceptance of time-step-dependent encounter feel.

Other focused evidence includes128 real normal spawn-hook iterations; immutable birth/no extra RNG; a real primary-projectile Mote defeat granting exactly5 XP, the24th Dart kill/pending reservation and one actually accepted bounty increment; no direct damage/grant for that defeat; repeated callback isolation; actual emitted projectile hull damage and ship-cell removal; actual shield absorption/Nova reflection of surviving emitted rays; damaged-pylon origins; shared multi-entity cap and no delayed replay; sanctuary body and across-site swept boundary guards; diagnostic actual-defeat/retreat no rewards; invalid retirement; and120 reused extractions with finite bounded packets and unchanged profile/time.

Final independent review pins this V2 manifest. QA reports **8/8** separately owned actual-core probes pass, including66→87 pulse timing, stress-mode ordinary spawn exclusion, two real-shot ordinary defeats preserving XP/bounty/G1 queue, diagnostic exact-profile/reload isolation, shared primary/Mote cap, gap lock and time/viewport edges. Its actual native checkpoint passes current strict TypeScript profile normalization and SaveStore persistence/reload/duplicate-ignore checks. Negative V1 timing and stress evidence remain preserved in its owned directory.

Engine reports the candidate's16 tests and **4/4** separately owned native probes pass: actual live-cell/material packing and nominal-emitter destruction at rotated ±999k coordinates, current seeded sanctuary pre/postguard parity, and portable far-world maximum-zoom approach. No additional source repair was requested. Those reports are separate artifacts under `work/radiation-production-qa/` and `work/radiation-production-engine-review/`; they do not establish browser rendering or phone FPS.

Root owns production composition, complete native regression, both WASM renderers, browser presentation, saved-profile reload behavior after integration and deployment. Human balance, physical Pixel6a/iPhone12 performance and touch-readability acceptance remain **Pending**. No FPS or playtest score is inferred from these portable tests.
