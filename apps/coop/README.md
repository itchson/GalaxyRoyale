# Private-room co-op signaling

A separate, dependency-free, ephemeral Node HTTP service exchanges WebRTC setup messages between one host and at most seven guests. It does not simulate the world, admit a Rust actor, transport gameplay, grant progression, or establish RTC connections. Host approval is a signaling prerequisite; `ready` remains a later signaling state.

The service/browser bundle must match this short-code contract. The gameplay protocol remains `galaxy-coop-1`; old long encrypted-envelope clients are not compatible. Existing `/v1/replace` replaces an entire session and is distinct from the new same-room `/v1/code` action.

## Launch and stop

Importing `server.mjs` opens no listener. Supported Node runtime: the repository's Node 24 runtime. Explicit local launch:

```powershell
$env:COOP_ALLOWED_ORIGINS = 'http://localhost:4173,http://127.0.0.1:4173'
$env:COOP_PORT = '4176'
node apps/coop/server.mjs
```

The CLI binds `127.0.0.1`; an occupied port fails visibly. Ctrl+C/SIGINT or SIGTERM ends rooms and closes the listener. Tests use temporary `127.0.0.1:0` listeners, all closed in cleanup. No persistent instance was launched for this implementation.

Every API request except `/health` requires an exact allowed Origin. Configure 1–16 HTTP(S) origins without paths, wildcards, or credentials. CORS is a browser boundary, not authentication; nonbrowser callers can forge Origin. Administration requires the host's separate bearer capability. There are no accounts, profiles, cookies, or arbitrary forwarding routes. The CLI logs startup/fixed listener failure only, never codes, credentials, SDP, or ICE.

## Room codes and authority

Each code uses `node:crypto.randomBytes(4)` and is displayed as `#A7C2-9F34`. Paste normalization removes ASCII whitespace (TAB through CR and space), hyphens and one optional leading hash. Exactly eight hex digits remain, case insensitive; raw input is at most 64 UTF-16 code units. Other characters, extra hashes or digit counts fail. Codes are lookup identifiers, not encrypted administration credentials.

Allocation checks all active and retired reserved codes, with at most 32 attempts. A room reserves at most 64 codes, including the initial code. Rotation keeps the original session, seed, host ID, absolute expiry and approved participants. Retired codes cannot join or be reused while their original room record remains; reservations are removed after room end plus 60-second retention. Restart clears all old rooms, codes and bearer credentials. The current code remains valid for its live room. Host departure, timeout, stop, restart or absolute one-hour room expiry ends its validity; rotation retires the old code without extending that room deadline. There is no independent invite timer.

All participant tokens are separate lowercase 64-hex secrets, stored as SHA-256 digests. Participant/session IDs are lowercase 32-hex. Only a current host token may rotate, change joining policy, accept/decline, or end its room. Codes and guest credentials cannot administer. Guest session projections always set `room_code:null`, including declined and retained terminal guests; only host-authenticated views reveal the current canonical code. Diagnostics return counts only, including `reserved_codes`.

## API and exact policy projection

Mutation bodies use `Content-Type: application/json`; authenticated calls use `Authorization: Bearer <token>`. Unknown fields/query keys reject. Errors are `{error:{code,message}}`; responses are `Cache-Control:no-store`.

| Method/path          | Body                                        | Result                                                     |
| -------------------- | ------------------------------------------- | ---------------------------------------------------------- |
| GET `/health`        | none                                        | status, gameplay protocol and eight-player limit           |
| POST `/v1/sessions`  | `{seed:uint32,protocol:"galaxy-coop-1"}`    | 201: `{session,invite,participant:hostWithToken,cursor:0}` |
| POST `/v1/join`      | `{invite}`                                  | 201: `{session,participant:pendingGuestWithToken,cursor}`  |
| POST `/v1/code`      | host bearer + `{}`                          | 200: `{session,invite}`; same-room rotation                |
| POST `/v1/joining`   | host bearer + `{locked:boolean}`            | 200: `{session}`                                           |
| POST `/v1/approval`  | host bearer + `{peer:32hex,accept:boolean}` | 200: `{session,participant}` without a token               |
| POST `/v1/replace`   | host bearer + create metadata               | 201: new room/host/code; old room ends                     |
| POST `/v1/heartbeat` | bearer + `{}` or `{ready:true}`             | 200: `{session,participant}`                               |
| POST `/v1/leave`     | bearer + `{}`                               | 200: `{session,participant}`                               |
| POST `/v1/end`       | host bearer + `{}`                          | 200: `{session}`                                           |
| POST `/v1/signals`   | bearer + `{to,kind,payload}`                | 200: `{accepted:true}`                                     |
| GET `/v1/events`     | bearer; `after=0&wait_ms=0` defaults        | 200: `{session,participant,events,cursor}`                 |

Every session view has `id,seed,protocol,host_id,expires_at,state,reason,joining_locked,room_revision,room_code`. `room_revision` starts at zero and advances for each code rotation and actual lock-state change; no-op unlocked updates preserve pending request validity. Hosts receive a complete current code/lock/revision tuple in every response, so newer lock/heartbeat/poll replies can supersede a delayed rotation reply without losing the new code. Guests receive the revision/lock fields with null code. Clients must reject older policy revisions within the same session. This policy revision does not replace transport or native gameplay sequence checks.

Host events use the existing peer object `{id,role:"guest",status:"pending"}`:

