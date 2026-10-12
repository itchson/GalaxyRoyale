# Actual baseline and dispatch

Root inspected source d2acfd2. Existing service uses long AES-GCM invitation envelopes and bounded256-bit participant bearer credentials. Native Rust supports one host plus seven guests and boxed local-world restoration. Browser controller currently automatically native-admits a guest join frame; the new flow needs an explicit approval boundary. Production main.ts does not pass RTC configuration, so existing generic transport configuration support does not establish configured TURN fallback.

Actual assignments reuse three clean attached worktrees/branches from d2acfd2: Engine service on codex/short-room-service, Browser UX/transport on codex/room-approval-browser, QA on codex/room-security-review. Their runs were dispatched through real followup_task calls; they are not simulated meetings. No new feature test, eight-client run or forced relay has passed merely because these assignments exist.

Root's single existing preview remains PID48668/4173 with compiled source1eca1c5 and unchanged runtime/package hashes. Studio4174/4175 and signaling4176 remain separate. The new service/browser candidates must be reviewed and rebuilt together before replacing that preview/signaling service. Old evidence remains source-specific and is not promoted to new room-flow acceptance.
