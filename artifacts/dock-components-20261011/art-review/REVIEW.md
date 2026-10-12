# Independent component and Forge review

Verdict: **no blocking recipe, migration, discovery or atomic transaction
defect found** in exact core commit
`373a1fd16ce02b7879cf95122087f555973d4558`.
This is source/native transaction and shell VM review, not browser, naturally
earned inventory, human balance, co-op or physical-device acceptance.

## Reviewed behavior

- Exactly three stable category IDs: Pulse Conductor (0 offense/RENKO), Aegis
  Lattice (1 defense/HUGO), Flux Bearing (2 passive/PAN). A stack is bounded to
  99 and remains banked inventory, outside material cargo/capacity.
- A real owned ordinary category ability R2+, an actually known same-category
  boss/champion blueprint, or retained owned part reveals the recipe. Registry
  membership and royal-only ownership do not reveal it.
- One dock craft requires normal gameplay, physical Haven, the active RAICHO's
  LV3, stack room and 2 banked T1 + 1 banked T2. Carried stock cannot pay. The
  existing shared T1 debit preserves named-subset accounting.
- Ordinary target R3/R4/R5 requires exactly 1/2/3 matching components alongside
  the old category points and exact-rank tier costs. Existing level/family
  gates run before debit. Failed calls alter no checkpoint or events; successful
  repeat calls are distinct next-rank transactions, not an idempotency promise.
- First craft, ordinary R2, the explicit corporation training first upgrade
  and royal reward restrictions keep their separate rules. No previous rank
  is downgraded, refunded or retroactively charged.
- Schema-7 JSON requires exact `[u32;3]` counts 0..99 and rejects missing,
  fractional, negative, overflowing, over-cap and future-schema values.
  Schemas through 6 ignore supplied component stacks and migrate to zeros;
  normal prior XP/ranks/material/identity data retain existing canonicalization.
  Direct typed Game inputs still sanitize counts, which is distinct from strict
  JSON import. The custom Profile field list covers every current profile field.
- Banked components survive death, abandonment and reload. Existing carried
  material loss remains explicit; no components or XP are fabricated.

The reviewed Forge uses game-provided discovered rows, recipes, eligibility and
reasons. It forwards exact component IDs, refreshes HUD, then snapshots the
profile. Repaint preserves workshop/category state and action focus; a fresh
Forge open or active hull change resets navigation. A native action error has
no invented save/success. Save completion stays separate from transaction
success, as verified with a deferred rejected save.

## Concrete source finding and resolution

The initial workshop icon CSS multiplied the intended 48 CSS pixels by
`--ui-scale`; at scale 2.4 this would be 115.2 physical pixels. This contradicted
the compact icon target. Root changed it to fixed `width:48px;height:48px`.
The final source read verifies that correction. No browser observation is
claimed. The art still has all 64×64 source cells; native continuous fitting
and shared pixel/work admission are separate Root integration tests.

## Actual commands and pins

In the frozen Gameplay worktree:

```text
cargo test --locked -p game_core component_tests -- --nocapture
```

**17 passed, 0 failed**, exit 0. This independently reruns the author's focused
native tests; it does not claim newly authored independent adversarial tests
or duplicate full-workspace/Clippy acceptance. `native-tests.log` retains output.

In the current Root checkout:

```text
node --test --test-name-pattern='component|Forge forwards|backend rejection|mutation success' scripts/test/loadout.test.mjs scripts/test/forge-integration.test.mjs
```

**5 passed, 0 failed**, exit 0. These execute the actual TypeScript view/shell
functions using fake DOM/VM dependencies. They do not exercise real WASM,
IndexedDB, browser geometry or persisted natural progression. `shell-tests.log`
retains output. No source/build/browser/service/API mutation was performed.

`pins.json` records six exact committed core input matches, five current Root
shell inputs, art source and both logs, including raw and canonical-LF hashes.
Root HEAD at final read was `d7217c6efdb8a5d186419ac2ded7877750203c87`.
Pins JSON SHA256:
`16f12be62497a226e485a6a51c8323adf71c52c09c7950e9322f0e7e6761d130`.

Real peer exchange before and after implementation covered category order,
banked-only spending, discovery, active-hull gates, legacy zero migration,
named-subset debit and royal/training exceptions. This records actual messages
and their resolved source contract, not a fabricated meeting. Normal browser
workshop transactions, progression pacing and physical devices remain pending
for this independent review scope.
