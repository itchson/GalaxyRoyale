# Bounded browser Wake transfer candidate

Branch `codex/coop-wake-browser`, explicit parent `d8ec6294976f54d26aab9ffaf5fc3dc251cb170a`. The frozen native trail `111a477fe507cd544902c8f44f07198dc31933d6` and separate sanctuary-order `d8ec629` refs are preserved. This candidate changes only `apps/game/src/coop/{wire.ts,controller.ts,wasm-ports.ts}`, focused consumer tests, a new test fixture namespace and this work evidence. Native source, Main/transport/signaling, audio/title, renderer, shared/package manifests and Root checkout source remain unchanged.

Read exact independent `wake-browser-integration-checklist/CHECKLIST.md` and earlier `wake-browser-integration-plan/PLAN.md`. Their raw hashes/locations are pinned in `fixture-input-pins.json`. This is the matching consumer implementation for the already frozen native contract, not a new protocol/channel/framework or JS trail simulation.

## Resulting flow

`wire.ts` accepts optional `wake_fx:{points:N}` on a snapshot. Omission is historical zero; supplied null/undefined/extra/missing keys, negative/fractional/non-u32 or193 reject. A body page with its own `points` key is distinguished before offset checks and must have exactly `session,tick,revision,points`. Geometry entity/cell pages retain their existing offset/dimensions/signature checks. Mixed variants reject.

Each point requires exactly `slot,epoch,born_tick,position,life`: slot0..7, nonzero u32 epoch, u32 birth<=page tick, strict slot/birth ordering,24 per slot and192 total. Raw life is finite positive <=Math.fround(3.4); converted f32 is finite positive within the same limit. Raw and f32 coordinates stay within±1,000,014. Short3.4 accepts; raw1e-46 and3.4000001 reject. No canonical equality requirement discards short valid spellings.

Values/encoding enforce the32768-byte serialized page bound. Incoming `decode` first parses and validates shape, then scans only the actual top-level raw payload span, honoring escaped keys, quoted strings/escapes and nested delimiters. It includes internal whitespace/numeric spelling and counts UTF-8 bytes, rather than trusting normalized JSON length. Duplicate top-level payload keys reject. Existing65536-byte frame checks remain. Empty pages are valid diagnostic shapes but receiving a zero/legacy header expects no page.

`Capture.wakeAppended` and `Receiving.wakeReceived` are the only new transfer flags, initialized complete for zero/omission and incomplete for a positive count. The producer finishes entity rows, makes one immutable `wake_page` native batch call with session/acquisition/revision, validates matching page tokens/count/roster, appends one existing body Piece, then handles cells/end. All peers share that capture/page. A body cache shortcut now positively identifies `cells`; it cannot skip points. `Counts.body_pages` and occupied/storage counts remain cells-only.

The receiver requires the positive page after entity completion and before cells. It validates exact page tokens, count and current slot/epoch roster, delegates through the existing ReplicaPort and marks received only after true. Points do not increment any cell/entity counters, body offsets or caches. Missing, zero-unexpected, duplicate, out-of-order, token/count/roster mismatch or native refusal never enables commit/ACK/cache promotion. `publication-end` requires the Wake flag along with existing body completeness; only successful native commit marks simulation connected and schedules ACK.

`wasm-ports.ts` routes points→`replica_wake`, entities→`replica_entities`, cells→`replica_body`, preserving the existing page API, source numbers, acquisition/revision and response/ownership checks. Native request bounds remain65536. Rejected operations preserve ownership tokens for end/recovery. Canonical same-tick set equality, expiry deadlines, previous Scene atomicity, newer authoritative revalidation and sample removal stay in native Replica. No JS timers, pose reconstruction, TTL refresh, birth lights, particles, audio or saved combat state are added.

The existing four append-work/two send-piece per frame,1MiB/80-frame per-peer rolling budgets, buffered limit, all-peer ACK capture barrier,30-second transfer timeout and blocked restoration remain unchanged. A delayed complete transfer may legitimately have no remaining dots after3.4 seconds; no visibility/FPS/throughput promise is made.

## Exact native fixtures and context

All22 native Value export files from Git111a are copied byte-for-byte canonically into `scripts/test/fixtures/co-op-wake/`. `provenance.json` and the focused hash test pin each file independently of Windows checkout CRLF. Modern actual-host/initial/empty/expired/maximum and legacy headers, diagnostic pages, raw life examples, transport wrappers, native operation wrappers and scalar stress routes are exercised.

