# Private co-op rooms — requested feature and implementation contract

This replaces the long invitation UI with short private-room codes. It is assigned work, not an accepted multiplayer release. Existing native host simulation and solo-world restoration are reused.

## Player flow

- Clicking the dedicated floating MULTIPLAYER spacehub opens a full-screen scene with Join and Host tabs. Join is the initial default; reactive updates preserve the selected tab. A multiplayer HUD icon reopens this same scene. The central spacehub is not the multiplayer entry point.
- Join accepts exactly eight hex digits, case-insensitive. Canonical display is `#A7C2-9F34`. An optional leading `#`, hyphens and ASCII whitespace may be removed on paste; other characters, additional hashes or a different digit count are rejected. Raw input is bounded to64 characters.
- Host starts a private world for eight players total: one host and at most seven guests. It displays the current code and offers Copy, Replace Code, Lock Joining and Stop Hosting. Approval requests have explicit Accept and Decline actions.
- Replacement changes only the join code, preserving the session, native host world and approved guests. Locking denies new joins. Both replacement and locking retire outstanding unapproved requests, keeping the trust boundary clear; already approved guests remain.
- Guests await approval before RTC negotiation or native admission. Refusal, timeout, full room, locked room, stale code and connection failure are understandable outcomes; none may leave a guest stuck or overwrite its local world.

## Signaling and authority contract

Codes are cryptographically random four-byte lookup identifiers. They are not encrypted administration credentials. A small ephemeral signaling service checks collisions against active/reserved codes, generates within a bounded retry budget, and never exposes its host bearer token in a displayed code. End, host timeout, session expiry or service restart invalidates room codes. Retired codes cannot resolve or be reused while their original room remains live. Code rotation is bounded and atomic.

Existing64-hex-character participant bearer credentials remain separate. Only the host bearer can rotate, lock, approve or stop its room; room codes and guest bearers cannot administer it. Existing bounded direct-client/global admission rates, session quotas, request bodies, event queues and no-secret diagnostics remain enforced. Pending approvals reserve capacity within the eight-player ceiling and expire after a documented bounded wait.

For this implementation pass, preserve the gameplay wire protocol and native Rust ownership model. The service's existing `invite` response/request key may carry the canonical short code to avoid unnecessary plumbing changes; the product calls it Room Code. Matching rebuilt service/browser clients are required; old long-envelope clients are not an accepted compatibility target.

The assigned service/browser contract is:

| Endpoint | Authority and operation |
| --- | --- |
| `POST /v1/sessions` | Create room; return host grant and canonical short code. |
| `POST /v1/join` | Normalize `{invite}`; reserve a pending approval; do not open RTC. |
| `POST /v1/code` | Host bearer, empty body; rotate code in the same room. |
| `POST /v1/joining` | Host bearer, `{locked:boolean}`; change joining policy. |
| `POST /v1/approval` | Host bearer, `{peer:string,accept:boolean}`; atomically resolve a pending request. |
| Existing heartbeat/events/end/leave/signals | Retain bounded credential/sequence/lifecycle behavior. |

The coordinated session view adds `joining_locked`, `room_code` and monotonic `room_revision`. Every newer authenticated host view includes both the current code and lock state so delayed replies cannot roll back either field. All guest views contain explicit null for the code, including retained terminal reads: an old rejected guest must not learn a replacement code. A host receives `join-request {peer,expires_at,room_revision}` for a pending guest. Approval atomically produces `peer-joined {peer}` for the host and `join-approved` for the guest. Participant status remains pending behind a private approval flag until RTC readiness; unapproved credentials cannot send offers, ICE or self-promote through heartbeat. Decline/rotation/locking/approval timeout produces a terminal guest outcome. Session seed/world/host ID remain stable across code rotation. A no-op unlock does not invalidate otherwise current requests.

The first bounded pass retains a20-second approval window and a separate20-second negotiation window from acceptance. It reserves at most64 code allocations per live room, retries code generation at most32 times, and rate-limits authenticated administration to30 attempts per minute before reading the body. These are implementation bounds, not measured human timing or public abuse acceptance. Candidate tests must cover response reordering, collisions, failed queue preflight and unchanged approved guests.

Codes remain valid for the lifetime of their live room, with no independent ten-minute invitation timeout. Host departure, host timeout, the absolute room lifetime, end or restart invalidates the code. Code rotation does not extend room lifetime. The service lifetime fixture advances a controlled clock; it is not a wall-clock soak test.

WebRTC DataChannels carry encrypted game traffic in a host-to-guest star. TURN fallback requires an actually configured relay and an authenticated short-lived credential path; never embed a permanent relay secret in the game bundle. Configuration support is distinct from a real forced-relay connection test. No camera/microphone is required. Native Rust remains authoritative for admission slots, inputs, collision, damage and world state. Guests share the host hub, retain their own boxed local worlds and progression, and return home on host departure or failed connection.

## Bounded work and evidence

Service specialist owns short-code lookup, collision/rotation/lock/approval rules and meaningful real-service tests. Browser specialist owns the parser, signaling/transport approval gates and reactive Join/Host panel; Root owns the HUD route/native icon and integration. QA reviews the actual candidates independently and derives a source-pinned validation matrix.

Record separate evidence for service capacity and eight real game clients; pending versus approved guests; old-code rejection and unchanged connected guests; unauthorized administration; repeated/invalid join attempts; decline/timeout; disconnect/host departure; failed ICE; forced TURN; unchanged local seeds, cargo and save digests. A mock RTC transport, native eight-pilot test or service full-room test does not establish eight-client gameplay. Real device/network gates remain pending until operated and captured.

Primary technical references: [Node cryptographic random bytes](https://nodejs.org/api/crypto.html#cryptorandombytessize-callback), [WebRTC ICE/STUN/TURN](https://developer.mozilla.org/en-US/docs/Web/API/WebRTC_API/Protocols), [WebRTC data-channel protocol](https://www.rfc-editor.org/rfc/rfc8831).
