//! First host-authority slice in the real Game ECS. Transport and shared rewards
//! are intentionally outside this module. A session restores the exact solo Game.
use super::*;
pub use crate::co_op_loadout::{OwnedAbility, OwnedLoadout};
use serde::{Deserializer, Serializer, de::Error as _};

pub const PROTOCOL: &str = "galaxy-coop-1";
pub const MAX_PARTICIPANTS: usize = 8;
pub const INPUT_TIMEOUT_TICKS: u32 = 15;
pub const TICK_SECONDS: f32 = 1.0 / 60.0;
pub const ENTITY_PAGE_SIZE: usize = 64;
pub const CELL_PAGE_SIZE: usize = 1024;
const MAX_NOTICES: usize = 128;

/// Authenticated signaling peer identifier. Parsing rejects aliases and lengths.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ParticipantId([u8; 16]);
impl ParticipantId {
    pub fn parse(value: &str) -> Result<Self, Error> {
        if value.len() != 32
            || !value
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(Error::InvalidId);
        }
        let mut bytes = [0; 16];
        for (index, byte) in bytes.iter_mut().enumerate() {
            *byte = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16)
                .map_err(|_| Error::InvalidId)?;
        }
        Ok(Self(bytes))
    }
    pub fn as_hex(self) -> String {
        self.0.iter().map(|byte| format!("{byte:02x}")).collect()
    }
}
impl Serialize for ParticipantId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.as_hex())
    }
}
impl<'de> Deserialize<'de> for ParticipantId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        Self::parse(&value)
            .map_err(|_| D::Error::custom("Expected 32 lowercase hexadecimal digits."))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Error {
    InvalidId,
    InvalidProtocol,
    InvalidIdentity,
    InvalidLoadout,
    InvalidSeed,
    NotAtDock,
    ActiveSession,
    NoSession,
    Full,
    DuplicateId,
    UnknownParticipant,
    HostOnly,
    StaleSession,
    StaleMembership,
    StaleSequence,
    InvalidInput,
    Paused,
    Exhausted,
    StaleSnapshot,
    InvalidEntity,
    InvalidOffset,
    PublicationBound,
    NoPublication,
    StalePublication,
}

/// Legacy v1 callers supply no combat inventory and retain factory admission.
/// The separate owned-loadout method never accepts a complete remote Profile.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParticipantSpec {
    pub id: ParticipantId,
    pub local_seed: u32,
    pub identity: Option<identity::KikaiIdentity>,
}
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
pub struct Admission {
    pub id: ParticipantId,
    pub slot: u8,
    pub epoch: u32,
}

#[derive(Clone, Copy)]
pub(super) struct ShotOwner {
    pub id: ParticipantId,
    pub epoch: u32,
}
pub(super) struct Pilot {
    pub admission: Admission,
    pub spec: ParticipantSpec,
    pub stats: Stats,
    pub selected_ship: u8,
    pub owned_loadout: Option<OwnedLoadout>,
    pub original_body: PixelBody,
    pub shield: f32,
    pub since_damage: f32,
    pub invulnerability: f32,
    pub damage_multiplier: f32,
    pub throttle: f32,
    pub destination: Option<Vec2>,
    pub fire_timer: f32,
    pub pulse_lens_rank: u8,
    pub primary_phase: u8,
    pub echo_rank: u8,
    pub echo_evolution: u8,
    pub echo_phase: u8,
    pub nova_rank: u8,
    pub nova_evolution: u8,
    pub nova_timer: f32,
    pub wake_rank: u8,
    pub wake_evolution: u8,
    pub wake_timer: f32,
    pub wake: Vec<crate::co_op_wake_fx::Sample>,
    pub input: InputFrame,
    pub last_sequence: Option<u32>,
    pub input_tick: u32,
    pub was_home: bool,
}
pub(super) struct Session {
    pub solo: Box<Game>,
    id: ParticipantId,
    host: ParticipantId,
    pub(super) tick: u32,
    revision: u32,
    next_epoch: u32,
    pub hit_owner: Option<ShotOwner>,
    notices: Vec<Destruction>,
    dropped_notices: u32,
    pub(super) nova_fx: crate::co_op_fx::Journal,
}

#[derive(Clone, Debug, Serialize)]
pub struct ParticipantState {
    pub id: ParticipantId,
    pub slot: u8,
    pub epoch: u32,
    pub local_seed: u32,
    pub identity: Option<identity::KikaiIdentity>,
    pub selected_ship: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owned_loadout: Option<OwnedLoadout>,
    pub position: [f32; 2],
    pub previous: [f32; 2],
    pub velocity: [f32; 2],
    pub heading: f32,
    pub previous_heading: f32,
    pub hull: f32,
    pub max_hull: f32,
    pub shield: f32,
    pub max_shield: f32,
    pub throttle: f32,
    pub last_sequence: Option<u32>,
}
#[derive(Clone, Debug, Serialize)]
pub struct ProjectileState {
    pub id: String,
    pub owner: Option<ParticipantId>,
    pub owner_epoch: Option<u32>,
    pub position: [f32; 2],
    pub previous: [f32; 2],
    pub velocity: [f32; 2],
    pub damage: f32,
    pub life: f32,
    pub remaining_hits: u8,
}
#[derive(Clone, Debug, Serialize)]
pub struct Snapshot {
    pub protocol: &'static str,
    pub session: ParticipantId,
    pub world_seed: u32,
    pub tick: u32,
    pub revision: u32,
    pub progression: bool,
    pub participants: Vec<ParticipantState>,
    pub projectiles: Vec<ProjectileState>,
    pub dropped_notices: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nova_fx: Option<crate::co_op_fx::Journal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wake_fx: Option<crate::co_op_wake_fx::WakeMeta>,
}
#[derive(Clone, Debug, Serialize)]
pub struct Destruction {
    pub tick: u32,
    pub id: String,
    pub kind: &'static str,
    pub owner: Option<ParticipantId>,
    pub owner_epoch: Option<u32>,
}
#[derive(Clone, Debug, Serialize)]
pub struct EntityState {
    pub id: String,
    pub kind: &'static str,
    pub participant: Option<ParticipantId>,
    pub position: [f32; 2],
    pub previous: [f32; 2],
    pub velocity: [f32; 2],
    pub heading: f32,
    pub previous_heading: f32,
    pub scale: f32,
    pub radius: f32,
    pub hull: f32,
    pub max_hull: f32,
    pub width: usize,
    pub height: usize,
    pub occupied: usize,
    pub body_signature: String,
}
#[derive(Clone, Debug, Serialize)]
pub struct EntityPage {
    pub session: ParticipantId,
    pub tick: u32,
    pub revision: u32,
    pub offset: usize,
    pub next_offset: Option<usize>,
    pub entities: Vec<EntityState>,
}
#[derive(Clone, Debug, Serialize)]
pub struct BodyPage {
    pub session: ParticipantId,
    pub tick: u32,
    pub revision: u32,
    pub id: String,
    pub width: usize,
    pub height: usize,
    pub occupied: usize,
    pub offset: usize,
    pub next_offset: Option<usize>,
    /// Original row-major occupied index and little-endian RGBA, not a bitmap.
    pub cells: Vec<[u32; 2]>,
}

