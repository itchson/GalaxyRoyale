# Independent node and live RAICHO integration review

Verdict: **no blocking integration finding** in exact Root commit **77442770e4abd9d6e36e9691df65512e385f6aa2**. This is a read-only review. It does not replace visible browser/device or natural encounter acceptance.

Reviewed integrations: seeded nodes **091751fc212d27594b6dd4a60eb4d2b2b4e781b9**, fine helper **a50aab678c98466e8c54838e39ca5a40a9f2b1b7**, real Game hook **8242c09650482a15019e8e1f0f96b35debed8d58**. Candidate reviews remain unchanged in sibling `seeded-node-independent-review` and `raicho-overlay-independent-review` folders.

## What survived integration

The integrated `regions.rs` and dedicated node tests are byte-identical to reviewed fe96d002. The integrated fine helper/tests, atlas and actual engine frame/pixel/affine/camera APIs are byte-identical to reviewed 1c73ad32. `source-checks.json` records all nine comparisons, 21 current committed input hashes and commit ancestry. No line-ending tolerance was needed. The three hook runtime/test files are unchanged between 8242c09 and the review HEAD; subsequent reviewed HEAD changes are documentation and shell CSS, not these runtime paths.

The canonical radius assignment remains `campaign_runtime.rs:289`. The independent seeded envelopes and Haven corridor therefore survive composition: site sanctuary, remote belt origin, arena wall and ID lookup use the same canonical Site. No new neighbor cache, palette selector, profile migration or simulation RNG dependency was introduced. Wider arenas and same-seed node/ring relocation remain intentional balance changes, not permanently empty corridors after drift.

The real extraction loop interpolates motion and heading once before either append path; the shared tint callback begins at `lib.rs:3562`. Only a real player with selected factory hull 0 and a saved identity enters the fine route, using its main corporation. Anonymous, crafted and Sovereign hulls retain their original append. The helper uses that actor affine and the original ACTOR material; no shader, geometry minimum, camera shake or zoom LOD was added.

The coarse callback preserves the original `animate_actor_cell` and real player tint. Fine colors receive the same health/maximum hull/recent-damage/cosmetic-time/Reduced Motion tint without recoloring neutral alloy or glass as corporate accents. Source signal breathing stays the helper's narrow RGB-only effect. No health, progression, collision occupancy, exhaust port or radius is mutated during render.

`lib.rs:3599` draws the identical existing live body whenever cosmetic append rejects. The helper preflights before callbacks and writes, so fallback cannot duplicate a partially appended pair. Each call has a fresh bounded local 3,000-cell/two-transform quota; this is not a global scene cap. After successful or fallback body extraction, the unchanged real exhaust and shield branches still execute. The original 48×48 body remains authoritative for damage and visibility; the overlay does not refill removed cells.

## Actual retained evidence

I inspected and copied Root's existing logs; I did not rerun tests, build, serve or use a browser. `raicho-game-hook.log` records two passing actual Game extraction tests: all three corporate fine counts, real PixelBody damage masking, profile preservation, world-viewport extremes, and actual hit/critical tint with unchanged alpha. The viewport loop checks packet geometry; it is not a GPU zoom visibility measurement. These fixtures do not force helper quota failure through Game; that rejection/no-partial-write contract is covered by the retained helper packet tests and reviewed fallback branch.

`nodes-fine-ui-native.log` records 631 native passes (37 engine+3 native-preview+351 core+209 web+31 layout), zero failures and one existing cosmetic export ignored. It includes the two Game tests, ten helper tests and ten node tests. Three retained logs record strict native, WebGPU-WASM and WebGL2-WASM Clippy success. These are Root-executed logs, not an independent execution or cryptographic build-source attestation. Current committed source and retained log bytes are pinned separately.

## Scope and remaining gates

No source, root ledger, service, package or candidate evidence was changed. Root's existing uncommitted ledger/captures were excluded from the review. The read-only verifier checks all 21 committed blobs, nine exact candidate comparisons and every locally frozen report/source/log byte.

Normal moving GPU coverage/contrast, damage visibility at supported zoom/viewports, browser arena/map behavior, runtime cold cache cost, natural play balance and physical devices remain outside this source review. There is no FPS, phone, global renderer budget or publication claim. No concrete source defect blocks those separate acceptance steps.
