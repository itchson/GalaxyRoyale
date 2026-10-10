# Haven Scout progression audit

The shipped Scout implements the requested slow start and gradual **drive** improvement. Its level-one speed/acceleration are 100/260 world units; XP adds 2.5% of those baselines per level through level twelve. Hull, shield, primary damage, fire rate and regeneration require category purchases. One real purchase defect emerged: fixed-60Hz primary cadence has plateaus at fire-rate ranks two and five.

This is a read-only source/native-simulation audit of the already-built production core at `9c290ae8fa7baa1a993a867a5e0d0178e0a8bde1`. No production edits, dependencies, manifests or publication were made. The work-only probe linked the existing `libgame_core-684f3d0a8e530f28.rlib`; it did not substitute a simplified controller. Raw evidence is [results.json](C:/Users/mitch/Documents/ChatGPT/galaxy-royale/work/scout-progression-audit/results.json), with [probe.rs](C:/Users/mitch/Documents/ChatGPT/galaxy-royale/work/scout-progression-audit/probe.rs). Native fixtures do not establish browser input, human difficulty or physical-phone performance.

## Actual source contracts

- [campaign.rs:36](C:/Users/mitch/Documents/ChatGPT/galaxy-royale/crates/game_core/src/campaign.rs:36): total XP floor for level L is `10 × L × (L−1)`; next floor is `10 × L × (L+1)`. Level caps at 100. Purchasable rank gates are levels **1 / 3 / 5 / 8 / 12**.
- [lib.rs:295](C:/Users/mitch/Documents/ChatGPT/galaxy-royale/crates/game_core/src/lib.rs:295): Scout speed is `100 × [1 + .025 × (min(L,12)−1)] × (1 + .04 × drive_rank)`; acceleration replaces 100 with 260. XP does not increase the other basic stats.
- [lib.rs:2563](C:/Users/mitch/Documents/ChatGPT/galaxy-royale/crates/game_core/src/lib.rs:2563): Haven-only purchase costs ten matching points, requires the next rank's selected-ship level, and is capped at rank five. A successful purchase repairs hull/shield; it does not award XP.
- [campaign_runtime.rs:26](C:/Users/mitch/Documents/ChatGPT/galaxy-royale/crates/game_core/src/campaign_runtime.rs:26): kill XP belongs to the currently selected ship. XP is retained across return/death/reload. Ordinary enemies award five, asteroids two, ordinary boss generations one through five award 25/35/45/55/65. A champion's first legitimate win awards 40; each won league match awards 30 once.
- Basic installed ranks are global and stay effective when switching to a low-XP ship. Only the **next purchase** is level-gated. The three ordinary boss signatures additionally use `min(installed_rank, selected_ship_rank_cap)`; their generation-derived evolution is separate. This difference is deliberate in the source/tests, not evidence of a bypass.

## Level one through twelve, drive ranks zero through five

Each cell below is **speed / acceleration**. The rank is the single drive node, not all hardware. Higher-than-cap cells cannot be bought on that ship at that level, but can be effective inherited hardware purchased while using another sufficiently leveled ship. The actual native fixtures explicitly granted equal basic ranks for measurement; they are not claimed as earned first-run progression.