fn entity_key(entity: Entity) -> String {
    format!("{:016x}", entity.to_bits().get())
}
fn kind_name(kind: Kind) -> &'static str {
    match kind {
        Kind::Player => "player",
        Kind::Asteroid => "asteroid",
        Kind::Enemy(_) => "enemy",
        Kind::Boss(_) => "boss",
    }
}
fn slot_position(slot: u8) -> Vec2 {
    Vec2::new(
        (f32::from(slot % 4) - 1.5) * 44.0,
        (f32::from(slot / 4) - 0.5) * 44.0,
    )
}
fn valid_input(input: &InputFrame) -> bool {
    input
        .joystick
        .is_none_or(|v| v.iter().all(|x| x.is_finite() && x.abs() <= 4.0))
        && [input.destination, input.aim, input.thrust]
            .iter()
            .all(|point| {
                point.is_none_or(|v| v.iter().all(|x| x.is_finite() && x.abs() <= WORLD_LIMIT))
            })
}
pub(super) fn guest_profile(spec: &ParticipantSpec) -> Result<Profile, Error> {
    if spec.local_seed == 0 {
        return Err(Error::InvalidSeed);
    }
    let mut profile = if let Some(id) = &spec.identity {
        identity::create_profile(
            match id.kind {
                identity::KikaiKind::Dora => "dora",
                identity::KikaiKind::Doro => "doro",
                identity::KikaiKind::Talo => "talo",
            },
            match id.corporation {
                identity::Corporation::Pan => "pan",
                identity::Corporation::Renko => "renko",
                identity::Corporation::Hugo => "hugo",
            },
            match id.sub_corporation {
                identity::Corporation::Pan => "pan",
                identity::Corporation::Renko => "renko",
                identity::Corporation::Hugo => "hugo",
            },
            &id.hex_id,
        )
        .map_err(|_| Error::InvalidIdentity)?
    } else {
        Profile::default()
    };
    // This temporary combat profile is not an accepted save. Seed remains the
    // guest's own metadata; every actual world query reads the host Game profile.
    profile.seed = spec.local_seed;
    Ok(profile)
}
fn pilot(
    admission: Admission,
    spec: ParticipantSpec,
    profile: &Profile,
    owned_loadout: Option<OwnedLoadout>,
) -> Pilot {
    let stats = crate::co_op_loadout::stats(profile, owned_loadout.as_ref());
    let echo_evolution = owned_loadout.as_ref().map_or_else(
        || bosses::evolution(profile, 0),
        |build| build.echo_evolution.unwrap_or(1),
    );
    let nova_evolution = owned_loadout.as_ref().map_or_else(
        || bosses::evolution(profile, 1),
        |build| build.nova_evolution.unwrap_or(1),
    );
    let wake_evolution = owned_loadout.as_ref().map_or_else(
        || bosses::evolution(profile, 2),
        |build| build.wake_evolution.unwrap_or(1),
    );
    Pilot {
        admission,
        spec,
        stats,
        selected_ship: profile.campaign.selected_ship,
        owned_loadout,
        original_body: identity::player_body(profile),
        shield: stats.shield,
        since_damage: 10.0,
        invulnerability: 0.0,
        damage_multiplier: shield_progression::incoming_damage_multiplier(profile),
        throttle: 0.0,
        destination: None,
        fire_timer: 0.0,
        pulse_lens_rank: ship_progression::equipped_rank(profile, 0),
        primary_phase: 0,
        echo_rank: ship_progression::equipped_rank(profile, 3),
        echo_evolution,
        echo_phase: 0,
        nova_rank: ship_progression::equipped_rank(profile, 4),
        nova_evolution,
        nova_timer: 0.0,
        wake_rank: ship_progression::equipped_rank(profile, 5),
        wake_evolution,
        wake_timer: 0.0,
        wake: Vec::new(),
        input: InputFrame::default(),
        last_sequence: None,
        input_tick: 0,
        was_home: true,
    }
}

