# Ephemeral co-op signaling

This is a separate, dependency-free Node HTTP service for exchanging WebRTC setup messages between one host and up to seven guests. It does not simulate the world, admit a Rust player, award progression, transport gameplay packets, or establish an RTC connection. A guest's `ready` state means signaling readiness only.

WebRTC applications exchange offers, answers and ICE candidates through an application-defined signaling channel. STUN/TURN configuration and relay availability remain separate requirements; this service supplies neither. See the [W3C WebRTC specification](https://www.w3.org/TR/webrtc/). No camera or microphone is used here.

## Run and stop

Requires the repository's supported Node runtime (tested with Node 24.18.0). Importing `server.mjs` starts no listener. Explicit local launch:

```powershell
$env:COOP_ALLOWED_ORIGINS = 'http://localhost:4173,http://127.0.0.1:4173'
$env:COOP_PORT = '4176'
node apps/coop/server.mjs
```

The listener always binds `127.0.0.1`. An occupied port fails visibly; it does not replace another process. Ctrl+C/SIGINT or SIGTERM ends sessions and closes the listener. This task did not launch a persistent instance. HTTPS/public hosting, TURN, trusted proxy handling and public abuse testing are pending integration work.

The exact Origin allowlist accepts 1–16 HTTP(S) origins with no path, credentials or wildcard. Every API request except `GET /health` requires an allowed `Origin`; browser preflight permits only GET/POST and Authorization/Content-Type. CORS is a browser boundary, not user authentication: clients outside a browser can supply an Origin. Session participation requires the issued bearer capability. There are no accounts, cookies or profiles.

## API version 1

All mutations use `Content-Type: application/json`. Authenticated calls use `Authorization: Bearer <token>`. Tokens are lowercase 64-hex strings and IDs are lowercase 32-hex strings. Unknown JSON fields and query parameters are rejected. Errors are `{error:{code,message}}`; responses use `Cache-Control: no-store`. No route fetches or forwards an arbitrary URL.

| Method / path        | Body / query                             | Result                                                       |
| -------------------- | ---------------------------------------- | ------------------------------------------------------------ |
| GET `/health`        | none                                     | `{status:"ok",protocol:"galaxy-coop-1",players:8}`           |
| POST `/v1/sessions`  | `{seed:uint32,protocol:"galaxy-coop-1"}` | 201: session, host credential, encrypted invite, cursor      |
| POST `/v1/join`      | `{invite}`                               | 201: session, pending guest credential, cursor               |
| POST `/v1/replace`   | host bearer + same metadata as create    | 201: new session/host/invite; old session ends               |
| POST `/v1/heartbeat` | bearer + `{}` or `{ready:true}`          | 200: session, participant; pending guest becomes active once |
| POST `/v1/leave`     | bearer + `{}`                            | 200: session, participant; host leave ends session           |
| POST `/v1/end`       | host bearer + `{}`                       | 200: ended session                                           |
| POST `/v1/signals`   | bearer + `{to,kind,payload}`             | 200: `{accepted:true}`                                       |
| GET `/v1/events`     | bearer; `after=0&wait_ms=0` defaults     | 200: session, participant, events, cursor                    |

Create/replace response:

```json
{
  "session": {
    "id": "<32hex>",
    "seed": 305441741,
    "protocol": "galaxy-coop-1",
    "host_id": "<32hex>",
    "expires_at": 0,
    "state": "open",
    "reason": null
  },
  "participant": {
    "id": "<32hex>",
    "role": "host",
    "status": "active",
    "token": "<64hex>"
  },
  "invite": "#<UPPERCASE encrypted hex>",
  "cursor": 0
}
```

`expires_at` is the service's absolute Unix millisecond deadline (the example's zero is a placeholder). Join has the same session and participant objects with guest/pending state, and omits `invite`. Other responses never return bearer tokens. Events are `{seq,type,...}`: `peer-joined`/`peer-ready` contain `peer:{id,role,status}`, `peer-left` contains `peer_id,reason`, `participant-left` and `session-ended` contain `reason`, and `signal` contains `from,kind,payload`.

### Signals and event reads

Only host↔guest traffic in the same session is allowed; self, guest↔guest and cross-session addressing fail with 403. Host sends `offer`; guest sends `answer` only after an accepted host offer. Both may send `ice`. The immutable seed/protocol cannot be modified by a guest.

SDP payload is exactly `{sdp:string}` with 1–16,384 UTF-8 bytes. ICE is `null` for end-of-candidates or `{candidate:string,sdpMid?,sdpMLineIndex?,usernameFragment?}`: candidate ≤2,048 UTF-8 bytes, optional string fields ≤64 UTF-16 code units, optional line index integer 0–32; optional fields may be null. These bounds validate transport structure, not browser SDP correctness. The service does not interpret SDP, connect to candidate addresses, or log these payloads.

