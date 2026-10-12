# Bounded native co-op wake presentation design

This is a read-only design against frozen native candidate `87de1f7afad1a53183fcca91e146c9a59ee012cd` in `C:/Users/mitch/.codex/worktrees/galaxy-pixel-art/galaxy-royale`. No source, branch, build, browser, server, or player-profile changes were made. Source/fixture pins are in `source-pins.json`. All implementation and test changes below are proposals, not passing results or runtime acceptance.

## Recommendation

Publish the complete set of currently active native wake samples as one bounded native presentation page in the existing immutable publication. Add only a count descriptor to the header. Treat the page as a third existing `body` payload variant, alongside entity and packed-cell pages; do not add another transport frame kind or a parallel event player. Commit it together with the existing fully validated entities/bodies, then draw the same native pixel-quads on host and guest.

The snapshot is actual current trail state, not a birth journal. A joining guest should see the active trails in its first complete scene. An empty new snapshot must clear prior trails, including a respawn that keeps the same membership epoch. Missing samples and removed memberships clear immediately on a subsequent complete commit. No sample is reconstructed from interpolated pilot motion.

## Inspected current behavior

- `crates/game_core/src/lib.rs:3487` draws five quads per solo `(position, life)` sample: size 2, radial offset 6, color `(132,255,202)`, alpha `(life/2*155) as u8`, phase `cosmetic_time()`. Rust's float-to-u8 cast saturates at 255. The sample is world anchored, culled with radius 12. `cosmetic_time()` returns zero for reduced motion (`lib.rs:783`). Solo wake does not currently thin these quads for reduced cosmetic density.
- Frozen `co_op_wake.rs` stores actual membership-local samples, with caps 12/16/24 and lifetimes 2/2.6/3.4 seconds. Its exact emission anchor is the pre-integration origin minus 14 forward using the current native heading. Motion publication is a sparse later capture, so a guest cannot reconstruct that anchor, the threshold crossing, cadence phase, respawn, or sample deletion reliably. Gameplay point/connected-segment damage remains host native.
- Host extraction in `Game::render_into_with_home` still reads only solo `Game.wake`; it does not draw `Pilot.wake`. Guest `Scene::append_visible` draws actual occupied bodies, projectiles and committed Nova rays, with no wake samples. `Galaxy.step`/draw already chooses the host Game extraction or the guest Replica scene; both feed the same FrameBuilder/renderer path (`crates/game_web/src/browser.rs:486`, `browser.rs:639`). No renderer shader/backend work is needed to append ordinary wake quads.
- `co_op_publish.rs` already freezes one native Snapshot plus entity rows and packed bodies under an acquisition/revision. Later ticks cannot alter that capture. `Replica` stages/validates the header and pages, replacing its visible Scene only after complete commit. It already records receiver receipt time and advances a monotonic presentation clock independently of clamped frame dt or pose interpolation (`co_op_replica.rs:851`, `co_op_replica_bridge.rs:127`).
- Current Nova is a short birth journal. Its first complete publication deliberately skips join history and subsequent commits deduplicate by journal sequence. Reusing that baseline-skip policy would hide genuine still-active 3.4-second wake trails. Reusing its receiver clock and complete-commit discipline is appropriate.
- Native host limits are 8 pilots, 180 bullets, 1100 ordinary particle entries, 32 transient lights, 128 retained events. Host light extraction selects at most four visible recent transient sources, two projectiles, engine and beacon, then truncates to eight. Guest Nova cache has its own 32-light bound and exposes at most four visible Nova sources before projectiles/beacon. Neither path publishes general wake-light admission/survival today.

## Proposed native state and wire

Add `born_tick: u32` to the authoritative membership-local sample when the real native wake stage admits it. Prefer one small native `Sample {position: Vec2, life: f32, born_tick: u32}` over parallel position/metadata vectors. Keep solo `Game.wake` unchanged. There is at most one new wake sample per pilot per authoritative fixed tick, so `(session, participant ID, epoch, born_tick)` is a stable visual key. Store and remove this metadata with the same sample as damage/TTL/cap/death/leave; do not maintain a second trail generator.

Resolve a compact wire `slot` through the pending header's actual participant table, and require matching epoch. The guest cache key uses the resolved participant ID as well as epoch and birth tick; it must not key only by a reused slot.

Illustrative contract shape, not an observed native export:

```json
{"snapshot":{"wake_fx":{"points":192}}}
```

```json
{
  "session":"00000000000000000000000000000064",
  "tick":600,
  "revision":608,
  "points":[
    {"slot":1,"epoch":2,"born_tick":594,"position":[786.125,300.5],"life":3.3000001907348633}
  ]
}
```

