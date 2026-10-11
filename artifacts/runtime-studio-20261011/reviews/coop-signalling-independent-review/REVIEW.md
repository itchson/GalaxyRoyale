# Independent co-op signaling review

Verdict: **no blocking source finding** in exact frozen commit **2fe20c6401e65f8dcc790b537269e2de18d85b4d**. I independently executed its focused suite from committed source snapshots: **20/20 pass**, zero failures/skips. This accepts the bounded signaling foundation only; it is not playable co-op or a security/deployment certification.

## Reviewed contracts

The four new files contain no frontend/Rust/world/reward integration or shared dependency changes. Importing the server starts no listener. The normal CLI binds loopback only and does not kill an occupied owner. Exact HTTP(S) Origin checks, fixed routes and JSON field/byte bounds guard the HTTP boundary. No route performs arbitrary URL forwarding. CORS remains a browser boundary rather than account authentication, as documented.

The store admits at most eight live or pending participants including the host. Synchronous admission and queue preflight prevent concurrent ninth-seat allocation. Host-only end/replacement and same-session host-to-guest addressing use separate random bearer capabilities retained as SHA-256 hashes. A guest cannot mutate immutable session seed/protocol or signal another guest/self/foreign participant.

Invites use Node AES-256-GCM with a fresh per-store 32-byte key, random 12-byte nonce, 16-byte tag and fixed versioned AAD. Only version/session/invite IDs/expiry are encrypted; participant credentials and player profiles are absent. Bounded optional # and case/whitespace normalization retains the canonical authenticated bytes. Tampered version/tag/bytes, expiry and service restart reject. This is a long authenticated capability, not a secure short numeric code.

Anonymous create/join attempts are charged before body parsing; replacement charges the global create and original owner-client bucket across changed credentials. Queue/event/byte/session/participant/client/socket limits are explicit. Full signaling queues reject without silently accepting transport; terminal notices replace old queued traffic. Event delivery is replay/ack-based, not exactly once. Queued transport is cloned rather than aliased to caller data.

Session and participant clocks are monotonic. Pending 20-second reservations do not extend through unready heartbeats; active guest/host heartbeat deadlines and absolute session expiry are separate. `#end` prepares every participant terminal queue before one change notification, avoiding partial terminal reads. Polls have one owner per participant, finite waits, subscribe/read-race closure and abort cleanup. Maintenance expiry wakes polls. `server.coop.stop()` ends sessions, refuses late mutations and closes sockets/timers/subscriptions; the suite checks these actual cleanup paths.

## Actual evidence and source identity

Exact committed snapshots are under `source/`. The independent command was `node --test work/coop-signalling-independent-review/source/scripts/test/coop-signalling.test.mjs`, recorded in `independent-tests.log`. Node 24.18.0/Windows x64 and its actual OpenSSL/V8 versions are recorded separately. Nineteen cases use owned 127.0.0.1:0 HTTP listeners and synthetic signaling bodies; one uses the timer-free store. Long lifecycle durations use declared injected time; one expiry-poll test uses the real 250ms maintenance interval. This is not a minute/hour soak.

Coverage includes concurrent host+seven pending guests, Origin/CORS/capabilities, invite copy/tamper/expiry/restart, roles/offer-answer ordering, liveness, all seven final guest notices, replacement rates and failed replacement preserving the prior session, replay/queue/byte/rate quotas, body/SDP/ICE schema limits and chunked overflow, poll abort/timeout, shutdown and retained credential pruning. Every fixture cleanup calls `stop()` and asserts closed listening state, no polls and no change subscribers; the test process exited normally. No persistent service, browser, game actor or RTC peer was started.

I separately verified all seven committed inputs from the peer manifest plus its final 20-pass log, and pinned the peer handoff/manifest. Those peer results are not substituted for my own 20-case run. All source/report/evidence hashes use their declared exact Git or raw-byte domain; working-tree CRLF aliases do not silently replace committed source.

## Acceptance limits

Actual RTC negotiation, ICE/NAT/STUN/TURN, WebRTC data channels, game command validation/host authority, profiles/rewards, HTTPS/public proxy/rate policy, frontend departure/RTC teardown, mobile/background recovery, physical devices and long-running public load remain Pending. No secret token/profile/SDP/ICE logging or outbound forwarding was found in the reviewed service, but this is a bounded code/test review rather than a general penetration test. Root must keep these infrastructure and gameplay gates distinct. No source repair was required by this independent review.
