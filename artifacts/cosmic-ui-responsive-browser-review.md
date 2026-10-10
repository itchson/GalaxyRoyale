# Responsive menu and touch geometry — final desktop review

Accepted within the declared desktop scope, with no blocking defect found in the recorded checks. The final local normal package is `galaxy-95d0a4d39116971e`; the actual browser page loaded `/assets/index-0nC89wsy.js` and `/assets/index-DcmMweY9.css`. Source and local package hashes are recorded in `release-source-pins.json`. This is a source freeze, not a claim of an already committed/published release identity.

## Implemented scope and ownership

This specialist edited only `apps/game/src/style.css`, `campaign.css`, `dialogue.css` and `debug-console.css`. Root supplied the six-page semantic menu, real Restore button/input integration, selected-tab scrolling and native scroll clipping. Engine supplied native flight HUD layout and shared touch-painter/reservation bounds. The root-reported 282 Rust / 89 shell / native and both-WASM strict Clippy / normal build results are separate author evidence; this specialist did not rerun or claim those tests.

The menu uses a fixed header, tabs and actions with one bounded scrolling page. At physical width <=800, the six tabs use three columns/two rows; narrow portrait <=420 uses two columns/three rows to retain labels. The exceptional <=240px height uses a 44px horizontal tab strip, with keyboard selection automatically scrolled into view. Settings tracks size to content; ranges retain their own height. Save/Restore controls are inside the Save page, Fullscreen is in Settings, and the footer contains only Resume/Title plus contextual Apply Update. Console remains an opaque top drawer; dialogue remains a bounded bottom panel. Both retain reduced-motion behavior.

## Method and actual final observations

An owned Chrome tab at `http://127.0.0.1:4201/?qa=1` used the normal Vite package and existing test-origin progression. All input came through documented CUA native keys/clicks/scrolling or semantic locators. Layout reads were read-only DOM measurements. The desktop viewport capability is not a physical phone emulator. No hidden game state, injected pointer events or browser GPU configuration was used.

The final package was activated through the actual waiting-update notice, Launch, E at Haven, Apply Update, and reload to title. In the later `?qa=1&touch=1` declared touch-layout surface, the actual `RENDERER` console command reported **WEBGL2 / 720 x 360 logical pixels**. Logs show WebGPU adapter unavailability followed by successful WebGL2; this session does not claim WebGPU execution. Extension content-script warnings are retained in raw logs, separately from the expected adapter fallback message. No further game error appears in the captured final-package warning/error slice.

| Actual final check | Recorded result |
| --- | --- |
| 320x180 menu, DPR1, ui-scale1, canvas320x180 | Menu304x164 at8,8. Tabs286x44 at17,17; selected Save scrollLeft406 and Home→Upgrades scrollLeft0. Page286x50 at17,65; fixed return action286x44 at17,119. No vertical menu overflow. |
| Tiny Save scrolling | Native scroll reached scrollTop316 of366 content pixels. Export/Restore row is visible within the page, with the return action still fixed. Narrow labels use the native painter's smaller crisp font. |
| 360x800 portrait Settings, DPR1, ui-scale2, canvas360x800 | Six complete tab labels in two columns/three rows. Settings client312px/scroll806px; ranges stay within their own rows. First ranges visible; lower settings intentionally require scroll. Footer/action fits. |
| 320x180 declared touch Haven | Live joystick[14,88,84,84], fire[236,94,72,72], pause[264,12,44,44], dock prompt[80,33,160,47]. Native capture shows separate touch pads and bounded compact telemetry/prompt. Full-hull Haven intentionally omits the hull meter through `!hud.dock || hull/max_hull < 1`; this is not meter occlusion. |
| 360x800 declared touch geometry | Actual logical width180 at ui-scale2: joystick[28,616,164,168] ends at192; fire[192,628,144,144] starts at192. Zero overlap. The portrait rotate guard is visible, so this is geometry-only, not a portrait flight/input acceptance. |
| Active hull at reset default1438x1192 | Actual documented `WARP 450 0` accepted with QA rewards disabled/run banked warning. The screenshot visibly shows separate hull100%, shield, LV1/cargo0/16 and touch controls. This is an isolated diagnostic HUD presentation check, not natural travel/combat. ui-scale is3.0025188916876573 and canvas1438x1192; no assumed integer layout ratio. |

Actual original captures are `release-save-tiny-320x180-top.jpg`, `release-tiny-home-navigation.jpg`, `release-save-tiny-320x180-bottom.jpg`, `release-settings-portrait-360x800.jpg`, `release-touch-hud-320x180.jpg` and `release-active-hull-default-webgl2.jpg`. Raw geometry is in `measurements.json` and `release-touch-portrait-bounds.json`. Page asset links and actual command text are preserved separately. DOM rectangles can include clipped/off-scroll content; they do not mean all page controls are simultaneously visible.

## Earlier observations retained with their exact scope

Historical `galaxy-965d6195d11e52a9` captures exposed clipped portrait labels and settings input spill, which the owned CSS corrected. The same-call `settings-720x360.jpg` was stale title content; the early portrait measurement retained the previous canvas before resize settled. Both are excluded from acceptance.

The `galaxy-f1e3594d12114f28` checkpoint independently showed readable720x360 Settings and all nine Settings plus Fullscreen fitting at1280x720, ui-scale2. Its tiny capture predates scrollbar:none and is superseded by the final50px page/44px strip measurement. Keyboard Tab from Export focused the real Restore BUTTON; Enter raised an actual single-file chooser at2026-10-10T22:23:23.012Z. No file was selected/imported. That chooser check is explicitly historical f1e evidence, not a newly repeated final-package import claim.

The first Settings click issued in the same call as the final portrait resize remained on Save while resize settled; a subsequent settled semantic click selected Settings correctly. The accepted screenshot and measurement use the settled layout. This does not establish a general click failure.

## Cleanup and remaining gates

The global viewport override was reset before root's default captures. Owned tab1254884522 was closed. The preview process44716 was stopped only after verifying its executable, exact4201 Vite command and creation timestamp; port4201 had zero remaining listeners. `cleanup.json` records this.

Physical phone safe-area insets/browser lifecycle, real simultaneous multitouch, both-renderer device coverage, sustained FPS/soak, full natural progression and moving/crowded human readability remain pending. The final360x800 geometry check is behind the rotate guard. The final active-meter check was at the reset desktop size, not an additional320x180 outbound meter test. Console/dialogue responsive CSS is implemented, but this final narrow menu session does not claim every dialogue/death/console content variant was manually tested. Private authenticated remote gameplay is outside this local review.