Rules:

- New captures contain `wake_fx:{points:N}` even when N is zero. Legacy missing `wake_fx` means zero supported points; explicit null or unknown descriptor fields reject. An empty descriptor needs no wake page and clears old visual state on successful commit. Metadata is not an `OwnedLoadout` or a save field.
- N is an integer 0..192. A positive count requires exactly one complete wake-state page containing exactly N points, with UTF-8 payload at most 32 KiB, before commit. Its normal `body` transport envelope and `replica_wake` bridge wrapper remain below the existing 65,536-byte limit. Enforce both point and serialized-byte bounds; no partial cosmetic truncation is needed.
- Page session/tick/revision must equal the retained pending capture. Strict page keys are exactly `session,tick,revision,points`. Standalone boundary checks limit array length, scalar types, coordinates and chronology. Contextual controller/native checks resolve membership and compare declared count. A second wake page for one acquisition rejects; no page is accepted if the header declares zero/legacy absence.
- Point keys are exactly `slot,epoch,born_tick,position,life`: slot 0..7, epoch nonzero u32 matching that slot's current member, birth tick u32 <= capture tick, exact two-element finite positions within native +/-1,000,000 bounds, positive finite remaining native life <=3.4 f32. At most 24 points per membership. Canonical order is slot then strictly increasing birth tick. Duplicate keys, mixed epochs for a slot, unknown members and oversized lists reject atomically.
- JavaScript's maximum life check must use `Math.fround(3.4)` (3.4000000953674316), matching Rust `f32`. The native Value/JSON bridge can expose that exact f32 value as f64; a literal JS `<=3.4` can reject a legitimate fresh point. Preserve actual native remaining life rather than deriving integer lifetimes from sparse motion or assuming expiry at exactly 204 steps; repeated f32 subtraction can differ at the final boundary.
- Captured points are complete current state. Keep the exact active positions/life in the immutable native Publication and serialize a single page on request. Repeated capture reads do not drain or mutate trails. No profile, damage, ranks, trigger requests, materials, boss history, arbitrary trails, or guest ability claims are accepted by this page.

A 192-row page remains small enough for one bounded message. Using conservative JSON scalar lengths (up to 26 chars per represented finite float, 10 per u32) gives under 30 KiB for these five fixed keys per row plus wrapper; enforce 32 KiB anyway and verify with the actual native bridge and browser serializer. This is an analytic schema bound, not a native measurement. One page avoids three extra publication frames and three native serialization calls.

## Complete commit, age, and removal

At stage/page time, store pending data only. No pending sample, deletion, epoch, deadline or light changes the currently committed Scene. Existing geometry/count/signature errors keep the preceding scene and presentation state; wake count/validation failures do likewise.

On a successful complete commit:

1. Age existing cached visual records using the current receiver monotonic clock.
2. Resolve the page's slot/epoch to exact current participant IDs. Remove departed IDs/epochs and all missing samples. An omitted/empty legacy state replaces the cache with empty wake presentation. Same-epoch native death has an empty/new trail snapshot, so it clears without needing a new membership epoch. Future actual births have greater birth ticks.
3. For a newly observed active key, expiry is `header_receipt_time + life_at_capture`. Commit assembly time is already included because the current clock is later than receipt. If expired during body/page transfer, do not draw it. This first snapshot may admit existing live dots; it does not replay an ability sound, particles, damage or a birth activation.
4. Keep at most 192 committed point records with their local deadlines, including records whose receiver deadline has expired; skip them when drawing rather than discarding their stable keys before a new complete capture. For a repeated same-native-tick capture, preserve immutable position and shorten its deadline to `min(old_deadline, header_receipt_time + newly_captured_life)`. This cannot restart or resurrect a same-tick sample on repeated acquisition, tiny frame deltas, visibility return or alpha interpolation. Reject a changed position for a key; native samples are stationary.
5. A strictly newer native tick is fresh authoritative evidence of currently active trail state, and establishes the new receipt-relative remaining-life deadline for points actually still present. This matters after host simulation pauses or runs slower than wall time: the host's existing wake TTL freezes with simulation, while a receiver reasonably expires stale presentation during the stall. A new positive-life native snapshot may show that same real sample again without replaying a birth event, sound, light or gameplay. Treating a receiver-only timeout as permanent native death would hide a still-damaging authoritative trail after recovery. Remove genuinely absent native keys/epochs on full commit; they never return from the current native trail implementation. Scene records/deadlines remain bounded to the complete current sample list, so no extra unbounded tombstone journal is required.

