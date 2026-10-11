# Co-op integration and evidence boundaries

## Current normal build

The dock's INVITE/JOIN panel is connected to the browser controller and real Rust Game/Replica ports. This is an experimental local co-op foundation, not a completed multiplayer release. One authoritative host can admit seven guests. The host's seeded universe and dock are shared; each client retains separate local progression. Exiting restores the boxed original solo Game. Shared rewards and missions remain disabled.

The controller uses ordered WebRTC DataChannels and ephemeral signaling. Channel opening and validated hello establish transport readiness; native admission and a complete committed publication establish game readiness separately. Session IDs, membership epochs and increasing sequences reject retired traffic. Remote inputs bind to their authenticated participants. Frames remain bounded to 64 KiB, buffered sends to 262,144 bytes, with no unbounded application send backlog.

Publication acquisition pins one native revision and bounded body pages. The guest stages a complete publication and commits it transactionally; an incomplete update does not replace its current native world. Body occupancy and authored transforms are native data. These mechanisms do not establish bandwidth, latency or eight-client performance acceptance.

## Owned equipment and native abilities

The native solo Profile exports a bounded `OwnedLoadout`: selected/owned hull declarations, patch/victory facts, selected-hull XP, six hardware ranks, three equipped ability/rank declarations, starter-training fact and relic count. Full Profile, currency, cargo, collection history and save containers are excluded. This is a self-declared offline equipment claim, not cryptographic proof of earned ownership. Native validation applies registry, hull compatibility and effective level/rank rules; the browser checks the bounded shape.

Current normal native gameplay has per-pilot primary attacks and the RENKO/Pulse Lens fifth-successful-primary extra bolt. The bolt carries participant ID and membership epoch, uses actual collision/destruction paths and respects the 180-projectile cap. Refused primaries do not advance its counter; a refused bonus is consumed rather than replayed. Native parity tests compare HUGO delayed recharge and PAN movement with corresponding solo builds. They do not prove all automatic abilities or audiovisual parity.

Native source now implements ordinary Pulse Echo fans per pilot. Earned evolution is carried by the bounded `echo_evolution` fact: an integer 1–5 attached only to equipped ordinary offensive ability 3; absence means evolution one, and native export omits the token at that default. Browser validation preserves it without inventing boss history. Nine native tests exercise successful-primary cadence, effective rank/evolution, eight staggered pilots, partial-cap admission, collision/debris, owner/epoch lifecycle and native publication/replica behavior. Real legacy/evolved native value fixtures agree across Root and specialist executions; their progression is configured for testing, not naturally earned. Matching rebuilt bundles and actual Echo RTC behavior remain separate acceptance gates. Mixed older clients have not been accepted; use matching tested bundles.

Repulsor Nova, Ion Wake, Royal/signature effects and guest transient ability lights/particles remain separate gaps. Published projectiles and damaged bodies do not establish complete remote effects, audio or progression.

## Actual validation scopes

See `artifacts/league-coop-native-20261012/README.md` and its reports for exact revisions and limitations:

- Factory-loadout browser runs use immutable normal 48bec87 bundles. WebGPU lifecycle and WebGL2 drive/fire, host end, distinct-session rejoin, guest exit and disposal were observed on two actual local Rust/WASM clients. These are not current owned-loadout tests.
- Owned-starter runs use normal b34d02c and helper 15b9e0f. A visible WebGPU retry passed native metadata/complete commits, movement/fire, lifecycle and unchanged native/IndexedDB save hashes. An earlier off-screen Drive timeout remains failed; eventual cleanup passed. Exact scheduling causation was not established.
- The later visibility safeguard refuses hidden/off-screen drive observations. Earlier successful runs do not execute that amended helper. A fresh source-pinned stage is required for a new claim.
- Normal 1b754da passed 805 Rust tests, 546 shell tests and strict native/WASM Clippy. Eight explicit native exporters and one existing Windows case were skipped. Final art captures do not extend the earlier RTC scope.

The source-pinned normal 6964645 run with the amended b048 helper now covers actual owned-starter WebGL2 RTC: channel/native admission, complete owned replica commits, visible movement/fire, host end, distinct-session rejoin, guest leave, unchanged native and independent-origin IndexedDB digests, and disposal with every created peer closed and zero requests/timers. Eight immutable Root per-action reports are retained in `artifacts/owned-webgl2-20261012/`. The guest recorded six body changes but zero same-count changes. The exact native runtime remains d749bc6 across the evidence-only HEAD.

Advanced owned builds, fifth-shot ownership through the browser, equal-count cell mutation over RTC, eight clients, Internet/NAT/TURN, physical phones, background recovery and performance soaks remain open. A native unit test, synthetic boundary fixture or channel-ready event is not a real gameplay run. This starter run does not establish Echo execution or full ability/audio/particle parity.

## Runtime and QA discipline

One normal game preview is retained. Studio and signaling are separate services; the shipped game does not depend on Studio. Temporary native RTC QA replaces regular signaling only for bounded tests. Dispose clients, stop the exact owned process and restore regular signaling afterward. No microphone/camera permissions or relaxed Origin policy are required.

Stage preparation pins source HEAD, runtime/package hashes, compilation time and addressed bundles before and after copying. Changed source or package must fail closed; never bypass the provenance guard. Unchanged compiled packages may be carried forward across evidence-only commits only after rechecking both hashes and preserving compilation time.

Diagnostic WARP retains isolated QA flags and disables progression. Native co-op startup rejects that world even after `WARP 0 0` returns to the dock. The panel can offer an invitation based on position, then display a generic failure after native refusal. Use a fresh unwarped expedition for co-op acceptance. Source inspection supports the retained failure, but its actual native error was not captured.