impl Game {
    pub fn begin_co_op(
        &mut self,
        protocol: &str,
        session: ParticipantId,
        host: ParticipantId,
    ) -> Result<Admission, Error> {
        if protocol != PROTOCOL {
            return Err(Error::InvalidProtocol);
        }
        if self.co_op.is_some() {
            return Err(Error::ActiveSession);
        }
        if self.arena.is_some()
            || self.diagnostic_scene
            || self.boss_qa_scene
            || self.stress_mode
            || !self.position().is_finite()
            || self.position().length_squared() > DOCK_RADIUS * DOCK_RADIUS
            || !self
                .world
                .get::<&Actor>(self.player)
                .is_ok_and(|a| a.health.is_finite() && a.health > 0.0)
            || !self
                .world
                .get::<&PixelBody>(self.player)
                .is_ok_and(|body| body.occupied_count() > 0)
        {
            return Err(Error::NotAtDock);
        }
        let admission = Admission {
            id: host,
            slot: 0,
            epoch: 1,
        };
        let profile = self.profile.clone();
        let spec = ParticipantSpec {
            id: host,
            local_seed: profile.seed,
            identity: profile.identity.clone(),
        };
        let mut active = Game::new(profile.clone());
        // Game::new's reload migration may discard flight cargo. This session
        // neither reloads nor commits a save: retain exactly the original profile.
        active.profile = profile.clone();
        active.publication_generation = self.publication_generation;
        active.viewport = self.viewport;
        active.reduced_motion = self.reduced_motion;
        active.reduced_cosmetics = self.reduced_cosmetics;
        active
            .world
            .insert_one(active.player, pilot(admission, spec, &profile, None))
            .unwrap();
        *active.world.get::<&mut Motion>(active.player).unwrap() =
            Motion::new(slot_position(0), Vec2::ZERO, -FRAC_PI_2);
        let solo = Box::new(std::mem::replace(self, active));
        self.co_op = Some(Session {
            solo,
            id: session,
            host,
            tick: 0,
            revision: 0,
            next_epoch: 2,
            hit_owner: None,
            notices: Vec::with_capacity(MAX_NOTICES),
            dropped_notices: 0,
            nova_fx: crate::co_op_fx::Journal::default(),
        });
        self.sync_co_op_host();
        Ok(admission)
    }

    pub fn join_co_op(
        &mut self,
        expected_session: ParticipantId,
        spec: ParticipantSpec,
    ) -> Result<Admission, Error> {
        self.check_co_op_session(expected_session)?;
        let profile = guest_profile(&spec)?;
        self.admit_co_op(spec, profile, None)
    }

    /// Offline owned-build declaration; ownership cannot be proven by a host.
    /// Neither this temporary build nor shared kills are written to any save.
    pub fn join_co_op_owned(
        &mut self,
        expected_session: ParticipantId,
        spec: ParticipantSpec,
        owned_loadout: OwnedLoadout,
    ) -> Result<Admission, Error> {
        self.check_co_op_session(expected_session)?;
        let profile = owned_loadout.profile(&spec)?;
        self.admit_co_op(spec, profile, Some(owned_loadout))
    }

    fn admit_co_op(
        &mut self,
        mut spec: ParticipantSpec,
        profile: Profile,
        owned_loadout: Option<OwnedLoadout>,
    ) -> Result<Admission, Error> {
        spec.identity = profile.identity.clone();
        let session = self.co_op.as_ref().ok_or(Error::NoSession)?;
        if spec.id == session.host {
            return Err(Error::DuplicateId);
        }
        let mut occupied = [false; MAX_PARTICIPANTS];
        for (_, p) in self.world.query::<(Entity, &Pilot)>().iter() {
            if p.admission.id == spec.id {
                return Err(Error::DuplicateId);
            }
            occupied[p.admission.slot as usize] = true;
        }
        let slot = occupied.iter().position(|used| !used).ok_or(Error::Full)? as u8;
        let epoch = session.next_epoch;
        let next = epoch.checked_add(1).ok_or(Error::Exhausted)?;
        let revision = session.revision.checked_add(1).ok_or(Error::Exhausted)?;
        let admission = Admission {
            id: spec.id,
            slot,
            epoch,
        };
        let p = pilot(admission, spec, &profile, owned_loadout);
        let scale = campaign::player_scale(p.selected_ship);
        let body = p.original_body.clone();
        self.world.spawn((
            Motion::new(slot_position(slot), Vec2::ZERO, -FRAC_PI_2),
            Actor {
                kind: Kind::Player,
                health: p.stats.hull,
                max_health: p.stats.hull,
                radius: pixel_assets::radius(&body, scale),
                scale,
                damage: 0.0,
            },
            body,
            p,
        ));
        let session = self.co_op.as_mut().unwrap();
        session.next_epoch = next;
        session.revision = revision;
        self.publication = None;
        Ok(admission)
    }

    /// Called by the local host controller, never by a remote channel.
    pub fn submit_host_co_op_input(
        &mut self,
        expected_session: ParticipantId,
        sequence: u32,
        input: InputFrame,
    ) -> Result<(), Error> {
        self.check_co_op_session(expected_session)?;
        let host = self.co_op.as_ref().ok_or(Error::NoSession)?.host;
        self.submit_co_op_input(host, 1, sequence, input)
    }
    /// Caller must map the authenticated channel to `id`; packet IDs are not
    /// authentication. This guest path cannot submit an input for the host.
    pub fn submit_guest_co_op_input(
        &mut self,
        expected_session: ParticipantId,
        id: ParticipantId,
        epoch: u32,
        sequence: u32,
        input: InputFrame,
    ) -> Result<(), Error> {
        self.check_co_op_session(expected_session)?;
        if self.co_op.as_ref().ok_or(Error::NoSession)?.host == id {
            return Err(Error::HostOnly);
        }
        self.submit_co_op_input(id, epoch, sequence, input)
    }
    fn submit_co_op_input(
        &mut self,
        id: ParticipantId,
        epoch: u32,
        sequence: u32,
        input: InputFrame,
    ) -> Result<(), Error> {
        if !valid_input(&input) {
            return Err(Error::InvalidInput);
        }
        let tick = self.co_op.as_ref().ok_or(Error::NoSession)?.tick;
        for (_, p) in self.world.query_mut::<(Entity, &mut Pilot)>() {
            if p.admission.id != id {
                continue;
            }
            if p.admission.epoch != epoch {
                return Err(Error::StaleMembership);
            }
            if p.last_sequence.is_some_and(|old| sequence <= old) {
                return Err(Error::StaleSequence);
            }
            p.input = input;
            p.last_sequence = Some(sequence);
            p.input_tick = tick;
            return Ok(());
        }
        Err(Error::UnknownParticipant)
    }
    pub fn leave_co_op(
        &mut self,
        expected_session: ParticipantId,
        id: ParticipantId,
        epoch: u32,
    ) -> Result<(), Error> {
        self.check_co_op_session(expected_session)?;
        let host = self.co_op.as_ref().ok_or(Error::NoSession)?.host;
        let entity = self
            .world
            .query::<(Entity, &Pilot)>()
            .iter()
            .find(|(_, p)| p.admission.id == id && p.admission.epoch == epoch)
            .map(|(entity, _)| entity)
            .ok_or(Error::StaleMembership)?;
        if id == host {
            self.restore_co_op();
            return Ok(());
        }
        let revision = self
            .co_op
            .as_ref()
            .unwrap()
            .revision
            .checked_add(1)
            .ok_or(Error::Exhausted)?;
        let shots: Vec<_> = self
            .world
            .query::<(Entity, &ShotOwner)>()
            .iter()
            .filter(|(_, owner)| owner.id == id && owner.epoch == epoch)
            .map(|(e, _)| e)
            .collect();
        for shot in shots {
            let _ = self.world.despawn(shot);
        }
        let _ = self.world.despawn(entity);
        self.co_op.as_mut().unwrap().revision = revision;
        self.publication = None;
        Ok(())
    }
    /// Restores the original world, profile, controls, effects and RNG exactly.
    pub fn end_co_op(&mut self, expected_session: ParticipantId) -> Result<(), Error> {
        self.check_co_op_session(expected_session)?;
        self.restore_co_op();
        Ok(())
    }
    pub(super) fn check_co_op_session(&self, expected: ParticipantId) -> Result<(), Error> {
        let session = self.co_op.as_ref().ok_or(Error::NoSession)?;
        if session.id != expected {
            return Err(Error::StaleSession);
        }
        Ok(())
    }
    pub(super) fn restore_co_op(&mut self) -> bool {
        let Some(session) = self.co_op.take() else {
            return false;
        };
        let generation = self.publication_generation;
        *self = *session.solo;
        self.publication_generation = generation;
        true
    }

