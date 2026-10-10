# Next physical-device and natural Scout playtests

This is an unexecuted worksheet and protocol. Every manual acceptance row below starts **Pending**. Fill observations from named devices and actual play; an empty cell is not a pass. Existing native, IndexedDB and two-backend WASM proofs remain valid for their declared automated scopes. Representative desktop browser captures do not pass this worksheet.

## Build and session record

Copy this block for each device, browser, backend and test session. Use the final published HTTPS game URL supplied by the Producer; `127.0.0.1` on a phone refers to the phone itself. The installed PWA must be tested from a secure origin, rather than the Vite development server.

| Field | Actual value to record |
| --- | --- |
| Session ID / tester / UTC start and finish | Pending |
| Published URL and actual loaded build/cache | Pending; current reviewed build is `galaxy-5a206c9f1e2acf40` |
| Source/release reference | Pending; attach the release record, not only a screenshot |
| Physical model / OS version / browser version | Pending |
| Browser tab or installed PWA | Pending |
| Backend reported by runtime | Pending; record `webgpu` or `webgl2`, not an inference from the browser |
| CSS landscape viewport / DPR / display refresh | Pending |
| Fullscreen, browser controls and safe-area/notch behavior | Pending |
| Battery start/end, charging, power-saving mode, ambient/device heat | Pending; unavailable thermal readings stay unavailable |
| Camera shake / Reduced Motion / cosmic density / zoom | Pending; record full and reduced cosmetics separately |
| Audio master/music/effects/voice settings and output device | Pending |
| Profile seed / schema / revision / selected ship / ship XP | Pending; begin natural play with a fresh Haven Scout |
| Capture method and its overhead | Pending; record whether profiling or screen recording was running |
| Evidence files and final status | Pending / Passed / Failed / Blocked, with the actual reason |

The stated target classes are **Pixel 6a and iPhone 12**, or explicitly named comparable devices. Neither has been tested in this session. Do not substitute the RTX 5090 workstation or desktop touch emulation. Record Safari, Chrome, browser/PWA and renderer results separately; unavailable features are **Blocked** or **Not supported**, rather than passed.

## Capture controls already implemented

For a natural run, append `?qa=1&metrics=1` to the game URL, with **no `scene` parameter** and no scene-toolbar actions. This exposes read-only capture methods without creating a diagnostic encounter. The metrics overlay shows backend/FPS, actual quads/entities/particles, p95 frame interval and CPU simulation/render time. If the overlay obstructs play, omit `metrics` and capture through a connected browser inspector instead. Record which method was used.

The following inspector expression only reads the game's existing QA surface. Save its actual JSON output at the start/end of a measured segment and at minutes 5, 10, 15, 20, 25 and 30 of a soak:

```javascript
JSON.stringify({
  capturedAt: new Date().toISOString(),
  visibility: document.visibilityState,
  report: window.galaxyQA.report(),
  hud: window.galaxyQA.hud(),
  frameIntervalsMs: window.galaxyQA.frames()
}, null, 2)
```

`window.galaxyQA.profile()` reads the actual checkpoint for progression comparisons. Settings → Export Save is the normal downloadable snapshot; retain a baseline and each milestone export. If a mobile inspector is unavailable, record the visible metrics and behavior with timestamps, and leave unavailable numerical evidence pending. Do not invent JSON values from a screenshot.

The backtick console has read-only `STATS`, `PROFILE` and `RENDERER` commands. Opening it pauses flight, so do not include that pause in an active-flight benchmark. A hardware keyboard or an attached inspector may be needed on a phone; there is no assumed touch console button. Settings exposes mobile zoom, volume, import/export and fullscreen controls.

Measurement limits from `apps/game/src/main.ts`:

