# Native Wake sanctuary predicate order

Separate candidate branch `codex/coop-wake-sanctuary-order`, based on frozen trail candidate `111a477fe507cd544902c8f44f07198dc31933d6`. The original `codex/coop-wake-presentation` ref is preserved unchanged. Root owns integration, protocol/bridges/JavaScript/render paths and live acceptance. This pass changes only gameplay `co_op_wake.rs`, one focused native test in `co_op_wake_tests.rs`, and this work evidence.

The minimum runtime change moves the existing point/link geometry predicate before `safe_zone_at(target)`. Hostile-kind rejection still comes first; a geometric hit must still pass canonical sanctuary protection before target collection or damage. Exact distance/segment mathematics, link sanctuary fences, owner sanctuary/aging/stamping, damage, boss multipliers, native death/slot attribution, `.replace(owner)` and all presentation/protocol source remain unchanged. No cache, new broad-phase structure, geometry approximation, allocator or instrumentation is added to production.

The new native functional test mixes 48 distant sanctuary-free misses with a true unprotected point hit and a true protected remote point hit outside the host's site cache. Actual health, protected packed body, Profile and attribution/notices verify that protection remains authoritative and only the unprotected target takes damage. It also executes the empty-trail stage and checks every target stays unchanged. The original ordinary rank/evolution differential, segment-crossing, boss/body, slot attribution, lifecycle and eight-pilot tests remain passing.

## Actual measurements and checks

The independent earlier auditor measured runtime `332ddd5` in a separate frozen snapshot. `reporter-input-pins.json` pins that input; its results are not relabeled as this newer presentation candidate. This pass executes a new before/after comparison based on `111a477`.

`native-snapshot/` is an ignored work-owned copy of 170 exact native/compiler inputs (11,495,987 source/asset bytes), copied from the frozen Git ref and its literal include files. Only the work-only allocator probe is appended to the archived wake test module. It adapts the auditor's tuple initializer to the existing native Sample helper and omits the old332 publication-equality/acquisition section because the newer runtime correctly publishes wakes. The five gameplay stage conditions, allocator, fixtures, batch counts and timing remain the same. All compiled inputs and the unchanged compiled probe are verified in `compiled-inputs.json`; the optimized snapshot differs only in `co_op_wake.rs`. The in-memory appended probe is LF; its exported work file is CRLF. These are separately pinned. An initial verification assumption incorrectly equated those raw bytes; `pin-newline-failure.log` reproduces the failure and the corrected verifier does not alter compiled source or measurements.

The work probe delegates a thread-local counting global allocator to System. It counts allocation/reallocation requests and requested sizes on the measured thread, not retained memory, peak RSS, leaks, all process allocations or hardware CPU throughput. It arranges eight ordinary E5/R5 builds, stationary pilots, long stamp timers and, when full, 24 Sample values per pilot with life3.4. Birth tags are not published by this isolated gameplay probe; it does not exercise publication validity. Forty-eight high-health Enemy0 miss targets are placed outside all point/link reach and, for remote cases, beyond the host site cache at actual canonical sanctuary-free positions. Each of five batches warms20 and measures120 real wake stages at dt1/60; assertions retain192 samples and184 candidate links throughout full runs. These are configured capacity stages, not earned progression or full fixed ticks.

Exact command, before and after (same compiler/profile/dependencies/target directory):

```text
cargo test --manifest-path work/coop-wake-sanctuary-order/native-snapshot/Cargo.toml --release --offline --locked -p game_core work_only_wake_sanctuary_order_probe -- --nocapture --test-threads=1
```

`CARGO_TARGET_DIR` is the owned `work/coop-wake-sanctuary-order/cargo-target`. Both actual probe runs passed. The five batch medians and constant per-stage allocator counts are:

| Native isolated stage | Before requests / bytes | After requests / bytes | Before / after instrumented median µs |
| --- | ---: | ---: | ---: |
| Empty, no targets |0 /0|0 /0|0.905 /0.895|
| Full192, no targets |1872 /179904|1872 /179904|122.457 /122.138|
| Full192,48 local misses |1872 /179904|1872 /179904|117.101 /114.350|
| Empty,48 remote misses |3840 /359424|0 /0|186.358 /1.857|
| Full192,48 remote misses |5712 /539328|1872 /179904|300.368 /114.796|

This confirms removal of3840 repeated allocation/reallocation requests and359424 requested bytes per distant-miss stage in these fixtures. The remaining full-trail1872 /179904 baseline includes the existing link sanctuary work, which is deliberately unchanged. Raw batches are retained in `baseline-release-probe.log` and `optimized-release-probe.log`; `measured-summary.json` summarizes them. Timings include instrumentation and local scheduling variation; no stable FPS, whole-game worst case, browser/GPU/RTC or physical-device performance improvement is claimed.

Actual production-source native checks:

- `cargo test -p game_core --lib co_op::wake_tests --offline -- --nocapture`:15 passed,0 failed (`focused-wake-tests.log`). This includes the new mixed-hit test plus the original25 configured rank/evolution solo differential builds and300 real eight-pilot fixed ticks.
- `cargo test -p game_core --lib co_op --offline`:129 passed,0 failed,4 explicit exports ignored (`native-coop-regression.log`). This includes occupied publication and all persistent Wake/Nova replica tests.
- `cargo clippy -p game_core --all-targets --offline -- -D warnings`:passed (`clippy.log`). Source diff whitespace check passed.

No full web build, browser, listener, render, audio audition or Root source change occurred. The audio V3 assessment preceding this pass was read-only; its existing audition/provenance was reported directly to Root and no new audio patch/render was made. The separate whole-edge asteroid rejection remains untouched. Independent delta review and Root integration remain separate gates. The original trail111a source review does not accept this later optimization automatically.

`source-pins.json` pins the two changed production/test sources, canonical Git blobs and unchanged presentation/protocol/title boundaries. `MANIFEST.json` pins the small evidence files. The large ignored native snapshot and target directory are not shipped or committed; their exact compiled input hashes are retained for review/reproduction. The frozen Git commit is delivered with this handoff.