    /// Exactly one authoritative fixed step; no remote dt/position/health input.
    /// Only primary weapons and ordinary families are enabled in this stage.
    pub fn tick_co_op(&mut self, expected_session: ParticipantId) -> Result<(), Error> {
        self.check_co_op_session(expected_session)?;
        if self.paused {
            return Err(Error::Paused);
        }
        let session = self.co_op.as_mut().ok_or(Error::NoSession)?;
        let next_tick = session.tick.checked_add(1).ok_or(Error::Exhausted)?;
        let next_revision = session.revision.checked_add(1).ok_or(Error::Exhausted)?;
        session.tick = next_tick;
        session.revision = next_revision;
        let tick = session.tick;
        let dt = TICK_SECONDS;
        self.time += dt;
        self.shake = (self.shake - dt * 6.0).max(0.0);
        for light in &mut self.transient_lights {
            light.life -= dt;
        }
        self.transient_lights.retain(|light| light.life > 0.0);
        for (_, motion) in self.world.query_mut::<(Entity, &mut Motion)>() {
            motion.previous = motion.position;
            motion.previous_heading = motion.heading;
        }
        self.previous_camera = self.camera;
        self.respawn_co_op_pilots();
        let mut fires = [None; MAX_PARTICIPANTS];
        let mut wake_origins = [None; MAX_PARTICIPANTS];
        for (entity, (motion, p, actor)) in self
            .world
            .query_mut::<(Entity, (&mut Motion, &mut Pilot, &mut Actor))>()
        {
            if tick - p.input_tick > INPUT_TIMEOUT_TICKS {
                p.input = InputFrame::default();
                p.destination = None;
            }
            p.since_damage += dt;
            p.invulnerability = (p.invulnerability - dt).max(0.0);
            p.nova_timer -= dt;
            if p.fire_timer > 0.0 {
                p.fire_timer -= dt;
                if p.fire_timer.abs() <= 0.000_001 {
                    p.fire_timer = 0.0;
                }
            }
            if p.since_damage >= 5.0 {
                actor.health = (actor.health + p.stats.regeneration * dt).min(p.stats.hull);
            }
            if p.since_damage >= 4.0 {
                p.shield = (p.shield + p.stats.shield_regeneration * dt).min(p.stats.shield);
            }
            *motion = co_op_control::control_motion(
                *motion,
                &mut p.throttle,
                &mut p.destination,
                &p.input,
                p.stats,
                dt,
            );
            p.input.destination = None; // A click command is applied once; held controls remain transient.
            // Solo stamps the pre-integration position with the current heading.
            wake_origins[p.admission.slot as usize] = Some((entity, motion.position));
            let safe = motion.position.length_squared() <= SAFE_RADIUS * SAFE_RADIUS
                || regions::protected(self.profile.seed, motion.position).is_some();
            if p.input.fire && !safe && p.fire_timer <= 0.0 {
                p.fire_timer += 1.0 / p.stats.fire_rate;
                fires[p.admission.slot as usize] = Some((
                    entity,
                    *motion,
                    p.stats,
                    ShotOwner {
                        id: p.admission.id,
                        epoch: p.admission.epoch,
                    },
                ));
            } else if !p.input.fire || safe {
                p.fire_timer = p.fire_timer.max(0.0);
            }
        }
        let mut bonuses = [None; MAX_PARTICIPANTS];
        let mut echoes = [None; MAX_PARTICIPANTS];
        for (entity, motion, stats, owner) in fires.into_iter().flatten() {
            let direction = forward(motion.heading);
            if let Some(shot) = self.spawn_bullet_entity(
                motion.position + direction * 19.0,
                direction * 430.0 + motion.velocity * 0.25,
                stats.damage,
                true,
                1,
            ) {
                self.world.insert_one(shot, owner).unwrap();
                let mut pilot = self.world.get::<&mut Pilot>(entity).unwrap();
                // Only admitted primaries count. This bounded phase is local to
                // the membership, never a saved or remotely supplied counter.
                pilot.primary_phase = (pilot.primary_phase + 1) % 5;
                if pilot.pulse_lens_rank > 0 && pilot.primary_phase == 0 {
                    bonuses[pilot.admission.slot as usize] = Some((motion, stats, owner));
                }
                if pilot.echo_rank > 0 {
                    let period = if pilot.echo_evolution >= 5 { 6 } else { 8 };
                    pilot.echo_phase = (pilot.echo_phase + 1) % period;
                    if pilot.echo_phase == 0 {
                        echoes[pilot.admission.slot as usize] =
                            Some((motion, stats, owner, pilot.echo_rank, pilot.echo_evolution));
                    }
                }
                drop(pilot);
                self.event("shot", motion.position, 0.6);
            }
        }
        // All queued primaries retain priority over bonus bolts at the shared
        // cap. A refused fifth bonus is consumed, never replayed on shot six.
        for (motion, stats, owner) in bonuses.into_iter().flatten() {
            let direction = forward(motion.heading);
            if let Some(shot) = self.spawn_bullet_entity(
                motion.position + direction * 21.0,
                direction * 500.0 + motion.velocity * 0.25,
                stats.damage * 1.5,
                true,
                1,
            ) {
                self.world.insert_one(shot, owner).unwrap();
            }
        }
        // Ordinary Echo follows primaries and Lens at the same shared cap.
        // Each authored fan projectile is a real piercing bullet; partial admission
        // consumes the trigger exactly as solo fire(), without a replay queue.
        for (motion, stats, owner, rank, evolution) in echoes.into_iter().flatten() {
            let position = motion.position + forward(motion.heading) * 19.0;
            self.transient_light(position, 95.0, [0.67, 0.38, 1.0], 1.1, 0.25);
            let damage = stats.damage * (0.5 + 0.1 * rank.saturating_sub(1) as f32);
            let angles: &[f32] = if evolution >= 3 {
                &[-0.32, -0.16, 0.0, 0.16, 0.32, PI - 0.18, PI + 0.18]
            } else if evolution >= 2 {
                &[-0.32, -0.16, 0.0, 0.16, 0.32]
            } else {
                &[-0.16, 0.0, 0.16]
            };
            let mut admitted = false;
            for angle in angles {
                if let Some(shot) = self.spawn_bullet_entity(
                    position,
                    forward(motion.heading + angle) * 460.0,
                    damage,
                    true,
                    if evolution >= 4 { 3 } else { 2 },
                ) {
                    self.world.insert_one(shot, owner).unwrap();
                    admitted = true;
                }
            }
            if admitted {
                self.event("ability-echo", motion.position, 0.65);
            }
            // Solo retains this bounded muzzle burst/light even if no fan fits.
            self.burst(position, 8, rgba(187, 159, 255, 255), 25.0);
        }
        self.refresh_sites();
        let positions = self.co_op_positions();
        for position in positions.into_iter().flatten() {
            self.update_chunks(position);
        }
        let targets = self.co_op_targets();
        let target = targets
            .into_iter()
            .flatten()
            .filter(|(_, safe)| !safe)
            .map(|(position, _)| position)
            .max_by(|a, b| a.length_squared().total_cmp(&b.length_squared()))
            .unwrap_or(self.position());
        if self.safe_zone_at(target).is_none() {
            self.expedition += dt;
        } else {
            self.expedition = 0.0;
        }
        self.update_spawns(target, dt);
        self.update_enemies(self.position(), dt);
        self.update_asteroid_orbits(dt);
        self.integrate(dt);
        self.enforce_asteroid_shields(true);
        self.enforce_actor_sanctuaries();
        self.confine_co_op_pilots();
        self.update_segments(dt);
        self.resolve_collisions(dt);
        self.enforce_asteroid_shields(false);
        self.confine_co_op_pilots();
        // No pickup grants. Existing fixtures/drops expire or disappear on contact.
        self.update_co_op_pickups(dt);
        self.respawn_co_op_pilots();
        self.repair_co_op_home_arrivals();
        self.update_co_op_wakes(wake_origins, dt);
        self.emit_projectile_trails(dt);
        self.update_particles(dt);
        self.sync_co_op_host();
        self.confine_world_geometry();
        self.camera += (self.position() - self.camera) * (1.0 - (-5.0 * dt).exp());
        Ok(())
    }

