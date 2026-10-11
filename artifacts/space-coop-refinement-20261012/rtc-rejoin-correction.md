# QA rejoin orchestration repair

Additive commit `3963f4ba0a5534f42712e67514edba8d182a6cd5`, parent `48207b861f9d6170badb2fb918cae1ff3cfc91bc`. Three approved helper/test/docs files only, tracked checkout clean. No production controller, native source, WASM bundle or save changed. Failed initialization and failed rejoin source/stages/captures remain preserved.

Root's actual first WebGPU run established initialization, real RTC/native admission, drive/publication and host-end restoration. Its subsequent New session action failed before creating a new PC: the helper unconditionally called host `end` again, while actual `CoopController.end` rejects `host_only` when the session is already ended. The repaired authored `restartSession` leaves both controllers through the actual idempotent leave path, then invokes fresh connection. Distinct service session and a new actual guest native commit remain mandatory.

Ten focused checks passed with zero skips, including the actual normal-artifact HTTP case. The new regression executes the authored restart function and actual CoopController lifecycle methods against an explicitly idle transport boundary: prior leave/end produces the real `host_only` rejection; repeated leave succeeds and both controllers are solo before fresh connect. No manufactured native reply, JS simulation, browser or RTC execution is attributed to that test. Existing new-commit, initialization, bounded deadline, report/privacy and pause-transition regressions remain. Syntax and diff checks passed. Test listeners were ephemeral and stopped.

Root subsequently clicked Dispose in the old actual WebGPU run. The retained report `a1f7e28099cd446d-1de8a92e…json` was independently read and validated here: both clients disposed, each 1 PC created/1 closed, zero transport requests/timers, fresh native/persisted Profile hashes equal to baselines and `dispose-clean:true`. It retains the earlier failed rejoin check; cleanup does not turn that failed action into acceptance. The source/artifact pins match the old immutable stage. Root reports verifying/stopping its exact QA listener; this agent did not control the process or browser.

## Fresh immutable stage

`stage-v1-rejoin-fixed` contains the same eight pinned normal48 bindgen files, with freshly compiled baseline27 controller/ports/transport/wire/signaling/SaveStore and corrected QA pages. No listener has been launched by this agent. Root owns start/stop and CUA actions:

```powershell
node "C:/Users/mitch/.codex/worktrees/galaxy-audio-refinement/galaxy-royale/scripts/qa/co-op-transport/server.mjs" --stage "C:/Users/mitch/.codex/worktrees/galaxy-audio-refinement/galaxy-royale/work/co-op-wasm-qa/rejoin-amendment/stage-v1-rejoin-fixed" --port 4189 --max-ms 600000
```

Same visible action sequence: Initialize → Create + join → Drive + fire → Host end → New session + join → Guest leave → Read local saves → Dispose. Presentation remains a finite three-minute clock from initialization; reload for a backend or a new timed run. Service uses one listener and exact parent/guest/service origins. Keep source frozen during use. Stop any separately owned signaling service before starting this QA service; close the page and stop QA after confirmed cleanup.

Actual browser rejoin, guest-leave and WebGL2 acceptance are still pending. Prior actual local WebGPU connect/drive/end/dispose evidence is retained separately. Same-page two-client loopback does not establish Internet/NAT/TURN, eight clients, owned equipment, special abilities, shared progression, mobile/background or performance acceptance.