World anchor and fade use actual remaining life; pose interpolation only moves pilots/projectiles/camera. The receiver knows capture-relative life plus header-to-commit elapsed time, not exact network latency before header receipt. Do not claim synchronized host/guest sub-frame TTL or orbit phase without a clock-offset mechanism. On a transport stall, receiver time still expires committed dots; repeated same-tick publications cannot leave immortal trails. Recovery from a strictly newer real tick revalidates actual remaining state. The review should explicitly confirm this distinction between an active-state snapshot and a one-shot birth journal; cache expiry must not silently become authoritative gameplay expiry.

## Native pixel extraction and budgets

Lift the five-dot append into one small shared native helper accepting an iterator of actual `(position, remaining_life)` plus explicit cosmetic phase and culling callback. Call it for solo `Game.wake`, host co-op pilots in stable slot order, and guest committed cache. Preserve the exact color, radial spacing, size, fade, and float-to-u8 saturation. Append wakes before actor bodies in both host and guest extraction, matching the existing solo layer order. Use `FrameBuilder::push_pixel` or its equivalent native affine-cell append; it reuses the fixed-size pixel transform/batch. Do not allocate PixelBody cells, spawn ECS actors/projectiles, or create JavaScript/canvas particles for trails.

At most 192 live samples produce 960 quads, with world-viewport culling. This is an explicit trail-geometry cap. They do not consume or inflate the existing 1100 particle-entry pool or Nova ray admission counter; those remain the same bounded systems. Existing solo wake already renders outside that particle pool. Both native extraction paths should honor reduced motion with phase zero, while TTL fade still advances. Keep all five dots in reduced density too, matching current solo and preserving a visible gameplay hazard; do not drop samples or their physics at lower quality.

For host co-op cosmetic phase, use the actual session tick/60 (plus bounded render interpolation if desired); guest uses the committed capture tick and explicitly defined presentation interpolation. The same helper can compare exact geometry when tests pass equal phase/remaining life. Normal network delay may create cosmetic phase differences. Reduced-motion geometry is phase-zero deterministic across both paths.

## Birth-light scope

The smallest persistent-trail patch should leave existing host birth-light behavior unchanged and add no guessed guest wake light. A live trail point is not proof its 0.3-second native birth light survived the shared 32-light pool; another effect may have evicted it. This patch can make both views show genuine persistent wake dots without claiming complete light/audio parity.

If matching birth lights is included in a subsequent bounded patch, tag the actual native transient light with the wake birth key. The current generic constructor keeps a None tag; wake admission supplies its exact key. Capture only surviving tagged light TTL associated with the actual sample, through a required nullable `light_life` field in the same page. `null` means no surviving native source; positive values are bounded by .3 f32. Do not infer or replay a light merely because a sample is young. Guest cache uses the same receipt/expiry rules and fixed native color/radius/intensity. Enforce one combined 32-transient-light cache bound and choose at most four visible transient sources across Nova and wake, then projectiles/beacon within the renderer's eight-light maximum. No extra per-point lights may bypass those limits.

That light increment also needs an explicit lifecycle decision: frozen 87 permits a host light's .3-second tail after death/leave. For immediate matching owner cleanup, remove only tagged wake lights on native death/leave and remove matching guest lights on committed empty/departed state. This is a proposed cosmetic amendment to frozen behavior requiring review, not already implemented parity. Keep all other effects untouched. Tests must verify actual eviction/admission and counters rather than treating a reconstructed source as host evidence. This optional wake-light increment still does not prove parity of every other transient light: the existing Nova guest cache is not a complete snapshot of all host hit/shield/explosion lights.

## Concrete future files

