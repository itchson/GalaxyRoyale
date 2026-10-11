# Diagnostic progression audit

**No reward leak reproduced; no production fix justified.** Frozen source is Root `65ecaf288d9abd55149beb3aa0a6ae40c9e1944d`, clean isolated branch `codex/qa-progression-audit`. The original uncontrolled browser observation remains unattributed. This audit does not reconstruct its later input, PWA transitions, SaveStore history or current progressed profile.

## Exact fresh-license replay

The issued browser identity `DORA-PAN-RENKO-7F8A52DAECE2ED52` was constructed through the real public `identity::create_profile()`. Its canonical lowercase hash derives **seed 2,371,994,270**. The native Game starts at LV1/XP0, points `[0,0,0]`, empty materials and acknowledged license stage 1.

The probe then used public `diagnostic("warp 3000 0")`, `diagnostic("combat")`, actual hostile ticks until death, the shell's actual pause/Retry-resume behavior, `diagnostic("boss-coil-1")`, another actual hostile death and Retry. Retry does **not** perform an additional core reset: death already respawns the Game, the browser pauses it, then Retry resumes it. Matching this exact seam avoids hiding leftover state with a second cleanup.

| Main manufacturer | Combat death tick | Coil1 death tick | Final after wait/reload |
| --- | ---: | ---: | --- |
| PAN (exact issued identity) | 539 | 928 | LV1 / XP0 / points 0,0,0 |
| RENKO (same declared hash/world) | 553 | 930 | LV1 / XP0 / points 0,0,0 |
| HUGO (same declared hash/world) | 662 | 983 | LV1 / XP0 / points 0,0,0 |

Each case additionally ran **36,000 fixed 1/60 ticks (ten simulated minutes)** stationary at Haven after Retry, alternating real fire input with finite aim. Actual current primary `shot` event count was zero. Every tick compared the **entire serialized Profile**, including revision, XP, currency, onboarding, discovery, histories, material labels, inventory and loadouts, against the initial checkpoint. Every comparison passed. Constructor reload also preserved the exact Profile. Eight complete checkpoints per case are archived in `public-checkpoints.json`.

## Actual diagnostic victory and separate natural flight

A separately declared already-qualified Sovereign checkpoint (XP1,320 per hull and points13/17/19; clearly a fixture, not earned or granted by a command) used public fire/aim/joystick input against a real Coil1 diagnostic. The actual primary/signature projectile path defeated it at tick **182**: 28 primary `shot` events,52 hit events, one explosion and no player death. Its **entire Profile remained identical** through the fight, another1,200 active-scene ticks, explicit reset and reload. No kill/damage/grant/private ECS helper was called by the probe.

A third, intentionally separate case used the exact fresh PAN identity, diagnostic death/Retry, then **declared normal controls**: eastward travel with fire for1,100 ticks, followed by swept aim/fire for the remainder of180 simulated seconds. It earned **7 real XP**, one ordinary Bastion defeat and three asteroid defeats, had one real death, and retained7XP on reload. This fixture collected **zero points/materials**; it is not evidence for the browser's particular5/5/2 points. The real last-run report records7XP, one enemy, three asteroids and a2,285.88-unit distance peak. This proves ordinary progression is restored after Retry; it does not attribute the browser's later gains.

## Source path and intended boundaries

- `lib.rs:954` `progression_enabled()` rejects diagnostic_scene, boss_qa_scene, stress and diagnostic arenas. `diagnostic()` sets the independent transient scene flag before immediate cleanup/setup; valid fixtures keep it. Invalid commands restore the old flag and do not discard the real run.
- `lib.rs:2390` `kill_actor()` exits before kill credit, XP, boss counters, pity and loot spawning when isolated. Diagnostic ordinary bosses have an additional component/boss-scene guard. `lib.rs:2606` `update_pickups()` removes isolated pickups before any currency/material credit.
- `campaign_runtime.rs:43/79/98/113` also gate banking, XP, points and run kill counts. Exploration, onboarding/training and champion contacts use the same progression boundary. Diagnostic arena outcomes carry their own explicit diagnostic flag.
- `lib.rs:1281` actual/forced death resolves once through `respawn_if_dead()`. It calls `reset_to_dock()`, which removes brains/segments, bullets and loot, resets course/motion/camera, and holds diagnostic isolation through dock cleanup. Only at its end (`lib.rs:3141`) is the transient flag cleared. **Subsequent natural play is intentionally normal**, including after browser Retry. Physical return to Haven during a still-active fixture does not clear the flag.
- `ship_progression.rs:193` constructor reload serializes progression only, drops unbanked material cargo and rebuilds at Haven. Arbitrary fixture motion/entities/diagnostic flags are not persisted. Reload therefore cannot retain a combat fixture as a reward-bearing run. Real earlier earned assets and reports are retained; starting a diagnostic while a real expedition is active can legitimately abandon cargo/finish that prior report or forfeit a league match. Those are existing explicit abandonment rules, not free awards.

Read-only shell review confirms `showDeath()` pauses the already-respawned Game and resets shell inputs/course; Retry is `resume()`. The console commands clear shell destination. Root separately reported later uncontrolled numeric-index navigation, a persistent destination and an emptied passive slot, plus a later increase to52XP/cargo2. Those are observations from Root, not rerun here. They make subsequent natural flight plausible, but no exact attribution is possible without the missing timeline/checkpoints.

## Evidence, reproduction and limits

`cargo build -p game_core --locked --release` produced the real source library. `run.py` verifies clean exact HEAD, pins committed core/engine/Cargo bytes, and links the public probe with rustc optimized ThinLTO and `-D warnings`. Its final compile/run succeeded. **24 retained diagnostic-filter native tests also passed** (301 filtered); `FOCUSED_TESTS.md` records that separate execution. No full suite, Clippy, WASM, public package, server, browser, Studio or save DB execution is claimed here.

The initial debug probe used an extra reset and was stopped before completion; it is not accepted evidence. The first direct release link lacked the required ThinLTO option and failed at the linker; that compile log is preserved. One preliminary successful run counted the retired name `shoot` for the Haven-silence assertion. That harness error was corrected to the actual current **`shot`** event and the final complete probe rerun; the earlier source/checkpoints/log are preserved with `first-wrong-quiet-event-*` names, not treated as the final assertion evidence. None of these harness repairs changed production source or Profile rules.

The simulation duration is native synthetic elapsed time, not a physical ten-minute browser soak or performance result. Frame/raster/audio/PWA behavior and uncontrolled UI navigation were not exercised. This audit supports the existing diagnostic isolation contract and normal-after-Retry behavior; it does not prove the original browser timeline, nor claim human gameplay acceptance. Runtime source remains unchanged.
