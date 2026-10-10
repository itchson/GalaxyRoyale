# Game console

`apps/game/src/debug-console.ts` supplies semantic keyboard controls and a modal overlay. The game shell supplies actual command results and controls simulation pause through the constructor callbacks. The module does not access saves, spawn enemies, or manufacture telemetry.

```ts
new DebugConsole({
  ready: () => runtimeReady,
  run: (command) => executeAllowedCommand(command),
  onOpen: () => pauseForConsole(),
  onClose: () => restorePreviousPauseState(),
});
```

The callbacks above describe the integration contract; they are not additional exported implementations. The shell must restore the pause state that existed before opening and reset held flight input. `run` must explicitly whitelist commands and validate their arguments. `HELP` and `STATS` should report the current implementation and runtime. Mutating `SCENE <NAME>` diagnostics are QA tools; enabling them and deciding whether they ship belongs to the shell. Scene diagnostics must not be presented as normal player progression.

Backquote toggles only when `ready()` returns true. Escape closes an open console. Enter submits a trimmed command capped at 256 characters; blank commands are ignored and one asynchronous command runs at a time. Up and Down browse the last 32 submissions, retaining the unfinished draft when returning to the newest entry. `CLEAR` clears local output without calling the game. Repeat Enter and Backquote key events do not issue duplicate commands or repeated toggles. Composition input is not submitted.

The overlay captures keyboard input while open and intercepts pointer events outside the card. Tab retains focus on its command field. `open()`, `close()`, `opened`, `transitioning`, `write(message)`, `clear()`, and `dispose()` are available for shell integration. The shell must not set the overlay's `hidden` attribute independently because that would bypass its pause callbacks.

The console is a top black drawer. Its card slides from `translateY(-100%)` to zero over 180 ms. Closing reverses that transform and leaves the overlay visible, `opened === true`, input capture active and simulation paused until the transform transition ends. Only then is `hidden` set and `onClose` called. A 220 ms fallback completes a transition if the browser does not deliver `transitionend`. Backtick during closing reopens from its current transform without repeating `onOpen` or briefly resuming the game. Repeated closes do not restart the timer; disposing completes closure once and cancels all scheduled work.

The root shell sets `body.reduced-motion` from its persisted setting. Both that class and the OS `prefers-reduced-motion: reduce` disable animation and make open/close immediate. Phase changes emit a bubbling `galaxy-console-transition` custom event from `#debug-console`, with detail `{ phase: "opening" | "open" | "closing" | "closed", durationMs: number }`. Start events report 180 ms (or zero for reduced motion); settled events report zero. Class/hidden mutations also invalidate the bridge. The root must invalidate/sync UI layout every render frame during the short transition so Rust follows the actual animated DOM bounds; observing only the initial class mutation cannot redraw each intermediate position.

Output uses `textContent` in `#console-output p` elements, with at most 80 lines of 256 characters each. Errors are the actual thrown message. `write()` splits multiline results into individual paragraphs. No command or output is interpreted as HTML. History is in memory and is not persisted.

The pixel UI bridge must observe `#debug-console` for child, text, visibility and class mutations, include `.console-card` as a UI root, clip paragraphs to `#console-output`, and read `#console-input.value` as an input node. Programmatic history and submission changes emit a bubbling `input` event so the bridge can redraw the current command. Existing input, scroll and focus listeners also invalidate its view. Once `body.pixel-ready` is set, the card is visually transparent while its semantic controls remain interactive and accessible; the engine must render its panel, text and input. Without that renderer integration the console would be invisible.

The CSS sizes the card at up to 640 by 180 logical pixels using `--ui-scale`, fills available width at the top and limits line width for reading. Its opaque black fallback has a thin bottom edge, aligned heading/help/output/input, and safe-area-aware interior padding. Output receives the remaining scrollable height. The engine should render its console panel as a plain black drawer with a thin bottom edge, rather than an ornate menu frame. Browser acceptance should verify opening after runtime readiness, visible intermediate slide positions, closing/reopening without a pause leak, reduced motion, typing without ship movement, history draft restoration, safe `CLEAR`, truthful `STATS`, pixel rendering, and compact landscape layout. Physical phone acceptance remains separate.
