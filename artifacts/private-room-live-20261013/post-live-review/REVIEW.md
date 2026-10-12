# Independent post-live helper amendment review

**No blocking source defect found in this bounded pass.** Reviewed Root's post-live `client.js`, `wasm-review.js`, `server.mjs` and two new checks in `coop-transport-wasm-qa.test.mjs`. No helper file was edited. Earlier positive browser captures remain evidence for the earlier frozen helper; they are not reinterpreted as execution of these amendments.

`send` now accepts an internal timeout budget while retaining the ordinary 75-second default. `approvePendingJoin` passes the remaining 15-second observation budget to both child reads, rejects an exhausted budget before sending another read, and rejects replies at or beyond the deadline before checking isolation or accepting. Repeated reads receive decreasing budgets. The existing per-request timer removes timed-out IDs; late replies find no pending request and cannot reopen it. Normal timely pending snapshots still proceed through the actual cumulative-peer/ready/admission gate.

The child message handler preserves the exact `approval_not_pending` enum alongside its existing controlled failure set. The parent's reply filter and server's strict client-failure allowlist now accept that enum. Arbitrary private error text still becomes `native_action_failed`. The new child-route test executes the actual event handler and server validation, rather than only calling `command` directly; the independent parent-route edge closes the other side of that path.

The budget limits **pending observation**, not the subsequent `approve-next` RPC, which retains the ordinary command timeout. Actual service expiry still prevents acceptance of a retired request. Throttled browser timers may delay rejection delivery; the post-reply wall-time check prevents a late pending observation from qualifying. A command timeout removes the reply mapping, not an in-flight child/service operation. No overall-action, physical timer or new live timing pass is claimed. The narrowed explicit save/flush guarantee does not change the separately documented `SaveStore.open` migration limit.

Actual execution, both exit 0:

- `node --experimental-transform-types --test --test-name-pattern='^Actual approval observation refuses late|^Actual child message route preserves' scripts/test/coop-transport-wasm-qa.test.mjs`: **2/2 pass**, `focused-tests.log`.
- `node --test work/private-room-post-live-review/edge-tests.mjs`: **3/3 pass**, `edge-tests.log`. Actual extracted parent functions/handler verify budget propagation, pending removal, controlled/raw reply filtering, ignored late replies, ordinary default timeout and decreasing observation budgets. Timer/message/state boundaries are declared fixtures; no browser or native peer is created.

`source-pins.json` records exact working and canonical HEAD source hashes plus evidence. This review did not rerun Root's full suite/typecheck or optional HTTP gates, and does not claim those as independent execution. No listener, browser, stage, normal build, native Game, RTC or saved-profile operation occurred.
