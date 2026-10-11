# Co-op stages and boundaries

The invitation/signaling service, browser transport and native authority are implemented independently. They are not yet wired into the shipped browser flight controller. The game still launches a solo expedition.

`game_core::co_op` operates on the actual Game ECS: one host and seven guests, fixed 60 Hz input, participant-owned primary shots, shared destructible bodies and targeted hostile hits. The entire original solo Game is retained and restored on exit. Guest metadata excludes Profile, currency, ranks, XP and inventory. This stage uses validated factory hulls; owned guest hulls/loadouts, automatic abilities and shared rewards remain required work. Local seeds never replace the host world seed.

`apps/game/src/coop` opens real ordered WebRTC DataChannels through the ephemeral signaling service. Transport readiness requires an opened channel and validated hello. Admission and simulation readiness are separate. Frames are versioned and bounded to 64 KiB; buffered sends have a 262144-byte ceiling with no application backlog. Session IDs, membership epochs and strictly increasing sequences reject retired traffic. Authenticated channel IDs must be mapped to the native participant API by the future adapter.

Actual same-page browser loopback connected two real RTCPeerConnection clients, delivered a declared join/admission/input fixture, ended, rejoined a distinct session and disposed all four created connections with zero tracked timers/requests. It did not call Game, render remote pilots or exercise separate machines/NAT/TURN. Earlier Origin and native-open failures are retained in the evidence. During host-end the guest observed a native SCTP close error; cleanup succeeded, but friendly termination wording remains a presentation task.

## Next integration gate

Publish coherent snapshots/body pages from one exact native revision. A 60 Hz tick invalidates an older page revision; a remote query cannot simply fetch arbitrary pages from the continuously advancing live Game. Cache one bounded publication set or introduce an explicit acquisition phase, then define update cadence/backpressure within the existing limits. The exercised full snapshot is approximately 45 KB for eight participants and 180 shots; this is a finite fixture, not a bandwidth/performance guarantee. Avoid transmitting the full world or Profile each frame.

Bind the WASM API, map local/remote inputs through authenticated membership, render interpolated replicas with damage holes and local camera/input targeting, and prove two actual browser game clients share destruction. Keep all collected progression transactions disabled until a reviewed reward/ownership policy exists. Test host join/title/quit/timeout restoration, stale callbacks, guest death, reconnect, CPU/memory, congestion and remote networks. Do not increase existing gameplay caps to hide a failure.

Temporary RTC QA uses one localhost listener, separate exact page/service origins and no microphone/camera permissions. It does not relax the signaling service's Origin policy and is excluded from the game package. The persistent game preview remains singular.
