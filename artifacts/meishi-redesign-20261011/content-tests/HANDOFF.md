# Compact MEISHI view tests

Frozen tests-only commit `81d0b6ff0aa80b958b52caff15dad2f3aaa7a19a`; exactly `scripts/test/meishi.test.mjs` changed. Parent `e900a83b258a26a80024bd12632cdf8990a77fba` separately contains the requested docs-only content review. The tracked checkout is clean.

Root's changed `meishi.ts` was copied temporarily as the actual test input, retained byte-for-byte in `root-meishi-input.ts` (SHA-256 `28bc379f64964dc68ab47229c114373553179151021e4728d33f51b83c9e8015`), and restored to the worktree's committed original before committing. No TS/CSS/runtime source, Root checkout, browser, server, native build or dist output was changed by this task. This tests-only commit expects integration with Root's new painter; running it against the branch's restored historical painter is deliberately not a passing integrated state.

The former two tests demanding removed live XP/vitals/loadout text are replaced with actual selected RAICHO icon/name/level, accessible level labels, stable LV1-to-LV100 refresh, unavailable invalid level and point values, and absence of removed rows. Existing identity/points/eight indexed socket/discovery/earned-loss/stable-node checks remain.

Two new regressions reject reading removed HUD fields through throwing getters, and require zero DOM writes plus identical nodes/focus/scroll when only those fields change. They execute the actual compiled TS painter, not a restated implementation.

Actual results:

- Old twelve tests against Root's new painter: ten pass/two fail at obsolete row access (`old-expectations-fail.log`).
- The two new tests against the committed historical painter: both fail, one on `ship_xp` getter and one on live-field DOM mutation (`new-regressions-old-painter-fail.log`).
- Updated MEISHI tests against retained Root input: fourteen pass (`focused-pass.log`).
- Final formatted tests plus actual bridge/Stats suites against retained Root input: forty-nine pass (`combined-pass.log`); fourteen MEISHI, twenty-one bridge and fourteen Stats.
- Prettier formatting and `git diff --check` passed. Only the test file is committed.

These are native Node/TypeScript VM DOM-adapter results. They do not establish browser BCR, visual fidelity, native packet budgets or responsive/touch acceptance. The earlier source-only regression review remains immutable under `work/meishi-layout-review/`.