| Level | Total XP | Purchase cap | R0 | R1 | R2 | R3 | R4 | R5 |
| --- | ---: | ---: | --- | --- | --- | --- | --- | --- |
| 1 | 0 | 1 | 100.00 / 260.00 | 104.00 / 270.40 | 108.00 / 280.80 | 112.00 / 291.20 | 116.00 / 301.60 | 120.00 / 312.00 |
| 2 | 20 | 1 | 102.50 / 266.50 | 106.60 / 277.16 | 110.70 / 287.82 | 114.80 / 298.48 | 118.90 / 309.14 | 123.00 / 319.80 |
| 3 | 60 | 2 | 105.00 / 273.00 | 109.20 / 283.92 | 113.40 / 294.84 | 117.60 / 305.76 | 121.80 / 316.68 | 126.00 / 327.60 |
| 4 | 120 | 2 | 107.50 / 279.50 | 111.80 / 290.68 | 116.10 / 301.86 | 120.40 / 313.04 | 124.70 / 324.22 | 129.00 / 335.40 |
| 5 | 200 | 3 | 110.00 / 286.00 | 114.40 / 297.44 | 118.80 / 308.88 | 123.20 / 320.32 | 127.60 / 331.76 | 132.00 / 343.20 |
| 6 | 300 | 3 | 112.50 / 292.50 | 117.00 / 304.20 | 121.50 / 315.90 | 126.00 / 327.60 | 130.50 / 339.30 | 135.00 / 351.00 |
| 7 | 420 | 3 | 115.00 / 299.00 | 119.60 / 310.96 | 124.20 / 322.92 | 128.80 / 334.88 | 133.40 / 346.84 | 138.00 / 358.80 |
| 8 | 560 | 4 | 117.50 / 305.50 | 122.20 / 317.72 | 126.90 / 329.94 | 131.60 / 342.16 | 136.30 / 354.38 | 141.00 / 366.60 |
| 9 | 720 | 4 | 120.00 / 312.00 | 124.80 / 324.48 | 129.60 / 336.96 | 134.40 / 349.44 | 139.20 / 361.92 | 144.00 / 374.40 |
| 10 | 900 | 4 | 122.50 / 318.50 | 127.40 / 331.24 | 132.30 / 343.98 | 137.20 / 356.72 | 142.10 / 369.46 | 147.00 / 382.20 |
| 11 | 1100 | 4 | 125.00 / 325.00 | 130.00 / 338.00 | 135.00 / 351.00 | 140.00 / 364.00 | 145.00 / 377.00 | 150.00 / 390.00 |
| 12 | 1320 | 5 | 127.50 / 331.50 | 132.60 / 344.76 | 137.70 / 358.02 | 142.80 / 371.28 | 147.90 / 384.54 | 153.00 / 397.80 |

The other node curves are independent of Scout level. Columns treat each node's own rank separately; relics can add up to 20 hull. Regeneration starts only after five undamaged seconds. Shield recharge starts after four undamaged seconds, at 15% of maximum shield per second.

| Node rank | Primary damage | Authored shots/s | Hull | Shield | Hull regen/s |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 0 | 12.00 | 5.0 | 100 | 0 | 0.0 |
| 1 | 13.44 | 5.5 | 115 | 20 | 0.4 |
| 2 | 14.88 | 6.0 | 130 | 30 | 0.8 |
| 3 | 16.32 | 6.5 | 145 | 40 | 1.2 |
| 4 | 17.76 | 7.0 | 160 | 50 | 1.6 |
| 5 | 19.20 | 7.5 | 175 | 60 | 2.0 |

## Observed drive and braking

All **72 actual Game::tick drive fixtures** (12 levels × 6 ranks) reached the tabulated speed, within 0.00023 world units/s. Each started at Haven with the normal north-facing ship, applied a full east stick for 0.75 seconds, then released. Every fixture stayed safe, took zero damage and kept XP unchanged. Time to 95% of maximum was **0.417s** for every case: speed and acceleration scale together, so experience increases travel speed rather than shortening this normalized acceleration time.

| Fixture | Actual peak speed | Release to <0.1 speed | Coast after release | Extra forward travel on 180° reversal | Reverse to 95% opposite speed |
| --- | ---: | ---: | ---: | ---: | ---: |
| L1 / drive R0 | 100.00 | 0.333s | 15.85 | 25.64 | 0.850s |
| L12 / drive R0 | 127.50 | 0.333s | 20.20 | 32.69 | 0.850s |
| L12 / drive R5 | 153.00 | 0.333s | 24.24 | 39.23 | 0.850s |

Actual mobile pointer-up sets the joystick to null, selecting the release path: throttle reaches zero over 0.15s while velocity decelerates afterward. A held **centered** stick follows exponential throttle smoothing instead; three separate fixtures took 0.383–0.400s and coasted 18.38/23.45/28.14 units. Those two inputs are intentionally distinguishable in the evidence. Reversal heading turns, but forward momentum lasts 0.467s before X velocity becomes negative; lateral travel was 16.26/20.73/24.88 units. That is real arcade drift, and the bigger absolute stopping envelope at higher speed requires human touch evaluation.

