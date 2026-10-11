# Supplied all-eight-earned MEISHI QA fixture

Import `all8-earned.json` only on the isolated QA origin. Its eight patch awards are deliberately supplied data, not naturally earned gameplay. It is a raw schema7 backup profile for the public Restore Backup action; no runtime diagnostic grant or production source change is involved.

Root's rejected fixture set all `campaign.patches` and chassis `campaign.blueprints` true but left champion ability IDs6–13 with `blueprint:false`. The actual TypeScript validator requires each of those blueprint flags to equal its corresponding patch. Native `ship_progression::initialize` enforces that same relationship, and `CampaignProgress::sanitize` mirrors chassis blueprints from patches. The repair changes exactly those eight ability blueprint flags to true; all their ranks stay0.

The supplied PAN/DORA/RENKO identity, seed, revision, factory XP200, point balances30/30/30, banked stock, starter ranks, loadouts and component counts are unchanged from the original. No champion chassis becomes crafted, no royal result/victory/credit is granted, and no champion ability is owned or equipped. All eight patch flags make the tournament available under the existing rule; they do not claim tournament completion. Onboarding remains null.

Executed on Node24.18.0 against Root's actual `apps/game/src/storage.ts`:

```powershell
node --experimental-transform-types work/meishi-layout-tests/earned-fixture/validate.mjs
```

Six checks passed: retained original rejection; exact minimal diff; repaired validity and idempotent normalization; sixteen individual patch/ability mismatch mutations rejected; no chassis/royal grants; actual `SaveStore.replace` followed by `open` preserves the exact fixture on an isolated in-memory `fake-indexeddb` adapter. The adapter touches no browser or user database. `validation.json` and `validation.log` record actual output and input hashes. No Rust executable, browser import, renderer or natural-earn result is claimed here. Root owns the separate public browser import/capture.
