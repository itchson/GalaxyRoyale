# Configured Wake QA final-health follow-through

Accepted for scoped integration/fresh testing at exact `f407dda355d92ec3ed2c6fc867d9eba0ffa9a6fd`, direct parent `1f28378871406383435f7ba103436d58026968d3`. No source blocker remains for the two preserved qualification findings. This decision is not actual native/WASM/RTC, save-store, canvas, visual or physical acceptance.

The runtime-facing change is confined to the QA parent `scripts/qa/co-op-transport/wasm-review.js`; two matching test files change. Native/gameplay/browser runtime, fixtures, report protocol/observation version and child ownership paths are unchanged. Three canonical/raw source pins,42 inherited raw/canonical inputs and14 original candidate canonical inputs were checked against the freeze; author raw evidence and preserved refs are separately pinned by `verify-evidence.py`.

## Corrected qualification boundaries

`wakeHealth()` checks both clients' current initialization readiness, null failure/reason, non-inconclusive Wake observations and exact current contexts. The expected live context is host/hosting plus guest/connected; after configured leave it is host/hosting plus solo/ended. These are real child observation fields, not new guessed world state or native operations.

`wakeRound()` applies that gate before dispatch, on each actual observation read, immediately before its awaited final verification and after that awaited verification. At the latter two boundaries it also rechecks the operation's current predicate. Stop/retreat require the latest captured point count to be0; historic expiry/removal flags alone cannot qualify a newly changed capture. The existing visibility latch still covers the visible Wake experiment through final verification.

The final verification runs `verify(5000,false)` to refresh both clients and compute the unchanged-profile/save fact without appending it yet. Only after current health/context/predicate checks does the parent append the save and operation facts. Thus the original delayed-final-read failure cannot create either a new Wake pass or a generic save pass first. A failed save comparison still records a failed check and triggers failed cleanup, as intended.

The new configured `wakeLeave()` observes the actual returned leave reply immediately, then checks current ended-guest/live-host context through owner-removal observation and before/after the final non-recording read. Clean peer/request/timer counters, actual host owner removal and actual guest local-view end must still hold at the final boundary. It appends restore/removal/save passes only afterward. Ordinary Guest leave retains its prior path, independently compared by the exact-function test.

Failure calls the existing bounded cleanup for both clients, with separate20-second replies. A timeout remains unresolved cleanup and does not set the parent's disposed flag. This does not prove late timeout cancellation, actual restore or free; those remain child/native/live gates. Earlier checks that actually completed remain append-only history. `action()` still captures a sanitized operation failure, and the unchanged report server does not rewrite prior report history merely because a later operation failed.

## Actual independent checks

We ran only `node --experimental-transform-types --test scripts/test/coop-wake-live-health-qa.test.mjs` in the clean frozen author worktree. All11 cases passed, zero skips/failures. The source was verified as f407 before/after; no listeners, browser, native build or real fault was executed. `focused-health.log` and its exit0 are retained here.

These use TypeScript AST extraction of the actual amended QA functions and immutable1f functions, evaluated with declared API reply doubles. They reproduce both old false passes, then verify delayed final-read failure, healthy-looking retirement, lost captured-owner/current predicate, each phase's final state, failure before final read, cleanup timeout, positive healthy phases, failed leave reply, delayed removal/read failure, lost host context, healthy leave, ordinary path equivalence and actual non-recording verify behavior. The old functions resolve incorrectly under those declared replies; the new ones reject and attempt both bounded disposals while preserving the earlier successful check. This is qualification evidence, not simulated native/RTC acceptance.

The author's frozen64-case evidence is48 final cases (11 new,18 inherited configured,10 RENKO,9 provenance) plus16 explicitly selected consumers. We verified source case counts and actual log summaries/commands; did not rerun the broad64. Format/syntax logs and tsc are retained; the tsc pass covers unchanged TypeScript inputs and does not typecheck this QA JS or compile WASM.

Our first byte verifier passed the source/evidence hashes but then failed its test-log pattern because Python's default Windows text decoding misread UTF8 status glyphs. The original script/log/exit are retained as `verify-evidence-first.py` and `verification-first-encoding-failure.*`. Explicit UTF8 decoding fixes the verifier; this was a reviewer packaging error, not an author test or source failure. The final verification passes3 changed,42 unchanged,14 original canonical sources,5 preserved refs,38 author evidence files,36 original raw files and4 reviewer seals.

## Preservation and remaining gates

The original reviewer report `b8586f...` and manifest `5f0612...`, corrected failure scripts/logs and the two earlier import setup failures are preserved unchanged in both the original review and author's copies. Import failures occurred before function evaluation and do not count as functional replays. The author's red baseline logs retain expected assertion exit1. Original1f and native/browser/fixture refs remain intact.

Old prepared configured stages must fail the changed source pins; Root must prepare a fresh stage with matching helper/build/carry provenance. Still open: real configured native ownership/page/commit/render expiry observations, healthy and failed teardown/save comparisons on both normal backends, human visual/physical inspection, eight real seats/Internet/TURN and performance. The source correction repairs two specific false-positive paths; it does not supply those live passes. Music-calibration/fairness evidence was neither changed nor inherited as approval.