The actual-host fixture has one host, not an absent guest2. Its batch response contains TWO repeated wake_page results; tests preserve both results and verify it cannot satisfy a one-operation controller call. The maximum native stage/page pair has matching actual tokens/roster. Scalar stress uses a separately constructed corresponding maximal-token/epoch header; it is not fed against the actual maximum acquisition2 or called billions-of-ticks gameplay. Initial/empty/expired/max provenance remains the native author's real-birth/death/elapsed-ticks/configured-capacity distinction.

Actual Node serialization/delegate measurements:

| Case | Bytes |
| --- | ---: |
| Maximum JS normalized page |17953|
| Test body frame with seq1 |18038|
| Native replica_stage request from maximum fixture |46589|
| Native replica_wake request from maximum fixture |18004|

The raw native frozen wrapper fixtures retain their larger original f32→f64 number spellings and are decoded successfully too. These checks prove fixture/boundary sizes, not every theoretical legal header spelling or actual RTC delivery.

## Actual verification

Node24.18.0 / TypeScript5.9.3. Final checks:

```text
node node_modules/typescript/bin/tsc --noEmit
node --experimental-transform-types --test scripts/test/coop-wake-wire.test.mjs scripts/test/coop-controller.test.mjs scripts/test/coop-wasm-ports.test.mjs scripts/test/coop-nova-fx-wire.test.mjs
```

Typecheck exited0. Final listener-free focused run:73 passed,0 failed,0 skipped (`listener-free-pinned.log`, `check-results.json`). Formatting targeted only the six owned source/test files and source diff whitespace checks passed. No native/workspace build or new render was run.

Wire tests cover all fixture routes/hashes, strict metadata, raw/f32 numeric/halo bounds, missing/mixed point fields, ordering,25/member,193 total, future births, exact raw32KiB boundary, over-limit whitespace/number spelling, escaped top-level/nested keys, duplicate payload keys and UTF-8 frame-byte checks.

Controller tests exercise actual controller/wire code with explicitly declared fake transport/native adapters. They cover warm body caches and subsequent captures still sending current Wake; one fetch per capture; backpressure refusing/retrying the same retained piece; missing/unexpected/duplicate/early/mismatched pages; native page rejection preserving incompleteness; pending Wake followed by bad cells/native commit refusal; zero/omission; same-tick forwarded data/changed life/key/position/absent sets reaching a configured native refusal; all-peer ACK and positive-page rolling quotas; end/dispose/ownership restoration. They do not execute Rust's comparison or a browser RTC connection. Port tests record exact requests and controlled unit replies; these are fake delegates, not native success fabrication or a Scene-render proof.

The initial five-fixture wire baseline failed on unknown metadata/old offset expectations (`wire-baseline-failure.log`). The later added same-tick controller test initially mixed the latest staged header/current rows with a previous capture's Wake page and correctly failed `publication_revision` (`focused-second.log`). Its original added fixture block from the author patch transcript is retained as `same-tick-fixture-original.mjs`. The correction selects the exact retained matching header/entity page; production guards were not weakened. Later final tests pass.

## Explicit execution deviation

I mistakenly included `scripts/test/coop-transport.test.mjs` in `focused-third.log`. That existing suite opens ephemeral localhost HTTP servers with `listen(0)`, outside this task's no-listener scope. Its111 tests passed; existing fixture teardown assertions closed each HTTP server/poll set, and the tool reported the Node process exit0. The full raw run is preserved. No persistent service or new browser/real RTC/render followed from it. Root was promptly told; final73 verification excludes that suite and is listener-free. The earlier task scope is not retroactively broadened or described as having had no listener activity.

Independent review, Root integration, matched native/JS normal dual builds, actual current host/guest RTC page/capture/commit/backpressure, visible trails on both renderers, delayed/hidden expiry, leave/end/save/blocked cleanup, multi-peer/physical performance and perceived AV remain open. Existing pre-feature room/native tests do not close these gates. The unrelated whole-edge asteroid/protocol limit is unchanged.

`source-pins.json` identifies the tested changed files, canonical Git blobs and unchanged native/protocol/manifests/title inputs. `MANIFEST.json` pins raw evidence; fixture provenance separately pins the22 exact input files. The author Git freeze is delivered with this handoff; review cannot be inferred from the author pass. Studio task remains unfinished while integration/live gates are open.