| File | Proposed change |
| --- | --- |
| `crates/game_core/src/co_op_wake.rs`, `co_op.rs` | Add birth-tick sample metadata to the real admitted trail; export bounded current presentation metadata/count; existing gameplay/lifecycle rules stay authoritative. |
| New `crates/game_core/src/co_op_wake_fx.rs`, registration in `lib.rs` | Tiny wire Point/Page/Meta types, shared five-dot append and bounded guest presentation cache. This is one effect helper, not a new framework. |
| `crates/game_core/src/lib.rs` | Route solo and host co-op wake extraction through that shared append, with ordinary modes/reduced motion preserved. |
| `crates/game_core/src/co_op_publish.rs` | Freeze exact wake points at acquisition, store them in Publication, provide `published_co_op_wake_page`; enforce count/byte bounds without changing entity/cell counts or signatures. |
| `crates/game_core/src/co_op_replica.rs` | Optional strict count descriptor, pending wake page validation/count/order/membership, atomic full-state reconciliation on commit, aging/end/removal, guest append. |
| `crates/game_web/src/co_op_bridge.rs` | Bounded `wake_page` native host operation referencing acquisition/revision; actual Value serialization and size check. |
| `crates/game_web/src/co_op_replica_bridge.rs` | `replica_wake` request; existing monotonic receipt clock and end behavior reused. |
| `apps/game/src/coop/wire.ts` | Optional descriptor strict allowlist and null/default semantics; third `body` page variant validator; full frame byte checks. Root owns this boundary. |
| `apps/game/src/coop/controller.ts` | Capture sends at most one wake page; receiving tracks exact wake completeness; explicit variant discrimination; only actual packed-body pages use body-signature cache skipping; commit waits for all advertised state. Root owns this integration. |
| `apps/game/src/coop/wasm-ports.ts` | Route points page to `replica_wake` instead of `replica_body`; retain acquisition/revision checks. |
| `crates/game_web/src/browser.rs` | Existing host/guest branching suffices for persistent dots. Birth-light integration may need the existing light-source composition, not renderer changes. |

No Main UI flow, assets, manifests, shader, renderer backend, SaveStore/Profile, or progression migration is required for the persistent dots. Do not write a second local Game on guests or copy combat state into saves.

## Meaningful future checks and observed limits

Native tests should generate actual points with real admitted ranks/evolutions, motion and fixed ticks; compare host extraction and fully committed guest geometry under controlled equal phase, then verify real lifetime/removal. Add tests for first populated commit, zero/legacy clear, immutable recapture during ongoing native motion, repeated same-tick acquisition/no lifetime restart or resurrection, missing/bad/duplicate wake page and body commit atomicity, capture age plus transfer delay, invalid/regressing/large monotonic clocks, pause/hidden expiry, death in the same epoch, leave/rejoin/slot reuse, session end, all 192 points/960 quads, viewport culling, and reduced motion/density without physics/profile changes. Existing Nova and particle/SFX tests must continue to demonstrate unchanged pools/counters.

Bridge/wire tests should use actual native serializer fixtures for legacy/empty/active/expired/max points and reject null, unknown/missing fields, zero/nonfinite/oversized life, malformed positions, future/duplicate birth ticks, wrong slot/epoch, 25 per pilot/193 total, wrong capture tokens and oversized bytes. Exercise the actual `co_op_batch` Value conversion plus `replica_wake` request, then the browser's encode/wrapper byte lengths. Maximum combined header fixtures must include 180 correctly owner-bearing shots, eight full admitted loadouts, 32 Nova records and wake metadata; the wake page's own maximum must exercise long float encodings. Counters alone are insufficient proof of the 65,536-byte frame limit.

Read-only observed fixture measurement: existing `scripts/test/fixtures/co-op-nova-fx/maximum-header.json` is 87,897 pretty-file bytes; Node JSON re-encoding gives 40,430 header bytes and 40,522 wrapped publication bytes, with 8 participants, 180 projectiles and 32 Nova records. Its first projectile is owner-null, and its fixture exporter deliberately arranges journal density. `fixture-size-observation.json`/`inspect-fixture.mjs` record this read/re-encode only; no native execution, transport test or proposed wake encoding was performed. The 25,014-byte headroom of that particular fixture is not proof that a complete inline trail fits every legal combined owned-shot/header state, which is why the one-page variant preserves the existing header budget.

The current controller targets capture every .05 seconds only when no preceding capture waits for acknowledgements. It serializes at most four pages per host frame, sends at most two pieces per peer/frame, limits each peer to 1 MiB/s and 80 frames/s, and retires stalled transfers after 30 seconds. Adding one wake page is one bounded extra call/frame per active capture, not a guarantee of 20 Hz delivery. For a warmed 120-row/two-entity-page capture with no changed body pages, header+entity pages+wake+end is five frames, making the 80-frame budget an upper ceiling of 16 publications/s; 512 rows gives 11 frames and an upper ceiling near 7.27/s. These are analytic ceilings, not measured throughput. Changing bodies, initial packed-cell downloads and slow-peer acknowledgements can be much slower.

Initial or dirty-body transfer may outlast a trail's 2..3.4-second lifetime. Under the existing atomic publication contract, expire late state honestly; do not draw replayed historical trails or invent missing new emissions. A following fresh full capture can restore currently active points. Continuous effects during a long blocked acquisition would require a separately reviewed independent presentation channel/membership discipline, outside this smallest design. Actual RTC, guest visual/light/audio behavior, mobile and physical play remain acceptance gates after implementation; source or CPU geometry tests alone cannot close them.
