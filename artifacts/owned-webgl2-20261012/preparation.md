# Owned RTC stage: 6964645

Prepared using the existing tracked-clean helper at b04860f6f9c47794dca822d211f5bcb8788708e4, branch codex/co-op-wasm-qa. No helper or production source changed. Preparation exited 0; this invokes lockfile-pinned TypeScript 5.9.3 module transpilation and copies actual normal WASM artifacts, without building the game or running tests.

Root source revision is 69646456ac3e104c94ca27bc79aeb774dd9ab757. Its evidence-only carry-forward stamp was checked against the complete live runtime and package hashes before and after preparation. Root HEAD stayed equal to this revision through preparation and independent verification. Source hash: dfbcd33501214c59dafcf3d4c766b84c20a47f7c79e4567cb05f5c66ab8be3ce. Package hash: 7b166b22eb7dd48bcb05936c6c6f734fdcc5074ea0a42854e01a6be1b9cd27c5. Original normal compilation time remains 2026-10-11T20:32:45.999Z; this task did not recompile the game.

Fresh immutable stage:
C:/Users/mitch/.codex/worktrees/galaxy-audio-refinement/galaxy-royale/work/co-op-wasm-qa/owned-v2/stage-6964645

Stage manifest SHA256: 38773dab964c857d8fc303d48ce7b5bd03e0735fd9e37a305322e24104b2ec2f.

The actual copied backend content hashes independently recompute as:
- WebGPU: 2b920069d8b54532d12135034fb963d70f124f16394b08cc58727eec25f245ac
- WebGL2: 669ab91d8ad1d7f71abef62f379a966a8931a7ea7f5b8a4ea0230bad36b20d9d

Independent read-only verification checked every one of the 18 prepared files, all ten live input pins, six canonical Git adapter/storage sources, and eight copied normal artifacts. It checked exact build-stamp equality, full runtime/package hashes, raw copy equality, bounded regular files, unchanged stage manifest and unchanged own tracked checkout. The b048 foreground/full-iframe visibility guard is copied byte-identically. This is preparation verification, not a test run or native/RTC execution. verification.json retains every hash and the actual checks. Earlier v1/v2 stages were untouched; obsolete c87 manifest remains 267c925d90f8e047f977eeab40adc108776ca013d273e08a6c854f1cdfa5b76c.

## Root-operated launch

From C:/Users/mitch/.codex/worktrees/galaxy-audio-refinement/galaxy-royale:

```powershell
node scripts/qa/co-op-transport/server.mjs --stage work/co-op-wasm-qa/owned-v2/stage-6964645 --port 4189 --max-ms 1200000
```

Root verifies and stops its existing owned 4176 signaling listener first, then launches this one finite QA listener. It performs strict live-source/stamp/Git/package and prepared-file validation at startup. If source, stamp or package changes, stop and preserve the failure; do not bypass the gate. No listener was started by this owner. Open http://localhost:4189, select WebGL2 before Initialize, keep the parent page foreground at 1280 x 900 and both real client iframes fully inside the viewport. Use visible controls for Initialize, Connect, Drive/fire, End, New session, Guest leave and Dispose. Preserve sanitized reports for both native-owned export and complete native-owned replica commit, movement/fire, restoration/rejoin and final cleanup. After Dispose, Root verifies and stops the exact QA listener and restores its ordinary signaling service.

## Acceptance limits

Owned WebGL2 RTC browser execution is pending at this preparation freeze. This stage uses two actual native Galaxies, ordered real RTC channels and independent local-save origins when Root runs it; this owner has not initialized them or produced browser results. The supplied starter owned-loadout scenarios do not cover Pulse Echo evolution or RENKO fifth-shot ownership merely because the compiled runtime supports them. Do not infer same-count destruction from across-session changes, nor Internet/NAT/TURN, eight clients, shared rewards, physical-device behavior or performance acceptance. Local profiles must remain hash-identical after end/leave/rejoin/dispose. Native commit readiness remains distinct from transport readiness. No credentials, SDP or ICE are included in this handoff.
