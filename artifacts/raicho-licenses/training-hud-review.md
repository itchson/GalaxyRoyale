# Independent training HUD copy review

Verdict: no blocking source defect found in `62f82d17164e5e8268e3dabf05b76bcc210d9c5a`, based on `03fc8b5bf60338b92cc680aead47bfb05519f10e`. The candidate checkout was clean before and after the focused native execution. This review is separate from the earlier frozen training-runtime review.

## Exact scope

The complete committed diff changes only `crates/game_core/src/identity.rs` and `crates/game_core/src/training_tests.rs`. The runtime hunk is confined to `identity::onboarding_hud`: it formats description strings from existing authoritative fields and reuses local copies of unchanged material, slot, cargo and stash fields. It adds no serialized field, transaction, event, reward, ability, portrait, material-pickup geometry, or API. Source snapshots, raw SHA256 values, committed blob IDs and unchanged dependency comparisons are retained in `source-pins.json`.

## Findings

- Stage 2 reports `asteroids_destroyed/10`, including 0/10 and each intermediate count. Its any-variety and Haven-belt versus outer reward text agrees with the existing actual defeat policy (`identity.rs:317`).
- Stage 3 reports admitted `materials_collected/10`, the corporation's native material name and its actual assigned birth variety: PAN → Phase Silicate/Porous; RENKO → Iron Filament/Metal; HUGO → Coolant KESSHO/Ice (`identity.rs:321`). It does not invent drops, collection progress, or a second material ledger.
- Stage 4 separately reports current carried stock, current banked stock and retained 10/10 collection history. Replacement after loss/spend and physical return are truthful; reload is not described as banking (`identity.rs:325`). The repeated /10 is a goal denominator, not a cap on legitimately larger stock.
- Stage 5's R1 branch names the exact ability, category slot and corporate starter RAICHO. The ten matching banked material/zero point/LV1 recipe agrees with the unchanged `training_upgrade_check` and transaction. Its separate owned-R2+ branch explicitly states recognition without another material cost (`identity.rs:328–335`). Real equipment/home/progression prerequisites remain authoritative. The unchanged `can_upgrade` is the debit-recipe availability; owned-upgrade recognition remains an equipment/home path rather than a second upgrade transaction.
- HUD extraction remains read-only: one new test compares exact profile JSON before and after extraction. Existing counter, ledger, migration, effective-rank, diagnostic, capacity, reward and atomicity tests still pass.

## Independently executed check

From the exact clean candidate checkout:

```powershell
cargo test -p game_core training::tests --locked -- --nocapture
```

Actual result: **16 passed, 0 failed, 0 ignored; 273 filtered**, completed in 1.73 seconds. The log is `native-training-tests.log`. This comprises the retained 13 runtime tests plus three new copy regressions. The new regressions use actual Game HUD extraction after real asteroid defeat/projectile paths, pickup admission, physical banking and public upgrade/equip operations. The R4 ownership and 560 XP input are declared fixtures, not claimed naturally earned progress; named pickup fixtures enter the actual capacity/pickup path. Lost stock is exercised through the actual constructor boundary; generic spend uses the actual shared debit helper. The existing combat replay's 39 matching-variety defeats per corporation remains a controlled fixture result, not balance or natural-play duration evidence.

Compiler: `rustc 1.97.0 (2d8144b78 2026-07-07)`. Gameplay's separate strict Clippy/format checks are peer-reported and are not additional executions by this reviewer.

## Remaining acceptance

No browser, normal WASM release, raster/layout, paragraph scroll, mobile device, new profile-save or publication was executed by this review. Root's actual serializer/browser check must confirm the new native descriptions are displayed completely and update after real events. The underlying frozen native/profile/portrait contracts were not changed by this two-file delta. Earlier runtime and licensed-portrait reviews remain immutable at their separate Root work paths.
