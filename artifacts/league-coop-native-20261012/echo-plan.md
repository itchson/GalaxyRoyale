# Next native co-op ability: ordinary Pulse Echo

Status: read-only design gap, implementation awaiting Root approval. No runtime edits or new tests, builds, exports, servers, or browser checks were performed for this report.

Reviewed native baseline: `41a54568780b4a89b7c6c44b7ae185e0393a8bb4` (Root integration of the passive parity tests). All 14 pinned source/test blobs also equal current Root `1b754da4e09fb2b8e6bbe123d8ac834750a97d99` and LF-normalized current working files. The working-tree raw hashes are recorded separately. Previous passive parity execution is a separate completed scope and is not new Echo evidence.

## Verified gap and selected slice

Choose equipped ordinary **Pulse Echo, ability ID 3**. It uses the successful-primary stream already present in co-op, so it requires a small per-pilot extension rather than a new ability framework. The current Pilot stores only `pulse_lens_rank` and modulo-five `primary_phase`; `tick_co_op` emits primary shots and Lens extras, with no Echo branch ([co_op.rs](C:/Users/mitch/Documents/ChatGPT/galaxy-royale/crates/game_core/src/co_op.rs:101), [successful primary](C:/Users/mitch/Documents/ChatGPT/galaxy-royale/crates/game_core/src/co_op.rs:659)).

Solo `Game::fire` emits the real piercing fan after successful primaries ([lib.rs](C:/Users/mitch/Documents/ChatGPT/galaxy-royale/crates/game_core/src/lib.rs:1418)). This is an observable missing combat effect, not just a missing tooltip or particle. Ordinary Echo can replace Lens in the offense slot; it must remain distinct from Lens's fifth-shot bolt. Royal Prism ID 14 and ship signature bundles are explicitly outside this first slice.

## Exact solo effect to preserve

- Gate by native equipped ordinary ID 3 effective rank, including ship compatibility and selected-hull XP cap. Rank affects fan damage, while **boss evolution is separate from rank** ([rank helper](C:/Users/mitch/Documents/ChatGPT/galaxy-royale/crates/game_core/src/ship_progression.rs:152), [evolution](C:/Users/mitch/Documents/ChatGPT/galaxy-royale/crates/game_core/src/bosses.rs:41)).
- Trigger every eighth admitted primary, or every sixth at evolution 5. Refused primaries do not advance or bank a trigger. Echo damage is `stats.damage * (0.5 + 0.1 * effective_rank.saturating_sub(1))`; `campaign::signature_rank(profile, 6)` resolves to family zero, not champion six.
- Evolution 1: angles `[-0.16, 0, 0.16]`; evolution 2: five angles `[-0.32, -0.16, 0, 0.16, 0.32]`; evolution 3+ adds rear angles `PI-0.18` and `PI+0.18`. Evolution 1..3 pierces two hits; evolution 4..5 pierces three.
- Every fan projectile starts at the same actual pilot front (`position + forward(heading)*19`), travels at `forward(heading+angle)*460` with **no inherited pilot velocity**, and uses normal player bullet life 1.5 seconds.
- The current solo 180-shot cap admits an ordered **partial fan**, one projectile at a time in authored angle order. Do not add atomic fan reservation or replay a consumed trigger. `ability-echo` is emitted only if any fan projectile was admitted; the existing transient light and eight-particle burst still occur on a due trigger even if the entire fan is refused.

## Necessary bounded admission fact

OwnedLoadout currently contains ten combat fields and reconstructs a temporary guest Profile with default boss-generation history ([co_op_loadout.rs](C:/Users/mitch/Documents/ChatGPT/galaxy-royale/crates/game_core/src/co_op_loadout.rs:18)). Its ability rank survives, but ordinary Echo evolution silently becomes 1. A rank-five declaration therefore cannot establish a seven-projectile evolved fan. The host has its actual retained native Profile and can derive evolution directly.

Recommended additive DTO: optional `echo_evolution: Option<u8>`, serde default and omitted when None. Export it from the actual native selected Profile only when ordinary ID 3 is equipped and evolution is above 1; absence means evolution 1 for historical v1/factory callers. Validate 1..5, reject a token attached to a different offense slot, and keep existing structural loadout validation. Resolve this effect-only fact into the Pilot cache; do not fabricate boss defeat records in a temporary Profile or transmit complete Profile/history/currency.

This remains a bounded **self-declared offline ownership claim**, as in the accepted owned-loadout contract, not cryptographic proof. Root owns the matching strict wire whitelist, native bridge, and browser port changes. Their current ten-field parsers will reject an uncoordinated eleventh field. Existing factory fixtures and absent-token serialization must remain byte-identical.

