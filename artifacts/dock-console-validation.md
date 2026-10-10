# Haven menus, console command line and renderer recovery

Final normal build: `galaxy-ac19daf73e83b006`. This report describes the frozen source and local browser checks before private publication; exact commit/archive/deployment identities are recorded separately.

The console command input is an opaque black quad surface with one subdued separator, aligned to the output column. Its prompt and text use RoyaleMono engine cells. The browser input owns keyboard editing and history; selection changes send a bounded codepoint caret to Rust. Rust preserves spaces, scrolls the visible line toward the caret and clips it inside the input. The caret blinks normally and stays visible with Reduced Motion. The console retains its existing top-down drawer animation and input capture.

Haven now has four authored world facilities for Forge, Hangar, Journal and Arenas. Left click or touch consumes an authoritative facility hit; right click remains aim/fire. Each facility opens its dedicated menu with a close icon. Escape opens Continue, Restart Flight, Settings and Return to Title. Restart waits for the save boundary before resuming. The save icon distinguishes pending, persisted, failed and memory-only storage. Pause buttons inherit their scroll viewport; very short layouts use one column.

Detached renderer preparation no longer holds a mutable live Game across the asynchronous adapter request. Installation is synchronous and uses current dimensions and modal intent. Actual WebGL loss/restoration events gate rendering and reconstruction. A native context-is-lost check also rejects preparation before the queued browser loss event. Stale candidates are disposed before preparing a fresh replacement. Semantic controls remain available while native drawing is unavailable.

Reduced cosmic detail retains cached complete planet/ring silhouettes, with atomic group admission and bounded region counts. Gameplay and save schema remain unchanged in this release.

Validation executed by root:

- 302 native Rust tests and134 shell/Studio tests passed. Strict TypeScript, native all-target Clippy, both locked WASM feature Clippy builds and the normal dual-backend/PWA build passed. Logs are `artifacts/dock-console-*.log`.
- The final normal game was loaded at isolated localhost4210. The actual Haven update control activated the new assets. Live console output identified WebGPU533×300 and WebGL2344×383 logical renderers. Rendered flat input, prompt, preserved spaces and Home/End long-command scrolling were inspected. Screenshots are in `artifacts/dock-recovery-browser/`.
- Final WebGPU device destruction/reconstruction was observed with the console open; the console and expedition remained usable afterward. Its captured `GPU device lost` error is the deliberately requested loss, not an unexpected crash. Reconstruction completed too quickly to measure the pending-adapter interval.
- Final WebGL `WEBGL_lose_context` loss/restoration was observed while the console was open; waiting/restored feedback and usable native console rendering followed. No warning/error log was captured on that tab. These were real extension calls and native events, not synthetic events.
- Earlier normal candidate scoped checks covered all four facilities, close/resume, compact pause, right-click remaining flight, Settings preservation through WebGL restoration, and the explicit early WebGPU-failure WebGL fallback. They apply to unchanged behavior and are identified as earlier candidate observations; the final native guard and input changes were rebuilt afterward.

Independent package review pins87 runtime/build inputs and seven static inputs, parses the actual51 WebGPU/47 WebGL exports and verifies the18 cached/19 served files. No QA-contract serialization exports, old asynchronous Game recovery export, Studio/API/source/work payload or credential is in the game package. See `artifacts/dock-recovery-final-package-review.md`.

Production's separate actual responsive review verifies final320×180 pause clipping and full-width44px actions, four facility routes, dedicated Settings/Backup and long-command Home/End caret at320×180 and720×360. Its temporary Reduced Motion setting was restored and owned viewport/server/tab cleaned up. The durable specialist report records the observations and screenshot identities.

Physical Android/iPhone testing, sustained30-minute percentile/memory capture, audible human review, natural full campaign balancing, authenticated remote gameplay and the complete timing/context-recovery matrix remain unmeasured. Native tests and local desktop screenshots do not claim those gates.

The subsequent cargo/ability request is separate: belt-origin level-one materials/basic points without XP, material loss on death, dock banking, ship-owned levels and compatible crafted loadouts. This UI package does not claim those gameplay changes are implemented.
