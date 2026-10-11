# Real two-Galaxy co-op review harness

Frozen helper commit: `e4d73c8a94b9369c507549ea4ceeb66073a63f99`, branch `codex/co-op-wasm-qa`, parent `27e0b403e96689598eb57fb91805d315de1c3e08`. Eight approved helper/test files only. No production modules, shared manifests, normal build outputs or Root source were edited. The prepared controller/ports/transport/wire/signaling/SaveStore are actual clean parent sources compiled with TypeScript 5.9.3. Native responses are forwarded from real Galaxy calls; no JavaScript simulation or manufactured native reply exists in this tool.

## Preserved normal artifacts

`stage-v1-ready` contains both normal bundles copied while Root's old `48bec87d504dc81ecf768fa22525d29cff8d6f37` dist existed. All eight bindgen files were byte-verified, then independently preserved under `pinned/dist`; the ready stage was prepared from that copy. This is a copy of the actual normal build, not a rebuild. Individual lengths and SHA256 values are in the stage manifest and this handoff manifest.

- WebGPU four-file content hash: `9cd54518ba647878a057c2afca1356dcbefb0e00532fedc28269a0ab73a0cfb6`; WASM 10,178,957 bytes, SHA256 `d477922baceaf54fdb09453f4b8af4f7b021c5d631cf6e0c4c9763706ef8ab76`.
- WebGL2 four-file content hash: `0072c7f366f41e6af5a4b85578075c4253e260df7add54226cc41713f68d5dcc`; WASM 13,783,473 bytes, SHA256 `eb884222512996340ff365398e04178499ecadb2111b90d9e2ab454be0b8b1e1`.

This is the v1 factory guest / ordinary primary-shot proof. New owned-loadout or per-pilot ability work is separate. No shared progression is awarded, and no remote Profile is transmitted or saved.

## Executed checks

`focused-freeze.log`: actual seven focused Node tests passed, zero failed/skipped, using the pinned normal bundles. The HTTP case served and hashed actual WASM bytes; checked exact host/origin/CSP/MIME/file gates, actual encrypted signaling create requests, immutable sanitized report capture and corrupt-stage rejection. Its listeners used OS-chosen ephemeral ports and were stopped. Four `node --check` checks and `git diff --check` passed. No game build, browser action or persistent listener was started by this agent.

`focused-first.log` preserves the first 4/5 result: Node Undici ignored a supplied Host header, causing the test to see host rather than guest role. The fixture now uses actual `node:http.request` Host handling; the server policy was retained. The log-print wrapper also encountered Windows cp1252 output limitations; saved test output remained intact. Subsequent raw logs distinguish these boundaries.

The source-function tests run actual authored function bodies against declared boundaries. They prove the frame calls Galaxy.step, pauses solo/guest, zeroes a mode-transition frame, and requires a new native commit after rejoin. They do not execute WASM or RTC. Browser initialization, WebGPU/WebGL2 CSP enforcement, actual channel traffic, native scene commit and IndexedDB equality are pending Root's real browser run.

## Root launch and visible actions

Stop the already verified owned signaling listener on 4176 before starting this single temporary listener. Root owns process launch/stop and CUA browser actions. The helper fails an occupied port, never kills a process, and stops on SIGINT/SIGTERM or deadline.

```powershell
node "C:/Users/mitch/.codex/worktrees/galaxy-audio-refinement/galaxy-royale/scripts/qa/co-op-transport/server.mjs" --stage "C:/Users/mitch/.codex/worktrees/galaxy-audio-refinement/galaxy-royale/work/co-op-wasm-qa/stage-v1-ready" --port 4189 --max-ms 600000
```

Open `http://localhost:4189/`. Select backend, then Initialize → Create + join → Drive + fire → Host end → New session + join → Guest leave → Read local saves → Dispose. The actual client frames use `localhost:4189` and `guest.localhost:4189`; signaling uses `127.0.0.1:4189`. These are two native Games/RTCPeerConnections and two origin-separated actual SaveStores on one browser page and one HTTP listener. Existing valid dedicated QA saves are retained; empty stores receive genuine new unprogressed licenses. No raised-rank or reward fixture is installed.

Connect waits at most 65 seconds for an actual new successful native commit, distinct from transport ready and native admission. Command timeout is 75 seconds. The normal QA presentation clock is finite at 180 seconds from initialization; proceed promptly. Drive + fire performs 360 explicit 1/60 native steps per client, paced by browser timers. This is an actual native input experiment and does not establish real-time FPS. A throttled IAB or timeout remains an actual failure. Reload for the second backend.

Each action stores a strict secret-free report under stage `captures`. Counts, controlled enums, nullable booleans and digests only; tokens/invites/SDP/ICE/Profiles/poses are rejected. Local Profile comparisons use canonical normalized hashes and actual persisted SaveStore reads. Body signature change counters are observations of complete native commits; zero means not observed. Unchanged signature comparisons are not a measured network cache-hit rate.

After Dispose verify every created PC is closed, transport request/timer counts are zero, both local Profiles and persistent save hashes match baseline, and native views were restored before Galaxy.free. Close the page to destroy iframe contexts/idle database connections, then stop the listener and restore the separate owned signaling service. Do not change this preparing checkout's source while Root uses the stage: startup validates source and stage byte pins.

## Remaining acceptance

Root browser execution is not claimed here. Two same-page clients do not establish separate-network/NAT/TURN operation, eight-client load, mobile/background recovery, latency/performance, owned-build effects or playable co-op release readiness. The regular shipped main integration is also outside this harness scope. Human/gameplay acceptance remains separate.
