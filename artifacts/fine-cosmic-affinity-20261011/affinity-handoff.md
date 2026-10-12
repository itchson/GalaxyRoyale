# Authoritative RAICHO affinity HUD

Frozen commit `9196a866ca89e636a21d23a874849281b76b0cae`, branch `codex/raicho-affinity-hud`, exact base `f69d6cbe8d58d182da6e45460b42b963655c5fbc`. The isolated tracked checkout is clean. Root's browser-tested f69 source/package was not edited or rebuilt.

The existing HUD had pilot license/role and discovered champion identities but no active craft affinity state. This adds one transient field, using the same manufacturer, category and equipped-item predicate as the gameplay multipliers:

```ts
ship_stats.affinity: null | {
  corporation: 'pan' | 'renko' | 'hugo';
  slot: 0 | 1 | 2; // offense / defense / passive
  active: boolean;
  source: 'factory_license' | 'roster_manufacturer' | 'sovereign_license';
}
```

`factory_license` identifies hull0's licensed main manufacturer. `roster_manufacturer` identifies hulls1..8 using the actual roster main, independent of pilot/sponsor/sub. `sovereign_license` explicitly describes the current hull9 licensed-main affinity policy; it must not be presented as an invented authored Sovereign manufacturer. Anonymous legacy hull0/9 have `null`. A known manufacturer remains present when its matching slot is empty, invalid, unowned or incompatible, with `active:false`.

Public Rust types are `game_core::RaichoAffinityHud` and `game_core::RaichoAffinitySource`. Enum serialization uses the exact snake_case values above. Numeric slot mapping is the existing canonical offense0/defense1/passive2. No new strings, per-frame container, saved field, profile migration, recipe, equipped item or balance rule is introduced. Fixed three-slot capacity and empty-slot behavior remain.

Four changed files: `shield_progression.rs` exposes the typed metadata and factors the existing category match once; `lib.rs` contains only the type re-export, `ShipStatsHud` field and HUD constructor assignment; new `raicho_affinity_hud_tests.rs`; approved `live_overlay_tests.rs` field-count9/null assertion plus exclusion of the new object from its unchanged numeric-field check.

## Actual checks

Four new tests execute actual public Game HUD serialization, constructors and transactions: nine factory main/sub pairs; all eight owned roster hulls with anonymous and three cross-corporation pilots (32 combinations); public matching-slot clear; invalid/unowned/wrong-category slots; exact checkpoint/reload stability; anonymous factory/Sovereign null and explicit Sovereign source. HUD reads preserve the entire checkpoint, and slot transactions retain stored identity/hash/XP. TATE's PAN pilot/sponsor/horns still serialize RENKO craft affinity.

`cargo test -p game_core shield_progression --locked` passed22/22 (18 retained plus4 new;296 filtered),2.39s. Whole core native suite passed318/318, zero ignored/filtered,7.92s. Strict native all-targets and WASM-target core Clippy passed with `-D warnings`. `git diff --check` passed before commit. Logs and exact Git-byte source/input hashes are frozen in `manifest.json`. No failing run is represented as passing; no negative runtime test result occurred in this bounded implementation.

Declared owned-hull/victory fixtures are not natural discovery/play evidence. WASM-target Clippy is compilation only, not actual serialized browser output or a normal dual-backend build. UI/TS binding, user-facing wording, GPU/physical display and human clarity acceptance belong to a separate Root integration gate. No preview, server, browser operation, WASM command export, push, Root source or shared catalog edit was performed.