- `frames()` and percentile reports retain at most **3,600 active-flight samples**. At approximately 60 FPS that is a trailing minute, not a complete 30-minute trace. Repeated snapshots can overlap; do not concatenate them as independent samples.
- `durationSeconds` is elapsed time since capture start and can include menu/console/background interruptions. Record active-play and pause durations separately. FPS is updated over approximately one second; frame percentiles measure presentation intervals, rather than GPU timestamp execution.
- `simulationMs` and `renderMs` are CPU timing scopes. `renderMs` includes extraction/upload/submission; it is not GPU duration. `memory` is optional browser JS-heap information, not WASM/GPU/process total memory. A null value means unavailable.
- Hidden/recovering/failed frames are skipped, and samples are recorded only while launched and unpaused. To assess full-soak stalls, also keep an observer log or a supported browser performance trace; a final rolling p95 alone cannot certify the entire session.

A separate disposable QA profile may use `?qa=1&scene=stress&metrics=1&toolbar=1`, or `scene=boss-dart-5`, `boss-bastion-5`, `boss-coil-5`. These are synthetic pressure/diagnostic scenes, not natural progression. Declare the actual loadout, resets, deaths and active duration. `renderer=webgl2` requests the real fallback backend; `fail-webgpu=1` is an explicit initializer-failure probe in QA mode. The existing QA toolbar has graphics-loss/recovery controls. These probes never count as natural boss kills or champion rewards.

## Physical-phone acceptance worksheet

Use a fresh/disposable browser profile for destructive lifecycle or diagnostic cases, preserving milestone exports. Begin in landscape at default zoom 1.0. Repeat the fit/input rows in browser-tab and installed-PWA mode, and repeat representative flight on each available actual backend.

| Case | Reproduction and acceptance | Status / observation / evidence |
| --- | --- | --- |
| Fresh launch and persistence | Launch a new Scout, export its valid checkpoint, close/reload and launch again. No invalid-profile error; schema/ranks/currency/seed remain valid, and reload returns to Haven. | Pending |
| Simultaneous multi-touch | Hold the left joystick with one finger and fire with another outside Haven for 60 seconds. Change direction continuously; release/repress each independently and reverse finger order. Each channel remains independent with no stuck thrust/fire. | Pending |
| Analog steering/release | Compare small, medium and full stick displacement in several directions. Heading follows direction and thrust is proportional; release fades thrust and centers the stick. Continue fire while releasing movement, then movement while releasing fire. | Pending |
| Touch cancellation and lifecycle | While holding both controls, background the browser, switch apps, rotate or trigger a browser interruption. Return and explicitly resume. No stale held input, automatic combat resume over a modal, audio continuation in the background or unintended purchase. | Pending |
| Safe area and compact menus | At Haven, arriving alone opens no services; tapping fire opens them. Open the collapsed flight icons, settings, journal and bottom dialogue. Labels/hit targets remain usable around browser controls/notches, five settings footer actions fit, and close restores the prior play state. | Pending |
| Zoom/fixed UI | Use the actual settings range at 0.6, 1.0 and 1.6; observe each family and change orientation. World scale changes while UI/control geometry stays fixed and readable. Record actual viewport and zoom. Mouse-wheel mapping is a separate desktop case. | Pending |
| Death and transitions | Die through actual hostile combat. Read the persisted expedition report; test Retry and Return to Title in separate deaths. No automatic home-services popup, stuck fire, background flight or stale async encounter reopening. | Pending |
| Audio and explicit voice replay | Unlock audio with a user gesture; compare music/effects levels and mute. Background/foreground and phone interruptions pause appropriately. Voice is initially disabled; explicit replay works when an actual platform voice is available. Neural character voices are not shipped. | Pending |
| Graphics startup/fallback/recovery | Record the actual backend. Separately force WebGL2 and, where available, the initializer-failure/device-loss QA probes. Recovery retains an ongoing expedition's position/hull/queues; permanent failure offers explicit retry and keeps play paused. | Pending |
| Installed PWA/offline | Load/cache the packaged game, record the actual active cache, disconnect networking, close/reopen/reload and launch flight. Verify title/WASM/audio behavior and persistent checkpoint. Service-worker presence alone is insufficient. | Pending |
| Dock-safe PWA update | With a deliberately newer release available, observe the waiting update. Apply at Haven after a saved checkpoint and verify the new active cache/build after reload. Away from Haven, update must not discard an active expedition. If no second release exists, this case is Blocked. | Pending |
| Profile reload and import/export | At a recorded milestone, export, reload, compare actual currencies/XP/ranks/matrix/cargo/patches. Round-trip that same export in its disposable profile. No repeated reward or invalid fresh `last_run=null` snapshot. | Pending |
| Two-tab save conflict | Where the device/browser supports it, start two tabs from one checkpoint. Advance/save A, then advance stale B. A's checkpoint survives; B visibly pauses on conflict. Capture actual resulting export. | Pending |
| Thirty-minute active-play soak | Warm up one minute, then accumulate 30 minutes of active mixed flight/combat including deep-boss pressure where qualified. Log menus/deaths/background interruptions separately; take timed metric windows and stall/thermal observations. No crash, stuck input, repeated graphics reset or unbounded memory trend. | Pending |

