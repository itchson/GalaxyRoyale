# Docked status integration review

This is a bounded source and VM-boundary review, before the normal HUD/audio build. No browser, public/dist build, production API or publication operation was performed. Historical cache `galaxy-810fccad52eabc00` evidence remains in its immutable browser folder and does not validate this new milestone.

The original shell handoff remains unchanged: `work/hud-status-shell-review.md`, SHA256 `3d4c2b68b21b61860d9a0709d1417cd95fb6c665bd0e3a28d36e93abbd8be490`. Its original CSS pin `2ac9916df3efdb3d1ae19bcb7a890a174292ae0705b08b08c1168386ff03ff4b` is historical. The approved recovery fallback changes current `apps/game/src/style.css` to `ba0a3856cd33c59dca5ad114e2d696c0a09deabc523f8b6f34df91e959722237`.

## Observed source consistency

- `syncFlightStatusLayout` consumes full logical-screen exports, including native safe insets, and multiplies once by `innerHeight / Galaxy.viewport()[1]`. CSS consumes physical variables without adding fallback env offsets when those variables exist.
- The pointer height agrees with CSS: `max(44px, 22logical × scale)`. At the scale-3 fixture it is 66px, not 44px. Cached safe top/bottom are physical values from the same env probe used for native inset conversion.
- The actual hit rectangle is tested against measured toolbar controls and, for touch, movement/fire controls. `data-context-interactive=false` removes its display even when the separate glyph slot remains available to a toast.
- The previously calculated 280×180 touch case now has an actual production-function VM regression. Its clear seven-logical text row cannot produce a hidden competing joystick/fire hit. The fixture also checks an overlapping map-button hit.
- Visible toast precedence is enforced in CSS and the bridge. Context elements carrying data-pixel-kind are skipped in the later generic loop, so they emit once. The bridge emits a separate nonpainting status-hit from the real measured bbox before substituting the exported text slot for glyph rendering.
- Boss exclusions preserve the clipped full status-hit rectangle. Attached-track avoidance reads the same scene hit rather than assuming the seven-logical text extent represents all pointer space.
- The bridge now observes both context availability/interactive attributes. Main also invalidates on resizing and normal semantic text changes. Its style-only observer optimization is not counted as a test of every future mutation path.
- Native status glyphs use the integer one-cell font, clip full cell extents to their seven-logical row, and retain unlit material. Existing native tests cover these mechanisms; this specialist read them but did not execute the native suite.
- Save footer state text is now concise (`Saved on this device.`, `Saving…`, `Save needs attention.`), with the full reason retained in its title. The eight-logical font and single-line CSS avoid the former 320px DOM/native wrap disagreement, pending actual final-package capture.

## Concrete repair found and applied

`onWebglLost` and `recover` remove `pixel-ready` while retaining `engine-flight`. With an empty native context export, flight toast CSS still used 0px width/height and overflow:hidden. The bridge's multiline branch depended on !engine-flight, so it did not make the real DOM loss/preparation message readable during that pending interval.

Root approved one bounded selector addition: `body:not(.pixel-ready) #toast` now uses the existing safe-bottom multiline DOM rule. It overrides the zero native slot only while normal native UI is unavailable. Pixel-ready flight keeps the quiet strip, and no runtime/class/API change was made. This is a source-confirmed edge and CSS repair; actual forced-loss browser execution remains pending the new normal build.

## Checks actually executed

`node --experimental-transform-types --test scripts/test/flight-status-layout.test.mjs` passed 4/4 after the collision guard and after the CSS repair. These execute the real extracted TypeScript function with declared geometry/VM stubs; they are not browser DOM or phone input tests.

`node node_modules/prettier/bin/prettier.cjs --check apps/game/src/style.css` passed. Prior markup/parser checks verified 81 unique IDs, retained context/telemetry IDs and absent tab navigation. No new broad test matrix was run.

Read-time source pins (not a release freeze):

| Input | SHA256 |
| --- | --- |
| `apps/game/index.html` | `d8ebd898007a868fecc8f4c4f2a7f1a72e2f9771741e8745404c0dbd85c8bc66` |
| `apps/game/src/main.ts` | `49396e1571e138062cb86a3b354c3de1d2d9af9d2e6b98448c53c1b021d557bd` |
| `apps/game/src/pixel-ui.ts` | `29b48dee69ef156c93f8313d3f72c9e468f65e912a8dbb4b786d0b1f40894ffa` |
| `scripts/test/flight-status-layout.test.mjs` | `a8716f21087d02d4c248a43fb1155e36e0445881be844ca28ff93a7412db753c` |

Engine and Audio freeze separately. Subsequent integration edits require final source/build pins rather than reusing these read-time hashes.

## Final package still pending

The owned Chrome normal WebGL2 check should cover compact toolbar, flat strip without duplicate context frames, actual prompt/toast precedence, short save footer, status/map/settings inner clipping at 1280×720/844×390/320×180, and retained same-profile PWA update. Root owns separate IAB WebGPU/audio/recovery. Physical touch/safe areas, hearing, sustained natural play, phone performance and authenticated private execution remain independent gates. No further blocking coordinate defect was established in this bounded review.