Events use a session-global monotonic sequence. A participant sees only its addressed messages/notices, so gaps are normal. Read with the last successfully processed response `cursor` as the next `after`. Repeating the same `after` replays retained events; advancing it explicitly acknowledges/removes older events. A live participant requesting an already acknowledged older cursor receives 409 `cursor_expired`. A future cursor fails with 400. Clients must process events before advancing; responses are not an exactly-once delivery guarantee.

At most one outstanding long poll per participant; another returns 409. `wait_ms` is 0–20,000. Timeout returns the current cursor and an empty event list. Disconnect/abort removes its timer and subscription. Heartbeat is explicit; reads and signals do not refresh participant liveness. A guest should become ready promptly, then both roles heartbeat more frequently than the timeouts below.

## Lifecycle and hard bounds

| Bound                                                               | Default                                          |
| ------------------------------------------------------------------- | ------------------------------------------------ |
| Alive participants, including host and pending guests               | 8                                                |
| Pending reservation deadline from join                              | 20 seconds, not extended by an unready heartbeat |
| Host / active guest heartbeat timeout                               | 30 / 45 seconds                                  |
| Invite / absolute session lifetime                                  | 10 / 60 minutes                                  |
| Terminal notice and departed credential retention                   | 60 seconds                                       |
| Sessions including ended tombstones                                 | 64                                               |
| Retained participant records per session                            | 64                                               |
| Open host sessions per salted direct client address                 | 4                                                |
| Body / SDP / ICE candidate bytes                                    | 24,576 / 16,384 / 2,048                          |
| Recipient event queue                                               | 64 events and 262,144 bytes                      |
| Signals per participant / session in a fixed 60-second window       | 60 / 240                                         |
| Create / join attempts globally per fixed 60-second window          | 12 / 120                                         |
| Create / join attempts per direct client per fixed 60-second window | 2 / 20                                           |
| Salted client rate records / connected sockets                      | 256 / 128                                        |

Invalid create/join attempts count against admission limits. Authenticated replacement attempts also consume the global create bucket and the original host client's create bucket before their body is parsed; new bearer credentials do not reset that rate. Invalid replacement metadata consumes an attempt while leaving the old session open. Fixed-window rate limits are not a distributed anti-abuse system. Forwarded headers are ignored; behind a proxy, clients would share the proxy's direct-address quota until a separately reviewed deployment policy exists.

Full queues return 429 without silently accepting/dropping the signal. Clients acknowledge promptly and may retry refused transport messages. If a host's saturated queue cannot admit a mandatory membership notice, the whole session ends with `signaling_overflow`. Terminal notices replace queued signaling traffic, stay bounded, and are prepared for every participant before polling readers are notified. Host leave/end/timeout, absolute expiry, replacement, and service shutdown all notify guests. Expired reservations and departed guests free an active seat immediately but their bounded notice/credential record remains temporarily. After retention, credentials are removed and reads return 401. Failed replacement validation or quota checks leave the old session open.

Invites encrypt only version, session ID, invite ID and expiry using AES-256-GCM with a random 12-byte nonce, 16-byte authentication tag and fixed versioned AAD. The hex envelope starts with version byte 1 and is emitted as `#` plus uppercase hex. Copy/paste accepts an optional `#`, either case and surrounding whitespace; the canonical hex must have an even length of 64–1,024 characters, with raw input at most 1,100 characters. This is a long authenticated capability, not a nine-character short-code service. The 32-byte key is generated in memory per `SessionStore`/service start. No IP/profile/seed is embedded in the invite; restart invalidates both invites and participant credentials. Participant bearer secrets are stored as SHA-256 digests; direct-address rate keys use a per-process salted digest. Invite and bearer capabilities must still be treated as secrets by clients. The CLI logs only startup address or a fixed listener failure, never request bodies, tokens, SDP or ICE.

`makeCoopServer()` is an importable test/integration seam. `server.coop.stop()` ends sessions, rejects subsequent mutations, closes sockets and clears maintenance/poll timers. `SessionStore` is synchronous and timer-free; its injected clock is monotonic. Limit overrides may narrow documented bounds only, with at least 256 queue bytes reserved for terminal notices.

## Executed evidence

```powershell
node --test scripts/test/coop-signalling.test.mjs
```

The focused suite uses actual Node HTTP listeners on `127.0.0.1:0` and fetch/chunked requests, with deterministic injected time for long expiries. Every listener is stopped in cleanup, and subscriptions/polls are asserted empty. Synthetic signaling fixtures prove admission, capabilities, authorization, lifecycle, queue/replay and cleanup behavior; they do not prove RTC negotiation, NAT traversal, data channels, host gameplay authority, browser integration or playable co-op.
