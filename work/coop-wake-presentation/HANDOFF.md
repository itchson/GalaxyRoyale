# Bounded native Ion Wake trail candidate

Native source/test candidate on `codex/coop-wake-presentation`, based on Root `332ddd5ec0370e13ad346ea4ea988f35b61ca3e7`. The frozen Git commit is delivered with this handoff; `source-pins.json` identifies every actual source byte and staged canonical Git blob. Previous gameplay branch `codex/coop-ion-wake` remains at `87de1f7afad1a53183fcca91e146c9a59ee012cd`. Root's `session.hit_owner.replace(owner)` fix is preserved. No audio/title source, saved profile format, manifest, JavaScript, renderer backend, browser, server or shared Root checkout was edited.

The native candidate adds actual per-pilot persistent trail dots. The baseline real fixed-tick test admitted one native sample but extracted zero host dots; `baseline-missing-host-quads.log` preserves the original failure. The final test extracts five. This is native behavior evidence, not live RTC, browser rendering, guest AV or perceived visual acceptance.

## Source and transport contract

`co_op_wake_fx.rs` holds the small DTOs, canonical capture/cache and shared five-dot append. Membership-local `Sample { position, life, born_tick }` replaces the prior tuple while preserving position/life/cadence/cap/damage behavior. `born_tick` comes from the actual authoritative fixed tick at admission. Samples remain native world anchors; nothing reconstructs them from sparse pilot poses. Death/respawn clears them within the same epoch; leave/rejoin/end retains existing ownership cleanup. The solo tuple trail and saved progression are unchanged.

Every new header publishes `wake_fx:{points:N}`, including N=0. Replica metadata may be omitted for legacy zero state, but explicit null and unknown metadata fields reject. N must be an integer0..192. `Counts.body_pages` remains occupied-body geometry only; it does not count this page.

The exact full page is `WakePage { session,tick,revision,points }`; each `WakePoint` has exactly `slot,epoch,born_tick,position,life`. The points are sorted by slot then strictly increasing birth tick, at most24 per admitted ID/epoch and192 total. The page contains no offset, next_offset, body signature or partial update. A positive advertised count requires exactly one complete matching page. Wrong count, second page, unknown slot/epoch, duplicate/future birth tick and stale capture token reject. For zero/legacy headers, no page is staged; a complete newer commit clears prior trails. The host accessor can export an empty page for diagnostics, but Replica rejects an unadvertised zero page.

The host batch operation is `{op:"wake_page",session,acquisition,revision}`. It reads the same immutable Publication as entity/body pages, is non-draining, and never queries later live state. The native guest request is `{v:1,op:"replica_wake",acquisition,page}`. The existing transport wrapper is `{v:1,session,seq,kind:"body",payload:page}`. The native bridge's actual raw page span, including internal whitespace or long numeric spelling, is limited to32768 bytes before its clock/pending state changes. The core API independently limits point count and compact DTO bytes. Existing native/transport wrappers remain bounded at65536 bytes.

Root must add the strict `points` variant before the existing offset discriminator, prevent cells-only body-cache shortcuts from skipping a positive wake page, keep wake page accounting independent of `body_pages`, and preserve complete-publication/backpressure/acquisition behavior. Root owns `wire.ts`, `controller.ts`, `wasm-ports.ts`, normal builds and integration.

## Canonical numbers and atomic lifecycle

The new page deserializes life/positions into raw f64. Life must be finite, >0 and <=f64::from(3.4_f32)=3.4000000953674316, then convert to a finite positive f32 <=3.4_f32. Raw1e-46 underflows to zero and rejects; raw3.4000001 would round into range but fails the raw bound. Short raw3.4 is valid and canonically equals the native Value spelling. JavaScript must perform both raw checks and `Math.fround` checks; it must not require raw equality with fround.

Coordinates must pass raw and post-f32 finite bounds Â±1,000,014 (`WORLD_LIMIT+14`), admitting legal inward-facing rear anchors without changing gameplay samples. Existing pilot/entity/projectile bounds stay Â±1,000,000. The exact integer1,000,014 is representable in f32. Only the positive x-edge anchor was executed here; other edges/corners are further coverage gates.

Pending points cannot move the visible Scene. Full entity/body signatures, membership and page completeness validate before canonical reconciliation and before taking the old Nova cache or moving the Scene. For each ID/epoch surviving a committed same-tick header, the entire canonical captured set must match: key, f32 position and f32 captured life. Sets are compared independently of raw decimal spelling, ordering across slots, receiver-aged remaining life and staged watermarks. Positiveâ†’zero/legacy, emptyâ†’positive and mutated keys/position/life for a surviving member reject. Leave/new epoch may remove that member independently. A stable key's world position must remain unchanged on newer ticks too.

The retained bounded canonical records include visually expired points, so an equivalent same-tick recapture cannot resurrect or refresh them. Deadlines use header receipt time+captured life, making assembly delay count. Same-tick equivalent captures retain the earlier deadline. A strictly newer real tick authoritatively revalidates current native life, including a still-live sample after receiver cosmetics expired during a host pause. Newer empty/legacy state clears. Receiver time advances through the existing monotonic bridge/presentation clock; invalid/regressing/nonfinite times reject. End drops pending/Scene state. There is no unbounded tombstone history or saved combat state.

## Native extraction and budgets

