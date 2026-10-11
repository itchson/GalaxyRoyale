# Ship-Local Storage Audit - schema4 Preparation

Bounded read-only worker audit. Generated 2026-10-11 (AEST). No production source or manifests edited.

## 0. Snapshot caveat - the tree moved during the audit

`apps/game/src/storage.ts` was edited by the producer WHILE this audit ran:
500 lines @09:35 -> 816 @09:36 -> 844 @09:43 -> 845 @09:49.
Schema4 code (`schema_version: 4`, tiered `[u32;5]` cargo, `abilities`, `loadouts`,
`materials_lost`, IDB version `2`) is ALREADY PRESENT in the working tree, not merely
planned. Line anchors below are the verified state at read time 09:49; re-verify before
wiring tests to exact line numbers.

## 1. Facts in current source (verified)

### Save normalizer - apps/game/src/storage.ts
- `validProfile` (L208) accepts schema 1|2|3|4 (L222-225). Schema4 branch (L319-391):
  `cargo_materials`/`stored_materials` must be 5 integer tiers with cargo sum <= 16
  (L321-322), 17 `abilities`, 10 `loadouts` (L324-327), plus rank/unlocked/`ranks`
  derivation cross-checks (L330-389). Schema3 branch (L390): scalar
  `cargo_materials` <= 16, scalar `stored_materials`.
- `last_run` on schema4 additionally requires the `materials_lost` tier array (L419).
- `normalizeProfile` (L440) always emits `schema_version: 4`; output `ranks`/`unlocked`
  are DERIVED from `loadouts[selected].hardware` + `abilities[3..6]` (L448-452).
- `migrateCampaign` (L482) copies campaign fields for 3/4 (L484-512); schema4 deep-copies
  tiers/abilities/loadouts (L514-523). Schema3 -> 4 (L526-538): cargo zeroed,
  `stored_materials = [min(0xffffffff, stored + cargo), 0,0,0,0]` - this is the one-time
  legacy total-asset preservation into tier-1 stash (old game retained cargo on death),
  saturating at u32 max, never wrapping. New non-legacy campaigns start XP0/LV1 from
  `emptyCampaign()`; recorded `ship_xp[10]` preserved verbatim (L495).
- `SaveStore.open` (L599): `indexedDB.open("galaxy-royale", 2)` (L612); `onblocked` (L613);
  `onversionchange` closes db + sets `invalidated` (L623-630); `VersionError` rethrown as
  `SaveConflict` (L667-671). Migration writes in ONE tx (L643-651); on failure keeps
  in-memory `loaded` and advises backup (L672-679).
- Queue semantics: `save()` (L684) rejects when `invalidated` (L685); returns existing queue
  while `imported` (L689 - old sim snapshots must not overwrite an import); skips
  `revision <= latest` (L694); re-reads persisted inside the readwrite tx and throws
  `SaveConflict` on fork (L701-714); puts `latest`->`previous` then `copy`->`current`
  (L715-716).
- `replace()` (L739) normalizes import, requires db, single pending-import guard
  (L751-754), then in-tx persists `persisted ?? latest` -> `previous` and `copy` -> `current`
  (L764-769). `flush()` drains queue (L792).

### Rust authority - crates/game_core/src/
- `campaign.rs` L19-22: `cargo_materials`/`stored_materials` are `[u32;5]` with
  `#[serde(deserialize_with = "deserialize_materials")]` (fn at L66) - scalar legacy JSON
  still deserializes. `sanitize` clamps cargo tiers to a combined 16 (L130-134).
- `ship_progression.rs` `initialize` (L166-216) is the CONSTRUCTOR BOUNDARY ONLY (comment
  L165: "Save validation must retain live flight cargo"): `legacy=true` folds total
  `stored+cargo` into `stored[0]`, cargo zeroed (L169-173); `legacy=false` captures
  `loss = cargo`, zeros cargo, and reports the loss so a schema4 reload abandons unbanked
  cargo.
- `campaign_runtime.rs` `bank_cargo` (L43-53): per-tier saturation move cargo -> stored;
  bumps revision only when something banked.
- `lib.rs`: `legacy = schema_version < 4` / `migrated = != 4` (L494-495); `initialize` +
  `schema_version = 4` + single revision bump when anything changed (L523-527); construction
  banks retained courier cargo at Haven (L568-571). Death path: `respawn_if_dead` calls
  `discard_cargo("death")` (L1110); `reset_to_dock` calls `discard_cargo("abandonment")`
  (L2628). `discard_cargo` (ship_progression.rs L333-345) zeroes cargo, records
  `material_loss` + reason, accumulates into `run.materials_lost[i]`.

### Existing tests - scripts/test/storage.test.mjs
Command run (actual): `node --experimental-transform-types --test scripts/test/storage.test.mjs`
Result: **22/22 pass, 0 fail** (exit 0, ~148ms), against the current schema4 tree (test file
mtime 09:42; the producer is concurrently editing it).
Worth retaining as-is:
- No-double-spend / equal-or-older revision rejection + snapshot isolation (L130, L341).
- Previous-snapshot recovery on corrupt current, incl. generation-counter variant (L165, L806).
- Import ordering, import-input capture, old-session save lockout (L195); invalid-import
  preservation (L219).
