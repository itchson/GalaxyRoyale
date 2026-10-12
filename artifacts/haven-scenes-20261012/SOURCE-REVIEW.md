# Haven scene source review

Task: read-only Root UX integration review. Reviewed the stable dirty source snapshot captured in `source-manifest.json`, based on Root HEAD **f4a6d46502850f189406862c5d1ebf1eeae7b39a**. Fourteen files were copied into this review's own work folder; their raw SHA256 pins and immediate unchanged-after-capture checks are recorded. Root may subsequently change these files. The frozen browser candidate 3c00f413 was not edited.

This is source/control-flow review only. No build, browser, rendered screenshot, server, worker, pixel comparison or physical-device validation was performed for this task.

## Findings

### P1 — Full-screen backing can cover menu feedback

`apps/game/index.html:22` places game-ui before the menu surface at230. `apps/game/src/pixel-ui.ts:215` visits roots in document order, emits the non-flight toast through the game-ui group at522–525 and appends each group at612. The later menu root emits kind scene at461. `crates/game_web/src/sports_ui.rs:218` gives that scene an opaque whole-rect backing; `ui_scene.rs:1841` iterates the groups directly. The renderer uses alpha blending without depth (`renderer.rs:255/263`), and `.pixel-ready` makes the semantic toast opacity0 (`style.css:1041`). Consequently a forge/save/action error written through `main.ts:591` can be hidden behind the subsequent full-screen background.

Root accepted this source finding and intends a final native feedback overlay with a reserved scene footer slot. Keep only one toast node, preserve correct clipping, and place it after menu chrome/content while retaining console/dialogue layering. Actual visual acceptance remains pending: cause a rejected forge action and a menu save error, confirm their visible native feedback and top-right close at short/wide/narrow aspects.

### P2 — DOM facility clicks bypass the stated native world-space footprint

The canvas route correctly checks the actual world point (`main.ts:897`). The semantic facility click handler (`main.ts:2098–2112`) instead calls dock_facility_at on the facility's constant center and then opens its page. Its DOM target is a square with minimum44px (`main.ts:1462`), while `docs/HAVEN_SCENES.md` specifies a circular24-world-unit hit radius. Thus square corners, and especially the enlarged minimum target on a short/zoomed-out view, can open a facility even though the actual cursor world point would fail the native test.

For pointer clicks, map event.clientX/Y through the existing unshaken camera/input conversion and require the returned facility index to match. For deliberate keyboard activation, retain the accessible center activation. If enlarged touch targets are intended, document and implement that as an explicit interaction rule rather than claiming the same native radius. This is a source mismatch; no inaccurate browser click was fabricated.

### P2 — Mission refresh replaces focused controls on unrelated live-state changes

`missions.ts:151–159` signs the entire campaign HUD even in Expeditions. On any changed signature, `current.render()` runs and `missions.ts:83` replaces the full body, including the APR contact button and League controls. `main.ts:1703` checks it each second while the universe runs. An encounter's changing health or another unrelated campaign field therefore recreates an Expeditions contact control; a focused League action also disappears when its HUD changes. The outer scroll container is preserved, but focused control identity and local League ledger state are not.

Use section-specific signatures and persistent/keyed action rows, or explicitly restore focus to the same semantic action after a necessary rebuild. Avoid letting a changing unrelated campaign field replace the Expeditions body. Validate with keyboard focus on CONTACT APR-1337 and on a League action while a relevant HUD change occurs; do not claim this from static screenshots.

### P3 — Mission acceptance still points to a removed Journal menu

`main.ts:2377` says `EVENT ACCEPTED / see Journal for progress`, while the route is now MISSIONS (`main.ts:235/1025`). Update that copy to MISSIONS. Internal quest_log names are acceptable; the player-facing destination should be the actual menu.

## Source contracts that are correctly retained

- Dedicated main route table is Forge/Hangar/Missions/Multiplayer (`main.ts:235`); the central interaction path has no direct open-coop special case. The explicit multiplayer HUD and facility3 route target the same semantic scene.
- Full-screen CSS width/height100%, opaque native kind Scene, content `min-height:0` + overflow:auto, and the absolute top-right close provide the intended layout structure (`style.css:329/1238/1262`). The later scene selector overrides the legacy co-op dialog dimensions at equal specificity. This does not prove actual fits at every aspect ratio.
- PixelUiBridge intersects every enclosing scroll rectangle on both axes (`pixel-ui.ts:279–304`), so Missions was added to the existing clipping path. The close is outside those scroll containers.
- MEISHI keeps its separate measured collectible frame inside the full-screen scene (`pixel-ui.ts:470–480`), and main hides collection navigation for the MEISHI page (`main.ts:1053`). Its existing portrait, corporation treatment, currencies and eight sockets remain owned by the card module. No new tabs or action buttons were added to the card.
- APR contact closes the menu before opening the bottom dialogue (`main.ts:457–461`); League still uses the authoritative campaign actions rather than client-owned rewards. Menus reset interrupted controls and continue simulation; dialogue/debug/lifecycle gates remain separate (`main.ts:795–873`).

## Next integration boundary

Root owns the toast fix, full-screen geometry, facility pointer routing, mission refresh/copy, native chrome and actual dual-backend checks. Latest user steering adds a distinct Vault, a garage-style Hangar with horizontal hull index and separate RAICHO detail, and purpose-specific Forge sections. Those forthcoming modules are not evaluated or claimed implemented by this frozen review. Root will assign the bounded Hangar module next; this review does not expand ownership into shared files.