The product target remains **60 FPS on the stated phone class**. Record refresh rate and actual distributions instead of certifying from a single FPS label. For each representative sustained workload, report median/p95/p99 frame interval, counts above 33.3/50/100 ms where a complete trace exists, CPU timing, visible quads/entities/particles and early-versus-late soak change. At 60 Hz the nominal frame budget is 16.67 ms. Any tolerance or acceptable stall rate must be agreed from the measured evidence; this worksheet does not silently relax the target. Test full cosmetics first, then reduced cosmetics as a separately named configuration. Do not pass full-density performance from a reduced-density run.

## Natural Haven Scout progression protocol

Run this separately from diagnostic scenes. Start with a fresh real profile and record seed, revision, currency `[0,0,0]`, zero hardware/boss defeats, selected ship 0 and its starting XP. Keep the Haven Scout selected through the ordinary-boss balance pass; purchased upgrades and naturally earned signatures are allowed and must be recorded. Do not import a prequalified loadout, use `SCENE`, or grant progression through an inspector. Split long progression into labelled sessions and retain each checkpoint. Unreached objectives remain Pending when the allotted session ends.

1. **Baseline flight.** Leave Haven through the belt into the first combat area. Record time/distance, stick or mouse method, responsiveness, camera follow and first damage. Compare 0.6/1.0/1.6 zoom while aiming; UI remains fixed. Return to Haven without firing: services stay closed. Fire to open them. Record whether fresh-Scout movement, aiming and threat readability feel controllable.
2. **Asteroids, cargo and banking.** Destroy real asteroids until actual material drops occur; a failed random drop is not a bug. Observe damage holes, impulses and debris. Accumulate cargo, verify the 16-unit bound, then return to Haven. Stored materials increase by the actual retained cargo once, and cargo clears. Reopen/reload and confirm no second banking. Death and reload banking are separate labelled cases.
3. **XP and hardware gates.** Record XP from actual kills/asteroids and the next-level HUD. At each attainable rank, attempt one purchase with insufficient points or ship level and confirm no deduction/rank change. Then buy with ten matching points at the required level. Rank 1/2/3/4/5 requires level 1/3/5/8/12; cumulative XP for those levels is 0/60/200/560/1,320. Each successful purchase deducts exactly ten once; no repeated tap can duplicate it. Export each milestone.
4. **First natural ordinary boss.** Reach 24 real kills of one family; record base kills, reserved G1 flag, queues and actual boss identity before/after spawn. Additional kills behind an active/reserved fight remain surplus. Observe charge, attack and exposed core through actual combat; compare armored/core hits and destroyed cells. Record the Scout level/ranks/signatures, damage taken, elapsed combat time, deaths, near misses and legibility. A defeat advances that family's G1 once; another family remains unchanged. Record awarded XP/signature separately from spawned loot and points actually collected.
5. **Failure and retry.** In a naturally outstanding fight, retreat to Haven, then revisit combat. In a separate attempt, die through actual enemy damage, read the report and choose Retry. Reload another unfinished attempt. The queue survives all three; no premature boss defeat/reward appears, and reload begins at Haven with live damage/entities transient. Export before/after counters rather than judging only the boss banner.
6. **Promotion and signatures.** Two G1 defeats queue G2; apply the same rule through G5 for each family. Highest queued generation gets ordinary encounter priority, with only one ordinary boss active. A full first G5 tree per family requires 16 G1, 8 G2, 4 G3, 2 G4 and 1 G5 defeat, supplied by at least 384 base kills. This is a multi-session objective, not a promised short playtest. Higher defeats must not consume reserved G1/surplus. Record actual signature unlock/pity and evolution; repeat reloads without duplicated credit. Collect the full fifteen-boss natural ledger below before accepting full natural progression.
7. **Sanctuary quest reward.** Discover a real site, read its actual description, accept a courier or family bounty, fulfil it and return to its source. Compare the ready/claimed state, three matching points and unique relic/max-hull gain. Reopen/reload/retry the claim: it stays once-only. For courier, separately verify retained cargo after death/reload and delivery at Haven. A native reward test does not pass this browser interaction.
8. **Campaign extension, separately timed.** After the Scout balance pass, challenge all eight real champions, checking once-only patch/blueprint/points/XP and no ordinary-generation credit. At Haven verify atomic craft costs: ten matching points plus six stored materials together; an insufficient resource leaves both unchanged. Successful crafting consumes both once; switching an owned ship costs nothing and cannot occur away from Haven. Record every selected ship when continuing, since its XP/rank cap changes the loadout.
9. **League win/loss/reload.** All eight actual patches reveal the beacon. Play the eight real player duels; record the other 28 pairings as explicitly simulated. A real death/retreat/home return records one loss; reloading an unfinished duel leaves it unplayed. A completed loss permits a new season and grants no Sovereign/credits. First place grants them once. Replay/reload must retain ownership without another award. Do not mark a complete league accepted from a prequalified profile or diagnostic champion scene.

