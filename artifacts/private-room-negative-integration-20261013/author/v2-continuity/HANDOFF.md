# Private-room negative QA v2 amendment

Frozen candidate `7c188d4198901ead15e4bffa2b0063b9c95f7674`, branch `codex/private-room-negative-qa-v2`, based on preserved original `1b14bff9e6da0a1a249f1bdc42ab49e59e74e52c`. Only `scripts/qa/co-op-transport/wasm-review.js` and its focused `scripts/test/coop-private-room-negative-qa.test.mjs` changed. Original branch, reports, logs and six-file candidate remain intact. Root shared checkout, child protocol, report server, HTML, README, preparation, runtime/service/native/package sources were not amended.

## Corrected acceptance boundaries

- Approved rotation, lock and unlock preserve the original private room/seed/peer/native/PC/epoch marks. After the awaited policy reply, `roomContinuity` reads a separate guest complete-commit floor. Only a strictly greater later `new_commits` count qualifies. Reading this floor does not replace the ownership marks.
- The existing visibility-loss latch activates before the first preflight observation. Every experiment command and polling observation guards visibility both before and after awaited replies. Hide followed by restore remains a failure. Preflight failure resets active qualification before any room mutation or attempt latch.
- Active `roomRead` completes both private `room-state` commands, then sends the existing public `state` commands. This ordering avoids concurrent commands on the same child's `commandPending` handler. Actual replies require initialization ready, null failure code, phase neither failed nor disposed, and finite positive private native lifetime. The native schedule exception path sets `stopped=true`, `failureCode=native_frame_failed`, and phase failed; it cannot complete Decline while a controller still appears live.
- Before the final cleanup, active qualification stops. Cleanup can still execute bounded actual leave/solo/PC/request/timer/digest checks after visibility or native failure. Cleanup does not erase the visibility latch or primary failure, and a failed experiment never enters the completed-case set. No experiment health/visibility guard prevents terminal cleanup.

## Actual evidence

Three baseline regression commands failed before the associated source amendment; raw logs and exact parent snapshots are retained:

1. `original-regression.log`: old `1b14` continuity accepted five commits received during the awaited policy reply; the required false predicate assertion failed (`true !== false`).
2. `visibility-original-regression.log`: old post-read acceptance missed visibility loss, including hide-restore; expected rejection was absent.
3. `health-preflight-original-regression.log`: three new cases failed. Actual child `schedule` and `observations` executed a declared throwing step and exposed `native_frame_failed/stopped`, but the original room read let the actual Decline orchestration complete. The failure/lifetime gate also missed expected rejection. Hidden-restored preflight reached the forbidden create fixture.

Final author checks, all actual exit 0:

- `node --experimental-transform-types --test scripts/test/coop-private-room-negative-qa.test.mjs`: **17/17 pass**, `focused-tests.log`.
- Selected existing WASM source/function/report checks: **16/16 pass**, `existing-wasm-functions.log`; the exact pattern is pinned. The supplied-artifact HTTP tests were excluded, so no listener was eligible.
- Existing Fifth source/function checks: **10/10 pass**, `existing-fifth-functions.log`.
- Parent `node --check` and scoped `git diff --check`: exit 0; local Prettier formatted only the two owned tracked files. Command-local established Git identity produced the frozen commit; tracked tree is clean.

These are actual helper-function tests with explicitly declared child/timer/message/native boundaries. They are not browser, RTC, native Game, stored-profile or live negative acceptance. No build, stage, listener or browser was launched. Native Wake audit has no running measurement process; only a separate work-owned source archive and compiler inspection existed when this amendment was prioritized.

## Observation limits

The parent health check validates existing child signals, not the private `stopped` flag itself. Source inspection maps actual native frame failure to failure code/phase, explicit Dispose to disposed phase, failed initialization to stage/failure, and exhausted native lifetime to `life_ms=0`. The child `pagehide` handler separately sets stopped and asynchronously disposes its controller without changing the child's own phase; controller disposal reports ended. Existing replies therefore cannot directly establish full loop liveness in that pagehide edge. Visibility/pagehide/live behavior remains an actual browser acceptance gate, not a source-function proof of every stopped path.

Root owns independent review, integration, fresh prepared provenance, live execution and disposal. Earlier captures retain their exact older helper sources. Eight clients, Internet/TURN, exact boxed-world/native restoration, zero migration writes, shipped UX and physical/mobile timing are not claimed.
