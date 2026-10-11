# Dock-crafted ability components

The Rust inventory has three stable component IDs. They are crafted items held at the dock, separate from tiered materials, named training-material subsets, mission packets and MEISHI point balances. Each stack holds at most 99. They do not consume cargo capacity and survive death, abandonment and reload. They are never granted by loot or by migration.

| Index / ability category | ID | Component | One-item recipe | Active RAICHO gate |
| --- | --- | --- | --- | --- |
| 0 / offense / RENKO points | `pulse-conductor` | Pulse Conductor | 2 banked T1 + 1 banked T2 | LV3 |
| 1 / defense / HUGO points | `aegis-lattice` | Aegis Lattice | 2 banked T1 + 1 banked T2 | LV3 |
| 2 / passive / PAN points | `flux-bearing` | Flux Bearing | 2 banked T1 + 1 banked T2 | LV3 |

Crafting a component costs no points. Its recipe becomes known when an ordinary ability in the same category reaches owned rank 2, or a same-category boss/champion blueprint is actually known. An already-owned component remains visible. Public registry membership and royal abilities alone do not discover recipes. Manufacture requires physical Haven, the active RAICHO's level, available banked stock, stack room and normal gameplay mode. Carried material cannot pay a dock recipe. Tier-1 spending uses the same existing unlabeled-first, stable-label debit as other dock recipes, keeping the three named subsets within T1.

Ordinary target ability ranks R3, R4 and R5 consume respectively 1, 2 and 3 matching crafted components **in addition to** the existing 10 matching category points and two materials of the target rank's exact tier. Existing active RAICHO level gates LV5/LV8/LV12 and chassis-family compatibility still apply. All costs are checked before any debit; failure changes no checkpoint, rank or event. A successful call consumes each cost once and advances one owned rank. Repeated calls are new next-rank transactions; there is no bulk action or automatic equipment change.

R1 crafting and ordinary R2 upgrades retain their existing recipes. The corporation starter's first training upgrade still uses 10 matching named T1 items, no points and no components, with its existing trained-hull0 R2 exception. Royal bundles remain victory rewards, cannot be crafted/upgraded and require their existing equipped slots. Previously owned higher ranks are retained without retroactive component payments or refunds. Three slots and PAN passive / RENKO offense / HUGO defense are unchanged.

## Save and API contract

`Profile.schema_version` is 7. `campaign.components` is exactly `[u32;3]` in the table's order. JSON schema-7 imports require the field, exact shape and integer counts 0..99. Missing, null, negative, fractional, string, over-cap and overflowing values reject deserialization; future versions reject. Schemas through 6 ignore any supplied component field and migrate to zero components, preserving prior inventory/ranks/XP/identity/training through existing canonicalization. This does not fabricate old provenance. A valid empty-cargo schema-7 reload is idempotent; carried material retains the existing explicit reload loss policy. Direct typed `Game::new` inputs still canonicalize corrupt counts to the finite bound; this is separate from strict JSON import.

`Game::craft_component(id: &str) -> Result<(), String>` crafts exactly one item. `Hud.components` contains discovered-only rows:

```text
ComponentHud { id, index, name, category, description, count, max_count,
               required_level, materials:[u32;5], can_craft, craft_reason }
```

`AbilityHud.required_component` is null when no next component recipe applies; otherwise:

```text
ComponentRequirementHud { id, name, amount, owned }
```

These fields are transient authoritative HUD data. The component workshop, autosave, IndexedDB version barrier and WASM delegates are separate shell integration owned by Root. Public diagnostics cannot manufacture/spend components, and this module adds no grant command. An explicitly supplied isolated QA checkpoint may exercise the real workshop without asserting naturally earned items.

The costs are authored progression values. Native transaction/schema/lifecycle tests verify their implementation; human balance, browser workshop interaction and device acceptance require separate evidence.