- Cross-session `SaveConflict` on stale writer, including newer proposed revision (L236, L778).
- Failed-write leaves accepted snapshot intact (L253); same-revision replay cannot mint
  currency/boss awards (L130, L539, L713).
- Legacy v1/v2 -> v4 migration preserves progression, blank campaign, no rewards (L263, L316, L476).

## 2. Producer-declared schema4 rules (NOT yet verified in source - do not treat as facts)
- Physical home arrival is what banks materials (`bank_cargo` exists; the exact "arrival"
  trigger and UI wiring were not audited end-to-end).
- Death/restart/return-title/reload abandonment forfeits UNBANKED cargo while
  currency/stash/shipXP persist. The Rust `discard_cargo` paths exist; the JS-side loss
  report and the single authoritative HUD/brief text summary were not fully traced.
- Three compatible ability slots + tiered crafting materials exist in types
  (`ShipLoadout.slots`, 5-tier arrays); gameplay balance/craft formulas are out of scope.

## 3. High-value new test cases (8)

1. **schema3 -> schema4 asset conservation**: scalar `stored_materials=S, cargo_materials=C`
   -> `stored_materials[0] === min(2**32-1, S+C)`, tiers 1-4 and all cargo tiers 0;
   currency/ranks/`ship_xp`/boss counters byte-identical; exactly one revision bump from
   `migrated`; no duplicate grant on re-save at the migrated revision.
2. **u32 saturation boundary**: schema3 `{stored: 0xffffffff, cargo: 16}` ->
   `stored_materials[0] === 0xffffffff` (no wrap), cargo zeroed; still one bump. Mirrors the
   `min(0xffffffff, ...)` at storage.ts L530-533 and `saturating_add` at
   ship_progression.rs L171.
3. **Idempotence + second-load stability**: `normalizeProfile(normalizeProfile(x))` deep-equal
   for schema3/legacy inputs; a second `SaveStore.open` of the migrated record performs no
   further migration put and no message change (current/previous byte-stable).
4. **Schema4 reload abandonment**: saved schema4 profile with nonzero tiered
   `cargo_materials`; reopen without banking -> cargo tiers all zero,
   `run.materials_lost` records the prior cargo (no double-count if `materials_lost` already
   nonzero), stored/currency/ship_xp untouched; `discard_cargo("death"|"abandonment"|...)`
   reasons distinguishable in the loss report.
5. **Home banking exactness**: bank with mixed-tier cargo -> each tier moves cargo->stored with
   per-tier u32 saturation (campaign_runtime.rs L47-49), totals conserved
   (`sum(stored_before + cargo) == sum(stored_after)`), cargo zeroed, and NO revision bump
   when cargo was already empty (L50-52).
6. **Previous-snapshot recovery after schema4 corruption**: valid schema4 `current` +
   byte-corrupt/invalid `previous` (e.g. `stored_materials` non-array, cargo sum > 16, or a
   slot reference violating `compatibleAbility`/`abilityCategories` L66-72) -> open keeps
   normalized current, message flags recovery, and the invalid `previous` never overwrites
   `current`; both-saved corrupt -> visible new-profile state, no silent zero.
7. **Import + concurrent stale session**: (a) `replace()` of a schema3 export yields normalized
   schema4 in `current` and demotes prior to `previous`; a second `replace()` while one is
   pending rejects and leaves `current` unchanged; an equal-or-older-revision import still wins
   over a queued simulation save (imported gate L689). (b) Tab A saves rev N; Tab B (stale
   latest) save throws `SaveConflict`, persisted record stays Tab A's, and Tab B recovers by
   reopening; after `onversionchange` fires in a tab, its queued save/replace reject via
   `SaveConflict` without writing (L685, L740).
8. **Legacy new-content guard**: schema1/2 profiles carry no campaign -> migrated `campaign`
   equals `emptyCampaign()` defaults (all-zero `ship_xp`, empty stash) with no material/XP
   grant; existing schema3 `ship_xp` roster preserved exactly through `migrateCampaign` L495.

## 4. Concrete risks noted
- Moving target: schema4 landed mid-audit; re-run this audit against the final tree before
  relying on the line anchors above (09:49 state).
- TS `migrateCampaign` saturates `stored[0]` via `Math.min(0xffffffff, ...)` but relies on
  `validProfile` for shape; the Rust `sanitize` (campaign.rs L116-134) clamps cargo to 16 but
  never re-clamps `stored`, so both sides assume well-formed stash values - pin the boundary
  with test #2.
- `save()` returns the shared `this.queue` when `imported` (L689): a caller awaiting that
  promise resolves on the import's queue, not a new write - tests should pin that the import
  result, not the stale caller snapshot, is what lands.
- No export/secret or save-format changes were made; no production files touched.
