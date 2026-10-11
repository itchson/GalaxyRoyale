# Final flat HUD: actual retained-Chrome WebGL2 review

This is an actual desktop browser observation of the normal game package on `http://127.0.0.1:4212`, using only documented CUA controls and read-only DOM/resource observations. It is not a phone, hearing, DSP, natural campaign or performance acceptance. The root-owned preview and origin storage were preserved.

## Identity and upgrade

- Selected Chrome browser `1`; fresh ephemeral replacement tab `1254884550` in the same retained profile and origin. The old agent-owned tab was already closed. No DB/cache/profile reset or extension/configuration change occurred.
- The initial page actually loaded historical `index-D39CEEWH.js`, `index-D-wW9yK0.css` and WebGL2 `e93f4425…504532`. A waiting-build message appeared. Launch, Save & Backup, and **Apply update at Haven** were used through the actual UI. The resulting title then loaded `index-5rV-IuNs.js`, `index-BLHOm9mL.css` and WebGL2 `42d9d5261cbd1e64442f8427726ed9425957e6e7c8302e0473c6c2423d46ff57` (`galaxy.js` and `galaxy_bg.wasm`). This was a real Apply transition, not manual tab-close activation.
- Expected normal dist cache: `galaxy-a9c3cb3540f78b9b`. This identifier is pinned from the local `dist/sw.js`; browser resource inventory proves the corresponding loaded JS/CSS/module pairing. No hidden Service Worker/cache API was queried.
- Retained actual readout: Haven Scout LV2 / XP51 / NEXT60; hull100/100, shield0/0, speed0/max103, acceleration267, primary12 damage/5 shots/s; three empty ability slots; cargo0/16; T1 stash1; offense/defense/passive points5/4/6. No fixture was imported and no reward was claimed.

## Accepted within this desktop scope

| Check | Actual evidence and limit |
| --- | --- |
| Final normal GL rendering | `desktop-flight.jpg` at 1280×720, `landscape-flight.jpg` at 844×390, `tiny-flight.jpg` at 320×180 show one flat bottom strip, compact four-icon toolbar and a ship-relative hull track. No old floating telemetry frames or fallback MAP/SET letters are painted over the native icons. |
| Coordinate agreement | Actual 1280×720 `--ui-scale=2`: strip `[12,664,1256,44]`, text `[20,688,1240,14]`; toolbar hit rectangles44×44. Both contextual action buttons were hidden at Haven, so this does not prove a live contact/touch hit region. |
| Live ship status | Actual profile/readout retained after Apply. At320×180, Close and all four actions have44px height; the status region is49px high with492px scroll content, kept below actions and clipped at the menu edge. This is a desktop tiny-viewport check, not physical multitouch. |
| Map and footer | At844×390 the map chart is clipped inside its dedicated menu. At320×180, keyboard focus scrolling brings selected-location details and Set Course into view; chart art does not leak above the header or below the content. `tiny-map-details-settled.jpg` shows the short save footer within the border. Actual footer `[17,155,286,8]`, client/scroll height8; the previous three-line clipped footer is corrected here. No Set Course navigation was activated. |
| Settings | Actual844 settings and320 scrolled action captures show bounded content, Close and Save/Backup/Title controls. No setting was modified; no Quit or backup/import operation was executed. |
| Real WebGL lifecycle | Declared QA URL adds `qa`, `metrics`, `toolbar`; actual **Lose WebGL context / Restore WebGL context** buttons exercise the real extension. Two loss/restoration cycles completed. The first opened Status → Settings during confirmed loss, and restoration preserved Settings with “Graphics restored. Your expedition is intact.” Closing Settings returned to rendered flight with retained profile. This is actual loss/restoration, not RECOVER-only or the manual Lose GPU flag. |
| Adaptive audio resource state | Existing DOM `data-report` observed after restoration: WebGL2/Haven, context running, theme0 weight1, hostiles0/combat0/alarm0. A declared reward-suppressed belt scene atposition950,0 observed context running, theme14 weight0.97385 and theme0 weight0.02615, six scheduled music voices, zero SFX/continuous voices. This proves source/resource state in this browser only; no output recording, RMS, peak, hearing or cue cancellation audibility was measured. |
| Application warnings | The captured warn/error inventory contains Chrome-extension warnings. Filtering source URLs leaves no application warn/error entries. Extension settings were untouched. |

## Concrete unresolved finding

**The real context-loss waiting reason expires while the browser remains lost.** In the second actual loss, `body.className` was `engine-flight` (pixel-ready removed). The toast contained “Graphics context lost. Waiting for browser restoration.”, was not hidden and had display:block, but its class was empty and computed opacity was0. `lost-toast-geometry.json` records this state. The earlier actual lost-flight capture shows a white lost canvas with fallback controls and no visible waiting reason; lost-settings is readable but also lacks that visible reason.

Current source `onWebglLost()` calls the ordinary `toast()`, whose3500 ms timeout removes `visible`. The approved non-pixel-ready CSS fallback changes placement, not this lifetime. Functional restore succeeded. Keep the unresolved-loss reason visible until restoration or an explicit replacement/error; this measured presentation finding was sent to Root before release review completion. No source edit was made by this reviewer. Loss background color is recorded as a visual observation, not a separately diagnosed renderer defect.

## Limits and cleanup

- Chrome's declared QA capture sampled approximately 1 FPS / 1004 ms intervals at the default1438×1192 viewport. These are actual sampled metrics, possibly affected by this browser/control environment; they are not a game performance claim, phone benchmark or 30-minute soak.
- Attached shield behavior with nonzero shield, low-hull bar pulse, crowded moving boss/core collisions, live contact action hit safety, touch overlap/cancellation, safe-area insets, mobile background/PWA lifecycle, actual-score DSP/listening and full natural progression remain pending here. Root's separate GPU observations are not counted as this GL execution.
- Temporary 1280×720/844×390/320×180 overrides were reset before the lifecycle phase, and reset again at cleanup. Only the sole owned tab was closed; retained origin DB/SW/profile and the root preview were left intact. `cleanup.json` records completion time. No source, dist, API or release metadata was changed.

`PROTOCOL.md` is the pre-execution plan; this review and raw evidence are the executed result. `pins.json` records read-time source/package hashes, and `hashes.json` freezes this owned evidence. These do not imply an authenticated private remote test or a committed release identity.