- `join-request`: `{seq,type,peer,expires_at,room_revision}`. The revision is the request's birth policy; clients discard requests older than their current policy.
- Acceptance: host gets `{seq,type:"peer-joined",peer}` and guest gets `{seq,type:"join-approved"}`. Both queues are prepared before consumers are notified. Guest participant status remains pending until ready; a private service flag separates approval from readiness.
- `peer-ready`: peer object with active status, once after approved guest `{ready:true}`.
- `peer-left`: `{seq,type,peer_id,reason}`. Terminal guest `participant-left` and room `session-ended` contain `seq,type,reason`.
- `signal`: `{seq,type,from,kind,payload}`.

Unapproved credentials cannot send or receive SDP/ICE or become ready. Pending requests reserve capacity within eight total participants. An unapproved request expires 20 seconds after join, even with heartbeats, reason `approval_timeout`. Acceptance starts a separate 20-second unready negotiation deadline, reason `reservation_timeout`. Active guest liveness then uses the 45-second heartbeat deadline. Decline produces guest left status and reason `join_declined`; rotation and lock cancel only unapproved requests with `code_replaced` and `joining_locked`. Approved pending/active guests remain. Unlocking allows future requests. Already-resolved/cross-room/host peer approval yields 409 `request_unavailable`.

Rotation, lock and decline preflight all required host notices before policy or request changes. Acceptance preflights both recipients before changing approval. Failed collision, code quota or event queue admission leaves the prior policy, code and participant approvals unchanged. Batched cancellations do not use ordinary saturated membership removal that can end the room. Typical errors: invalid/unknown code 401 `invalid_invite`, retired code 410 `stale_invite`, ended room 410 `session_ended`, locked room 409 `joining_locked`, full room 409 `session_full`, and preapproval negotiation 403 `approval_required`.

## Transport, lifecycle and bounds

Only same-room host-to-guest signals are accepted. Host offers; approved guests answer only an outstanding offer. ICE may be null or an exact candidate object. SDP is 1–16,384 UTF-8 bytes; candidate is at most 2,048 bytes, optional strings 64 code units, optional line index 0–32. These are transport structure checks, not SDP correctness or actual network connection proof.

Events retain the existing global monotonic sequence and addressed queues. Process a response before advancing its cursor; gaps are normal. Unacknowledged requests/events can be replayed. Acknowledged older cursors fail. At most one long poll per participant, wait at most 20 seconds; abort and shutdown remove its timer/subscription. Heartbeats are explicit; polling and signaling do not extend liveness.

| Bound                                                        | Default                     |
| ------------------------------------------------------------ | --------------------------- |
| Live participants including host and pending requests        | 8                           |
| Approval wait / approved negotiation wait                    | 20 / 20 seconds             |
| Host / active guest heartbeat timeout                        | 30 / 45 seconds             |
| Current code / absolute room lifetime                        | Live room / 60 minutes      |
| Terminal credentials/notice/code retention after room end    | 60 seconds                  |
| Rooms including ended records / participant records per room | 64 / 64                     |
| Reserved codes per room / allocation attempts                | 64 / 32                     |
| Host policy/approval attempts per minute                     | 30                          |
| Event queue                                                  | 64 events and 262,144 bytes |
| Request body                                                 | 24,576 bytes                |
| Signals per participant / room per minute                    | 60 / 240                    |
| Create / join attempts globally per minute                   | 12 / 120                    |
| Create / join attempts per direct client per minute          | 2 / 20                      |
| Open rooms per direct client / salted client rate records    | 4 / 256                     |
| Sockets                                                      | 128                         |

Malformed create/join attempts consume existing direct/global rates before body parsing. Host policy/approval attempts use a separate bounded authenticated bucket before parsing; invalid bodies count, unauthorized guests do not consume a host's bucket. Whole-session replacement retains original-owner create rate limits. Forwarded headers remain ignored. Limits may narrow only; terminal queues require at least 256 bytes. The store is synchronous, timer-free and monotonic-clock bounded; the server owns maintenance and poll timers.

Host departure, timeout, absolute expiry, whole-session replacement and shutdown prepare every terminal room notice before notifying readers. Ordinary saturated mandatory membership queues retain the existing `signaling_overflow` end policy. Departed records stay bounded, then credentials and reserved room codes are erased. No permanent TURN secrets exist in this service or the game bundle.

## Executed focused evidence and remaining gates

```powershell
node --test scripts/test/coop-signalling.test.mjs scripts/test/coop-private-rooms.test.mjs
```

32 focused tests passed during this implementation: real HTTP capacity, admin authority, pending/approval signals, collision retry failure, stale codes, expiry, same-room rotation/locking, aggregate queue preflight, complete policy snapshots, strict input schemas, rates, long-poll abort/shutdown and cleanup. Code-collision cases use an injected four-byte generator; production allocation uses crypto randomness. Transport fixtures are explicitly synthetic; they do not create RTC/game clients.

Browser integration, actual eight-client gameplay, physical networks, forced TURN, public HTTPS deployment and profile/save/guest-world restoration proof remain separate integration gates. This service neither issues relay credentials nor proves a relay connection. Primary references: [Node crypto randomBytes](https://nodejs.org/api/crypto.html#cryptorandombytessize-callback), [WebRTC protocols](https://developer.mozilla.org/en-US/docs/Web/API/WebRTC_API/Protocols), [RFC 8831 DataChannels](https://www.rfc-editor.org/rfc/rfc8831).
