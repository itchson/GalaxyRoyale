# Sponsored KIKAI competitors

Implementation commit `7be061bb387fac2c1a1155edabe4e25b42dd638a` from Root baseline `f6c7140348f752d53c3e30f96cf09376c256c455`. Owned changes only: `campaign.rs`, `campaign_runtime.rs`, `champion_roster_tests.rs`. The eight established competitive aliases, arena grids, indices, body assets, ability IDs, construction categories, hull tuning and actual attack controllers remain in their previous order.

| Index / alias | Type / main / sub | Compact identity | Sponsor | Current model / historical model alias |
|---|---|---|---|---|
| 0 Lyra Voss | DORA / PAN / PAN | APP#F344FEE | PAN | AKATSUKI / Dawnneedle |
| 1 Korr-9 | DORO / RENKO / PAN | ORP#2323 | PAN | TATE / Bulwark |
| 2 Sable Myrr | TALO / HUGO / RENKO | THR#A91CE204 | RENKO | YORUMICHI / Nightrail |
| 3 Xil of Aurora | TALO / PAN / HUGO | TPH#C7E5503B | HUGO | KESSHO / Prism |
| 4 Maera Quill | DORA / PAN / HUGO | APH#1A2B44D0 | PAN | KAZEHAYA / Windrunner |
| 5 Orun Vale | DORO / HUGO / HUGO | OHH#90BDE511 | RENKO | KUROGANE / Ironwake |
| 6 Vexa Null | DORA / RENKO / RENKO | ARR#E0A66F42 | HUGO | UZUSHIO / Voidcoil |
| 7 Atlas-0 | DORO / HUGO / PAN | OHP#7FA70C09 | HUGO | HINOHOKO / Sunspear |

These are authored NPC identifiers, including the user's exact APP/ORP examples; they are not claimed to be cryptographically generated player licenses. Current species is KIKAI. Historical encounter labels remain explicit traceability aliases, not a claim of surviving biological humans or new biological aliens. The eight new RAICHO model labels are Japanese-inspired fictional display names. This implementation makes no unsourced linguistic-translation claim.

Manufacturer and sponsor are independent data. Type controls the portrait silhouette; main manufacturer controls hair color; sub-manufacturer controls crystal color. Sponsorship does not recolor anatomy, alter a loadout or grant an ability. Existing craft silhouettes/palettes are untouched in this roster pass; no eight newly authored portrait faces or upgraded hull artwork is claimed.

## Exact runtime HUD additions

`ChampionHud` retains index/id/name/species/ship/ability/silhouette/family/category/position/patch/blueprint/crafted/discovered. Additions:

```ts
identity: null | {
  kind: "dora" | "doro" | "talo";
  corporation: "pan" | "renko" | "hugo";
  sub_corporation: "pan" | "renko" | "hugo";
  hex_id: string; // Authored lower-case NPC hex; variable length permitted.
  designation: string; // TYPE-MAIN-SUB-UPPERHEX
  short_designation: string; // A/O/T + MAIN initial + SUB initial + #UPPERHEX
};
sponsor: null | "pan" | "renko" | "hugo";
display_model: string;
legacy_model: string;
legacy_species: string;
introduction: string;
```

Known `ship` equals the current display model; `legacy_model` retains the former English label. Known `species` is KIKAI. The source-authored introduction distinguishes callsign, competitive alias, both manufacturers, arena sponsor, model alias and existing ability. Root owns binding this introduction and the typed identity to campaign/dialogue/native portrait UI. No shell/Wiki mutation is included here.

Existing production discovery predicates remain patch OR blueprint OR crafted OR persistent canonical-site discovery. Unknown entries expose `identity:null`, `sponsor:null`, empty display/legacy/introduction/reward text and `Unknown champion`. The fixed eight-entry array and indices remain, so arena interaction stays positional. Standings mask unknown display names too; seeded NPC results, player match results, points and tiebreak arithmetic do not change.

## Evidence and boundaries

Seven focused native tests on final source passed. They cover all authored identities/corporations, exact historic tuning and ability compatibility, actual public HUD serialization with no unknown metadata, real joystick movement into the discovery radius and exact checkpoint reload, legacy patch recognition, all eight real occupied-cell swept-projectile victories with exact once-only reward indices, and 60 diagnostic ticks for each encounter with exact profile equality. The victory fixture declares a10,000-damage shot; this proves reward/collision plumbing, not player balance. The approach fixture declares the initial world position, then uses real Game ticks for admission; it is not a browser playtest.

The full296-test core suite passed on the runtime changes before a final wording adjustment (`formerly listed`→`also known`) and test-only Clippy iterator cleanup. Final focused7/7 passed; strict all-target core Clippy, rustfmt and diff check passed. The initial stationary-projectile fixture failure is preserved separately: the opponent moved away before its tick-time collision check; the final fixture uses an actual swept shot through a near-center occupied cell.

No Profile/schema migration or save field was added; currency, ship XP, carried/stored material ledgers, tutorial facts, kills, patches, league histories and reward exception are unchanged. No main/UI, engine, gameplay asset, shared manifest, dist, server, ledger, push or publication change is included. Root integration/Wiki updates, actual browser presentation and human/physical acceptance remain separate.
