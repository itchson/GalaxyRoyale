# Owned guest RAICHO admission handoff

Source commit `75b9c2535bbb66c098e7fa66a05f56db9355cab6` is frozen on `codex/co-op-owned-loadout`, base `27e0b403e96689598eb57fb91805d315de1c3e08`. The tracked checkout was clean after committing only co_op.rs, co_op_replica.rs, the lib.rs module declaration and new co_op_loadout.rs/co_op_loadout_tests.rs. Public API and validation limits are in CONTRACT.md.

Actual verification:

- `cargo test --locked -p game_core co_op -- --nocapture`: 48 passed, 0 failed, 2 existing explicit export tests ignored, 389 filtered out. This includes authority/publication/replica and nine owned-loadout tests. Log focused-final.log predates only an additional null-identity assertion within an existing test.
- After that assertion, `cargo test --locked -p game_core co_op_loadout -- --nocapture`: all nine passed, 0 ignored, 430 filtered out. Log legacy-final.log is at final source.
- `cargo clippy --locked -p game_core --all-targets -- -D warnings`: exit 0 at final source. Log clippy-final.log. rustfmt on owned modules and `git diff --check` passed.

Tests cover all ten selected hulls; actual retained solo exports/restoration; 17 malformed semantic builds without epoch or publication mutation; unsigned/array/unknown-field rejection; effective rank and trained R2/Sovereign exceptions; real two/eight-pilot movement, exact selected occupied bodies and native primary bullets; all eight champion manufacturers across three pilot corporations; actual publication-to-Replica HUD/render/body-signature coherence; invalid and same-epoch changed build rejection preserving the old scene; unchanged factory metadata/retained fixture; and the full eight-participant/180-projectile frame bound.

Measured serialization, not performance claims: an actual owned declaration is 238 UTF-8 bytes. The retained two-player selected-hull samples have 1,342-byte headers. The actual eight-pilot plus 180-projectile framed publication is 50,071 bytes, below the existing 65,536-byte wire limit. Sample owned-loadout.json is extracted from the native test's real serialized stdout, not a mocked bridge response. These finite fixtures do not prove every possible JSON pose size or browser transport throughput.

No full workspace/WASM build, server, browser, GPU, device or shared reward verification was run by this task. The existing general co-op performance test ran as part of the focused command; its CPU-only output is retained without claiming an owned-loadout performance benchmark. Root owns combined builds and the real browser integration. The offline ownership and unfinished ability/reward boundaries in CONTRACT.md remain explicit.

source-pins.json records committed source under canonical UTF-8/LF and working-tree raw hashes separately, plus unchanged native inputs and raw evidence. verify.py checks the same committed/working domains. Earlier logs/handoffs are preserved.
