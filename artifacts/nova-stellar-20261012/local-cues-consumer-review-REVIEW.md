# Exact role-aware cue candidate amendment

Read-only source review of `27262a42302506e3724246dbb237a15e73b996c9`, parent ab71442. No scoped consumer-routing blocker found. The earlier pre-fix/proposal review is retained unchanged; no tests, native/browser death actions, listener, build or stage were run by this reviewer.

Final respawn routing uses death1 for host admission slot zero and existing explosion1 for all nonhost slots. This avoids main.ts showDeath/finishDeath and browser.rs death-only vitals reset for a remote pilot, while preserving the common death/explosion light branch and body burst. Host-local behavior remains deliberately unchanged.

The final hit implementation refines the draft: it calls event(hit,.9) first, then changes only the newly admitted nonhost event record to .25. Therefore the existing impact light remains intensity1.0 exactly. The earlier proposal's possible .675 light-strength implication is not the final behavior. Queue index is captured before event insertion: if the 128-event queue is full, get_mut(128) returns None, so an older host-local hit cannot be reclassified. There is no intervening event insertion or asynchronous callback in this Rust path.

AudioEngine already maps hit>=.8 to player-hit and .25 to ordinary hit; explosion is supported and never invokes the shell death lifecycle. No consumer/event ABI/network changes are needed for this repair. Shield/world cues remain unchanged. Guest Replica steps still return no events, so guest-own death/hit UI/audio and remote spatial attenuation remain separate missing scopes.

New native test source uses actual spawned high-speed enemy projectiles and Game ticks for guest death, guest shield hit and host positive routing, checking native vitals, respawn placement and retained Profile. A separate deliberately arranged eight-dead-pilot case checks one local death/seven world explosions and existing particle/light bounds; it is not eight naturally occurring browser deaths. The full-queue case preserves existing event bytes while actual guest damage still resolves. Engine reports the retained old-source1pass/3fail, final5focused/83co-op passes and strict Clippy; those are Engine-owned execution evidence, not reruns by this review.

Fresh stage remains blocked until Root integrates this candidate, completes its normal build and supplies the exact coherent HEAD/stamp. Actual main-app guest death/continued host clock/audio acceptance has not been demonstrated here.