| Family | G1 actual defeat/loadout/evidence | G2 | G3 | G4 | G5 | Natural acceptance |
| --- | --- | --- | --- | --- | --- | --- |
| Dart | Pending | Pending | Pending | Pending | Pending | Pending |
| Bastion | Pending | Pending | Pending | Pending | Pending | Pending |
| Coil | Pending | Pending | Pending | Pending | Pending | Pending |

For every attempt, record family/generation, seed, ship level/ranks/signatures, pre/post matrix and queues, time to first hit, death or actual defeat, controller type, damage taken, reloads/retreats, and player comments. Separate player skill/learning from code failures. If a boss kills the player before its first readable charge or spawns a tail across the ship, preserve the actual video/seed/position rather than reducing it to “too hard.” Existing native Coil initialization tests prevent one measured overlap; they do not certify all human encounter balance.

## Closure and evidence handoff

Use one evidence folder per session with the filled worksheet, build/cache record, timed telemetry, screenshots/video where needed, baseline/milestone/end exports and a short defect list. Each defect needs actual reproduction, expected/observed behavior and the affected build/device/profile; cosmetic, persistence, control and performance findings stay distinct.

| Acceptance scope | Current status before running this worksheet |
| --- | --- |
| Automated native boss mechanics/progression and real fifteen-boss projectile sweep | Passed declared CPU/test scopes; not a Scout human-balance pass |
| Automated IndexedDB migration/conflict and real two-backend WASM snapshots | Passed declared automated scopes |
| Representative desktop pixel UI/WebGPU/WebGL2 capture review | Passed representative presentation; all-content choreography remains Pending |
| Natural fifteen-boss Scout progression and balance | Pending |
| Full real eight-champion/league browser playthrough | Pending |
| Browser quest completion/reward-claim interaction | Pending |
| Physical Pixel 6a/iPhone 12 multi-touch/layout/audio/PWA lifecycle | Pending |
| Physical sustained 60 FPS / thirty-minute soak / total-memory trend | Pending |

Source references: `docs/QA_BOSSES.md`, `docs/BOSSES.md`, `docs/CAMPAIGN.md`, `docs/ENGINE.md`, `apps/game/src/main.ts`, `apps/game/src/input.ts`, `apps/game/src/storage.ts` and the durable `scripts/qa/boss-live/` harness. Mission Control owns final acceptance status; this unexecuted worksheet does not move any task to Complete.
