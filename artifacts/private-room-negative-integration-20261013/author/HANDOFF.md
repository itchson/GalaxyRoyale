# Bounded private-room negative QA candidate

Candidate **`1b14bff9e6da0a1a249f1bdc42ab49e59e74e52c`**, branch `codex/private-room-negative-qa`, based exactly on Root helper checkpoint **`fbb8b1c640f0e11d495cf00c11bf557500701674`**. Checkout: `C:/Users/mitch/.codex/worktrees/galaxy-audio-refinement/galaxy-royale`. The tracked tree was clean before branching and after committing. Prior `codex/haven-scene-fixtures` and ignored work/checkpoint directories were retained. Root shared testing was never checked out or edited here.

Six tracked files changed: existing QA client, parent script/page, report server, README, and new focused `coop-private-room-negative-qa.test.mjs`. No production game/service/native/package source changed. Actual preparation file sets and legacy allowlists are unchanged; the helper needs a newly prepared source-pinned stage after Root integration. Prior browser captures retain their original helper provenance.

## Implemented behavior

- **Decline:** real current `JoinRequest.id`, host/exactly-one guard, and `controller.approve(id,false)`. Pending observation verifies no new cumulative PCs/native guest entry/ready/admission. Actual `join_declined` must restore guest solo and terminal resources while retaining the host room/native ownership.
- **Replace:** real `controller.replaceCode()` returns the private new grant. An already approved owned guest must keep room/seed/host/peer identity, committed participant epoch, PCs and successful native ownership counts, and commit additional complete frames. Only then does the guest leave before trying the old code (`stale_invite`). A fresh pending guest is separately cancelled by another rotation (`code_replaced`). No host recreate occurs inside policy phases.
- **Lock:** strict boolean `controller.setJoiningLocked`. It checks pending cancellation and locked lookup (`joining_locked`), unlock/revision progression, fresh approval/owned commit, then approved-guest continuity with additional commits across both locking and unlocking. Two-client approved and pending phases are separate, not simultaneous three-client proof.

Private marks use actual transport diagnostics, current committed publication epochs, cumulative real PC factory/close counts, and successful actual `co_op_batch` / `co_op_replica` begin/join/leave/end operations. Raw code/session/seed/self/peer/epoch values stay in private commands/memory. Public client report shape remains unchanged; the server admits only the new fixed boolean check names. Unknown terminal strings are not forwarded. Epoch/progress checks reject old-session headers and use new commits rather than lifetime tick maxima.

Every negative case requires visible foreground clients in owned default mode, both solo/clean, and at least 140 seconds of each native loop lifetime remaining. One attempt per page is latched even on failure. The case has a 120-second budget, bounded reply budgets and post-reply deadline checks; cleanup gets at most another 20 seconds. The original failure is retained if cleanup also fails. Cleanup attempts actual leave on both controllers, checks solo/PC/request/timer closure and normalized digest equality. Dispose remains the operation that frees Galaxies. A timed-out child operation can still finish; failure requires actual cleanup/disposal before reload.

## Actual author verification

All commands were run in the isolated checkout, exit 0:

1. `node --test scripts/test/coop-private-room-negative-qa.test.mjs`: **11/11 pass**, `negative-tests-final.log`.
2. Targeted existing WASM source/function/report checks (exact pattern in `source-pins.json`): **16/16 pass**, `existing-wasm-functions.log`. Supplied-artifact HTTP tests were excluded, so no listener was eligible.
3. `node --experimental-transform-types --test scripts/test/coop-transport-fifth-qa.test.mjs`: **10/10 pass**, `existing-fifth-functions.log`.
4. `node --check` for client, parent and server; scoped `git diff --check`: exit 0. Local Prettier formatted only owned files.

The new checks execute actual helper functions/handlers against explicitly declared native/timer/message fixtures. They cover false/true current-ID decisions, strict lock inputs, private output filtering, successful real bridge observer wiring, preapproval isolation, wrong terminal/ownership/epoch/old-commit rejection, replacement ordering, lock phase ordering, deadline/lifetime/one-attempt boundaries and cleanup preserving the primary failure. They are not native/browser/RTC acceptance.

No test failed in this author pass; initial 8-case and expanded 10-case logs are retained. The first Git commit command failed because no identity was configured; its output is retained in `original-commit-failure.log`. The successful retry used the established prior `Codex <codex@openai.com>` identity through command-local `-c` flags, with no Git configuration changes. `commit.log` records the successful commit.

Studio task `task-71611b9e` was actually dispatched: exact revision1 was updated through the existing API to active revision2, and running record `run-5695acbc` was appended. Payloads/responses are retained. Candidate source work is ready for independent review; live acceptance and task acceptance criteria remain open.

## Remaining gates and limits

Root owns independent review, integration, fresh stage provenance, listener/browser execution on both backends and final capture curation. Run one of the three labeled buttons per fresh initialized page within 40 seconds, then Dispose and reload for the next case. No listener, browser, normal build, stage, native Game, actual RTC, stored-profile or live negative operation was performed by this candidate author.

Normalized digest checks do not establish zero migration writes or exact boxed ECS/RNG/body/presentation restoration. `SaveStore.open` may migrate data. Native deterministic restoration proof remains separate. Shipped panel/Main entry UX, three simultaneous game clients, eight players, Internet/TURN, physical/mobile timing, performance and reference music remain unaccepted. No fake native result or studio-service dependency was added to the shipped game.
