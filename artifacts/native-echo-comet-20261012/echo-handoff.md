# Ordinary Pulse Echo co-op handoff

Source frozen at `84c485a858464ed45e6f34ef56a6baee69757763` on `codex/co-op-pulse-echo`, base `378293d208bf56d18601c55f301af880bfddcaea`. Exactly three tracked files changed: co_op.rs, co_op_loadout.rs and new co_op_echo_tests.rs. The worktree is tracked-clean. Root owns wire/ports, integration, normal WASM builds and browser acceptance.

## Behavior and contract

Validated ordinary equipped ability ID3 now has a native per-pilot effective-rank/evolution/phase cache. Only an actually admitted primary advances its bounded phase (period8, or6 at evolution5). A fixed eight-entry queue executes after all primaries and existing Lens extras. Each real fan projectile uses actual Motion/Stats, the authored three/five/seven angles, front19 position, velocity460 without inherited pilot velocity, rank-scaled damage, native piercing2/3 and normal player life1.5. Every projectile receives authenticated participant ID plus membership epoch.

The existing180-shot cap remains authoritative: fan attempts are individually admitted in authored order; refused/partial fans consume their trigger and are never replayed. The ability event requires at least one accepted fan projectile; the existing bounded purple burst/light still occurs on a due trigger even when none fit, matching solo. Cross-pilot primaries→Lens→Echo ordering is the existing co-op priority policy, not a claim of identical sequential solo ordering. Maximum due Echo commands are8; maximum fan attempts are56/tick, still refused by the unchanged cap.

Shared collisions reuse the actual cell/segment damage, impulse/debris, piercing, owner attribution and no-PvP/reward-isolation paths. Phase survives pause/release/timeout, Haven repair and respawn; new membership starts at0. Leave removes owner+epoch shots, rejoin rejects old epochs, and end restores the original solo Game/Profile/shot counter.

`OwnedLoadout.echo_evolution: Option<u8>` is defaulted and skipped whenNone. Export reads the actual native Profile and emits only equipped ordinary Echo evolution>1. Incoming present values must be1..5 with ordinaryID3 in offense slot; absence keeps evolution1. No defeat records, Profile, currency or saved counters are transmitted or synthesized. This is a bounded offline self-declaration, not a proof of remote ownership. The temporary guest Profile still has zero boss-generation records; Pilot uses the validated effect-only token. Native serde null is None; Root's strict wire rejects presentnull, and native export never emits it.

## Actual verification

- Focused `cargo test --locked -p game_core co_op::echo_tests -- --nocapture`: **9 passed**,0failed/ignored,452filtered. `focused-final.log`.
- Joined `cargo test --locked -p game_core co_op -- --nocapture`: **67 passed**,0failed,**2 existing exporters ignored**,392filtered. `joined-final.log`. This includes prior real movement/loadout/authority/publication/replica/Lens/passive cases; it is not a full workspace suite.
- `cargo clippy --locked -p game_core --all-targets -- -D warnings`: exit0. `clippy-final.log`.
- Scoped rustfmt check and git diff check: exit0. No full game build, WASM build, listener, server or browser was run here.

Nine cases prove actual solo/session tick equivalence for all5 evolutions ×3 stored/effective-rank builds (including storedR5 at XP0→effectiveR1), eight staggered independent pilots/moving fans, full-cap primary refusal, all available1..8 partial admissions, primary/Lens priority, pause/safe/timeout/respawn/leave/rejoin/end, ordinary-slot exclusion of Lens/empty/Royal bundles, actual asteroid/enemy/connected-segment cell collisions/debris/impulse, correct destruction attribution, noPvP/noLoot/noProfile grants, and a real native publication/Replica fan+occupied-hole packet.

The packet case produced **8 real projectiles**, **3,085 bytes** for the serialized native capture header (not a framed transport envelope), **1,999 native replica cells**, and **356/400 remaining cells** in its arranged damaged rock. It uses current native publication and receiver validation without fixture replies; it does not establish browser rendering or transport success.

## Actual serialized fixtures

`legacy-loadout.json` and `evolved-echo-loadout.json` were copied verbatim from actual `println!` native serializer output in focused-final.log, with a finalLF added. They are **183/218 JSON bytes** excludingLF. The legacy ten-field string is asserted against the previous exact schema. The evolved fixture is a fixture-configured licensed RENKO hull0, stored ordinaryEchoR5, selectedXP0/effectiveR1, with a canonical configured defeat chain `[16,8,4,2,1]` yielding evolution5. It is **not** a naturally earned player save. No Profile or defeat history appears in the DTO.

Initial diagnostic logs are retained: first-check had test-only Profile equality/TransientLight field compilation mistakes; first-execution used noncanonical fixture defeat prerequisites and queried particles as ECS entities; third-execution expected yellow instead of the existing purple piercing-projectile color. Those fixtures/assertions were corrected, and the final actual checks above pass. They were not runtime defect proofs.

## Limits

Ordinary Echo only. Royal Prism/ship signatures, Repulsor Nova, Ion Wake and shared progression/rewards remain outside this slice. Host event/burst/light uses existing native presentation; guest transient ability lights/particles/audio are still omitted by the current publication and are not claimed equivalent. Offline host validation cannot prove another browser's durable ownership. Native fresh-membership counters start0, not the pre-session solo firing phase. Final combined WASM/RTC/GPU/GL checks remain Root-owned.

Source-only peer review by Production is finishing at this same immutable commit. One bounded read-only Spark metadata reviewer was invoked; its output is pending and is not counted as verification here. Neither reviewer is credited as executing these native tests.