    pub(super) fn co_op_positions(&self) -> [Option<Vec2>; MAX_PARTICIPANTS] {
        let mut positions = [None; MAX_PARTICIPANTS];
        if self.co_op.is_some() {
            for (_, (motion, actor, p)) in self
                .world
                .query::<(Entity, (&Motion, &Actor, &Pilot))>()
                .iter()
            {
                if actor.health > 0.0 {
                    positions[p.admission.slot as usize] = Some(motion.position);
                }
            }
        }
        positions
    }
    pub(super) fn co_op_targets(&self) -> [Option<(Vec2, bool)>; MAX_PARTICIPANTS] {
        self.co_op_positions()
            .map(|position| position.map(|p| (p, self.safe_zone_at(p).is_some())))
    }
    fn confine_co_op_pilots(&mut self) {
        for (_, (motion, _)) in self.world.query_mut::<(Entity, (&mut Motion, &Pilot))>() {
            let bounded = motion
                .position
                .clamp(Vec2::splat(-WORLD_LIMIT), Vec2::splat(WORLD_LIMIT));
            if bounded.x != motion.position.x {
                motion.velocity.x = 0.0;
            }
            if bounded.y != motion.position.y {
                motion.velocity.y = 0.0;
            }
            motion.position = bounded;
            motion.previous = motion
                .previous
                .clamp(Vec2::splat(-WORLD_LIMIT), Vec2::splat(WORLD_LIMIT));
        }
    }
    pub(super) fn cancel_co_op_course(&mut self) {
        if let Ok(mut pilot) = self.world.get::<&mut Pilot>(self.player) {
            pilot.destination = None;
            pilot.input = InputFrame::default();
        }
    }
    fn sync_co_op_host(&mut self) {
        if let Ok(p) = self.world.get::<&Pilot>(self.player) {
            self.throttle = p.throttle;
            self.destination = p.destination;
            self.shield = p.shield;
            self.since_damage = p.since_damage;
            self.invulnerability = p.invulnerability;
        }
    }
    fn repair_co_op_home_arrivals(&mut self) {
        for (_, (motion, actor, body, p)) in self
            .world
            .query_mut::<(Entity, (&Motion, &mut Actor, &mut PixelBody, &mut Pilot))>()
        {
            let home = motion.position.length_squared() <= SAFE_RADIUS * SAFE_RADIUS;
            if home && !p.was_home {
                actor.health = p.stats.hull;
                p.shield = p.stats.shield;
                *body = p.original_body.clone();
            }
            p.was_home = home;
        }
    }
    fn respawn_co_op_pilots(&mut self) {
        let mut dead = [None; MAX_PARTICIPANTS];
        for (entity, (motion, actor, body, p)) in self.world.query_mut::<(
            Entity,
            (&mut Motion, &mut Actor, &mut PixelBody, &mut Pilot),
        )>() {
            if actor.health > 0.0 && body.occupied_count() > 0 {
                continue;
            }
            dead[p.admission.slot as usize] = Some((entity, motion.position, p.admission));
            *motion = Motion::new(slot_position(p.admission.slot), Vec2::ZERO, -FRAC_PI_2);
            actor.health = p.stats.hull;
            *body = p.original_body.clone();
            p.shield = p.stats.shield;
            p.invulnerability = 2.0;
            p.since_damage = 10.0;
            p.throttle = 0.0;
            p.destination = None;
            p.fire_timer = 0.0;
            p.wake.clear();
            p.wake_timer = 0.0;
            p.input = InputFrame::default();
            p.was_home = true;
        }
        for (_, position, admission) in dead.into_iter().flatten() {
            // Only the host's local death may drive its death screen. A remote
            // respawn remains a world explosion with the same light/burst.
            self.event(
                if admission.slot == 0 {
                    "death"
                } else {
                    "explosion"
                },
                position,
                1.0,
            );
            self.burst(position, 40, rgba(255, 130, 70, 255), 90.0);
            let shots: Vec<_> = self
                .world
                .query::<(Entity, &ShotOwner)>()
                .iter()
                .filter(|(_, owner)| owner.id == admission.id && owner.epoch == admission.epoch)
                .map(|(e, _)| e)
                .collect();
            for shot in shots {
                let _ = self.world.despawn(shot);
            }
        }
    }
    fn update_co_op_pickups(&mut self, dt: f32) {
        let positions = self.co_op_positions();
        let mut remove = Vec::new();
        for (entity, (motion, loot)) in self.world.query_mut::<(Entity, (&Motion, &mut Loot))>() {
            loot.life -= dt;
            if loot.life <= 0.0
                || positions
                    .iter()
                    .flatten()
                    .any(|p| p.distance_squared(motion.position) < 144.0)
            {
                remove.push(entity);
            }
        }
        for entity in remove {
            let _ = self.world.despawn(entity);
        }
    }

