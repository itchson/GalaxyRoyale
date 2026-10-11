# Owned-loadout RTC harness amendment

Frozen helper/test source: `15b9e0f018d427c0c6594af27385a07c64b54ff7`, seven owned files only, atop corrected factory harness `3963f4b`. The tracked worktree is clean. No production app/native code, manifests, build outputs, root records or live services were edited.

The new explicit `--owned-loadout --source-root --source-commit` preparation mode uses exact Git adapter/storage bytes, live equivalents and the actual complete runtime/package hashes from the normal build record. It checks before and after copying; server startup repeats the guard. Source and binary mismatches fail visibly. Version-2 factory manifests/reports remain supported; owned manifests/reports are version 3 and cannot be exchanged.

The actual native `readOwnedLoadout` export accompanies local seed/identity when the controller starts. Profile/currency remains local. Native-export checks and guest matching equipment after successful replica commit are separate visible checks. Channel readiness alone is insufficient. Host authoritative build metadata is not confused with the guest's optional snapshot declaration. Body-comparison state resets on a new service session to avoid rejoin entity-ID reuse masquerading as equal-count damage.

Executed checks: 15/15 focused tests, zero skips, including legacy transport QA, the retained factory normal bundles, and source-pinned current owned normal bundle HTTP/provenance preparation. Tests execute actual staged adapter code against declared response boundaries; they do not execute native WASM/RTC in Node. Every ephemeral listener is closed in test cleanup. `focused-first.log` and `focused-final.log` preserve actual outputs. Syntax checks and `git diff --check` passed. No full game build or browser was run by this agent.

Fresh immutable stage:

`C:/Users/mitch/.codex/worktrees/galaxy-audio-refinement/galaxy-royale/work/co-op-wasm-qa/owned-v2/stage-b34d02c`

It pins Root's exact `b34d02c` final normal build, actual full commit/build stamp and both full-SHA normal backend bundles in its manifest. It does not use the earlier b263 binaries or overwrite any factory stage/capture. Detailed pins are in `manifest.json` alongside this handoff and the stage manifest.

Root startup, from this isolated worktree after stopping the owned regular signaling service:

```powershell
node scripts/qa/co-op-transport/server.mjs --stage work/co-op-wasm-qa/owned-v2/stage-b34d02c --port 4189 --max-ms 600000
```

Root then opens `http://localhost:4189/`, selects a backend, and uses Initialize, Create + join, Drive + fire, Host end, New session, Guest leave, Read local saves, Dispose. Initialize adds `native-owned-export`; connection adds `native-owned-commit`. Reports show controlled export/commit flags, actual selected craft and slot counts, and declaration digests. Existing valid dedicated QA saves remain intact; empty stores receive actual unprogressed licensed profiles. No boosted profile/admission or remote Profile is fabricated.

After Dispose, inspect unchanged native/persisted profile digests, all PCs closed, zero transport requests/timers; close the tab and stop the exact owned listener. Restore the regular service separately. Do not mutate the stage or its source while active. Startup fails if the retained normal runtime source/package changes; prepare another fresh stage after a new normal build.

Pending physical acceptance: this new owned mode has not yet been driven in a browser. A fresh starter export covers only its actual equipped starter, not every advanced hull/rank. The current native fifth-shot addition is present in the pinned build, but this report makes no browser cadence/effect acceptance claim. Real equal-count body mutation over RTC, individual pilot ownership, eight clients, remote networks/NAT/TURN, mobile/background, performance and shared progression remain separate gates. Factory two-backend evidence is independently reviewed under `../browser-review/factory-both-backends/REVIEW.md`.
