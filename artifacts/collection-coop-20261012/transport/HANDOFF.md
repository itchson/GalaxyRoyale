# WebRTC transport source freeze

Candidate `4f89b53d86711699acb6f912fb3d64f7720aac3c`, parent `5aa5cfaeb4db4e71511ac4dc251ce17d1b423923`, branch `codex/co-op-transport`. Nine owned files; tracked checkout clean. No Rust, manifests, shell integration, game build, renderer, Profile or save writes. Existing `apps/coop` service is unchanged.

## Executed evidence

Node24.18.0, installed lockfile TypeScript5.9.3, Windows. `node --experimental-transform-types --test scripts/test/coop-transport*.test.mjs` passes **26/26** (24 transport/framing cases,2 QA helper cases). `pnpm exec tsc --noEmit` and `git diff --check` pass. All actual ephemeral HTTP test listeners/polls/clients close; injected deterministic RTC APIs are expressly lifecycle models, not native RTC acceptance. No full game build or full shell suite executed by this task.

Retained first/second/third failing logs are genuine: delayed creation reopened the error path after cancellation; queued retired-peer ICE could end unrelated host membership; expiry raced a queued signal request and hid its terminal status. The fixes have focused regressions. The final test log is `final-test.log`; earlier logs stay historical. The local Spark wrapper executed but returned only an intent to read, with no substantive review. No Spark review is claimed.

`check-native.mjs` read the peer's actual Game API exports without modifying them. `native-compatibility.json` pins all five files: snapshot8 pilots/180 projectiles,64 entity rows,1024 packed cells,actual input and8 admissions. Exact current validators accept them; framed sizes44,924/20,818/18,373/198 bytes respectively. Revision and dropped_notices retained. Numeric JSON values survive decode unchanged; JS JSON formatting may shorten Rust's float spelling.

## Integration contract

`CoopTransport({service,rtcConfiguration?,onChange?,onFrame(peerId,frame)?,onWritable?})` imports no storage/Game. APIs: `create(nonzeroUint32Seed)`, `join(invite)`, explicit secret `invite()` copy getter, `send(peerId,kind,payload):boolean`, `markSimulation(peerId,running)`, `leave()`,host-only `end()`, `dispose()`, sanitized `diagnostics()`.

Diagnostics distinguish session/signaling phase,actual channel transport readiness,explicit native admission and simulation waiting/running. Ready requires opened ordered reliable DataChannel **and** expected service-peer hello. It does not admit a Rust actor. `markSimulation` is a caller declaration and must follow actual authoritative admission/application, never a button merely opening a channel. The QA page intentionally leaves it waiting.

Wire `{v:1,session:32lowerhex,seq:uint32>0,kind,payload}`; channel `galaxy-coop-1`. `onFrame` sender is bound to authenticated service peer and expected offer direction; payload has no sender claim. Pass frame.session to ALL native expected-session parameters; for guests call submit with this sender ID,then payload epoch/seq/input. Host submit is separate. Rejoin must obtain a new service ID/admission epoch. Strict per-channel ordered sequence disallows replay/gaps/wrap.

Join spec is **only** `{local_seed:nonzeroUint32,identity:KikaiIdentity|null}`. Identity exact enums dora/doro/talo + main/sub pan/renko/hugo + lowercase16hex. Never accept remote Profile,seed mutation,save or arbitrary cells/damage as authoritative. Host frames may transmit native snapshots/pages to guests; there is no API here that imports them into local saves.

Input payload `{epoch,seq,input}`. Snapshot exact native `{protocol,session,world_seed,tick,revision,progression:false,dropped_notices,participants,projectiles}`. Entity/body responses retain `{session,tick,revision,...}`; query `{kind:'entities'|'body',expected_revision,offset,id?}`. Same-tick join/leave changes revision. Native pages must reject stale expected revision/session; integration must discard pages no longer matching its applied snapshot revision. The transport validates shape/session but cannot prove native revision freshness or guest rendering.