    pub(super) fn hurt_co_op_pilot(&mut self, entity: Entity, damage: f32, local: Option<Vec2>) {
        let Some((motion, actor)) = self
            .world
            .get::<&Motion>(entity)
            .ok()
            .zip(self.world.get::<&Actor>(entity).ok())
            .map(|(m, a)| (*m, *a))
        else {
            return;
        };
        if !damage.is_finite() || damage <= 0.0 || self.safe_zone_at(motion.position).is_some() {
            return;
        }
        let Ok(mut p) = self.world.get::<&mut Pilot>(entity) else {
            return;
        };
        if p.invulnerability > 0.0 || actor.health <= 0.0 {
            return;
        }
        p.invulnerability = 0.45;
        p.since_damage = 0.0;
        let damage = damage * p.damage_multiplier;
        let absorbed = p.shield.min(damage);
        p.shield -= absorbed;
        let hull_damage = damage - absorbed;
        let nova = if p.nova_rank > 0 && p.nova_timer <= 0.0 {
            p.nova_timer = 18.0 - 2.0 * p.nova_rank.saturating_sub(1) as f32;
            if p.nova_evolution >= 4 {
                p.nova_timer *= 0.85;
            }
            Some((
                p.nova_evolution,
                ShotOwner {
                    id: p.admission.id,
                    epoch: p.admission.epoch,
                },
            ))
        } else {
            None
        };
        drop(p);
        if hull_damage > 0.0 {
            if let Ok(mut a) = self.world.get::<&mut Actor>(entity) {
                a.health -= hull_damage;
            }
            if let Ok(mut body) = self.world.get::<&mut PixelBody>(entity) {
                body.damage(local.unwrap_or(Vec2::new(0.0, 3.0)), 2.5 / actor.scale);
                if body.occupied_count() == 0
                    && let Ok(mut a) = self.world.get::<&mut Actor>(entity)
                {
                    a.health = 0.0;
                }
            }
        }
        if absorbed > 0.0 {
            self.event("shield", motion.position, 0.8);
        }
        let cue_index = self.events.len();
        self.event("hit", motion.position, 0.9);
        // Preserve the actual impact light; only classify an admitted remote
        // cue as a world impact rather than the host's personal hit sound.
        if entity != self.player
            && let Some(cue) = self.events.get_mut(cue_index)
        {
            cue.intensity = 0.25;
        }
        if let Some((evolution, owner)) = nova {
            self.emit_repulsor_nova(motion.position, evolution, Some(owner));
        }
    }

