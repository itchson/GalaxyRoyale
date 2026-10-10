# Dock prompt hitbox review

Read-only source review prompted by the root's isolated normal WASM dev-tab observation on port 4186: at Haven, a CUA role-targeted dock click stayed in flight, while E opened services. That observation was supplied by the root, not independently replayed by this reviewer. No production, browser DOM, input handler, stylesheet, service or deployment was modified here.

## Concrete source candidate

`apps/game/src/style.css:202` positions both `.dock-prompt` buttons at `left:50%` and centers them with `transform:translateX(-50%)`. The global `button:active` rule at line 826 replaces the entire transform with `translate(2px,2px)`. Its selector specificity exceeds `.dock-prompt`, and the later `.dock-prompt` sizing rule has no active transform override.

Let the prompt's untransformed left be L and its width be W. Its resting rectangle starts at `L − W/2`; its center is L. While active, the rectangle starts at `L + 2`. A pointer targeted at its resting center L therefore falls two pixels left of the active rectangle. The hitbox moved by `W/2 + 2` pixels horizontally, not by a harmless two-pixel pressed offset. This is a deterministic CSS geometry conflict. Whether it caused the particular CUA failure still needs a real pointer event/rectangle trace.

For uncaptured mouse clicks, down and up targeting different siblings can cause click dispatch on their common ancestor rather than the dock button. Touch may differ because direct-manipulation inputs normally receive implicit pointer capture. These distinctions follow the [W3C Pointer Events dispatch rules](https://www.w3.org/TR/pointerevents3/#the-click-auxclick-and-contextmenu-events) and [implicit capture rules](https://www.w3.org/TR/pointerevents3/#implicit-pointer-capture); they do not constitute a browser reproduction.

The same transform conflict applies to `#site-button`, which shares `.dock-prompt`. Ordinary flow-positioned buttons experience the small two-pixel shift, while these centered absolute buttons lose their positioning transform.

## Handler and layer checks

- `main.ts:1187` binds the dock click directly to `pause(true)`. `main.ts:1292` routes E at `hud.dock && !paused` to that same function. `pause` exposes the menu, pauses Rust, resets flight input and selects upgrades. There is no distinct Rust dock command in the click path that could explain E succeeding while an actual click handler runs but silently fails.
- `paintHud` shows the dock button when `hud.dock && !paused && launched`. Its opacity becomes zero under `.pixel-ready` because Rust draws the visual; opacity alone does not remove semantic hit testing. `#game-ui` has pointer events disabled, while buttons explicitly restore them. No dock-specific pointer-events override was found.
- `FlightInput` captures mouse pointers only in the canvas's own pointerdown listener. The canvas and `#game-ui` are siblings. A fresh pointerdown on the prompt does not bubble through the canvas. Joystick/fire capture is likewise confined to those controls.
- The dock has no pointerdown handler that cancels its click, captures elsewhere or stops propagation. The bridge's document hover/focus listeners only update semantic rendering state. No persistent document-level capture loop was found that automatically redirects a fresh prompt click.
- An already held canvas/joystick/fire drag can remain captured until release; that is a separate input sequence and must be distinguished from a fresh down/up click. Hidden menus, dialogue and console overlays can also affect hit testing when actually open, but the supplied observation was flight at Haven with E available.

## Bounded verification and repair handoff

Root should capture the resting and active `getBoundingClientRect()`, computed transform, `elementFromPoint` at the initial center, and actual pointerdown/pointerup/click target sequence in the visible isolated dev tab. Record pointer type and whether the initiating control has capture. Repeat with a fresh center mouse click after all buttons are released, and with E as the control case. If pointerup targets the canvas/common ancestor while the dock moved as predicted, the source candidate is confirmed.

The smallest repair is an explicit `.dock-prompt:active` rule preserving `translateX(-50%)` without moving this semantic target. An alternative is leaving all semantic hitboxes stationary and using engine color/pixel feedback for pressed visuals. Removing centering or replacing it with `transform:none` would reproduce the same positioning loss. No input-capture workaround is needed merely to conceal the transform conflict.

After repair, verify actual center and inside-edge mouse clicks, keyboard Enter/Space on the focused semantic button, E, site prompt activation, and touch down/up on a compact landscape view. Check that the down/up remain on the intended hit target and the menu opens once without a course or shot being submitted. Include any actual touch-pad/prompt overlap measurements at small widths; this source review does not establish phone hitbox acceptance.

Existing `scripts/test/input.test.mjs` checks flight input and Haven's short weapon-press interception using EventTarget surfaces. It does not model CSS `:active`, browser hit testing, DOM transforms or native click target selection, so those tests cannot accept this prompt behavior. No new passing test or browser screenshot is claimed here.

## Reviewed source fingerprints

| File | SHA-256 at review |
| --- | --- |
| `apps/game/src/style.css` | `72507CA0E813D09054E61E9F113EFE1066B157B45BE696B6BB1047EC9A40A21F` |
| `apps/game/src/main.ts` | `959644E093A58C8602D8275F214D809433275CEB6ED89AD353B1DE15B8F2657E` |
| `apps/game/src/input.ts` | `85BA91369AD506CBC60D5B314F7FB3F9370D1A15B71F7FB1E886575793011159` |

Conclusion: a real positioning conflict is present in source and coherently explains the reported mouse failure. The specific browser incident and its eventual repair remain root-owned validation; no confirmed browser regression or shipped fix is asserted by this read-only review.