If Root approves zero DTO change instead, the honest first stage is ordinary evolution-1 Echo only; evolved owned guest parity would remain incomplete. The recommended next slice includes the small optional fact above.

## Minimal native hooks and lifecycle

1. Cache `echo_rank`, `echo_evolution`, and a bounded per-pilot `echo_phase` (period 8 or 6) during validated admission. Increment only after `spawn_bullet_entity` returns Some for that pilot's primary. Keep Lens state unchanged. No remotely supplied counters or durable Profile mutation.
2. Reuse the existing fixed eight-pilot queue pattern: at most one due Echo command per pilot, retaining actual Motion, Stats, effective rank/evolution and ShotOwner. All queued primaries retain the existing co-op priority, then existing Lens extras, then Echo attempts. This cross-pilot primaries-first order is an established co-op policy and differs from sequential solo firing; do not claim byte-identical multiplayer ordering. There are at most 56 fan spawn attempts per tick (8*7), all subject to the existing global 180 bullet cap. No heap queue, raised cap, or effect dispatcher framework is needed.
3. Attach authenticated participant ID plus membership epoch to every fan projectile exactly as for current primary/Lens shots. Reuse actual Bullet, collision, piercing, damage, body holes, impulse/debris and destruction attribution. Player bullets already skip Player actors, so this must not introduce PvP. The session kill guard returns before progression/XP/drop grants ([collision](C:/Users/mitch/Documents/ChatGPT/galaxy-royale/crates/game_core/src/lib.rs:1942), [damage](C:/Users/mitch/Documents/ChatGPT/galaxy-royale/crates/game_core/src/lib.rs:2174), [kill guard](C:/Users/mitch/Documents/ChatGPT/galaxy-royale/crates/game_core/src/lib.rs:2415)).
4. Use the real pilot cadence/safe-zone/input gates before primary admission. Hold phase through fire release, pause, input timeout, Haven repair and pilot respawn. Start at zero for a fresh membership, matching current Lens admission semantics; this does not continue a pre-session solo shot counter. Leave removes all owner/epoch fan shots; rejoin gets a fresh epoch/phase; end restores the exact boxed solo Game including its original solo counter/Profile.

The existing host bounded transient-light/particle path can reproduce the solo burst without extra quotas. **Guest transient ability lights/particles remain a separate presentation gap**: the current publication omits those facts and the native replica explicitly does not invent them ([co_op_replica.rs](C:/Users/mitch/Documents/ChatGPT/galaxy-royale/crates/game_core/src/co_op_replica.rs:179)). Published real fan projectiles and damaged occupied bodies can render already; that does not establish full guest audiovisual parity.

## Meaningful acceptance checks for the proposed implementation

- Real solo/session successful-primary comparisons for evolution 1..5 and effective rank caps, including high stored rank with low selected-hull XP. Prove three/five/seven fans, correct sixth/eighth cadence, and absence with Lens/empty offense slot. Use two and eight independent pilots with staggered input rather than direct counter assignment as the main oracle.
- Moving-pilot fans prove actual front-19 positions, authored angle order, 460 velocity without inherited motion, rank-scaled damage, two/three piercing hits and authenticated owner/epoch. Test all queued primaries retain priority at the shared cap.
- At 180 shots, refused primaries leave phase unchanged. At short remaining capacity, prove zero/partial fans and no replay on the next primary. Keep the existing solo event-versus-burst distinction explicit.
- Real cell collisions against asteroids/enemies/segments prove holes, impulse/debris, multi-target pierce, correct destruction attribution, no player damage and no duplicated rewards/Profile changes. Reuse actual collision paths and existing test fixtures.
- Pause/release/stale input, Haven repair, respawn, leave/rejoin and end prove independent phases, owner cleanup, fresh epochs and exact original solo restoration.
- Native export/validation and real publication/replica cases prove the optional evolution token, malformed/wrong-slot rejection, old absent-token byte parity, fan metadata and occupied holes. Do not claim guest transient effects from projectile-only publication.

Existing solo tests inspected, not rerun in this planning scope: `ordinary_boss_echo_stays_a_separate_eighth_shot_ability`, `automatic_echo_tracks_actual_primaries_after_cadence_idle_and_pause`, and Sovereign/ordinary evolution tests. Production specialist independently reread the exact baseline and confirmed the gap, rank/evolution separation, partial-cap policy and host FX distinction; that was source-only, with no test execution.

Recommended implementation ownership after approval: co_op.rs plus one dedicated focused Echo test file and the narrow co_op_loadout.rs metadata/export/validation path. Root coordinates strict wire/bridge support. Repulsor Nova, Ion Wake, Royal/ship signature effects, shared rewards and guest transient FX remain explicit later work.
