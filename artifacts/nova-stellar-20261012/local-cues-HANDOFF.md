# Co-op host-local cue routing repair

Frozen candidate `27262a42302506e3724246dbb237a15e73b996c9`, parent `ab71442b9540c3f33c61a418d445323a1b9ccc1e`, branch `codex/co-op-local-cue-routing`. Three owned files: `co_op.rs`, `co_op_tests.rs`, new `co_op_local_cue_tests.rs`. Tracked checkout is clean. No Root edits, wire/ABI/Profile changes, listener, browser, WASM/package build or push.

## Actual defect and repair

The old authority emitted `death` for every participant respawn. Host `Galaxy.step()` drains those cues, and main calls `audio.play()` then unconditionally `showDeath()` for `death`. That handler pauses the shared Game, disables host input and opens the host death screen. The native guest-death fixture already proved that a surviving host received one death cue. Likewise every guest impact emitted `hit` intensity .9; the current AudioEngine treats any hit >=.8 as its stronger personal `player-hit` sound, without spatial attenuation.

The candidate keeps slot-0 host `death` and host impact .9. Remote respawns emit the existing world `explosion` at intensity1. Its light mapping matches the existing death mapping, and the 40-particle burst/respawn/body repair/shot cleanup remain unchanged. Remote impacts retain the exact existing light and damage, while only their newly admitted cue is reclassified to ordinary impact .25. The pre-event queue index is checked with `get_mut`, so an already-full 128-cue queue cannot mutate a previous cue. Shield absorption cues remain truthful and unchanged.

This repairs host-local routing; it does not provide guest-owned death UI, spatial remote audio, replicated transient FX or shared rewards. Native guest respawn remains automatic. Host death still follows the existing local death-screen policy. The earlier independent future-FX proposal is preserved separately, not implemented here.

## Actual evidence

`baseline-failure.log`: new real authority collision/simultaneous tests against unchanged ab71442 runtime produced **1 pass/3 failures**. Guest death entered the death cue, guest hit was .9 rather than .25, and eight simultaneous respawns emitted eight local death cues. These are concrete runtime cue defects, not mock UI/browser failures. The host-positive test passed before the fix.

After repair, four collision/cap tests passed (`focused-candidate.log`). An additional queue-overflow test and the final formatted source then passed:

```powershell
cargo test --locked -p game_core local_cue_tests -- --nocapture
cargo test --locked -p game_core co_op -- --nocapture
cargo clippy --locked -p game_core --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Final results: **5 new focused passes**, **83 co-op passes/2 existing fixture exporters ignored**, strict core all-target Clippy and formatting/diff checks exit0. Full co-op scope includes current Echo/Nova, publication, replica, passive and restoration checks. Its existing CPU capture prints are incidental existing tests, not a new performance audit.

The tests use real swept hostile Bullet collisions to prove guest damage/respawn, unchanged host HUD/vitals/save, remote ordinary impact and normal host death/personal hit. Eight same-tick deaths prove one local death/seven world explosions, repaired participant hull/shield, no repeated next-tick cue, and unchanged 1,100-particle/32-light capacities. A full native cue queue proves no earlier local hit is reclassified. No browser death-screen run was performed; Root must integrate/rebuild before normal bundle acceptance.

`manifest.json` pins actual raw working bytes, Git LF blobs, unchanged consumer/native dependencies and retained logs. Formatter/diff success had no output; `check-observations.json` records observed exit codes rather than inventing a log. `verify.py` is read only.
