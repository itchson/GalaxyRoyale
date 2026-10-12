# Per-pilot Pulse Lens parity

Frozen commit `f79e27835ba01fa12a539350ff291e405ed160ff`, branch `codex/co-op-pulse-lens`, base `2d494ebb3b1ae46495545a8af80d0fa78ca3e58b`. Only co_op.rs and new co_op_starter_tests.rs are committed; the tracked checkout is clean. The previous read-only design remains immutable under work/co-op-starter-plan.

Pilot caches the actual native effective Pulse Lens rank at admission and owns a bounded successful-primary phase 0..4. Only spawn_bullet_entity success advances that pilot's phase. Every fifth success with equipped Pulse Lens attempts the exact solo stronger bolt: forward offset 21, speed 500 plus 25% actual pilot velocity, damage 150% of native primary damage, player=true/hits1. The effect follows the equipped ability, not the license corporation. Primary damage already includes the native rank and active hull affinity.

All queued primary attempts run before the fixed eight-entry bonus queue, under the unchanged global 180 projectile cap. Refused primaries do not count; refused fifth bonuses are consumed and never replayed. Each bolt carries the same participant ID/membership epoch. Counter/rank stay native transient components, with no wire/schema/Profile field. Phase survives safe hold, release, pause, timeout, home repair and respawn, matching the current solo cycle. New admission/rejoin starts 0; end restores the boxed original solo counter/world. No shared rewards or remote save writes are added.

Actual checks at final source:

- `cargo test --locked -p game_core co_op::starter_tests -- --nocapture`: eight passed, no failures/ignores, 440 filtered. focused-final.log.
- `cargo test --locked -p game_core co_op -- --nocapture`: 56 passed, no failures, two existing explicit exporters ignored, 390 filtered. co-op-final.log.
- `cargo test --locked -p game_core starter_ability_tests -- --nocapture`: five solo tests passed, no failures/ignores, 443 filtered. solo-parity.log.
- `cargo clippy --locked -p game_core --all-targets -- -D warnings`: exit 0. clippy-final.log. Scoped rustfmt and git diff --check pass.

New actual tick probes cover staggered two/eight pilots (20 primaries/4 bonuses each); native factory/owned/legacy-null builds, equipped versus corporation and rank 1/trained 2/rank 5; inherited moving velocity/spawn direction; cap refusal, primary-first contention and no replay; lifecycle and fresh epoch; real surviving-cell damage and one destruction attribution with no PvP/progression; actual coherent publication-to-Replica assembly and exact two-bolt quad centers. The first lifecycle fixture incorrectly fired the out-of-sanctuary host while asserting no guest-safe-zone shots; its failure is preserved in initial-fixture-failure.log. The corrected fixture isolates guest input without weakening its phase/lifecycle assertions.

The actual two-projectile Header is 1,562 UTF-8 bytes. The assembled native Replica emits 95,093 cells, including actual streamed host-world bodies plus exactly six projectile quads (three per bolt); this is a finite native packet proof, not a new renderer cap or performance claim. No runtime packet files, images, export tests, servers, GPU, browser/RTC, WASM package or full workspace build were executed here. Root owns combined integration and real browser acceptance. Current earlier RTC results remain pinned to their own ordinary-primary bundles.

Other triggered family/champion/Royal abilities, shared rewards/missions and offline ownership verification remain separate unfinished scopes. The primary-first policy does not claim fairness beyond that finite ordering. source-pins.json contains exact committed LF source/input pins, working raw hashes and raw evidence hashes; verify.py checks them without changing files.