Host-only admission payload `{accepted:true,slot:1..7,epoch}` or `{accepted:false,reason?}`. Rejection frees guest signaling membership. Admission is fixture-only on this tool; regular gameplay must source it from real native join result. Host-only snapshot/body/notice,guest-only join/input/body-query; hello both.

## Bounds and lifecycle

Host max7 peers,guest1. Frame65,536 UTF8 bytes,buffer262,144 bytes,low threshold65,536;SCTP maxMessageSize honored. Send120 frames/s;receive120 frames/s and2MiB/s. Refused backpressure/rate sends do not queue or advance wire sequence. Caller should replace stale input with latest input when writable. JSON finite/depth12/nodes16,384/array1024/string16,384. Snapshot8/180;pages64/1024. Plain records only; explicit Profile/save keys rejected. These are transport sanity caps,not authority.

ICE max16 events including end marker per peer/direction;local signal queue20;SDP16KiB/ICE2KiB. Service's independent60 signals/min per participant,240/session can reject excessive multi-interface candidate bursts; this client never bypasses service limits. There is no ICE restart/autoreconnection. Negotiation15s,disconnection grace5s;heartbeat5s with3s timeout and3 network failure bound;event longpoll20s with25s request deadline and finite retry. Exact fixed service URL,no redirects/cookies/arbitrary forwarding. Terminal teardown immediately aborts controllers,clears clocks,closes and detaches all RTC callbacks,erases private capabilities; best-effort leave/end gets1.5s. Lost notices remain bounded by service TTL.

No default public STUN/TURN or media tracks. Configure ICE servers explicitly for later deployed environments. Public HTTPS/signaling deployment,TURN/NAT/separate networks,mobile/hidden browser lifecycle and playable shared-world rendering remain untested. Source RTC channel behavior follows [W3C WebRTC](https://www.w3.org/TR/webrtc/); buffered backpressure uses the actual [RTCDataChannel.bufferedAmount](https://developer.mozilla.org/en-US/docs/Web/API/RTCDataChannel/bufferedAmount). ICE is held until remote description before [addIceCandidate](https://developer.mozilla.org/en-US/docs/Web/API/RTCPeerConnection/addIceCandidate).

## Root-owned native browser loopback

Tracked dev-only helper is `scripts/qa/co-op-transport`; excluded from game imports/build. Prepare compiles three actual current TS modules with installed5.9.3,records source/output SHA256 and copies visible page. Serving rejects changed source/outputs and exposes only allowlisted paths. No historical/snapshot dependency.

From this checkout (or re-prepare in integrated root):

```powershell
node scripts/qa/co-op-transport/prepare.mjs --stage work/co-op-transport/stage
node scripts/qa/co-op-transport/server.mjs --stage work/co-op-transport/stage --port 4189 --max-ms 1200000
```

Launch only after Root ownership check. Listener127.0.0.1:4189,one actual service/page process; occupied port fails,no process killed. Automatic stop20minutes,SIGINT/SIGTERM close signaling/socket/polls. If backgrounded use Start-Process -WindowStyle Hidden. No listener is currently left running by this task.

Root CUA controls at `http://127.0.0.1:4189`: **Connect**,**Send join/input fixture**,**Host end**,**Rejoin new session**,**Dispose**. Two actual browser RTC clients on the same page;no replacement RTC APIs. Admission message is declared framing fixture,simulation waiting. Host end/rejoin checks real peer closure,new session,pending re-admission. Visible sanitized report +immutable per-action JSON under stage/captures;private invite/token/SDP/ICE never in DOM/artifacts. Capture schema admits only declared check names/booleans/bounded counts. Helper HTTP test's supplied report is metadata validation,not RTC execution.

Native browser RTC execution is **Pending** until Root actually runs these controls. One-page loopback proves no NAT/separate-device/gameplay claims. No human hearing/visual-gameplay claim applies.