Solo, actual host and committed guest use the same existing pixel-quad style: five size2 quads, radial offset6, `(132,255,202)` and saturating `(life/2*155) as u8`, radius12 culling, appended before actor bodies. Tests compare actual solo, actual host and guest frame paths at equal phase/life. Host phase uses the real session tick/60; guest phase is held at its committed capture tick/60 between publications. Reduced motion freezes phase at0 while expiry continues; reduced density preserves all five dots. This increment does not claim synchronized sub-frame host/guest phase or network latency compensation.

Actual native sample caps bound this to192*5=960 quads. No PixelBody cells, trail entities/projectiles, particle entries or extra Nova rays are allocated by rendering. Existing particle1100, transient-light32, event128 and drawn-light8 limits remain. Actual host birth lights are untouched. Guest trails do not synthesize birth lights, particles, audio or gameplay damage from a persistent active snapshot.

## Executed evidence

- `workspace-native-final.log`: `cargo test --workspace --offline`, exit0;928 passed,0 failed,19 explicit ignored exports (53 engine lib+3 engine integration+541 game core+300 game web+31 layout). No normal web bundle or browser was built/run.
- `clippy-author-final.log`: `cargo clippy --workspace --all-targets --offline -- -D warnings`, exit0. Initial Clippy failure for a redundant test f32 cast remains in `clippy-first.log`.
- `presentation-frozen-source.log`:11 persistent-trail tests passed,1 explicit export ignored. Workspace final ran the culling/invalid-clock assertions; the later frozen focused run also executed explicit same-tick key and life mutations. `wake-regression-first-pass.log` records14 gameplay/host Wake tests, including25 rank/evolution solo differential builds and300 actual eight-pilot ticks; all remain passing in final workspace execution.
- `export-frozen-source.log`: explicit actual core capture/Value serializer exporter passed. Initial actual eight-pilot samples, real same-epoch death empty state and210-tick later expired state are exported. A separate capacity fixture arranges native Sample/Bullet/ShotOwner/Journal data in actual admitted ECS; it is not earned progression or a claim of natural192-point density. The first exporter failure from reusing a birth key with a changed anchor is retained; the fixture was corrected without weakening validation.
- `host-bridge-first.log`: actual `co_op_batch` operation test passed after360 real fixed ticks;17 naturally admitted points,2014 compact Value bytes, frozen repeated reads and strict operation shape. Its exact actual header/page/batch response are exported separately.
- `replica-maximum-boundary-frozen.log`: the actual native request parser/Replica admitted the exported maximum stage and wake wrappers, rejected a duplicate page, and kept Scene absent before complete geometry. Full maximum body/Scene commit is exercised by the core exporter. This bridge check does not claim a full browser publication.
- `raw-page-bound-first.log`: actual raw-page whitespace/escaped-key rejection test passed before creating/moving a view. Original compiler/first execution failures are retained as raw logs.

Compact byte lengths exclude each fixture file's final newline:

| Fixture | Bytes |
| --- | ---: |
| Configured192-point native Value page |18401|
| Exact maximum transport publication wrapper |49452|
| Exact maximum native replica_stage wrapper |49389|
| Exact maximum body transport wrapper |18495|
| Exact maximum native replica_wake wrapper |18452|
| Separate declared maximal-u32 page |21745|
| Separate declared maximal-u32 body wrapper |21839|
| Separate declared maximal-u32 native wake wrapper |21805|

The maximum header has8 full admitted ordinary Echo/Nova/Wake evolution-five loadouts,180 correctly owner-bearing shots,32 configured journal records and wake metadata. The page fixture includes f32â†’f64 decimal expansions and halo anchors. Maximal-u32 fixtures stress integer lengths separately and are explicitly DTO scalar fixtures, not actual billions-of-ticks execution. These measurements prove the executed fixtures fit; they do not prove every theoretical legal header spelling fits the existing65KiB bound. `MANIFEST.json` pins all raw evidence/fixtures; `source-pins.json` pins12 changed sources and17 unchanged boundaries/manifests/title inputs.

## Separate observed whole-edge limit

`edge-first-inspection.log` preserves an unfiltered real tick/capture failing `Replica::entities` with `invalid_entity`. Test: `co_op_replica::tests::wake_fx::native_edge_stamp_keeps_legal_rear_anchor_halo`. Setup admitted two ordinary evolved Wake pilots, first native tick, guest at(WORLD_LIMIT,0) inward headingPI and -0.75*speed, real seq2 input then native tick. Sample x=1,000,014 was legal. The capture had99 actual ECS rows,17 asteroids outside the older Â±1,000,000 vector bound. First: `000000010000004b`, kindasteroid, position[1000036.94,-229.91548], previous[1000036.94,-229.80978]. Existing `vector` checks in `co_op_replica.rs` reject these entity rows. Projectile position/previous/velocity use the same bound in source; no out-of-bound projectile execution was observed.

The final wake-only edge test explicitly removes incidental non-pilot fixture entities after the actual birth tick. That narrows coverage to the real pilot/sample/page path. It does not repair or accept whole-world-edge replication. No existing bounds were widened or native anchors clamped. Root owns a separate follow-up for that reproduced existing limit.

Independent source review is pending at freeze. Matching strict JavaScript/controller/ports, dual normal build, actual current host/guest RTC delivery/backpressure/expiry/cleanup, WebGPU/WebGL2 trail visibility, perceived quality and physical AV remain open. Historical loopback evidence and source/tests do not close those gates. Studio records describe the actual bounded native run and leave the product task unfinished.