Nine real single-click courses to 40/120/200 world-unit targets settled without overshoot. L1/R0 took 0.783/1.583/2.383s; L12/R0 0.683/1.317/1.950s; L12/R5 0.633/1.167/1.683s. Final error was 0.876–1.262 units, inside the authored four-unit arrival tolerance. These are unobstructed Haven courses, not asteroid-navigation or browser-camera acceptance.

## Measured primary cadence discontinuity

The six five-second firing probes used the actual public diagnostic fight/shot path, a Scout with explicitly inherited armor/shield/regen R5, variable firing rank, and rightward micro-thrust. Every probe remained at 175 hull and kept ship XP unchanged. The counted `shot` event marks each actual primary firing invocation, with an unsaturated projectile budget. There were no signatures to add extra shots.

| Fire-rate rank | Authored shots/s | Observed interval, ticks at 60Hz | Observed steady shots/s | Primary shots in five seconds |
| ---: | ---: | ---: | ---: | ---: |
| 0 | 5.0 | 13 | 4.615 | 24 |
| 1 | 5.5 | 11 | 5.455 | 28 |
| 2 | 6.0 | 11 | 5.455 | 28 |
| 3 | 6.5 | 10 | 6.000 | 30 |
| 4 | 7.0 | 9 | 6.667 | 34 |
| 5 | 7.5 | 9 | 6.667 | 34 |

[lib.rs:951](C:/Users/mitch/Documents/ChatGPT/galaxy-royale/crates/game_core/src/lib.rs:951) decrements the timer each fixed tick; [lib.rs:1116](C:/Users/mitch/Documents/ChatGPT/galaxy-royale/crates/game_core/src/lib.rs:1116) resets it to a complete `1 / rate` period. This discards overshoot and exposes floating-point boundary delay. Consequently **R1→R2 and R4→R5 cost ten offensive points but give zero primary cadence improvement at 60Hz**. Starting nominal primary throughput 60 hull/s is actually approximately 55.38 before misses/mitigation in this fixture. This is an implementation finding, not an argument to increase enemy difficulty or change authored weapon balance.

Next correctness work should preserve timer overshoot with a bounded one-primary-shot-per-fixed-tick policy, handle inactive/sanctuary cooldown without a banked burst, and prove strictly increasing long-window cadence for all six ranks. It needs its own production edit/review; none was made here.

## Switching and progression limits

Crafted ships start at their own XP, while inherited global hardware remains active. Their base drive is **180/400**, so a new ordinary craft is a deliberate step above the Scout's base, even if the crafted ship is level one. Bulwark is 165.6/400 plus 20 hull; Windrunner is 198/460; Ironwake adds ten hull. Other crafted baseline drives are 180/400. The same XP multiplier and purchased drive bonus apply to them. This is a ship-reward discontinuity, not continuous Scout leveling.

A validated Royale Sovereign has intrinsic hardware/signatures rank five and all signature evolution tier five. Its no-relic baseline is **240 hull / 100 shield / 248.4 speed / 480 acceleration / 24 primary damage / authored 9 shots/s / 3 regen/s**, plus its radial burst and repair. It has a minimum displayed level twelve, and does not acquire additional drive from XP. Selecting it does not overwrite ordinary ranks/unlocks/currency. Returning to Scout restores Scout XP/stats rather than copying the intrinsic ranks. Ownership requires legitimate persisted campaign victory and is sanitized against the actual table.

Switching uses [campaign_runtime.rs:180](C:/Users/mitch/Documents/ChatGPT/galaxy-royale/crates/game_core/src/campaign_runtime.rs:180) and [lib.rs:2478](C:/Users/mitch/Documents/ChatGPT/galaxy-royale/crates/game_core/src/lib.rs:2478): it is Haven-only, rebuilds the selected hull, repairs it and clears all enemies/projectiles. This prevents a live shot from an old ship earning XP after switching. The independent QA agent additionally exercised purchase atomicity, crafting, per-ship XP, inherited/capped ranks, Sovereign return and persistence; its evidence is separately owned. No new farm/switch duplication exploit was demonstrated by this audit.

