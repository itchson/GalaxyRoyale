# Co-op signaling source handoff

Frozen commit **2fe20c6401e65f8dcc790b537269e2de18d85b4d**, branch `codex/coop-signalling`, parent **a50aab678c98466e8c54838e39ca5a40a9f2b1b7**. Exactly four added files: `apps/coop/store.mjs`, `apps/coop/server.mjs`, `apps/coop/README.md`, `scripts/test/coop-signalling.test.mjs`. Tracked checkout is clean. No shared manifest, frontend, Rust, game build, Studio, browser, API or push mutation.

The Node-only service implements synchronous atomic host + seven guest reservations (pending guests count), encrypted authenticated invite capabilities, hashed participant credentials, exact Origin allowlist, explicit heartbeat/readiness, host-only end/replacement, host↔guest offer/answer/ICE transport, bounded replay queues and long polling. Invites emit `#` plus uppercase AES-256-GCM hex; optional `#`, outer whitespace and either case are accepted after strict bounded even-length validation. They are long authenticated capabilities, not invented nine-digit secure short codes. Participant credentials remain 64 lowercase hex.

The complete exact request/response contract, error/lifecycle behavior and quotas are in the committed README. Default port4176 binds127.0.0.1 only. Import does not listen. We launched **no persistent instance**. Every actual test listener bound127.0.0.1 port0 and stopped through `server.coop.stop()` in cleanup, with listener/poll/subscription assertions. Final Node process exited normally; no service was left running.

Default timeouts: pending20s from join, host30s, active guest45s, invite10min, absolute session60min, departed/session terminal retention60s. Expiry tests use a declared injected monotonic clock through actual HTTP; the host-timeout pending-poll case exercises the real250ms service maintenance interval. There is no minute/hour wall-time soak claim. Host departure/end/expiry/replacement/shutdown prepares all terminal queues before notifying pollers. Terminal notices replace old signaling data rather than being starved by it. Stopping also rejects late mutations. Client frontend departure/RTC teardown and replacement of the actual shared world remain Root integration work.

Global fixed-window create/join attempt limits12/120 per minute, direct salted-client2/20,64 sessions including tombstones,64 retained participant records/session,4 open host sessions/client,256 rate-client records,128 sockets. Authenticated replacements consume the global create and original host client-create buckets **before body parsing**; changing credentials cannot reset the rate. Failed validation consumes an attempt but preserves the old session. No accounts/public distributed anti-abuse claim; forwarded addresses are ignored and public HTTPS/proxy/TURN policy remains pending.

## Actual checks

Node **24.18.0**, Windows x64. Final command `node --test scripts/test/coop-signalling.test.mjs`: **20 passed,0 failed,0 skipped**, raw `final-tests.log` (711.3582ms reported total). Nineteen cases use real ephemeral HTTP/fetch/chunked requests and the remaining case tests timer-free store cloning/monotonicity. Some HTTP cases also directly inspect bounded internal diagnostics/configuration. All body/SDP/ICE data are explicitly synthetic signaling fixtures; no RTC peer, camera/microphone, data channel or game actor is created.

Covered: concurrent ninth-player rejection with eight total credentials; exact Origin/CORS; optional invite copy/paste variants plus authenticated byte/version/tag tamper, expiry and restart; role/self/guest↔guest/stranger restrictions; answer-after-offer; pending/active liveness; all seven guest terminal long-poll notices; end/leave/timeout/absolute expiry/replacement; replacement rates and failed-replacement atomicity; replay/ack/queue-event/byte limits; terminal overflow; unauthenticated invalid-attempt/global/client/session quotas; sender/session signaling limits; declared-length and actual chunked body overflow; one poll per participant; abort/finite wait/maintenance wake; shutdown and retained-record pruning; payload cloning.

Executed Node syntax checks for all three code/test files, installed Prettier formatting/check for all four files, and staged `git diff --check`; all passed. No game/full repository build or broad unchanged suite was run.

## Preserved negative evidence

- `first-tests.log`:14/15. The guest expiry fixture omitted a timely host heartbeat; real service correctly returned host_timeout instead of the expected guest_timeout. Fixture was corrected without relaxing lifecycle rules.
- `copy-paste-rate-tests.log`:18/20. During the new replacement-rate test update, the author inserted the keepalive loop into the wrong test, leading to a departed guest410 and replacement429. The loop was moved to the intended replacement fixture; limits stayed unchanged. Corrected run20/20 is `copy-paste-rate-fixed-tests.log`.
- Earlier positive17/17 and19/19 logs remain historical. The terminal-notice publish-order risk was found statically and fixed before the first test run; there is no fabricated executed negative claim for it.

`frozen-hashes.json` pins exact committed bytes and working-file bytes separately (Git line-ending handling), seven source inputs and raw logs/handoff. The source snapshots contain no runtime credentials or user profile.

## Acceptance boundaries

This is signaling foundation only. It does not authorize Rust actors/rewards, replicate a world, validate gameplay commands, or prove playable co-op. Browser integration, actual offer/answer/ICE interoperability, NAT/STUN/TURN, HTTPS/public hosting, mobile/background disconnects, physical devices, multi-client gameplay and long-running public-service load are Pending. The service performs no arbitrary forwarding and logs no credentials, SDP or ICE. The [W3C WebRTC specification](https://www.w3.org/TR/webrtc/) is the primary reference for the separate signaling/peer/STUN/TURN boundary.
