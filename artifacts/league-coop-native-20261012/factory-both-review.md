# Independent retained factory RTC evidence review

Reviewed 13 immutable reports from the corrected stage authored at `3963f4ba0a5534f42712e67514edba8d182a6cd5`: six WebGPU and seven WebGL2. Root performed the actual browser actions. This reviewer read the captured observations and checked their byte identity and provenance; no browser, Game, listener or GPU run was performed for this review.

`verify-captures.mjs` passed: all 18 staged files match the frozen manifest, both four-file normal WASM bundle hashes match their addressed paths, all 13 report filenames match raw SHA256, and each report matches the stage's source-input/backend/artifact metadata. Strict version-2 report validation passed. Every recorded check is true and every client failure marker is null. The script retained byte-identical report copies and a finite summary in `validation.json`.

| Actual run | Reports | Drive/fire in this run | Complete guest commits at disposal | Maximum shared projectiles |
| --- | ---: | --- | ---: | ---: |
| WebGPU `95d6f45c84179f49` | 6 | No | 428 | 0 |
| WebGL2 `deb1cc150573cc68` | 7 | Yes, 360 explicit native steps | 512 | 16 |

Both runs record real channel readiness separately from native admission and complete native replica commit. Host end, a distinct new session, guest leave and final disposal passed. At final disposal each client had created two PCs and closed two PCs, with zero transport requests/timers. Native Profile and actual origin-persisted SaveStore digests both equal their initial baseline for each client. Independent origin persistence was recorded. Root separately reports the owned listener stopped and the regular signaling service restored; this review did not operate those processes.

The earlier WebGPU run `a1f7e28099cd446d` did drive/fire and is preserved in the separate initial review; it is not silently attributed to the corrected WebGPU run above. Its failed rejoin and successful later disposal remain historical evidence. This review does not overwrite those failures.

Each corrected run has one same-count signature-change observation, first appearing at rejoin. The historical client retained comparison rows across sessions, where ECS IDs can be reused for different objects. This does **not** establish same-count body destruction. The new owned harness resets that comparison baseline on session changes. Ordinary GL2 play did record body-signature changes before rejoin, but the aggregate counters do not identify a specific damaged cell or prove each pilot's shot ownership.

Scope accepted: the retained reports substantiate the tested two-client local factory loopback, complete native commits, the stated drive/fire run, distinct-session lifecycle and local save restoration. They do not establish owned-loadout behavior, fifth-shot parity, eight simultaneous clients, remote networks/NAT/TURN, mobile/background behavior, throughput/latency/FPS, subjective rendering quality, advanced abilities, shared rewards or playable co-op release acceptance.

Next bounded checks: a coherent owned-loadout source/binary stage; an actual native equal-count color/topology mutation transmitted over RTC and atomically committed; separately observable pilot shot ownership and fifth-shot behavior. Existing native signature/replica tests cover equal-count invalidation, but that is separate from physical RTC evidence. Any controlled mutation must be an explicit native QA scene/export, with no manufactured JavaScript native replies or progression grants.

The first reviewer-script invocation had a wrong relative import and did not inspect reports; its error log is retained as `validation.log`. The corrected invocation passed in `validation-final.log`. A delegated local worker returned only an acknowledgement and no substantive review; it is excluded from acceptance and test counts.
