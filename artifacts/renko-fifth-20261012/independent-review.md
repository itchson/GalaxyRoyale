# Independent actual RENKO loopback artifact review

Root performed the browser run on 2026-10-12 (Brisbane). QA independently read the eight saved raw captures under `artifacts/renko-fifth-20261012`, recomputed their complete byte hashes and filename hashes, checked their strict version-four report schemas, compared all provenance to the previously verified immutable stage, inspected the actual native/helper consumers, and visually inspected the saved cadence and disposed screenshots. QA did not run another browser, native Game, RTC session, listener, build or cleanup operation.

Source: `650a546b8f8c07150d4df27f8de7cb1d4e13d4e8`. Runtime source hash: `84bdd7de9392ed0aa48157fbba5658c874039dc260098274d1762fede0eb849a`; package hash: `c57689500c7acfccc1d70c401a9c572343505d16e1a952fe041a684284edd96f`; compilation: `2026-10-11T22:20:11.266Z`. Stage manifest SHA256: `d4f77f50bb1ec93dcd142b208e4904f136802b7e609e7ed396e8a47ab78e7332`. Every capture carries those same source/build/artifact/input pins. The saved stage manifest exactly matches the independently verified nineteen-file stage.

## Findings

No blocker found for the bounded **RENKO host first-session cadence on WebGL2, two-client same-machine loopback, and the tested lifecycle/save-restoration path**. The independently executed artifact checker passed; its actual output is `check.log`, with raw data pins and operation boundaries in `verification.json`.

| Saved action | Cumulative checks | Guest complete commits | Host/guest observed ordinals |
| --- | ---: | ---: | --- |
| Initialize | 5 | 0 | 0 / 0 |
| Create + join | 9 | 1 | 0 / 0 |
| RENKO ten primaries | 11 | 417 | 10 / 10 |
| Host end | 13 | 896 | 10 / 10 |
| Distinct new session + join | 18 | 899 | 10 / 10 |
| Guest leave | 20 | 1,196 | 10 / 10 |
| Read durable saves | 21 | 1,196 | 10 / 10 |
| Dispose | 22 | 1,196 | 10 / 10 |

The final twenty-two checks contain fourteen unique check names; repeated saves/channel/admission checks are separate operation observations, not fourteen or twenty-two independent feature scenarios. All raw check lists retain the previous list as an exact prefix, all entries pass, and capture timestamps increase. The saved summary's final object exactly equals the final raw capture. The checker did not derive a pass by trusting only the summary.

Both observers have identical ten-row cadence evidence. Ordinals 1–4, 6–9 contain one new primary projectile. Ordinals 5 and 10 each contain two, with matching owner and membership epoch, damage ratio `1.5000000328400214`, and velocity difference `70.00000000000037`, within the helper's actual 0.0001 tolerance of native ×1.5 and +70. Ordinal six is a single primary, so it supplies the requested no-replay observation after the fifth bonus. Host actual admitted-primary count is ten; the silent HUGO guest's primary count is zero. The guest's completed observer consumes the native replica's exact committed session/tick/revision for each held host capture; it does not simply trust an RTC-ready flag.

The source supports this interpretation: native `tick_co_op` advances Pulse Lens phase only after primary bullet admission, gives primaries priority before bonuses at the existing shared bullet cap, and emits its real primary `shot` cue. The helper pauses actual native simulation after that accepted cue, waits for native capture, then lets the guest complete all bodies and compare actual new projectile metadata. Fresh native `register_droid` supplies RENKO R1 Pulse Lens and HUGO R1 shield; retained SaveStore profiles are refused before creating the scenario. No fixture native response, injected Profile, forced phase, or diagnostic warp is part of the observed helper path.

The saved cadence screenshot visibly shows two whole native client canvases and the WebGL2/Fresh RENKO selection. Host has left the protected dock; guest remains at the HUGO hub. The disposed screenshot shows both clients restored to their respective corporation homes and both labels disposed/solo. These stills corroborate the bounded rendered scene and final UI status; they cannot independently prove the timing of each individual firing event or continuous visibility throughout the run.

Every raw capture preserves each client's own native profile hash and persisted hash equal to its baseline. The two baseline hashes remain distinct. The final clients are disposed, solo, created two peer connections each and closed two each, pending requests zero, tracked transport timers zero, admitted/ready peers zero, and progression disabled. Actual helper disposal awaits controller restoration, clears its scheduled loop, and calls `Galaxy.free()`. The saved reports substantiate the tested client cleanup. Root separately reported verifying/stopping its exact temporary listener and restoring ordinary signaling; QA did not independently re-execute that process operation.

## Limits

The cadence rows persist after the first session. A distinct second session is explicitly observed, with new admission/complete owned commit and leave/restoration, but no second-session firing pass occurred. Neither accumulated counters nor the final retained cadence rows prove repeated cadence after rejoin.

This is a controlled native hold-and-transfer proof on WebGL2 using two actual clients and loopback RTC. It does not prove real-time firing latency, WebGPU cadence, guest RENKO cadence, every ability, advanced Nova/Echo visuals, same-count body destruction (the saved count is zero), shared rewards/currency, Internet/NAT/TURN, eight simultaneous clients, physical-phone performance, or a 30-minute soak. Store hashes prove no mutation during this transient scenario; they do not establish every multiplayer progression flow.
