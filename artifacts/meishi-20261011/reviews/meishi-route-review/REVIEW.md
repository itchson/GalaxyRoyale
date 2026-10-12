# Standalone MEISHI routes and collection scroll review

No scoped source blocker in Root `69db89ccf5895390807383fcdd37c3413ba2dc68`. This is an independent read-only source review plus three executed DOM-adapter scroll probes, not browser or native visual acceptance.

## Route and accessibility findings

At historical `e6be2e847bedd1e477e4f3576b7e6bf654f7492b`, both Escape and the status toolbar opened MEISHI. Removing the card's navigation alone would have left Cargo/Stats reachable from Haven Forge/Hangar but absent from ordinary away-from-home flight. That concern was reported before Root's correction.

The final runtime route is explicit: Escape and the existing status toolbar invoke `pause(false)`, whose default is now `pause`/RAICHO Stats (`main.ts:762–764,1974–1976,2282–2286`). Stats exposes the existing MEISHI/Cargo/Hulls/Abilities/Stats buttons. Haven Forge and Hangar also expose this navigation (`main.ts:1001–1006,1887–1896`; `index.html:233–273`). No additional toolbar hitbox is required. Collection browsing outside Haven remains distinct from authoritative home-gated transactions.

MEISHI has the requested standalone presentation. Its generic heading, description, footer and collection navigation are hidden; all other menu pages, including Settings, are hidden (`main.ts:988–1031`). There is no separate MEISHI Settings action in the HTML or bindings. The retained Close button and Escape return to flight. To enter another inventory after viewing the card, close it and reopen Stats; this deliberate route preserves the card-only request.

The card dialog's accessible name points to a real `meishi-card-title` heading created by the actual painter (`main.ts:1027–1029`, `meishi.ts:68–69`). The close label becomes “Close MEISHI card”. Card contents are read-only and contain no action buttons. The final keyboard follow-up gives `meishi-content` a labelled `role="group"` and `tabindex="0"` and explicitly includes it in the Tab trap, so Close and the read-only scroll area can receive keyboard focus (`index.html:274–281`, `main.ts:2256–2277`). Hidden ancestors are filtered; the global `[hidden] { display: none !important; }` prevents flex rules from reviving hidden menu navigation/footer (`style.css:76–78`). Stable card nodes update text/attributes rather than rebuilding on each HUD paint. These checks establish source semantics, not actual keyboard scrolling, screen-reader or browser-focus acceptance.

The bridge now emits `meishi-panel` for the card surface (`pixel-ui.ts:382–391`). Its native surface integration is Root/Engine-owned; this report does not claim it was rendered or judge visual clipping, glyph fit or physical touch targets.

## Executed scroll evidence

Root's scroll patch retains the existing collection scroll during category/filter changes, while entering/leaving the ability collection resets to the overview. The real `paintForge`/`paintShipHangar` functions already capture and restore the pre-rebuild offset. Reading `target.scrollTop` after painting a filter therefore preserves the restored, potentially browser-clamped offset. Focus restoration uses `{ preventScroll: true }`.

`scroll-probe.mjs` imports a pinned copy of the actual Root `loadout.ts` through the existing test fixture. Three independent probes passed:

- Ability filter retains scroll 63, restores its control focus and leaves HUD data unchanged.
- Same-collection category change retains scroll 71; Back resets to 0 and focuses the matching Manage control.
- Hull filters retain scroll 89 and accept a declared shorter-content clamp to 20, with filter focus retained.

The adapter explicitly models `replaceChildren` resetting scroll and a bounded scroll setter. It is a fake DOM, not an actual browser layout measurement. The final committed source differs from the executed snapshot solely by Prettier wrapping the `previousScroll` assignment; exact snapshots and both hashes are retained. No new test rerun is represented by this final source comparison.

## Pins and limits

`source-hashes.json` records 14 exact committed source/test inputs and their observed working-byte hashes. Runtime route changes are in `2bab5b1e3b8a14d813f4827ff5623d09d1af7427`; the reviewed head also includes the native surface, documentation and `69db89c` keyboard-scroll follow-up. The first pin attempt correctly rejected a working source mismatch when this follow-up landed during review; its three-file diff was then read before pinning the new head. Earlier source finding and executed scroll input remain separately pinned.

Root reported 413 shell passes, one existing Windows skip and a passing TypeScript check before the keyboard-scroll follow-up; those are attributed integration evidence and were not rerun here. Its new modal fixture checks forward and reverse Tab between Close and the scroll area; it was inspected, not executed by this reviewer. No source, server, Studio, public build, dist or user profile was changed by this review. Browser BCR/scroll clamping, native text/portrait budget, screen-reader focus behavior and physical input acceptance remain separate Root checks.
