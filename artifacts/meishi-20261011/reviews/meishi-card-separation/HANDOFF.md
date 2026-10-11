# Standalone MEISHI CARD documentation correction — frozen

Commit `7f1e487782087ab9af1ce6b7c9c4be1e4ca4c1e6`, parent1bb6efac5ae4a124d83dd691f0da425af7e67ad8. No root cherry-pick in this isolated branch.
Only the four assigned docs changed; no source/build/Studio/service/API or push.

Current contract: MEISHI CARD is standalone read-only, with Close only. No cargo/hull/ability/stats navigation, tabs, Settings or save/update footer belongs inside it. It retains selected animated native profile/currentRAICHO/XP/loadout/corporatebalances/exactly8earnedpatchpositions as readouts. Dedicated native meishi-panel belongs to the same local group. Escape/status open live RAICHO menu; the MEISHI entry and separate collections are accessed there. Card/collection current browser status remains verification-pending.

Actual source inspection confirmed root pause defaults to pausepage, card hides menu-footer and collection-navigation, card-specific Close labeling, and bridge meishi-panel route. Exact read source hashes are retained in catalog-source-check.log; this is source inspection, not route execution or GPU acceptance.

Actual7/7 existing Wiki fact tests pass; catalog validates37stableentries, cardcontrolfact Close only and pendingstatus. git diff --check passes.

- `docs/KIKAI_LORE.md` rawSHA256 `965126fb301ca8a87c5e180a4a08a4bad91dd41bfd987c40a700c0e83a8c3faa`
- `docs/DEV_BIBLE.md` rawSHA256 `0c96762987c0b8e20270ecfff1d90d3c735252faf5789f4fe332d4231ec042e1`
- `docs/UI_REFINEMENT.md` rawSHA256 `bcdd2ef555ad18572acf1498f437a6754f970c98abf46850019ea557d6671e30`
- `docs/wiki/catalog.json` rawSHA256 `d87a4ac10c1650d483e2b2b800a59d873f558e9b3c92e2214a5384c64821e383`

Logs: work/meishi-card-separation/wiki-facts.log and catalog-source-check.log. Previous handoff remains unchanged.
