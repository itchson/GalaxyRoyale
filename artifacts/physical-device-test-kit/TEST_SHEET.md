# Physical phone acceptance — human run sheet

**All device checks below are Not run.** This kit creates no device measurements and changes no game/build/profile. Test on an actual **Pixel 6a + Chrome** and **iPhone 12 + Safari**, then their installed Home Screen/PWA mode. Record exact OS/browser versions and actual renderer; comparable hardware needs a separate named result. Desktop emulation is not a substitute.

## Identify the package before starting

Target reviewed normal build: **galaxy-95d0a4d39116971e**,18 cached game assets; script `/assets/index-0nC89wsy.js`, stylesheet `/assets/index-DcmMweY9.css`. `expected-build.json` pins current source/WASM/package hashes. At kit preparation, authoritative private metadata still identified the previous8b19c6c/37e3441c build; do not attribute the new UI to that older release. Before a remote run, fill the matching final runtime commit/deployment from producer-updated `artifacts/private-release.json`. An owner login gate is a prerequisite, not game acceptance; no credentials belong in evidence.

Use the deployed HTTPS game or an explicitly approved equivalent HTTPS test origin. A phone's `localhost` is the phone, not the Windows preview. Preserve a baseline through **Save → Export Backup** and use a dedicated QA progression/session. Keep one game tab active for ordinary save checks. Record cache/loaded asset names after any waiting-build notice is activated through **Haven → Apply Update**, then verify the profile again. Do not clear browser storage to hide a defect.

Fill: date/time; tester; model; OS/browser; browser-tab/PWA; renderer; URL origin; build/cache; battery/charging/brightness/power-save state; refresh setting; CSS viewport/DPR/canvas/logical UI/zoom/safe insets; profile seed/revision/selected ship/XP/currency; capture method. Use `session-template.json` and `soak-template.csv`.

## Functional pass — repeat per device and browser/PWA mode

| Check | Reproduce with real fingers | Required observation / evidence |
| --- | --- | --- |
| Landscape safe area | Rotate both landscape directions with browser bars visible/hidden; repeat installed mode. | Six pages, current tab, footer, HUD, prompts and touch controls stay clear of notch/home indicator and each other. Capture both sides with actual safe insets. Portrait menu remains usable; portrait flight shows rotate guard. |
| Menu/navigation | Visit Upgrades/Hangar/Campaign/Journal/Settings/Save, scroll each; use Return/Title. Open Restore then cancel the native chooser. | Full selected label, page-only scrolling, reachable44px actions, no click-through or swallowed tap. Portrait Settings fields stay within rows. No restoration is claimed from cancellation alone. |
| Thrust power and release | Fly outside Haven; hold stick at25/50/100% radius in eight directions; release. | Smooth facing/thrust, short thrust decay, then coast; no stuck input or discontinuity. Camera follows without HUD movement. Record video; judge handling separately from FPS. |
| Simultaneous fingers | Outside Haven, hold left thrust + right fire for10s, steer while firing; release each first; drag fingers beyond pads. Repeat10times. | Movement and heading-based mobile shots remain independent; releasing one does not cancel/stick the other. No synthetic pointer event replaces this. |
| Cancellation/modal | During both holds: open Pause, rotate, invoke OS app switch/home or notification overlay, lock20s; return. | No firing/thrust resumes without a fresh touch; pause/menu is truthful; saved progress remains. Residual physical velocity is not stuck thrust. Record each actual interruption. |
| Haven and death | Tap fire at Haven; later allow a natural death outside. | Fire opens services safely. Death report has Retry/Title, retained progression and honest last-recorded history; no automatic services popup. Do not use SCENE die as a natural-death pass. |
| Zoom/readability | Set0.60/1.00/1.60 in Settings and resume each. | UI/hit targets stay fixed; aim/control geometry remains aligned; ship, hostile shots, boss core/phase and Mote warning can be distinguished while moving. Test full and reduced motion/detail separately. |
| Save/relaunch | Export baseline and earned progress; close app/tab, reopen, reload. | Returns to Haven; XP, currency, crafted ship, cargo/stash handling and quest state follow the actual schema. No duplicate reward or Invalid profile message. Record revisions/exports. |
| PWA lifecycle | After full load/cache verification, install with the device's supported UI; reopen offline, launch flight, close/reopen; reconnect and apply a genuinely newer build at Haven. | Real game/WASM loads offline, settings/progress retained, update activates only at Haven. No available newer build means update row Not run; an install shortcut alone is not offline proof. |
| Audio/recovery | Resume after background/lock, change volume/mute; separately request existing QA Recover GPU if available. | No stuck audio/input or lost expedition; capture actual message/backend. Explicit RECOVER is API-requested recovery, not native device/context-loss proof. Forced/native loss remains a separate gate. |

Write Passed/Failed/Blocked/Not run beside each case, with actual evidence path. Unsupported backend/fullscreen/inspection is reported honestly. Stop the test on a reproducible critical persistence/input defect, preserving the failed profile and steps.

## Thirty-minute active-play soak

Run **30 active minutes per declared device/backend/density/mode**, after1min warm-up. Use natural flight/belt/waves and the heaviest legitimately available combat. State loadout and route; if only early content is reached, deep-boss workload stays untested. A separate `scene=stress` run is useful synthetic pressure, never natural progression or a substitute for its evidence.

Log active minutes1,5,10,15,20,25,30 plus exact pause/background/death durations, stalls, resets, battery/thermal observations and visible workload counts. Capture both early/late comparable combat windows. Full cosmetics at defaultzoom is the primary configuration; reduced detail/motion is a separately labeled run, not a full-quality pass. Record profiling/screen-recording/USB charging overhead.

The target is60FPS, nominal16.67ms at60Hz. Report actual rollingp50/p95/p99, workload counts and any complete-trace33.3/50/100ms stalls. Do not approve from a single FPS label or silently choose a new tolerance. No crashes, repeated resets, stuck input, lost/duplicated progress or unexplained persistent memory growth. Quantitative whole-soak frame distribution/total memory remains Pending when the capture method cannot provide it; see `CAPTURE.md`.