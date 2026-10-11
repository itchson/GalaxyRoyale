# Docked status shell handoff

This is source/layout groundwork for the subsequent HUD/audio milestone, not normal-browser or phone acceptance. The previous `work/universe-map-browser-qa/REVIEW.md` is frozen evidence for cache `galaxy-810fccad52eabc00`; it must not be counted as execution of these new source changes.

## Owned source freeze

- `apps/game/index.html`: `d8ebd898007a868fecc8f4c4f2a7f1a72e2f9771741e8745404c0dbd85c8bc66`
- `apps/game/src/style.css`: `2ac9916df3efdb3d1ae19bcb7a890a174292ae0705b08b08c1168386ff03ff4b`
- `docs/STYLE_SYSTEM.md`: `2a38c13a8b12b67782680ca49c6b8439b58d45f806845439c592665895313167`

The semantic toolbar stays top-right. Existing live sector/currency/hull/distance IDs remain inside accessible offscreen telemetry. Existing contact buttons and the live toast share `#flight-status-strip`; native rendering owns the flat strip and projected tracks. No frame was added for a contact or toast. CSS consumes Root's physical strip/text bounds and `--flight-hit-y`, preserving `--ui-hit = max(44px, 22logical × uiScale)` rather than assuming every hit is exactly 44px.

The prompt is display:none during a visible toast, when no context slot exists, or when `data-context-interactive=false`. Root owns that last real-hit intersection guard; CSS does not derive a second native layout. Native `status-hit` must reserve the measured full button hit region while `status-text` paints only the seven-logical text row. Full accessible text is retained in DOM.

The menu save footer now has one integer eight-logical font line with ellipsis. Root owns concise committed/pending/error text and full tooltip/save-page reason. Initial HTML copy says `Saves on this device.` and does not assert a committed save before initialization.

## Checks actually executed

Prettier parsed/formatted and then checked the three owned files successfully. A bounded markup check found 81 unique IDs and verified the preserved live fields/contact IDs, native kind attributes and absence of tab navigation. Root's actual-AST physical geometry suite passed 3/3 before the subsequent hit-intersection guard was added. That run does not validate the new collision guard or native/browser paint parity. No dist/public build or API write was performed by this specialist.

## Calculated touch case found and handed off

At supported logical viewport 280×180, touch enabled, uiScale 1, and a measured toolbar spanning y=12..56, the gap between pads is under 120. The native above-pad strip can be `[6,62,268,22]` with context `[10,74,260,7]`. Centering a 44px hit gives x=10..270 and y=55.5..99.5. It intersects joystick x=14..98/y=88..172 and fire x=196..268/y=94..166 although the seven-logical text itself is clear. Safe-top/bottom clamping alone cannot resolve this.

Root accepted a separate `data-context-interactive` flag driven by intersection with measured real joystick/fire/header geometry. CSS hides the unsafe prompt target rather than moving its glyph slot or inventing another layout. At scale 2 the actual 44px minimum can still protrude roughly half a logical pixel into the joystick in this same above-pad arrangement. Conversely, a sufficiently wide touch center gap can safely retain its centered hit without a blanket above-pad clamp. These are source-derived cases, not observed phone input measurements.

## Pending

The final integrated native painter/boss exclusions and normal dual-backend browser package still need review and visible capture. Test the real safe-area and interactive guards, prompt/toast precedence, small/tiny geometry, recovery warnings, short save footer, projected-track zoom/motion, and uncluttered live menu transitions. Physical touch/phone performance, sustained play, adaptive audio listening and owner-authenticated private execution remain separate gates.
