# First native host-authority slice

Frozen source: `60934969913ad1e49a3ce4b2b63af514d890518b`, branch `codex/co-op-authority`, parent `5aa5cfaeb4db4e71511ac4dc251ce17d1b423923`. Seven source files are committed; tracked checkout was clean. This is an unfinished authority stage, not released or browser-validated co-op.

One actual `Game` ECS contains the host and up to seven participant-owned player bodies. Solo and session steering share the extracted existing control math. Primary projectiles carry stable participant ID and membership epoch; ordinary enemies choose the nearest living unprotected participant, hostile swept hits damage that participant, and player shots do not PvP. Tests fire two participants into one shared asteroid, observe actual occupied-cell holes, and verify one destruction notice without XP, currency, materials or loot grants.

Beginning requires the actual living player inside the 128-world-unit dock, correct protocol, no existing session/arena/diagnostic/stress. The original complete solo `Game` is retained in one `Box<Game>`; the active world is a fresh host-seeded Game. `profile()` returns the retained solo profile. End, host leave, or reset restores that exact instance, including cargo, damaged body, shots, RNG, time, controls and effects. The session cannot recursively begin another session. Session purchases/crafting/equipping/components/champions/tournaments/site claims/license acknowledgement/credits/diagnostics are blocked; solo tick cannot accidentally advance the authority world.

IDs are canonical 32-lowercase-hex strings. Guest specs accept only `id`, nonzero `local_seed` and optional validated Kikai `identity`; remote Profile/progression/loadout fields are rejected. Guest local seed/identity remain separate from the host world seed. Guests temporarily use their actual factory hull and starter stats. Authenticated channel-to-ID mapping remains the caller's responsibility; supplied IDs are not authentication. Each new service session must have a fresh session ID.

Every mutating/public page call carries `expected_session`. A different session rejects delayed packets even when IDs/epochs were reused. Rejoin within a session obtains a new membership epoch; sequence numbers must strictly increase and do not wrap. Input expires after 15 fixed ticks without a refresh. Join/leave/tick advance `revision`, independently of physics `tick`, so same-tick membership changes invalidate old pages. Overlay cancellation clears the host Pilot's held input/course and preserves normal momentum/throttle decay.

## Native API

Types live in public `game_core::co_op`; `ParticipantId::parse(&str) -> Result<ParticipantId, Error>`. Protocol is `galaxy-coop-1`. Methods are on `Game`:

```rust
begin_co_op(&mut self, protocol: &str, session: ParticipantId, host: ParticipantId)
    -> Result<Admission, Error>
join_co_op(&mut self, expected_session: ParticipantId, spec: ParticipantSpec)
    -> Result<Admission, Error>
submit_host_co_op_input(&mut self, expected_session: ParticipantId,
    sequence: u32, input: InputFrame) -> Result<(), Error>
submit_guest_co_op_input(&mut self, expected_session: ParticipantId,
    id: ParticipantId, epoch: u32, sequence: u32, input: InputFrame) -> Result<(), Error>
tick_co_op(&mut self, expected_session: ParticipantId) -> Result<(), Error>
leave_co_op(&mut self, expected_session: ParticipantId,
    id: ParticipantId, epoch: u32) -> Result<(), Error>
end_co_op(&mut self, expected_session: ParticipantId) -> Result<(), Error>
co_op_snapshot(&self) -> Option<Snapshot>
take_co_op_destructions(&mut self, expected_session: ParticipantId)
    -> Result<Vec<Destruction>, Error>
co_op_entity_page(&self, expected_session: ParticipantId, revision: u32,
    offset: usize) -> Result<EntityPage, Error>
co_op_body_page(&self, expected_session: ParticipantId, revision: u32,
    key: &str, offset: usize) -> Result<BodyPage, Error>
```

`tick_co_op` advances exactly 1/60 second; clients cannot submit dt/positions/HP/cells. Host input has a separate local entry point; the guest method rejects the host ID. The transport input payload is `{epoch,seq,input:InputFrame}`; the envelope session and authenticated channel peer ID are supplied separately to Rust. Optional joystick components are finite with absolute value <=4; destination/aim/thrust coordinates are finite within +/-1,000,000 world units.

## Serialized data and bounds

Snapshot: `{protocol,session,world_seed,tick,revision,progression:false,participants,projectiles,dropped_notices}`. Participants (max8) include ID/slot/epoch/local_seed/identity/selected_ship, position/previous/velocity/heading/previous_heading, hull/max_hull/shield/max_shield/throttle/last_sequence. Projectiles (max180) include 16-hex ECS key, nullable owner/epoch, position/previous/velocity, damage/life/remaining_hits. Lists are stable-sorted by slot or ECS key.

Entity pages are at most64 real ECS rows with `{id,kind,participant,position,previous,velocity,heading,previous_heading,scale,radius,hull,max_hull,width,height,occupied}`. Body pages are at most1024 `[rowMajorIndex,rgbaLE]` pairs from the current actual damaged PixelBody. Pages include session/tick/revision/offset/next_offset; body pages also identify the ECS key/dimensions/occupied count. Stale session/revision, invalid generation/key and invalid offset reject. Detached segment maximum hull is0 because the existing segment component does not store original maximum health.

Destruction notices carry tick/ECS key/kind/nullable shot owner+epoch. Queue is capped128; overflow is counted in `dropped_notices`. Draining is explicit and session-bound. This queue is informational, never reward authority. Existing world caps remain:112 rocks,48 enemies,180 projectiles,1100 particles; streaming retains at most eight participant neighborhoods. No global cap was increased.

Actual fixture files under `serialization/` come from the explicitly ignored native exporter, not a network or natural-play capture. Eight real admitted ECS pilots advance one fixed tick; native fixture setup then fills the actual180-shot cap. Before final LF: snapshot45,003 bytes; entity page20,813; body page18,288; admissions489; input116. The independent edge-size assertion fixture is44,828 bytes. These finite fixtures leave transport-envelope room under64KiB; this is not a general compression/performance claim. Gameplay's separate transport validator accepted all five exports without changing them.

## Checks and remaining work

Actual final checks: full core401 passed,0 failed,4 explicit export tests ignored; focused17 passed,0 failed,1 exporter ignored; explicit serializer exporter1 passed; strict native core all-target Clippy and scoped rustfmt/diff checks passed. Logs are pinned by `manifest.json`. Shared solo cursor/thrust/cadence, sanctuary, training, save and reward tests ran in the full core suite. No full WASM build, browser, network, GPU, performance, RSS or device claim.

Historical failures are retained: one initial cancellation fixture wrongly expected immediate velocity reduction despite existing throttle decay; one exporter initially moved a non-Copy input; a broad final acknowledgement guard initially changed diagnostic Ok behavior. The final guard is session-only, and the unchanged full diagnostic tests pass.

Pending: validated guest-owned hull/loadout admission, automatic abilities and advanced signatures, special NPC/boss/campaign encounters, all shared rewards/missions/onboarding, guest exhaust/extended audio and local camera presentation, WASM wiring, client replica/render/interpolation, actual transport integration and browser/device acceptance. Session loot does not grant resources. This temporary factory restriction is not a final design requiring invited players to lose their owned RAICHO. Tick/snapshot/page allocation and cold second-Game initialization remain unprofiled. Root owns all subsequent binding and integration work.

Production peer review is separate under its own `work/co-op-authority-review`; source is frozen pending concrete findings.
