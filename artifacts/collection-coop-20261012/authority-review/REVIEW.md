# Independent final co-op authority review

**Verdict: no scoped source blocker found at `60934969913ad1e49a3ce4b2b63af514d890518b`.** Reviewed 2026-10-12. The engine checkout was clean at that exact commit. The seven changed Rust files and three Cargo dependency inputs are pinned in `source-pins.json`, with raw checkout hashes and Git-blob hashes; CRLF-normalized checkout bytes must equal each committed blob. Earlier `DRAFT_REVIEW.md` remains a historical source-review checkpoint.

## Actual independent execution

Ran `cargo test -p game_core co_op_tests --locked -- --nocapture` in `C:/Users/mitch/.codex/worktrees/galaxy-hud-navigation/galaxy-royale`. Exit 0: **17 passed, 0 failed, 1 explicit serialization exporter ignored**. Output is retained in `focused-native.log`. I did not run the exporter, regenerate peer evidence, run a full build, start a server or exercise network/browser co-op.

The focused tests execute the actual `Game` ECS. They cover two/eight participants moving independently; factory guest identity/stats and host world seed separation; stale session/membership/sequence and invalid input rejection; host course cancellation with continued momentum decay; real admitted shot ownership and member-shot removal; hostile swept hits and absent player friendly fire; nearest living unsafe guest targeting; shared asteroid cell destruction and one destruction record without rewards; per-pilot respawn; every public progression mutation barrier; exact solo restoration through end/host leave/reset; complete paged body cells and same-tick membership revision invalidation; existing solo cursor/thrust/cadence quality parity.

The actual bounded fixture containing eight participants and 180 projectiles serialized to **44,828 bytes**. Entity outputs remain at most 64 rows; body outputs remain at most 1,024 occupied cells. This fixture fits the declared 64 KiB transport-envelope allowance; it is not a throughput, congestion or network-delivery measurement.

## Resolved review findings

- Initial `Game.cancel_course()` did not clear the host `Pilot` route. Final source clears the host route and queued input before synchronizing, retaining normal velocity/throttle decay. The actual host-only cancellation test passes without cancelling the guest route.
- Initial resettable membership epochs could match delayed input in another session. Final join/input/leave/end/step/page/notice calls validate the explicit expected session before mutation, and membership epochs distinguish departure/rejoin. The actual replay/impersonation test passes. Future transport must use unique session identifiers and map authenticated channels to participant IDs; supplied packet IDs alone are not authentication.
- Initial training acknowledgement could return success while suppressing its mutation in-session. Final `acknowledge_license()` explicitly rejects the isolated session. The identified-host training test passes with unchanged retained save.

The author's independent same-tick page finding is also fixed: checked revision advances on join, leave and step, and both page APIs reject an obsolete revision. This is recorded as author work, not another reviewer-discovered defect.

## Source constraints and practical limits

The complete original solo `Game` is boxed and restored, preserving actual world bodies/handles, profile, clocks, RNG, controls, camera, effects and queued events. `profile()` returns that retained solo profile during the isolated session. Session entry requires the real dock, a finite living actor and a nonempty body. Guests supply only strict identity/seed metadata; canonical fresh factory profiles determine their stats. Shared progression, owned guest RAICHO transfer, boss/champion/special-enemy execution and secondary/signature weapon support are outside this stage. This implementation explicitly enables primary weapons and ordinary enemy families.

Inventory mutations, arena/site rewards, credits/training acknowledgements and diagnostic movement are blocked or suppressed; actual kills/pickups do not add XP, currency, cargo or crafted inventory. Actual created primary bullets carry participant/epoch ownership, friendly bullets skip all player bodies, hostile bullets damage the contacted pilot, and departure/death removes only that participant's shots. Existing world limits and global actor/projectile/particle caps remain.

Snapshot/page output is bounded, but page creation collects/sorts current entity metadata, scans occupied cells and allocates bounded output vectors. Exact solo retention also retains a second world for the session's lifetime. No zero-allocation, fixed total memory, sustained performance, authenticated transport, browser multiplayer, physical phone or long-soak acceptance is claimed here. Root owns subsequent integration/browser evidence.