    /// Shared actual effect; solo retains its own dispatch/cooldown, while co-op
    /// admits ordinary ID4 only and supplies transient membership ownership.
    pub(super) fn emit_repulsor_nova(
        &mut self,
        position: Vec2,
        evolution: u8,
        owner: Option<ShotOwner>,
    ) {
        let reach = if evolution >= 4 {
            120.0
        } else if evolution >= 2 {
            90.0
        } else {
            70.0
        };
        let (light, life) = crate::co_op_fx::lights(position)[0].clone();
        self.transient_light(position, light.radius, light.color, light.intensity, life);
        if evolution >= 3 {
            let mut reflected = [None; 16];
            let mut count = 0;
            let mut cleared = Vec::new();
            for (entity, (motion, bullet)) in self
                .world
                .query_mut::<(Entity, (&mut Motion, &mut Bullet))>()
            {
                if !bullet.player && motion.position.distance(position) < reach {
                    if evolution >= 5 && count < reflected.len() {
                        bullet.player = true;
                        bullet.remaining_hits = 2;
                        bullet.previous_hit = None;
                        bullet.life = 1.5;
                        motion.velocity = -motion.velocity;
                        motion.heading = motion.velocity.y.atan2(motion.velocity.x);
                        reflected[count] = Some(entity);
                        count += 1;
                    } else {
                        cleared.push(entity);
                    }
                }
            }
            for entity in reflected.into_iter().flatten() {
                if let Some(owner) = owner {
                    self.world.insert_one(entity, owner).unwrap();
                }
            }
            for entity in cleared {
                let _ = self.world.despawn(entity);
            }
        }
        let targets: Vec<_> = self
            .world
            .query::<(Entity, (&Motion, &Actor))>()
            .iter()
            .filter(|(_, (m, a))| {
                a.kind != Kind::Player
                    && (m.position - position).length() < reach
                    && self.safe_zone_at(m.position).is_none()
            })
            .map(|(e, (m, _))| (e, *m))
            .collect();
        // Attribute both immediate kills and later reflected bullets to the
        // triggering membership, then restore the enclosing collision's owner.
        let previous_owner = self
            .co_op
            .as_mut()
            .map(|session| std::mem::replace(&mut session.hit_owner, owner));
        for (entity, motion) in targets {
            let direction = (motion.position - position).normalize_or_zero();
            self.damage_actor(entity, 12.0, Vec2::ZERO, direction * 110.0);
        }
        if let Some(previous) = previous_owner {
            self.co_op.as_mut().unwrap().hit_owner = previous;
        }
        let before_rays = self.particles.len();
        for i in 0..crate::co_op_fx::RAYS {
            let (offset, velocity, color, size) = crate::co_op_fx::ray(i);
            self.particle(
                position + offset,
                velocity,
                crate::co_op_fx::RAY_LIFE as f32,
                color,
                size,
            );
        }
        self.event("ability-nova", position, 1.0);
        if let Some(owner) = owner {
            // Only an actual active, triggering native membership can publish.
            let active = self
                .world
                .query::<&Pilot>()
                .iter()
                .any(|p| p.admission.id == owner.id && p.admission.epoch == owner.epoch);
            if active
                && position.is_finite()
                && let Some(session) = &mut self.co_op
            {
                session.nova_fx.record(
                    session.tick,
                    owner.id,
                    owner.epoch,
                    position,
                    (self.particles.len() - before_rays) as u8,
                );
            }
        }
    }
    pub(super) fn co_op_contacts(
        &mut self,
        actors: &[(Entity, Motion, Actor)],
        segments: &[(Entity, Motion, Entity, f32, f32)],
    ) {
        let mut pilots = [None; MAX_PARTICIPANTS];
        for (entity, (motion, actor, p)) in self
            .world
            .query::<(Entity, (&Motion, &Actor, &Pilot))>()
            .iter()
        {
            pilots[p.admission.slot as usize] = Some((entity, *motion, *actor));
        }
        for (player, motion, actor) in pilots.into_iter().flatten() {
            if actor.health <= 0.0 || self.safe_zone_at(motion.position).is_some() {
                continue;
            }
            for (entity, other, other_actor) in actors {
                if other_actor.kind == Kind::Player || !self.world.contains(*entity) {
                    continue;
                }
                let delta = motion.position - other.position;
                let radius = actor.radius + other_actor.radius;
                if delta.length_squared() >= radius * radius {
                    continue;
                }
                let direction = if delta.length_squared() > 0.01 {
                    delta.normalize()
                } else {
                    Vec2::X
                };
                if let Ok(mut m) = self.world.get::<&mut Motion>(player) {
                    m.position = other.position + direction * radius;
                    m.velocity += direction * 45.0;
                }
                if let Ok(mut m) = self.world.get::<&mut Motion>(*entity) {
                    m.velocity -= direction * 25.0;
                }
                self.hurt_co_op_pilot(player, other_actor.damage, None);
            }
            for (entity, other, owner, _, radius) in segments {
                if self.world.contains(*entity)
                    && motion.position.distance_squared(other.position)
                        < (actor.radius + radius).powi(2)
                {
                    let boss = self
                        .world
                        .get::<&Brain>(*owner)
                        .is_ok_and(|brain| brain.boss);
                    self.hurt_co_op_pilot(player, if boss { 13.0 } else { 7.0 }, None);
                }
            }
        }
    }
    pub(super) fn record_co_op_destruction(&mut self, entity: Entity, kind: Kind) {
        let Some(session) = &mut self.co_op else {
            return;
        };
        if session.notices.len() >= MAX_NOTICES {
            session.dropped_notices = session.dropped_notices.saturating_add(1);
            return;
        }
        session.notices.push(Destruction {
            tick: session.tick,
            id: entity_key(entity),
            kind: kind_name(kind),
            owner: session.hit_owner.map(|owner| owner.id),
            owner_epoch: session.hit_owner.map(|owner| owner.epoch),
        });
    }
    pub fn take_co_op_destructions(
        &mut self,
        expected_session: ParticipantId,
    ) -> Result<Vec<Destruction>, Error> {
        self.check_co_op_session(expected_session)?;
        Ok(self.co_op.as_mut().map_or_else(Vec::new, |session| {
            std::mem::replace(&mut session.notices, Vec::with_capacity(MAX_NOTICES))
        }))
    }

