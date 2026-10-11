# Historical live status/map browser review — 810 milestone

This is a desktop Chrome WebGL2 observation of `galaxy-810fccad52eabc00`, not acceptance of the subsequent HUD/audio milestone. The actual observed page loaded `index-D39CEEWH.js`, `index-D-wW9yK0.css` and content-addressed e93f WebGL2 glue/WASM. The normal local package had 19 service-worker assets. Exact observed URLs, package hashes, measured rectangles and file pins are in `BROWSER_EVIDENCE.json` and `HASHES.json`.

The same selected Chrome profile and `4212` origin were retained throughout. No save import, engine injection, hidden cache/registration API, profile reset or extension-setting change was used. Agent-created replacement tabs are explicitly recorded. Only owned tabs were closed; the preview service was left running and the viewport override was reset.

The original stable-path WASM mismatch was reproduced earlier and preserved separately. The content-addressed build successfully loaded new shell/runtime together before applying its waiting worker; Launch → Save → Apply at Haven → title → Launch → status worked for the 965c checkpoint. A later cached 965c shell did not show the next update, even after reopening Save. This failure is preserved in the interim report. Root-authorized manual close/reopen allowed ordinary activation of 3aad; that build then showed a real waiting notice and its Haven Apply action loaded 810. Manual close/reopen is not presented as an automatic update or a second 965c Apply success.

Actual final observations:

- Live status displayed current Scout identity, level/XP, effective drive/weapon statistics, empty automatic slots, carried tier inventory and retained bank/points. A returned DOM observation after ordinary reload retained LV2/XP51, bank T1=1, points5/4/6 and empty slots. The initial interim observation had XP4; no cause or unmeasured progression change is inferred. A later saved snapshot was the title and is named accordingly.
- All four flight toolbar semantic hitboxes measured 44×44. Map/settings now had opacity0 alongside status/save on the pixel-ready page, so native icons were visible without DOM MAP/SET overpainting.
- Real map data showed Haven radius240, belt700–1400 and zero charted discoveries. It did not invent unseen beacons. Keyboard ArrowRight changed center0→1750; the actual Home control returned it to0. No populated discoveries or natural navigation trial is claimed.
- Desktop map scrolling kept art below the content/header boundary and selected-location text/actions above the saved-status footer. The original footer crossing is preserved in `map-desktop.jpg`; corrected captures use the `final-map-*` names.
- Settings opened through its dedicated icon. Its Return to title action worked. Quit was semantically present and was not clicked. At320×180 it was below the initially visible title area; no initial native Quit-visibility pass is claimed there.

| Requested viewport | Actual UI scale | Observed scope |
| --- | --- | --- |
|1280×720|2|Status, map before/after scroll, settings, toolbar rectangles/opacity|
|844×390|1|Status, map with retained scroll, settings|
|320×180|1|Status with fixed44px actions and49px inner scroll area; map controls/details scroll; settings; initial title; toolbar|

Tiny pages require scrolling. Status actions and Close remained44px tall while its text was clipped to the inner scroll region. Map art/details remained clipped away from the fixed heading/Close and footer; the real Set course action could be reached by scrolling. This is resized desktop browser evidence, not a physical phone test. Forge regression was not completed in this session; earlier Forge captures remain historical and Root owns transaction validation.

One concrete narrow-footer issue remains in this historical milestone. At320×180 its DOM text was `Progress saved automatically on this device.`, 12px RoyaleMono, 15px line height, text rectangle `[17,133,286,30]` (two DOM rows), within footer `[17,128,286,35]`. Actual native pixel text wrapped to three rows and clipped the last word at the border. Proposed next copy contract: committed `Saved on this device.`, pending `Saving…`, attention `Save needs attention.` Retain full reasons in the save icon tooltip/accessibility and the dedicated backup page. No source fix is claimed here.

Warnings captured by the supported browser logger were attributed to a Chrome extension; no new game exception appeared in the captured warning/error list. Browser UA is unavailable through the supported read-only DOM evaluator and is not invented.

Physical touch/safearea/orientation, performance/soak, nonempty loadout fixtures, populated-map exploration, full natural campaign, human pacing/listening and owner-authenticated remote execution remain pending. This session did not test forced GPU loss or the newer HUD/audio source.
