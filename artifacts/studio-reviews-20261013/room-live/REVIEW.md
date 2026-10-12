# Independent recorded private-room negative live review

Accepted as six completed, cleanly disposed two-client local runs: Decline, Replace Code and Lock/Unlock on WebGPU and WebGL2. A seventh WebGPU Lock attempt failed before peer creation and was also cleanly disposed; its failure remains separate. This review inspected existing actual Root browser evidence and source provenance; it did not operate another browser, listener, native Game or RTC connection.

## Frozen provenance

Prepared stage `work/private-room-negative-integration/live-stage/manifest.json` identifies helper/source commit `e97c29ca262e83dfc46c3a3d4faa10703fa0f64c` and normal binaries compiled at `332ddd5ec0370e13ad346ea4ea988f35b61ca3e7`. The compilation timestamp is retained; carry metadata does not claim a new compilation. I verified all twenty prepared file hashes and all eight bindgen artifacts' hashes/lengths, both content-addressed backend directory hashes and equality with the corresponding existing normal dist artifacts. Both manifests' seven canonical runtime input digests agree; the declared normal-build dependency paths have no Git diff between332 ande97.

Fourteen raw prepared inputs and seventeen canonical source snapshots (including preparation/report/hash helpers) are pinned with raw-versus-LF distinctions. All seven staged JavaScript modules were independently reproduced with the exact TypeScript5.9.3 settings, canonical source SHA comment and import rewrite, matching their stage hashes byte for byte. This pure source check created no stage or bundle and started no service.

All twenty-one captures were independently hash-checked against their immutable `run_id-SHA256.json` filename. Every report's source inputs, normal artifacts, source commit, build stamp and canonical input digests match the stage. The actual exact frozen report validator was applied to the original report portion of each capture, excluding server-added metadata: all twenty-one satisfy the strict allowlist and bounded counts. No private room code, raw participant/session identity, admin credential, token or private request facts occur in that report schema. An initial source-inspection VM omitted the Fifth-reasons constant; its setup was corrected by extracting the exact source constant. This was an inspection setup error, not a live case failure or runtime defect.

## Actual case progression

Each run has three timestamp-ordered immutable captures: Initialize, case outcome and Dispose. Initial captures contain the five passing setup/save/export checks and two ready idle solo clients. Every case returns both clients to ended/solo; every final capture has `dispose-clean:true`, two disposed/solo clients, created PCs equal closed PCs, and zero pending requests, timers, transport-ready peers and admitted peers. Both normalized native and persisted profile hashes equal their per-client baselines throughout, with distinct host/guest profiles and owned exports.

| Backend | Case | Result | Final PC created/closed per client | Guest complete commits | Host native ticks |
| --- | --- | --- | --- | ---: | ---: |
| WebGPU | Decline | passed/disposed |0/0|0|0|
| WebGPU | Replace Code | passed/disposed |1/1|3|100|
| WebGPU | Lock initial attempt | failed/disposed |0/0|0|0|
| WebGPU | Lock/Unlock fresh attempt | passed/disposed |1/1|6|113|
| WebGL2 | Decline | passed/disposed |0/0|0|0|
| WebGL2 | Replace Code | passed/disposed |1/1|5|107|
| WebGL2 | Lock/Unlock | passed/disposed |1/1|6|117|

Decline reports pass pending isolation, retained host policy, exact decline terminal and cleanup, with no PC/native tick/complete-commit activity. Replacement reports pass approved continuity, old-code rejection after actual guest leave, fresh pending-code replacement, retained host and cleanup. Lock reports pass pending cancellation, locked lookup refusal, unlock/rejoin and approved continuity through lock and unlock. Duplicate `room-policy-keeps-host`/`room-lock-approved` check names are deliberate multiple assertions in the reviewed orchestration, not extra distinct test cases. The reviewed v2 source retains original private ownership marks, requires commits beyond the post-policy floor, latches active visibility and refuses recorded native failure/lifetime signals.

The failed run preserves `operation-failed:false` alongside passing cleanup and final disposal. Its screenshot shows generic `room_case_failed`; reports have no underlying service error. The default two creates/minute and chronology make a third-create rate limit a source/timing-consistent hypothesis only. No raw429/rate_limit response is pinned, so the exact underlying cause is unproven. The later unchanged-source fresh run passed; this does not replace the original failure.

## Screenshots and visibility

All seven existing screenshots were visually inspected and raw-hash pinned without modifying originals. Six show the expected completed case status and one shows the failed Lock status. Both full side-by-side native canvases and their ended/ready/solo footer state are visible. They show expected corporate dock scenes under the selected backend. The raw files have `.png` names but contain JPEG1265×988 pixels. Root operated a1280×1000 viewport; that is distinct from the stored raster dimensions. This is filename/capture metadata, not proof of a renderer sizing defect. No image conversion, cropping or rerender was performed by this review.

Continuous visibility is supported by the reviewed active source guard/latch and accepted recorded outcomes, plus the visible screenshots. I did not observe the whole live interval independently, and these screenshots are post-case observations rather than final disposal images; the separate final captures establish disposal. Parent screenshots show later frame counters because the unchanged render loop continued between case capture and screenshot, not additional combat ticks or falsely matched report bytes.

## Limits

This closes the bounded recorded two-client local negative-case gate for the pinned helper and compiled332 binaries. It is not eight-client, Internet/NAT/TURN, physical-device, audio/ear, performance, shipped menu or every-ability acceptance. It does not prove exact boxed ECS/RNG/body preservation or zero save migration writes; normalized profile/persisted digest evidence and the separate native boxed-world regression have different scopes. It does not prove new persistent Ion Wake presentation from111a477, which was not in these binaries, or world-edge replication.

The manifest's prepared `browser execution pending` scope remains an original pre-run record, while immutable reports are the later execution evidence. Root's reported QA listener termination/normal-preview restoration is outside this review's independent process observation; no port or PID was probed and no new instance was launched. Source-only, recorded live and future device/network gates remain distinct.
