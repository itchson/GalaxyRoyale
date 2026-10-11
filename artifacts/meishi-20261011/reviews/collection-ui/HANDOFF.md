# Discovered collections UI handoff

Frozen commit `37cdd564b55476b242f65660518d72af159e9c7b` on `codex/collection-ui`, parent `bc910fabaed6695c502f50ff3f86945817584156`. Tracked checkout is clean. Seven owned files changed: campaign.ts/css, loadout.ts/css and their campaign/loadout/Forge integration tests. No Root source, main/index/bridge, gameplay/save, manifest, build/dist, Studio, API, server or browser changes.

The existing `paintForge`, `resetForge`, `paintShipHangar`, `paintCampaign` exports and callback/HUD contracts remain intact. All filters are semantic buttons supported by the existing native painter; no search/input/caret API was added. Root owns the separate MEISHI/Cargo/Hulls/Abilities navigation integration.

## Result and integration

- HULL COLLECTION uses only the core-provided genuinely found hull list. All shows every supplied entry; Owned uses `owned`; Blueprint uses `craftable && !owned`. A found but undefeated arena hull remains visible in All with `FOUND / BLUEPRINT REQUIRED`, without pretending it is craftable. Owned and active hulls have separate states; core construction/selection booleans and reason strings remain authoritative.
- Forge retains the three slot overview and six hardware actions. `BROWSE ABILITIES` opens ABILITY COLLECTION across all categories; existing MANAGE opens that category. Category buttons select All Slots/Offensive/Defensive/Passive. All/Owned/Blueprint/Compatible filters run only after `blueprint || owned`; Blueprint means known and not crafted. Compatibility uses the core flag, never a guessed manufacturer or family. Replaced items remain owned; crafting does not auto-equip.
- Card recipes distinguish MEISHI from actual banked tier materials or the matching named training subset. Required RAICHO level, owned/effective rank and compatibility are shown from HUD fields. Reasons and `can_*` gates remain authoritative even when visible balances appear contradictory in a declared test fixture. No client transaction/affordability rule was introduced.
- Repeated cargo/stash tier inventory and loss messages were removed from collection pages. Actual banked quantities remain alongside each recipe. Root's Cargo page owns cargo/risk/loss inventory presentation.
- Campaign's existing discovery gate remains. Found/blueprint/owned hull status now names the Hull Collection destination without granting rewards or exposing unknown champions.
- Ephemeral WeakMaps hold only view category/filter/active hull, not inventory or progression. Forge reset/ship change returns the overview. HUD repaints restore action focus and scroll; a successful craft can move focus to its real Equip button. Outside focus is not stolen. Empty filters show a truthful empty message. All game-provided strings continue through `textContent`.

## Executed evidence

Node v24.18.0 / installed locked TypeScript 5.9.3 and Prettier 3.9.9. Commands ran in this isolated checkout:

```text
node --experimental-transform-types --test scripts/test/loadout.test.mjs scripts/test/campaign.test.mjs scripts/test/forge-integration.test.mjs
pnpm typecheck
pnpm test
pnpm exec prettier --check [the seven owned files]
git diff --check
git diff --cached --check
```

Focused suite: **33/33 pass**. Full shell suite: **404 pass / 1 existing skip / 0 failures** (405 total). Strict TS, formatting and diff checks pass. The focused tests execute the actual view/shell functions with fake DOM/Game/save adapters; they prove filter selection, discovery-first hiding, unchanged callback arguments, no HUD mutation, action gates, named recipes, category/reset state, focus/scroll continuity, cargo separation and owned/effective disclosure. They do not execute Rust transactions or prove browser layout/native glyph clipping/GPU budgets.

Negative evidence remains in `first-focused-fail.log` (16/26 pass: a new source convenience relied on a fake-DOM missing lastElementChild getter; it was simplified to an explicit node reference, plus old cargo/PTS assertions superseded by the approved UI contract). `new-filter-fixture-fail.log` retains the later 32/33 result: an owned incompatible fixture incorrectly left can_craft=true; it was corrected to the real core contract without changing sort/runtime gates. Final logs are separate. No passing result is attributed to a peer review.

## Existing recipe gap for the next milestone

Current authoritative recipes still consume materials, not pre-crafted components: ordinary hull construction is 10 matching category points plus 6 T1; starter ability craft is 3 points plus 2 T1; other craftable abilities are 10 plus 2 T1; normal ability upgrades cost 10 plus 2 of the next-rank tier; hardware costs 10 plus 1 of that tier. The verified starter training first upgrade uses 10 banked matching named T1 and no points. Royal reward items are not construction recipes. There is no crafted-component inventory/recipe requirement in this change; it requires a separate real core/save contract. UI labels do not relabel tier materials as crafted items.

Root must compose this commit with its newer navigation/menu work and run the final integrated checks/browser inspection. No preview service/game build/push or actual browser/phone/user readability acceptance occurred here.

`frozen-hashes.json` pins exact committed relevant inputs, current executed source bytes and raw logs. Prior PAN dock review and signaling freezes remain unchanged.
