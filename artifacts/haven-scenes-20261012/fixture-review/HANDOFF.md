# Haven shell fixture migration

Frozen commit `5df19ae371cae78b167e64921d84040d64cb4011`, parent `8c284637f3dfd4f86982f2dc01b10fb8a70c8162`, branch `codex/haven-scene-fixtures`. Only `scripts/test/dock-menu.test.mjs` and `scripts/test/forge-integration.test.mjs` changed. Runtime, art, shared manifests and production state are unchanged.

The fixtures now extract the actual current shell transitions/registered handlers and bind the real `forge-scene`, `hangar-scene`, `flight-menu` and `loadout` modules. CSS imports are explicit no-op model boundaries; the cosmetic portrait port is declared and the default legacy HUD has no portrait. Fake DOM/Game/storage adapters remain declared. These tests do not execute Rust, IndexedDB, a browser, GPU rendering or RTC.

The routing fixture uses all five current facility IDs/centers: Forge, Hangar, Missions, Multiplayer and Vault. Its boundary adapter reads actual native center/radius constants and models the native circle; the actual `FlightInput.worldPoint` method projects pointer coordinates. Added cases cover inclusive circle edges, square-corner/outside rejection, three zoom values, invalid coordinates and keyboard center activation without pointer projection. The central dock now explicitly fails to open multiplayer without changing flight/progression. Native circle physics itself still requires native/browser tests.

Forge opens on the discovered ability index and retains its local mode across category/component/hardware callbacks and mutation refreshes. New checks retain the actual navigation nodes and action focus, verify the blueprint craft argument, suppress selection in construction mode, and assert that craft does not auto-switch. The real Hangar view carries per-hull XP, uses the legacy Scout art fallback, opens the separate expanded statistics view and keeps its back node/handler outside the repainted statistics target.

All earlier cases remain, including delayed saves/flush, save conflict, memory-only storage, backend rejection without HUD/save/audio or optimistic success, quit/restart failures, hidden/resume/modal/recovery gates, audio-unavailable success, and post-mutation authoritative snapshot saves. Four case names were updated for changed runtime behavior; none were dropped. Six cases were added: **33 → 39**.

Actual evidence:

- `baseline-targeted.log`: 33 cases, 16 passed, **17 failed** on the unmodified isolated 8c source. Root's original full-shell failure log is separate and remains untouched; no later pass overwrites it.
- `migrated-first.log`: 36 cases, 32 passed, **4 failed**. These exposed the model's missing hidden-ancestor visibility rule and remaining outdated MEISHI navigation/Forge copy expectations. The fixture now models the hidden menu ancestor; the source-specific assertions were replaced with current facility navigation, ability index and authoritative XP/statistics assertions.
- `migrated-second.log`: 39 passed, 0 failed/skipped after those corrections and the additional mode/craft/detail cases.
- `committed-targeted.log`: actual final committed source, **39 passed, 0 failed, 0 skipped**.
- Pinned local Prettier formatting/check and Git diff checks passed. Production TypeScript was unchanged; no new typecheck or broad suite/build was run by this task.

Root owns integration and the frozen complete check/build. Actual WebGPU/WebGL2 layout, native world/transaction/save behavior, physical-device and performance validation remain separate. RTC preparation is checkpointed in `work/private-room-rtc-qa/CHECKPOINT.md` and unfinished; no stage/listener/client operation was performed or claimed.

`manifest.json` pins both Git sources, unchanged executed runtime inputs and preserved raw evidence. `verification.json` records source/count/scope checks.