    pub fn co_op_snapshot(&self) -> Option<Snapshot> {
        let session = self.co_op.as_ref()?;
        let mut participants: Vec<_> = self
            .world
            .query::<(&Motion, &Actor, &Pilot)>()
            .iter()
            .map(|(m, a, p)| ParticipantState {
                id: p.admission.id,
                slot: p.admission.slot,
                epoch: p.admission.epoch,
                local_seed: p.spec.local_seed,
                identity: p.spec.identity.clone(),
                selected_ship: p.selected_ship,
                owned_loadout: p.owned_loadout.clone(),
                position: m.position.to_array(),
                previous: m.previous.to_array(),
                velocity: m.velocity.to_array(),
                heading: m.heading,
                previous_heading: m.previous_heading,
                hull: a.health,
                max_hull: a.max_health,
                shield: p.shield,
                max_shield: p.stats.shield,
                throttle: p.throttle,
                last_sequence: p.last_sequence,
            })
            .collect();
        participants.sort_by_key(|p| p.slot);
        let mut projectiles: Vec<_> = self
            .world
            .query::<(Entity, (&Motion, &Bullet, Option<&ShotOwner>))>()
            .iter()
            .map(|(entity, (m, b, owner))| ProjectileState {
                id: entity_key(entity),
                owner: owner.map(|o| o.id),
                owner_epoch: owner.map(|o| o.epoch),
                position: m.position.to_array(),
                previous: m.previous.to_array(),
                velocity: m.velocity.to_array(),
                damage: b.damage,
                life: b.life,
                remaining_hits: b.remaining_hits,
            })
            .collect();
        projectiles.sort_by(|a, b| a.id.cmp(&b.id));
        let nova_fx = (session.nova_fx.sequence > 0).then(|| {
            let mut journal = session.nova_fx.clone();
            journal.records.retain(|r| {
                f64::from(session.tick.saturating_sub(r.tick)) / 60.0 < crate::co_op_fx::LIFE
                    && participants
                        .iter()
                        .any(|p| p.id == r.emitter && p.epoch == r.epoch)
            });
            journal
        });
        Some(Snapshot {
            protocol: PROTOCOL,
            session: session.id,
            world_seed: self.profile.seed,
            tick: session.tick,
            revision: session.revision,
            progression: false,
            participants,
            projectiles,
            dropped_notices: session.dropped_notices,
            nova_fx,
            wake_fx: Some(crate::co_op_wake_fx::WakeMeta {
                points: self
                    .world
                    .query::<&Pilot>()
                    .iter()
                    .map(|p| p.wake.len())
                    .sum::<usize>()
                    .try_into()
                    .unwrap_or(u16::MAX),
            }),
        })
    }
    pub fn co_op_session(&self) -> Option<ParticipantId> {
        self.co_op.as_ref().map(|session| session.id)
    }
    pub(super) fn co_op_entity_rows(&self) -> Vec<EntityState> {
        let mut rows: Vec<_> = self
            .world
            .query::<(
                Entity,
                (
                    &Motion,
                    &PixelBody,
                    Option<&Actor>,
                    Option<&Segment>,
                    Option<&Pilot>,
                ),
            )>()
            .iter()
            .map(|(entity, (m, body, actor, segment, pilot))| EntityState {
                id: entity_key(entity),
                kind: actor.map_or("segment", |a| kind_name(a.kind)),
                participant: pilot.map(|p| p.admission.id),
                position: m.position.to_array(),
                previous: m.previous.to_array(),
                velocity: m.velocity.to_array(),
                heading: m.heading,
                previous_heading: m.previous_heading,
                scale: actor.map_or_else(|| segment.map_or(1.0, |s| s.scale), |a| a.scale),
                radius: actor.map_or_else(|| segment.map_or(0.0, |s| s.radius), |a| a.radius),
                hull: actor.map_or_else(|| segment.map_or(0.0, |s| s.health), |a| a.health),
                max_hull: actor.map_or(0.0, |a| a.max_health),
                width: body.width,
                height: body.height,
                occupied: body.occupied_count(),
                body_signature: co_op_publish::body_signature(body),
            })
            .collect();
        rows.sort_by(|a, b| a.id.cmp(&b.id));
        rows
    }
    pub fn co_op_entity_page(
        &self,
        expected_session: ParticipantId,
        revision: u32,
        offset: usize,
    ) -> Result<EntityPage, Error> {
        self.check_co_op_session(expected_session)?;
        let session = self.co_op.as_ref().ok_or(Error::NoSession)?;
        if session.revision != revision {
            return Err(Error::StaleSnapshot);
        }
        let rows = self.co_op_entity_rows();
        if offset > rows.len() {
            return Err(Error::InvalidOffset);
        }
        let total = rows.len();
        let entities = rows
            .into_iter()
            .skip(offset)
            .take(ENTITY_PAGE_SIZE)
            .collect::<Vec<_>>();
        let next = offset + entities.len();
        Ok(EntityPage {
            session: session.id,
            tick: session.tick,
            revision,
            offset,
            next_offset: (next < total).then_some(next),
            entities,
        })
    }
    pub fn co_op_body_page(
        &self,
        expected_session: ParticipantId,
        revision: u32,
        key: &str,
        offset: usize,
    ) -> Result<BodyPage, Error> {
        self.check_co_op_session(expected_session)?;
        let session = self.co_op.as_ref().ok_or(Error::NoSession)?;
        if session.revision != revision {
            return Err(Error::StaleSnapshot);
        }
        if key.len() != 16
            || !key
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(Error::InvalidEntity);
        }
        let entity = u64::from_str_radix(key, 16)
            .ok()
            .and_then(Entity::from_bits)
            .ok_or(Error::InvalidEntity)?;
        let body = self
            .world
            .get::<&PixelBody>(entity)
            .map_err(|_| Error::InvalidEntity)?;
        if offset > body.occupied_count() {
            return Err(Error::InvalidOffset);
        }
        let cells = body
            .cells_indexed()
            .skip(offset)
            .take(CELL_PAGE_SIZE)
            .map(|(index, _, color)| [index as u32, color])
            .collect::<Vec<_>>();
        let next = offset + cells.len();
        Ok(BodyPage {
            session: session.id,
            tick: session.tick,
            revision,
            id: key.into(),
            width: body.width,
            height: body.height,
            occupied: body.occupied_count(),
            offset,
            next_offset: (next < body.occupied_count()).then_some(next),
            cells,
        })
    }
}

#[cfg(test)]
#[path = "co_op_starter_tests.rs"]
mod starter_tests;

#[cfg(test)]
#[path = "co_op_passive_tests.rs"]
mod passive_tests;

#[cfg(test)]
#[path = "co_op_echo_tests.rs"]
mod echo_tests;

#[cfg(test)]
#[path = "co_op_nova_tests.rs"]
mod nova_tests;

#[cfg(test)]
#[path = "co_op_wake_tests.rs"]
mod wake_tests;

#[path = "co_op_wake.rs"]
mod wake;