There are two presentation plateaus to state honestly. Beyond level twelve, additional XP levels do not improve basic drive or purchase caps. At level 100 (99,000 total XP), `next_xp(100)` remains 101,000 and the menu still says “next level,” although level 101 is unreachable; the QA agent independently reproduced that public-HUD issue. A low-XP crafted ship can display an installed signature's global rank above its current effective rank. Showing effective rank/cap would make that intentional rule clearer, without weakening permanent hardware.

## Early natural-run evidence and limited next tuning recommendation

Six new **actual natural** 30-second launches used fresh profiles, three declared seeds (`0x47524f59`, 1, 17), the default 533×300 world viewport and constant full-east stick, each with/without held fire. They used no teleport, reward grant or boss diagnostic. This is an intentionally unskilled straight-input probe, not a human success rate.

| Seed | Held fire | Belt time | First combat time / hull | XP at 30s | Collected category points | Minimum hull |
| --- | --- | ---: | --- | ---: | --- | ---: |
| Default | No | 7.23s | 14.63s / 76 | 0 | 0 / 0 / 0 | 26.8 |
| Default | Yes | 7.23s | 14.23s / 100 | 8 | 0 / 1 / 0 | 92 |
| 1 | No | 7.42s | 14.42s / 92 | 0 | 0 / 0 / 0 | 75 |
| 1 | Yes | 7.23s | 14.23s / 92 | 6 | 0 / 1 / 0 | 92 |
| 17 | No | 7.23s | 14.47s / 84 | 0 | 0 / 0 / 0 | 41 |
| 17 | Yes | 7.23s | 14.23s / 100 | 13 | 1 / 0 / 1 | 100 |

All six survived the finite probe; none reached level two or ten matching points. Held fire destroyed actual rocks along these particular paths, reducing asteroid contact losses. These six tracks cannot establish typical first-upgrade time, player strategy, Coil difficulty or level pacing. The prior 45-case boss study remains an artificial stationary/telemetry targeting study: same-Scout G1 floors are already level four after 24 base kills, and its Coil losses show that human turning/segment targeting needs explicit testing, not a blanket difficulty change.

Analytical affordability is a stronger next question than increasing the drive curve: absent other income/pickup loss, ten matching points take on average 150 ordinary kills (750 XP, level nine) or 120 asteroids (240 XP, level five). Bosses award one point of each category; missions/champions award three matching points, so intentional mission use can change that significantly. These expectations use actual drop probabilities and do not predict a human session's duration.

After fixing cadence and the false max-level prompt, keep the 100/260 start and current drive curve unchanged for the first human comparison. Measure fresh runs' **time/XP at first collected ten-category-point purchase**, hull on first combat entry, voluntary returns and whether the 20-XP level-two drive improvement is perceptible. If affordability consistently trails the unlocked ranks, compare one clearly disclosed onboarding reward/mission improvement with the current income, rather than unmeasured general speed or enemy nerfs. Physical landscape-touch braking/aiming, real phone frame time and human G1 Coil acceptance remain Pending.

## Reproduction identity

The existing core rlib SHA256 used here was `5AEC559EA5A1DC1C423FAC226CA300CBED2AB7438EAF6CBE0E98F8EC099DE843`. Read source hashes at measurement: `lib.rs` `E2EF687C9F14D8B75AF6B36FBB220B8244979FF35A042D8B0CE6D579FABD6702`; `campaign.rs` `CE3701246D366042841CCA3A61CE1C83EA15BBFCD312A05FE5D6D7F991D0B5BC`; `campaign_runtime.rs` `BEAA27A19AF8C62A23DF0B6D6F820E924AF51ED40F1D1E78A5CAC25F8D7A4E58`.

Actually executed successfully, without a new Cargo manifest:

```powershell
rustc --edition=2024 work/scout-progression-audit/probe.rs -L dependency=target/debug/deps --extern game_core=target/debug/deps/libgame_core-684f3d0a8e530f28.rlib --extern serde_json=target/debug/deps/libserde_json-b89616ee5c7ce7b6.rlib -o work/scout-progression-audit/probe.exe
.\work\scout-progression-audit\probe.exe
```

The probe passed 72 drive fixtures, three centered-stick stops, nine click courses, three reversals, six cadence probes and six natural launches. Rebuilding against changed production source is a new observation; preserve this result as evidence of the measured version.
